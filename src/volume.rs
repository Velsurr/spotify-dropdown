//! Spotify's Windows audio-session volume. Never reads or writes endpoint/system volume.
//! All COM interfaces stay on one MTA worker; slider requests share a single coalescing slot.
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex, mpsc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Media::Audio::{
            AudioSessionStateActive, AudioSessionStateExpired, DEVICE_STATE_ACTIVE,
            IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume,
            MMDeviceEnumerator, eRender,
        },
        System::{
            Com::{
                CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
            },
            Threading::{
                OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
                QueryFullProcessImageNameW,
            },
        },
    },
    core::{Interface, PWSTR, Result},
};

const REFRESH_INTERVAL: Duration = Duration::from_secs(3);
const WRITE_INTERVAL: Duration = Duration::from_millis(40);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VolumeState {
    pub available: bool,
    /// Stored session gain from 0.0 to 1.0, preserved independently of mute.
    pub level: f32,
    pub muted: bool,
    pub error: Option<String>,
}

#[derive(Default)]
struct Request {
    level: Option<f32>,
    mute_toggles: bool,
    stop: bool,
}

pub struct VolumeBackend {
    shared: Arc<Mutex<VolumeState>>,
    request: Arc<(Mutex<Request>, Condvar)>,
    worker: Option<JoinHandle<()>>,
    done: mpsc::Receiver<()>,
}

impl VolumeBackend {
    pub fn start(ctx: eframe::egui::Context) -> Self {
        let shared = Arc::new(Mutex::new(VolumeState::default()));
        let request = Arc::new((Mutex::new(Request::default()), Condvar::new()));
        let worker_state = Arc::clone(&shared);
        let worker_request = Arc::clone(&request);
        let (finished, done) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("spotify-volume".into())
            .spawn(move || {
                worker_loop(worker_state, worker_request, ctx);
                let _ = finished.send(());
            });
        let worker = match worker {
            Ok(worker) => Some(worker),
            Err(error) => {
                shared.lock().unwrap_or_else(|e| e.into_inner()).error = Some(format!(
                    "Não foi possível iniciar o controle de volume: {error}"
                ));
                None
            }
        };
        Self {
            shared,
            request,
            worker,
            done,
        }
    }

    pub fn state(&self) -> VolumeState {
        self.shared
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn set_level(&self, level: f32) {
        if !level.is_finite() {
            return;
        }
        let (lock, wake) = &*self.request;
        let mut request = lock.lock().unwrap_or_else(|e| e.into_inner());
        if !request.stop {
            request.level = Some(level.clamp(0.0, 1.0));
            if level > 0.0 {
                // A later positive slider adjustment supersedes earlier pending mute clicks.
                request.mute_toggles = false;
            }
        }
        drop(request);
        wake.notify_one();
    }

    pub fn toggle_mute(&self) {
        let (lock, wake) = &*self.request;
        let mut request = lock.lock().unwrap_or_else(|e| e.into_inner());
        if !request.stop {
            // Preserve double-click parity without queuing an unbounded sequence.
            request.mute_toggles = !request.mute_toggles;
        }
        drop(request);
        wake.notify_one();
    }
}

impl Drop for VolumeBackend {
    fn drop(&mut self) {
        let (lock, wake) = &*self.request;
        lock.lock().unwrap_or_else(|e| e.into_inner()).stop = true;
        wake.notify_one();
        if let Some(worker) = self.worker.take() {
            // Audio drivers can stall inside COM. Do not hold the GUI indefinitely on exit.
            // A detached worker has no external process and ends with this application.
            if self.done.recv_timeout(Duration::from_millis(500)).is_ok() {
                let _ = worker.join();
            }
        }
    }
}

struct Apartment;
impl Apartment {
    fn new() -> Result<Self> {
        // SAFETY: this dedicated thread initializes and releases its own COM apartment.
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        }
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

struct Process(HANDLE);
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn is_spotify_process(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    // SAFETY: read-only process query with the least required access; no elevation or injection.
    let Ok(handle) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
        return false;
    };
    let process = Process(handle);
    let mut path = vec![0u16; 32768];
    let mut length = path.len() as u32;
    if unsafe {
        QueryFullProcessImageNameW(
            process.0,
            PROCESS_NAME_WIN32,
            PWSTR(path.as_mut_ptr()),
            &mut length,
        )
    }
    .is_err()
    {
        return false;
    }
    let path = String::from_utf16_lossy(&path[..length as usize]);
    path.rsplit(['\\', '/'])
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("spotify.exe"))
}

struct Session {
    volume: ISimpleAudioVolume,
    active: bool,
}

fn sessions() -> Result<Vec<Session>> {
    // SAFETY: all interfaces are created, used, and dropped on the initialized MTA worker.
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let devices = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
        let mut result = Vec::new();
        let mut processes = HashMap::<u32, bool>::new();
        let mut first_error = None;
        let mut readable_devices = 0;
        for index in 0..devices.GetCount()? {
            let endpoint = (|| -> Result<_> {
                let device = devices.Item(index)?;
                let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
                let enumerator = manager.GetSessionEnumerator()?;
                let count = enumerator.GetCount()?;
                Ok((enumerator, count))
            })();
            let (enumerator, count) = match endpoint {
                Ok(endpoint) => {
                    readable_devices += 1;
                    endpoint
                }
                Err(error) => {
                    first_error.get_or_insert(error);
                    continue;
                }
            };
            for index in 0..count {
                let session = (|| -> Result<Option<Session>> {
                    let control = enumerator.GetSession(index)?;
                    let status = control.GetState()?;
                    if status == AudioSessionStateExpired {
                        return Ok(None);
                    }
                    let extended: IAudioSessionControl2 = control.cast()?;
                    let pid = extended.GetProcessId()?;
                    if !*processes
                        .entry(pid)
                        .or_insert_with(|| is_spotify_process(pid))
                    {
                        return Ok(None);
                    }
                    Ok(Some(Session {
                        volume: control.cast()?,
                        active: status == AudioSessionStateActive,
                    }))
                })();
                // A single disappearing/protected session must not disable other endpoints.
                if let Ok(Some(session)) = session {
                    result.push(session);
                }
            }
        }
        if readable_devices == 0
            && let Some(error) = first_error
        {
            return Err(error);
        }
        result.sort_by_key(|session| !session.active);
        Ok(result)
    }
}

fn read_state(sessions: &[Session]) -> VolumeState {
    let mut last_error = None;
    for session in sessions {
        let result = unsafe {
            session.volume.GetMasterVolume().and_then(|level| {
                session
                    .volume
                    .GetMute()
                    .map(|muted| (level, muted.as_bool()))
            })
        };
        match result {
            Ok((level, muted)) if level.is_finite() => {
                return VolumeState {
                    available: true,
                    level: level.clamp(0.0, 1.0),
                    muted,
                    error: None,
                };
            }
            Ok(_) => {}
            Err(error) => {
                last_error = Some(format!(
                    "O Windows não informou o volume do Spotify: {error}"
                ))
            }
        }
    }
    VolumeState {
        error: Some(
            last_error
                .unwrap_or_else(|| "Nenhuma sessão de áudio local do Spotify disponível.".into()),
        ),
        ..Default::default()
    }
}

fn publish(shared: &Mutex<VolumeState>, ctx: &eframe::egui::Context, value: VolumeState) {
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    if *state != value {
        *state = value;
        drop(state);
        ctx.request_repaint();
    }
}

fn worker_loop(
    shared: Arc<Mutex<VolumeState>>,
    request: Arc<(Mutex<Request>, Condvar)>,
    ctx: eframe::egui::Context,
) {
    let _apartment = match Apartment::new() {
        Ok(apartment) => apartment,
        Err(error) => {
            publish(
                &shared,
                &ctx,
                VolumeState {
                    error: Some(format!(
                        "Não foi possível acessar o áudio do Windows: {error}"
                    )),
                    ..Default::default()
                },
            );
            return;
        }
    };
    let mut current = Vec::new();
    let mut refresh_at = Instant::now();
    let mut last_write = Instant::now() - WRITE_INTERVAL;
    loop {
        if Instant::now() >= refresh_at {
            match sessions() {
                Ok(sessions) => {
                    current = sessions;
                    publish(&shared, &ctx, read_state(&current));
                }
                Err(error) => {
                    current.clear();
                    publish(
                        &shared,
                        &ctx,
                        VolumeState {
                            error: Some(format!("Controle de volume indisponível: {error}")),
                            ..Default::default()
                        },
                    );
                }
            }
            refresh_at = Instant::now() + REFRESH_INTERVAL;
        }
        let (lock, wake) = &*request;
        let request = lock.lock().unwrap_or_else(|e| e.into_inner());
        let timeout = refresh_at.saturating_duration_since(Instant::now());
        let (mut request, _) = wake
            .wait_timeout_while(request, timeout, |request| {
                !request.stop && request.level.is_none() && !request.mute_toggles
            })
            .unwrap_or_else(|e| e.into_inner());
        if request.stop {
            return;
        }
        if request.level.is_none() && !request.mute_toggles {
            continue;
        }
        let wait = WRITE_INTERVAL.saturating_sub(last_write.elapsed());
        if !wait.is_zero() {
            let (updated, _) = wake
                .wait_timeout_while(request, wait, |request| !request.stop)
                .unwrap_or_else(|e| e.into_inner());
            request = updated;
            if request.stop {
                return;
            }
        }
        let level = request.level.take();
        let toggle_mute = std::mem::take(&mut request.mute_toggles);
        drop(request);
        if level.is_some() || toggle_mute {
            // Read the actual representative Spotify session at command time, so an
            // external mixer mute is respected even between periodic refreshes.
            let before = read_state(&current);
            let mute = if level.is_some_and(|value| value > 0.0) {
                Some(toggle_mute)
            } else if toggle_mute && before.available {
                Some(!before.muted)
            } else {
                None
            };
            let mut failures = 0;
            for session in &current {
                // SetMute never rewrites the gain. Sessions with different levels
                // therefore retain their own previous volume across mute/unmute.
                let result = (|| -> Result<()> {
                    if let Some(level) = level {
                        unsafe {
                            session.volume.SetMasterVolume(level, std::ptr::null())?;
                        }
                    }
                    if let Some(mute) = mute {
                        unsafe {
                            session.volume.SetMute(mute, std::ptr::null())?;
                        }
                    }
                    Ok(())
                })();
                failures += usize::from(result.is_err());
            }
            last_write = Instant::now();
            let mut state = read_state(&current);
            if failures > 0 {
                state.error = Some("O Windows não aceitou o ajuste em uma ou mais sessões do Spotify. Tente novamente após a atualização.".into());
                refresh_at = Instant::now();
            }
            publish(&shared, &ctx, state);
        }
    }
}

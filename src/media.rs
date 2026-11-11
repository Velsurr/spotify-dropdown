//! Windows GSMTC integration. All COM and image work stays off the UI thread.
use crate::model::{Capabilities, Playback, TICKS_PER_SECOND, Timeline, is_spotify_source};
use std::{
    io::Cursor,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows::{
    Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as SessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    },
    Storage::Streams::DataReader,
    Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
    core::{Error, HRESULT, Result, RuntimeType},
};
use windows_future::{AsyncStatus, IAsyncOperation};

#[derive(Clone)]
pub struct Artwork {
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub connected: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub source: String,
    pub playback: Playback,
    pub capabilities: Capabilities,
    pub timeline: Option<Timeline>,
    pub artwork: Option<Arc<Artwork>>,
    pub problem: Option<String>,
}

#[derive(Clone, Default)]
pub struct BackendState {
    pub snapshot: Snapshot,
    pub busy: bool,
    pub notice: Option<(String, Instant)>,
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    Previous,
    Toggle,
    Next,
}

pub struct Backend {
    shared: Arc<Mutex<BackendState>>,
    commands: SyncSender<Command>,
}

impl Backend {
    pub fn start(ctx: eframe::egui::Context) -> Self {
        let shared = Arc::new(Mutex::new(BackendState::default()));
        let (commands, receiver) = mpsc::sync_channel(1);
        let worker_state = Arc::clone(&shared);
        if let Err(error) = thread::Builder::new()
            .name("spotify-media".into())
            .spawn(move || {
                worker(receiver, worker_state, ctx);
            })
        {
            shared
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .snapshot
                .problem = Some(format!("Não foi possível iniciar a integração: {error}"));
        }
        Self { shared, commands }
    }

    pub fn state(&self) -> BackendState {
        self.shared
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn send(&self, command: Command) {
        let mut state = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        if state.busy {
            return;
        }
        state.busy = true;
        if self.commands.try_send(command).is_err() {
            state.busy = false;
            state.notice = Some((
                "A integração não está disponível. Feche e abra o aplicativo.".into(),
                Instant::now(),
            ));
        }
    }
}

struct Apartment;
impl Apartment {
    fn new() -> Result<Self> {
        // SAFETY: this dedicated thread has no previous COM initialization.
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED)?;
        }
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: paired with successful RoInitialize on this same thread.
        unsafe {
            RoUninitialize();
        }
    }
}

/// Bound each WinRT operation so an unavailable provider cannot stall the worker forever.
fn wait<T: RuntimeType + 'static>(operation: IAsyncOperation<T>) -> Result<T> {
    let deadline = Instant::now() + Duration::from_secs(4);
    while operation.Status()? == AsyncStatus::Started {
        if Instant::now() >= deadline {
            let _ = operation.Cancel();
            return Err(Error::new(
                HRESULT(0x800705B4u32 as i32),
                "O Windows demorou para responder.",
            ));
        }
        thread::sleep(Duration::from_millis(15));
    }
    operation.GetResults()
}

struct Reader {
    manager: Option<SessionManager>,
    artwork_key: String,
    artwork: Option<Arc<Artwork>>,
}

impl Reader {
    fn new() -> Self {
        Self {
            manager: None,
            artwork_key: String::new(),
            artwork: None,
        }
    }

    fn manager(&mut self) -> Result<&SessionManager> {
        if self.manager.is_none() {
            self.manager = Some(wait(SessionManager::RequestAsync()?)?);
        }
        Ok(self.manager.as_ref().expect("manager initialized"))
    }

    fn refresh(&mut self) -> Result<Snapshot> {
        let Some(session) = select_spotify(self.manager()?)? else {
            self.artwork = None;
            self.artwork_key.clear();
            return Ok(Snapshot::default());
        };
        let source = session.SourceAppUserModelId()?.to_string_lossy();
        let playback_info = session.GetPlaybackInfo()?;
        let playback = match playback_info.PlaybackStatus()? {
            Status::Playing => Playback::Playing,
            Status::Paused => Playback::Paused,
            _ => Playback::Stopped,
        };
        let controls = playback_info.Controls()?;
        let capabilities = Capabilities {
            previous: controls.IsPreviousEnabled().unwrap_or(false),
            play: controls.IsPlayEnabled().unwrap_or(false),
            pause: controls.IsPauseEnabled().unwrap_or(false),
            toggle: controls.IsPlayPauseToggleEnabled().unwrap_or(false),
            next: controls.IsNextEnabled().unwrap_or(false),
        };
        let properties = wait(session.TryGetMediaPropertiesAsync()?)?;
        let title = properties.Title()?.to_string_lossy();
        let artist = properties.Artist()?.to_string_lossy();
        let album = properties.AlbumTitle()?.to_string_lossy();
        let key = format!("{source}\0{title}\0{artist}\0{album}");
        if self.artwork_key != key || self.artwork.is_none() {
            self.artwork_key = key;
            self.artwork = properties.Thumbnail().ok().and_then(|reference| {
                let stream = wait(reference.OpenReadAsync().ok()?).ok()?;
                let length = stream.Size().ok()?;
                if length == 0 || length > 4 * 1024 * 1024 {
                    return None;
                }
                let input = stream.GetInputStreamAt(0).ok()?;
                let reader = DataReader::CreateDataReader(&input).ok()?;
                let loaded = wait(reader.LoadAsync(length as u32).ok()?).ok()?;
                let mut bytes = vec![0; loaded as usize];
                reader.ReadBytes(&mut bytes).ok()?;
                let _ = reader.Close();
                let mut decoder = image::ImageReader::new(Cursor::new(bytes))
                    .with_guessed_format()
                    .ok()?;
                let mut limits = image::Limits::default();
                limits.max_image_width = Some(4096);
                limits.max_image_height = Some(4096);
                limits.max_alloc = Some(64 * 1024 * 1024);
                decoder.limits(limits);
                let image = decoder.decode().ok()?.thumbnail(256, 256).to_rgba8();
                Some(Arc::new(Artwork {
                    size: [image.width() as usize, image.height() as usize],
                    rgba: image.into_raw(),
                }))
            });
        }
        let timeline = session.GetTimelineProperties().ok().and_then(|timeline| {
            let mut position = timeline.Position().ok()?.Duration;
            if playback == Playback::Playing {
                // Position is anchored at LastUpdatedTime, not necessarily at this poll.
                let updated = timeline.LastUpdatedTime().ok()?.UniversalTime;
                let now = windows_now_ticks();
                let age = now.saturating_sub(updated);
                if updated > 116_444_736_000_000_000
                    && (0..86_400 * TICKS_PER_SECOND).contains(&age)
                {
                    position = position.saturating_add(age);
                }
            }
            Timeline::from_ticks(
                timeline.StartTime().ok()?.Duration,
                timeline.EndTime().ok()?.Duration,
                position,
                playback == Playback::Playing,
            )
        });
        Ok(Snapshot {
            connected: true,
            title,
            artist,
            album,
            source,
            playback,
            capabilities,
            timeline,
            artwork: self.artwork.clone(),
            problem: None,
        })
    }

    fn execute(&mut self, command: Command) -> std::result::Result<String, String> {
        let session = select_spotify(self.manager().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
            .ok_or("Nenhuma sessão do Spotify disponível.")?;
        let playback = session.GetPlaybackInfo().map_err(|e| e.to_string())?;
        let controls = playback.Controls().map_err(|e| e.to_string())?;
        let operation = match command {
            Command::Previous if controls.IsPreviousEnabled().unwrap_or(false) => {
                session.TrySkipPreviousAsync()
            }
            Command::Next if controls.IsNextEnabled().unwrap_or(false) => {
                session.TrySkipNextAsync()
            }
            Command::Toggle if controls.IsPlayPauseToggleEnabled().unwrap_or(false) => {
                session.TryTogglePlayPauseAsync()
            }
            Command::Toggle
                if playback.PlaybackStatus().ok() == Some(Status::Playing)
                    && controls.IsPauseEnabled().unwrap_or(false) =>
            {
                session.TryPauseAsync()
            }
            Command::Toggle
                if playback.PlaybackStatus().ok() != Some(Status::Playing)
                    && controls.IsPlayEnabled().unwrap_or(false) =>
            {
                session.TryPlayAsync()
            }
            _ => return Err("O Spotify não disponibilizou este controle agora.".into()),
        }
        .map_err(|e| e.to_string())?;
        if wait(operation).map_err(|e| e.to_string())? {
            Ok("Comando aceito pela sessão do Spotify.".into())
        } else {
            Err("O Spotify recusou o comando. Confira a reprodução no aplicativo.".into())
        }
    }
}

fn windows_now_ticks() -> i64 {
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (unix.as_nanos() / 100 + 116_444_736_000_000_000).min(i64::MAX as u128) as i64
}

fn select_spotify(manager: &SessionManager) -> Result<Option<Session>> {
    let current = manager.GetCurrentSession().ok().filter(|s| {
        s.SourceAppUserModelId()
            .is_ok_and(|id| is_spotify_source(&id.to_string_lossy()))
    });
    if current.as_ref().is_some_and(is_playing) {
        return Ok(current);
    }
    let sessions = manager.GetSessions()?;
    let mut first = None;
    for index in 0..sessions.Size()? {
        let session = sessions.GetAt(index)?;
        if !session
            .SourceAppUserModelId()
            .is_ok_and(|id| is_spotify_source(&id.to_string_lossy()))
        {
            continue;
        }
        if is_playing(&session) {
            return Ok(Some(session));
        }
        if first.is_none() {
            first = Some(session);
        }
    }
    Ok(current.or(first))
}

fn is_playing(session: &Session) -> bool {
    session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .ok()
        == Some(Status::Playing)
}

fn worker(
    receiver: Receiver<Command>,
    shared: Arc<Mutex<BackendState>>,
    ctx: eframe::egui::Context,
) {
    let _apartment = match Apartment::new() {
        Ok(apartment) => apartment,
        Err(error) => {
            shared
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .snapshot
                .problem = Some(format!(
                "Não foi possível iniciar a API do Windows: {error}"
            ));
            ctx.request_repaint();
            return;
        }
    };
    let mut reader = Reader::new();
    let mut command = None;
    loop {
        let executed_command = command.is_some();
        let notice = command
            .take()
            .map(|command| reader.execute(command).unwrap_or_else(|error| error));
        let snapshot = match reader.refresh() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                reader.manager = None; // Reconnect after provider/service failures.
                Snapshot {
                    problem: Some(format!("Falha ao consultar a mídia do Windows: {error}")),
                    ..Default::default()
                }
            }
        };
        let interval = if snapshot.playback == Playback::Playing {
            Duration::from_secs(1)
        } else {
            Duration::from_secs(3)
        };
        {
            let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
            state.snapshot = snapshot;
            if executed_command {
                state.busy = false;
            }
            if let Some(notice) = notice {
                state.notice = Some((notice, Instant::now()));
            }
        }
        ctx.request_repaint();
        match receiver.recv_timeout(interval) {
            Ok(next) => command = Some(next),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

/// Read-only probe, including all session IDs to explain why Spotify was not selected.
pub fn diagnostic_report() -> String {
    let mut report = format!(
        "Spotify Dropdown v{} — diagnóstico local\nData: {:?}\n",
        env!("CARGO_PKG_VERSION"),
        SystemTime::now()
    );
    let _apartment = match Apartment::new() {
        Ok(apartment) => apartment,
        Err(error) => return format!("{report}Falha ao iniciar WinRT: {error}\n"),
    };
    let mut reader = Reader::new();
    match reader.manager() {
        Ok(manager) => {
            report.push_str("API de sessões de mídia: OK\n");
            if let Ok(sessions) = manager.GetSessions() {
                let count = sessions.Size().unwrap_or(0);
                report.push_str(&format!("Sessões encontradas: {count}\n"));
                for index in 0..count {
                    if let Ok(session) = sessions.GetAt(index) {
                        let source = session
                            .SourceAppUserModelId()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default();
                        report.push_str(&format!(
                            "- {source} | Spotify: {}\n",
                            is_spotify_source(&source)
                        ));
                    }
                }
            }
        }
        Err(error) => return format!("{report}Falha ao acessar API de sessões: {error}\n"),
    }
    match reader.refresh() {
        Ok(s) => {
            report.push_str(&format!("Spotify conectado: {}\nFonte: {}\nFaixa: {}\nArtista: {}\nEstado: {:?}\nAnterior: {} | Play/Pause: {} | Próxima: {}\nTimeline disponível: {}\nCapa disponível: {}\n",
                s.connected, s.source, s.title, s.artist, s.playback, s.capabilities.previous, s.capabilities.can_toggle(s.playback), s.capabilities.next, s.timeline.is_some(), s.artwork.is_some()));
            if let Some(timeline) = s.timeline {
                report.push_str(&format!(
                    "Posição: {:.1}s | Duração: {:.1}s\n",
                    timeline.position.as_secs_f64(),
                    timeline.duration.as_secs_f64()
                ));
            }
        }
        Err(error) => report.push_str(&format!("Falha ao ler Spotify: {error}\n")),
    }
    report
}

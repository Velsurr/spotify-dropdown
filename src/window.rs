//! Native placement and a cursor observer that also wakes a completely hidden window.
use crate::dropdown::{Hit, Slide, hit_test};
use crate::preferences::{
    Alignment, Bounds, EdgeAction, EdgeWatcher, MIN_HEIGHT, MIN_WIDTH, Mode, Preferences,
};
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    marker::PhantomData,
    rc::Rc,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DwmSetWindowAttribute},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST,
            MONITORINFOEXW, MonitorFromWindow, ScreenToClient,
        },
        UI::{
            HiDpi::{GetDpiForMonitor, GetDpiForWindow, MDT_EFFECTIVE_DPI},
            Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON},
            Shell::{
                DefSubclassProc, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE,
                NIM_SETVERSION, NOTIFYICON_VERSION_4, NOTIFYICONDATAW, RemoveWindowSubclass,
                SetWindowSubclass, Shell_NotifyIconW,
            },
            WindowsAndMessaging::*,
        },
    },
    core::{BOOL, Error, Result, w},
};

const SUBCLASS_ID: usize = 0x53504432;
const WM_EDGE_ACTION: u32 = WM_APP + 0x321;
const WM_TRAY_ACTION: u32 = WM_APP + 0x322;
const TRAY_ID: u32 = 1;

#[derive(Clone, Copy)]
struct ResizeAnchor {
    screen: Bounds,
    alignment: Alignment,
    scale: f32,
    start: Bounds,
}

impl Default for ResizeAnchor {
    fn default() -> Self {
        Self {
            screen: Bounds {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            alignment: Alignment::Center,
            scale: 1.0,
            start: Bounds::default(),
        }
    }
}

fn tray_data(hwnd: HWND, icon: usize) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
        uCallbackMessage: WM_TRAY_ACTION,
        hIcon: HICON(icon as *mut std::ffi::c_void),
        ..Default::default()
    };
    let tip: Vec<u16> = "Spotify Dropdown · dois cliques para abrir"
        .encode_utf16()
        .collect();
    data.szTip[..tip.len()].copy_from_slice(&tip);
    data
}

fn add_tray(hwnd: HWND, icon: usize) -> bool {
    let mut data = tray_data(hwnd, icon);
    // SAFETY: the icon is owned by NativeWindow and retained until the icon is removed.
    if !unsafe { Shell_NotifyIconW(NIM_ADD, &data) }.as_bool() {
        return false;
    }
    data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
    let _ = unsafe { Shell_NotifyIconW(NIM_SETVERSION, &data) };
    true
}

#[derive(Clone)]
pub struct Monitor {
    pub device: String,
    pub bounds: Bounds,
    pub work: Bounds,
    pub scale: f32,
}

impl Monitor {
    pub fn label(&self, index: usize) -> String {
        format!(
            "Tela {} · {} × {}",
            index + 1,
            self.bounds.width,
            self.bounds.height
        )
    }
}

fn bounds(rect: RECT) -> Bounds {
    Bounds {
        x: rect.left,
        y: rect.top,
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    }
}

fn monitor_info(handle: HMONITOR) -> Option<Monitor> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    // SAFETY: MONITORINFO is the initial field of MONITORINFOEXW, with cbSize set above.
    if !unsafe { GetMonitorInfoW(handle, &mut info.monitorInfo) }.as_bool() {
        return None;
    }
    let mut x = 96;
    let mut y = 96;
    // SAFETY: valid monitor handle and writable scalar outputs.
    let _ = unsafe { GetDpiForMonitor(handle, MDT_EFFECTIVE_DPI, &mut x, &mut y) };
    let length = info
        .szDevice
        .iter()
        .position(|&v| v == 0)
        .unwrap_or(info.szDevice.len());
    Some(Monitor {
        device: String::from_utf16_lossy(&info.szDevice[..length]),
        bounds: bounds(info.monitorInfo.rcMonitor),
        work: bounds(info.monitorInfo.rcWork),
        scale: x.max(96) as f32 / 96.0,
    })
}

unsafe extern "system" fn enumerate_monitor(
    handle: HMONITOR,
    _: HDC,
    _: *mut RECT,
    data: LPARAM,
) -> BOOL {
    if let Some(monitor) = monitor_info(handle) {
        // SAFETY: EnumDisplayMonitors invokes this synchronously with our live Vec pointer.
        unsafe {
            (&mut *(data.0 as *mut Vec<Monitor>)).push(monitor);
        }
    }
    true.into()
}

fn monitors() -> Vec<Monitor> {
    let mut result = Vec::<Monitor>::new();
    // SAFETY: callback and reference remain valid until enumeration returns.
    let _ = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(enumerate_monitor),
            LPARAM(&mut result as *mut _ as isize),
        )
    };
    result.sort_by_key(|monitor| (monitor.bounds.x, monitor.bounds.y));
    result
}

struct HookData {
    ctx: egui::Context,
    watch: Mutex<EdgeWatcher>,
    resize_anchor: Mutex<ResizeAnchor>,
    dismiss_menu: AtomicBool,
    explicit_hidden: AtomicBool,
    tray_icon: usize,
    taskbar_created: u32,
    action: AtomicU8,
    interacting: AtomicBool,
    moved: AtomicBool,
    resized: AtomicBool,
    activated: AtomicBool,
    displays_changed: AtomicBool,
    menu_open: AtomicBool,
    collapsed: AtomicBool,
    dropdown: AtomicBool,
    animating: AtomicBool,
    alive: AtomicBool,
    stop: AtomicBool,
    wake: Condvar,
}

fn observe_cursor(hwnd_value: usize, state: Arc<HookData>) {
    // HWND is only used for read-only queries and PostMessage on this thread.
    // Window mutations remain on the window's UI thread in window_hook/update.
    let hwnd = HWND(hwnd_value as *mut std::ffi::c_void);
    let mut foreground = 0usize;
    let mut last_raise = Instant::now();
    let mut was_pressed = false;
    loop {
        let watcher = state.watch.lock().unwrap_or_else(|e| e.into_inner());
        let menu_open = state.menu_open.load(Ordering::Relaxed);
        let interval = if watcher.enabled || menu_open {
            Duration::from_millis(30)
        } else {
            Duration::from_secs(1)
        };
        let (mut watcher, _) = state
            .wake
            .wait_timeout(watcher, interval)
            .unwrap_or_else(|e| e.into_inner());
        if state.stop.load(Ordering::Acquire) || !state.alive.load(Ordering::Acquire) {
            break;
        }
        if (!watcher.enabled && !menu_open)
            || state.explicit_hidden.load(Ordering::Acquire)
            || unsafe { IsIconic(hwnd) }.as_bool()
        {
            continue;
        }
        let mut point = POINT::default();
        if unsafe { GetCursorPos(&mut point) }.is_err() {
            continue;
        }
        let now = Instant::now();
        let active = unsafe { GetForegroundWindow() }.0 as usize;
        let changed = active != foreground;
        foreground = active;
        let pressed = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0;
        let any_pressed = pressed || unsafe { GetAsyncKeyState(VK_RBUTTON.0 as i32) } < 0;
        let inside = watcher.bounds.contains([point.x, point.y]);
        if menu_open && !inside && any_pressed && !was_pressed {
            state.dismiss_menu.store(true, Ordering::Release);
            state.ctx.request_repaint();
        }
        was_pressed = any_pressed;
        // An integral settings screen or volume panel only holds the dropdown while
        // the pointer is still inside. Leaving it restarts the usual hide delay.
        watcher.hold_open = menu_open && inside;
        let action = watcher.poll(
            [point.x, point.y],
            pressed || state.interacting.load(Ordering::Relaxed),
            now,
        );
        let raise = watcher.enabled
            && watcher.expanded
            && (changed
                || (watcher.bounds.contains([point.x, point.y])
                    && now.duration_since(last_raise) >= Duration::from_millis(400)));
        let code = match action {
            Some(EdgeAction::Reveal) => 1,
            Some(EdgeAction::Collapse) => 2,
            None if raise => {
                last_raise = now;
                3
            }
            None => continue,
        };
        drop(watcher);
        if state
            .action
            .compare_exchange(0, code, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            // A native message wakes winit even when its window is hidden and has no redraws.
            if unsafe { PostMessageW(Some(hwnd), WM_EDGE_ACTION, WPARAM(0), LPARAM(0)) }.is_err() {
                state.action.store(0, Ordering::Release);
            }
        }
    }
}

fn screen_point(value: LPARAM) -> POINT {
    POINT {
        x: value.0 as i16 as i32,
        y: (value.0 >> 16) as i16 as i32,
    }
}

unsafe extern "system" fn window_hook(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    data: usize,
) -> LRESULT {
    // SAFETY: NativeWindow owns this stable Arc and unregisters before releasing it.
    let state = unsafe { &*(data as *const HookData) };
    if message == state.taskbar_created && message != 0 {
        // Explorer can restart independently; re-register the same icon and HWND.
        let _ = add_tray(hwnd, state.tray_icon);
        return LRESULT(0);
    }
    match message {
        WM_NCHITTEST if !state.collapsed.load(Ordering::Relaxed) => {
            if state.animating.load(Ordering::Relaxed) {
                return LRESULT(HTCLIENT as isize);
            }
            let mut point = screen_point(lparam);
            let mut client = RECT::default();
            // SAFETY: the callback's HWND is live; both outputs point to stack values.
            unsafe {
                let _ = ScreenToClient(hwnd, &mut point);
                let _ = GetClientRect(hwnd, &mut client);
            }
            let scale = unsafe { GetDpiForWindow(hwnd) }.max(96) as f32 / 96.0;
            let x = point.x as f32 / scale;
            let y = point.y as f32 / scale;
            let width = client.right as f32 / scale;
            let height = client.bottom as f32 / scale;
            let hit = match hit_test(x, y, width, height, state.dropdown.load(Ordering::Relaxed)) {
                Hit::Client => HTCLIENT,
                Hit::Caption => HTCAPTION,
                Hit::Left => HTLEFT,
                Hit::Right => HTRIGHT,
                Hit::Top => HTTOP,
                Hit::Bottom => HTBOTTOM,
                Hit::TopLeft => HTTOPLEFT,
                Hit::TopRight => HTTOPRIGHT,
                Hit::BottomLeft => HTBOTTOMLEFT,
                Hit::BottomRight => HTBOTTOMRIGHT,
            };
            return LRESULT(hit as isize);
        }
        WM_NCRBUTTONUP => return LRESULT(0),
        WM_NCLBUTTONDBLCLK if state.dropdown.load(Ordering::Relaxed) => return LRESULT(0),
        WM_ENTERSIZEMOVE => {
            state.interacting.store(true, Ordering::Relaxed);
            let mut rect = RECT::default();
            if unsafe { GetWindowRect(hwnd, &mut rect) }.is_ok() {
                state
                    .resize_anchor
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .start = bounds(rect);
            }
        }
        WM_MOVING => {
            state.moved.store(true, Ordering::Relaxed);
        }
        WM_SIZING => {
            state.resized.store(true, Ordering::Relaxed);
            if state.dropdown.load(Ordering::Relaxed) && lparam.0 != 0 {
                let anchor = *state
                    .resize_anchor
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                // SAFETY: WM_SIZING supplies a live writable RECT. Returning TRUE lets
                // Windows apply this anchored rectangle throughout its native size loop.
                let proposed = unsafe { &mut *(lparam.0 as *mut RECT) };
                let delta = if wparam.0 == WMSZ_BOTTOMLEFT as usize {
                    anchor.start.x - proposed.left
                } else {
                    proposed.right - (anchor.start.x + anchor.start.width)
                };
                let width = if anchor.alignment == Alignment::Center {
                    anchor.start.width.saturating_add(delta.saturating_mul(2))
                } else {
                    proposed.right - proposed.left
                };
                let min_width = (MIN_WIDTH * anchor.scale).round() as i32;
                let max_width = ((1400.0 * anchor.scale).round() as i32)
                    .min(anchor.screen.width)
                    .max(1);
                let min_height = (MIN_HEIGHT * anchor.scale).round() as i32;
                let max_height = ((900.0 * anchor.scale).round() as i32)
                    .min(anchor.screen.height)
                    .max(1);
                let width = width.clamp(min_width.min(max_width), max_width);
                let height = (proposed.bottom - anchor.screen.y)
                    .clamp(min_height.min(max_height), max_height);
                let rect =
                    Bounds::anchored(anchor.screen, [width, height], anchor.alignment, false);
                *proposed = RECT {
                    left: rect.x,
                    top: rect.y,
                    right: rect.x + rect.width,
                    bottom: rect.y + rect.height,
                };
                state.ctx.request_repaint();
                return LRESULT(1);
            }
        }
        WM_EXITSIZEMOVE => {
            state.interacting.store(false, Ordering::Relaxed);
            state.ctx.request_repaint();
        }
        WM_ACTIVATE => {
            if wparam.0 & 0xffff != WA_INACTIVE as usize {
                state.activated.store(true, Ordering::Relaxed);
            } else if state.menu_open.load(Ordering::Relaxed) {
                state.dismiss_menu.store(true, Ordering::Release);
            }
            state.ctx.request_repaint();
        }
        WM_TRAY_ACTION => {
            if lparam.0 as u32 & 0xffff == WM_LBUTTONDBLCLK {
                state.explicit_hidden.store(false, Ordering::Release);
                state.action.store(4, Ordering::Release);
                unsafe {
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                    let _ = SetForegroundWindow(hwnd);
                }
                state.ctx.request_repaint();
            }
            return LRESULT(0);
        }
        WM_DISPLAYCHANGE | WM_DPICHANGED => {
            state.displays_changed.store(true, Ordering::Relaxed);
            state.ctx.request_repaint();
        }
        WM_EDGE_ACTION => {
            if state.action.load(Ordering::Acquire) == 1
                && !state.explicit_hidden.load(Ordering::Acquire)
            {
                // Showing without activation preserves the foreground application's keyboard focus.
                unsafe {
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                }
            }
            state.ctx.request_repaint();
            return LRESULT(0);
        }
        WM_NCDESTROY => {
            state.alive.store(false, Ordering::Relaxed);
            state.wake.notify_all();
            unsafe {
                let _ = Shell_NotifyIconW(NIM_DELETE, &tray_data(hwnd, state.tray_icon));
                let _ = RemoveWindowSubclass(hwnd, Some(window_hook), SUBCLASS_ID);
            }
        }
        _ => {}
    }
    // SAFETY: preserve all normal winit processing for messages we do not consume.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

struct NativeWindow {
    hwnd: HWND,
    data: Arc<HookData>,
    observer: Option<JoinHandle<()>>,
    topmost: Option<bool>,
    visible: bool,
    tray_icon: HICON,
    _ui_thread: PhantomData<Rc<()>>,
}

impl NativeWindow {
    fn new(
        cc: &eframe::CreationContext<'_>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let handle = cc.window_handle()?.as_raw();
        let RawWindowHandle::Win32(handle) = handle else {
            return Err("Esta versão requer Windows.".into());
        };
        let hwnd = HWND(handle.hwnd.get() as *mut std::ffi::c_void);
        // Reuse the existing application icon; the copied handle belongs to us.
        let mut icon_value = unsafe {
            SendMessageW(
                hwnd,
                WM_GETICON,
                Some(WPARAM(ICON_SMALL as usize)),
                Some(LPARAM(0)),
            )
        }
        .0 as usize;
        if icon_value == 0 {
            icon_value = unsafe { GetClassLongPtrW(hwnd, GCLP_HICONSM) };
        }
        if icon_value == 0 {
            icon_value = unsafe {
                SendMessageW(
                    hwnd,
                    WM_GETICON,
                    Some(WPARAM(ICON_BIG as usize)),
                    Some(LPARAM(0)),
                )
            }
            .0 as usize;
        }
        let tray_icon = if icon_value != 0 {
            unsafe { CopyIcon(HICON(icon_value as *mut std::ffi::c_void))? }
        } else {
            unsafe { CopyIcon(LoadIconW(None, IDI_APPLICATION)?)? }
        };
        let data = Arc::new(HookData {
            ctx: cc.egui_ctx.clone(),
            watch: Mutex::new(EdgeWatcher::default()),
            resize_anchor: Mutex::new(ResizeAnchor::default()),
            dismiss_menu: AtomicBool::new(false),
            explicit_hidden: AtomicBool::new(false),
            tray_icon: tray_icon.0 as usize,
            taskbar_created: unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) },
            action: AtomicU8::new(0),
            interacting: AtomicBool::new(false),
            moved: AtomicBool::new(false),
            resized: AtomicBool::new(false),
            activated: AtomicBool::new(false),
            displays_changed: AtomicBool::new(false),
            menu_open: AtomicBool::new(false),
            collapsed: AtomicBool::new(false),
            dropdown: AtomicBool::new(false),
            animating: AtomicBool::new(false),
            alive: AtomicBool::new(true),
            stop: AtomicBool::new(false),
            wake: Condvar::new(),
        });
        // SAFETY: install on the window's own UI thread, retaining a stable allocation until Drop.
        if !unsafe {
            SetWindowSubclass(
                hwnd,
                Some(window_hook),
                SUBCLASS_ID,
                &*data as *const _ as usize,
            )
        }
        .as_bool()
        {
            unsafe {
                let _ = DestroyIcon(tray_icon);
            }
            return Err("Não foi possível habilitar o arraste nativo da janela.".into());
        }
        let mut native = Self {
            hwnd,
            data,
            observer: None,
            topmost: None,
            visible: true,
            tray_icon,
            _ui_thread: PhantomData,
        };
        // The root is a tool window for its entire lifetime. There is no helper
        // HWND and reveal/hide never toggles taskbar or Alt+Tab eligibility.
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                (style & !(WS_EX_APPWINDOW.0 as isize)) | WS_EX_TOOLWINDOW.0 as isize,
            );
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER,
            )?;
        }
        if !add_tray(hwnd, tray_icon.0 as usize) {
            return Err("Não foi possível criar o ícone na bandeja do Windows. O aplicativo não será ocultado.".into());
        }
        // egui paints its own corners: suppress a second Windows 11 corner mask.
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &DWMWCP_DONOTROUND as *const _ as *const _,
                std::mem::size_of_val(&DWMWCP_DONOTROUND) as u32,
            );
        }
        let state = Arc::clone(&native.data);
        let hwnd_value = hwnd.0 as usize;
        native.observer = Some(
            thread::Builder::new()
                .name("dropdown-edge".into())
                .spawn(move || observe_cursor(hwnd_value, state))?,
        );
        Ok(native)
    }

    fn rect(&self) -> Result<Bounds> {
        let mut rect = RECT::default();
        unsafe {
            GetWindowRect(self.hwnd, &mut rect)?;
        }
        Ok(bounds(rect))
    }

    fn scale(&self) -> f32 {
        unsafe { GetDpiForWindow(self.hwnd) }.max(96) as f32 / 96.0
    }

    fn place(&mut self, rect: Bounds, topmost: bool, force_raise: bool) -> Result<()> {
        if self.rect()? != rect || self.topmost != Some(topmost) || force_raise {
            // SAFETY: only reposition this app's HWND; showing the dropdown never steals focus.
            unsafe {
                SetWindowPos(
                    self.hwnd,
                    Some(if topmost {
                        HWND_TOPMOST
                    } else {
                        HWND_NOTOPMOST
                    }),
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    SWP_NOACTIVATE | SWP_NOOWNERZORDER,
                )?;
            }
            self.topmost = Some(topmost);
        }
        Ok(())
    }

    fn show(&mut self, visible: bool) {
        if self.visible != visible {
            unsafe {
                let _ = ShowWindow(self.hwnd, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE });
            }
            self.visible = visible;
        }
    }
}

impl Drop for NativeWindow {
    fn drop(&mut self) {
        self.data.stop.store(true, Ordering::Release);
        self.data.wake.notify_all();
        if let Some(observer) = self.observer.take() {
            let _ = observer.join();
        }
        if self.data.alive.load(Ordering::Relaxed) {
            // SAFETY: same UI thread, unregister before HookData is freed.
            unsafe {
                let _ =
                    Shell_NotifyIconW(NIM_DELETE, &tray_data(self.hwnd, self.tray_icon.0 as usize));
                let _ = RemoveWindowSubclass(self.hwnd, Some(window_hook), SUBCLASS_ID);
            }
        }
        unsafe {
            let _ = DestroyIcon(self.tray_icon);
        }
    }
}

pub struct WindowManager {
    pub preferences: Preferences,
    pub monitors: Vec<Monitor>,
    pub error: Option<String>,
    native: NativeWindow,
    collapsed: bool,
    place_pending: bool,
    force_raise: bool,
    slide: Slide,
    scene_height: f32,
}

impl WindowManager {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        preferences: Preferences,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            preferences,
            monitors: monitors(),
            error: None,
            native: NativeWindow::new(cc)?,
            collapsed: false,
            place_pending: true,
            force_raise: true,
            slide: Slide::open(),
            scene_height: 340.0,
        })
    }

    pub fn collapsed(&self) -> bool {
        self.native.data.explicit_hidden.load(Ordering::Acquire)
            || (self.collapsed && !self.slide.moving(Instant::now()))
    }

    pub fn scene_height(&self, visible_height: f32) -> f32 {
        if self.preferences.mode == Mode::Dropdown
            && !self.native.data.interacting.load(Ordering::Relaxed)
        {
            self.scene_height
        } else {
            visible_height
        }
    }

    pub fn set_hold_open(&self, hold: bool) {
        let changed = self.native.data.menu_open.swap(hold, Ordering::AcqRel) != hold;
        self.native
            .data
            .watch
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .hold_open = hold;
        if changed {
            self.native.data.wake.notify_all();
        }
    }

    pub fn take_dismiss_menu(&self) -> bool {
        self.native.data.dismiss_menu.swap(false, Ordering::AcqRel)
    }

    /// Explicitly hidden panels are restored only by a tray double-click, never
    /// by a cursor touching the edge. This is separate from automatic collapse.
    pub fn hide(&mut self) {
        self.native
            .data
            .explicit_hidden
            .store(true, Ordering::Release);
        self.native.data.action.store(0, Ordering::Release);
        self.set_hold_open(false);
        self.native
            .data
            .watch
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .enabled = false;
        self.native.show(false);
        self.native.data.wake.notify_all();
    }

    fn current_monitor(&self) -> Option<Monitor> {
        monitor_info(unsafe { MonitorFromWindow(self.native.hwnd, MONITOR_DEFAULTTONEAREST) })
    }

    fn selected_monitor(&self) -> Option<Monitor> {
        self.monitors
            .iter()
            .find(|monitor| monitor.device == self.preferences.monitor)
            .cloned()
            .or_else(|| self.current_monitor())
    }

    fn save(&mut self) {
        self.error = self
            .preferences
            .save()
            .err()
            .map(|error| format!("Não foi possível salvar as preferências: {error}"));
    }

    pub fn apply(&mut self, mut preferences: Preferences) {
        if preferences == self.preferences {
            return;
        }
        if preferences.mode != self.preferences.mode {
            if self.preferences.mode == Mode::Free
                && let Ok(rect) = self.native.rect()
            {
                preferences.free_position = Some([rect.x, rect.y]);
                preferences.free_size = [
                    rect.width as f32 / self.native.scale(),
                    rect.height as f32 / self.native.scale(),
                ];
            }
            if preferences.mode == Mode::Dropdown && preferences.monitor.is_empty() {
                preferences.monitor = self
                    .current_monitor()
                    .map(|monitor| monitor.device)
                    .unwrap_or_default();
            }
        }
        self.preferences = preferences;
        self.collapsed = false;
        self.slide = Slide::open();
        self.place_pending = true;
        self.force_raise = true;
        self.native
            .data
            .watch
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .reset();
        self.save();
        self.native.data.ctx.request_repaint();
    }

    pub fn reveal(&mut self) {
        self.slide.target(true, Instant::now());
        self.collapsed = false;
        self.place_pending = true;
        self.force_raise = true;
        self.native.data.ctx.request_repaint();
    }

    pub fn collapse(&mut self) {
        if self.preferences.mode == Mode::Dropdown {
            self.slide.target(false, Instant::now());
            self.collapsed = true;
            self.place_pending = true;
            self.native.data.ctx.request_repaint();
        }
    }

    pub fn update(&mut self, hold_open: bool) {
        if let Err(error) = self.update_inner(hold_open) {
            self.error = Some(format!("Falha ao ajustar a janela: {error}"));
        }
    }

    fn update_inner(&mut self, hold_open: bool) -> Result<()> {
        if self
            .native
            .data
            .displays_changed
            .swap(false, Ordering::Relaxed)
        {
            self.monitors = monitors();
            self.place_pending = true;
        }
        if !self.native.data.interacting.load(Ordering::Relaxed)
            && !self.collapsed
            && !self.slide.moving(Instant::now())
        {
            let moved = self.native.data.moved.swap(false, Ordering::Relaxed);
            let resized = self.native.data.resized.swap(false, Ordering::Relaxed);
            if moved || resized {
                let rect = self.native.rect()?;
                if moved {
                    self.preferences.mode = Mode::Free;
                    self.slide = Slide::open();
                }
                if self.preferences.mode == Mode::Free {
                    self.preferences.free_position = Some([rect.x, rect.y]);
                }
                self.preferences.set_size([
                    rect.width as f32 / self.native.scale(),
                    rect.height as f32 / self.native.scale(),
                ]);
                self.place_pending = true;
                self.save();
            }
        }
        if self.native.data.activated.swap(false, Ordering::Relaxed) && self.collapsed {
            self.reveal();
        }
        match self.native.data.action.swap(0, Ordering::Relaxed) {
            1 => self.reveal(),
            2 => {
                self.native.data.dismiss_menu.store(true, Ordering::Release);
                self.set_hold_open(false);
                self.collapse();
            }
            3 => self.force_raise = true,
            4 => {
                self.native.data.dismiss_menu.store(true, Ordering::Release);
                self.reveal();
            }
            _ => {}
        }
        if self.native.data.explicit_hidden.load(Ordering::Acquire) {
            self.set_hold_open(false);
            self.native
                .data
                .watch
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .enabled = false;
            return Ok(());
        }
        let now = Instant::now();
        let animating = self.slide.moving(now);
        let fraction = self.slide.value(now);
        let dropdown = self.preferences.mode == Mode::Dropdown;
        if dropdown
            && !self.native.data.interacting.load(Ordering::Relaxed)
            && let Some(monitor) = self.selected_monitor()
        {
            let mut anchor = self
                .native
                .data
                .resize_anchor
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            anchor.screen = monitor.bounds;
            anchor.alignment = self.preferences.alignment;
            anchor.scale = monitor.scale;
        }
        self.native.data.dropdown.store(dropdown, Ordering::Relaxed);
        self.native
            .data
            .animating
            .store(animating, Ordering::Relaxed);
        if (self.place_pending || animating || self.force_raise)
            && !self.native.data.interacting.load(Ordering::Relaxed)
        {
            let current = self.native.rect()?;
            let rect = match self.preferences.mode {
                Mode::Dropdown => {
                    let monitor = self.selected_monitor().ok_or_else(|| {
                        Error::new(
                            windows::core::HRESULT(0x80004005u32 as i32),
                            "Nenhum monitor disponível.",
                        )
                    })?;
                    let size = self
                        .preferences
                        .size()
                        .map(|value| (value * monitor.scale).round() as i32);
                    let mut full =
                        Bounds::anchored(monitor.bounds, size, self.preferences.alignment, false);
                    self.scene_height = full.height as f32 / monitor.scale;
                    full.height = ((full.height as f32 * fraction).round() as i32).max(1);
                    full
                }
                Mode::Free => {
                    let work = self
                        .monitors
                        .iter()
                        .find(|monitor| {
                            self.preferences
                                .free_position
                                .is_some_and(|point| monitor.bounds.contains(point))
                        })
                        .cloned()
                        .or_else(|| self.current_monitor())
                        .map(|monitor| monitor.work)
                        .unwrap_or(Bounds {
                            x: 0,
                            y: 0,
                            width: 1920,
                            height: 1080,
                        });
                    let [width, height] = self
                        .preferences
                        .free_size
                        .map(|value| (value * self.native.scale()).round() as i32);
                    let [x, y] = self
                        .preferences
                        .free_position
                        .unwrap_or([current.x, current.y]);
                    Bounds {
                        x,
                        y,
                        width,
                        height,
                    }
                    .fit_inside(work)
                }
            };
            self.native
                .data
                .collapsed
                .store(self.collapsed, Ordering::Relaxed);
            self.native.place(
                rect,
                self.preferences.mode == Mode::Dropdown || self.preferences.always_on_top,
                self.force_raise,
            )?;
            self.native.show(!dropdown || animating || !self.collapsed);
            // Keep one final frame to place the exact endpoint and hide at zero.
            self.place_pending = animating;
            self.force_raise = false;
        }
        if animating {
            self.native
                .data
                .ctx
                .request_repaint_after(Duration::from_millis(16));
        }
        self.set_hold_open(hold_open);
        let rect = if dropdown && self.collapsed && !animating {
            let monitor = self.selected_monitor().ok_or_else(|| {
                Error::new(
                    windows::core::HRESULT(0x80004005u32 as i32),
                    "Nenhum monitor disponível.",
                )
            })?;
            Bounds::anchored(
                monitor.bounds,
                self.preferences
                    .size()
                    .map(|v| (v * monitor.scale).round() as i32),
                self.preferences.alignment,
                true,
            )
        } else {
            self.native.rect()?
        };
        let mut watch = self
            .native
            .data
            .watch
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if watch.bounds != rect || watch.expanded == self.collapsed {
            watch.reset();
        }
        watch.enabled = dropdown;
        watch.expanded = !self.collapsed;
        watch.auto_hide = self.preferences.auto_hide;
        watch.bounds = rect;
        watch.hide_delay = Duration::from_millis(self.preferences.hide_delay_ms);
        drop(watch);
        self.native.data.wake.notify_all();
        Ok(())
    }
}

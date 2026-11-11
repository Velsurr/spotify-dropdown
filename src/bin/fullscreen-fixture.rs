//! Plain Win32 QA surface: a separate opaque, topmost, borderless full-screen window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
    core::{Result, w},
};

unsafe extern "system" fn procedure(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_KEYDOWN if wparam.0 == 27 => {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            unsafe {
                let dc = BeginPaint(hwnd, &mut paint);
                let _ = SetBkMode(dc, TRANSPARENT);
                let _ = SetTextColor(dc, COLORREF(0x00F0F4EF));
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                rect.top = rect.bottom / 2 - 40;
                let mut message: Vec<u16> = "Teste local: outro aplicativo em tela cheia sem bordas\nEncoste o cursor no topo para abrir o dropdown.\nEsc fecha esta janela de teste. Tela cheia exclusiva não é simulada.".encode_utf16().collect();
                DrawTextW(dc, &mut message, &mut rect, DT_CENTER | DT_TOP);
                let _ = EndPaint(hwnd, &paint);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn main() -> Result<()> {
    // All HWND/GDI objects below belong to this short-lived, separate QA application.
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let brush = CreateSolidBrush(COLORREF(0x002F211B));
        let class = w!("SpotifyDropdownFullscreenFixture");
        let definition = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            hbrBackground: brush,
            lpszClassName: class,
            ..Default::default()
        };
        if RegisterClassW(&definition) == 0 {
            return Err(windows::core::Error::from_thread());
        }
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_APPWINDOW,
            class,
            w!("Spotify Dropdown — teste de tela cheia"),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
            None,
            None,
            Some(instance.into()),
            None,
        )?;
        let _ = ShowWindow(hwnd, SW_SHOW);
        let mut message = MSG::default();
        loop {
            let result = GetMessageW(&mut message, None, 0, 0).0;
            if result <= 0 {
                break;
            }
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        let _ = UnregisterClassW(class, Some(instance.into()));
        let _ = DeleteObject(brush.into());
    }
    Ok(())
}

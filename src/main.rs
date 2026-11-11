#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dropdown;
mod layout;
mod media;
mod model;
mod player;
mod preferences;
mod ui;
mod window;

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--diagnostico") {
        let path = std::env::args()
            .nth(2)
            .unwrap_or_else(|| "artifacts/diagnostico.txt".into());
        let report = media::diagnostic_report();
        if let Err(error) = std::fs::write(&path, report) {
            show_error(&format!("Não foi possível salvar o diagnóstico: {error}"));
            std::process::exit(1);
        }
        return;
    }
    let preferences = preferences::Preferences::load();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(format!("Spotify Dropdown · v{}", env!("CARGO_PKG_VERSION")))
            .with_inner_size(preferences.size())
            .with_min_inner_size([preferences::MIN_WIDTH, preferences::MIN_HEIGHT])
            .with_max_inner_size([1400.0, 900.0])
            .with_resizable(true)
            .with_decorations(false)
            .with_transparent(true)
            .with_icon(ui::app_icon()),
        renderer: eframe::Renderer::Glow,
        centered: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        ..Default::default()
    };
    if let Err(error) = eframe::run_native(
        "spotify-dropdown",
        options,
        Box::new(move |cc| Ok(Box::new(player::SpotifyDropdown::new(cc, preferences)?))),
    ) {
        show_error(&format!("Não foi possível abrir a janela:\n{error}"));
        std::process::exit(1);
    }
}

fn show_error(message: &str) {
    use windows::{
        Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW},
        core::{HSTRING, PCWSTR, w},
    };
    let message = HSTRING::from(message);
    // SAFETY: the UTF-16 strings are valid for the duration of this synchronous call.
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(message.as_ptr()),
            w!("Spotify Dropdown"),
            MB_OK | MB_ICONERROR,
        );
    }
}

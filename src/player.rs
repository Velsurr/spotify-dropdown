use crate::{
    layout::PlayerLayout,
    media::{Artwork, Backend, Command, Snapshot},
    model::{Playback, format_time},
    preferences::{Alignment, Mode, Preferences},
    ui::{BORDER, DIM, GREEN, Icon, MUTED, TEXT, draw_icon, icon_button, text, text_block},
    window::WindowManager,
};
use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, TextureHandle, pos2, vec2,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuTab {
    Settings,
    About,
}

pub struct SpotifyDropdown {
    backend: Backend,
    window: WindowManager,
    artwork: Option<Arc<Artwork>>,
    texture: Option<TextureHandle>,
    menu_open: bool,
    menu_position: Pos2,
    menu_tab: MenuTab,
    size_draft: [f32; 2],
    delay_draft: f32,
}

impl SpotifyDropdown {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        preferences: Preferences,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.window_fill = Color32::from_rgb(20, 27, 23);
        visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
        visuals.selection.bg_fill = GREEN.gamma_multiply(0.25);
        visuals.selection.stroke = Stroke::new(1.0_f32, GREEN);
        visuals.window_corner_radius = 12.into();
        cc.egui_ctx.set_visuals(visuals);
        cc.egui_ctx.style_mut(|style| {
            style.animation_time = 0.08;
            style.spacing.item_spacing = vec2(8.0, 8.0);
        });
        let size_draft = preferences.size();
        let delay_draft = preferences.hide_delay_ms as f32 / 1000.0;
        Ok(Self {
            backend: Backend::start(cc.egui_ctx.clone()),
            window: WindowManager::new(cc, preferences)?,
            artwork: None,
            texture: None,
            menu_open: false,
            menu_position: pos2(80.0, 50.0),
            menu_tab: MenuTab::Settings,
            size_draft,
            delay_draft,
        })
    }

    fn update_artwork(&mut self, artwork: &Option<Arc<Artwork>>, ctx: &egui::Context) {
        let unchanged = match (&self.artwork, artwork) {
            (Some(old), Some(new)) => Arc::ptr_eq(old, new),
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }
        self.artwork = artwork.clone();
        self.texture = artwork.as_ref().map(|artwork| {
            ctx.load_texture(
                "album-cover",
                egui::ColorImage::from_rgba_unmultiplied(artwork.size, &artwork.rgba),
                egui::TextureOptions::LINEAR,
            )
        });
    }

    fn open_menu(&mut self, position: Pos2) {
        self.menu_open = true;
        self.menu_position = position;
        self.size_draft = self.window.preferences.size();
        self.delay_draft = self.window.preferences.hide_delay_ms as f32 / 1000.0;
        self.window.set_hold_open(true);
    }

    fn menu(&mut self, ctx: &egui::Context) {
        let mut open = self.menu_open;
        let mut preferences = self.window.preferences.clone();
        let old_mode = preferences.mode;
        let mut collapse = false;
        let available_height = (ctx.content_rect().height() - 100.0).max(160.0);
        egui::Popup::new(egui::Id::new("settings-menu"), ctx.clone(), self.menu_position, egui::LayerId::new(egui::Order::Foreground, egui::Id::new("settings-anchor")))
            .kind(egui::PopupKind::Menu)
            .frame(egui::Frame::popup(&ctx.style()).fill(Color32::from_rgb(20, 27, 23)))
            .open_bool(&mut open)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .width(300.0)
            .show(|ui| {
                ui.set_width(280.0);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.menu_tab, MenuTab::Settings, "Ajustes");
                    ui.selectable_value(&mut self.menu_tab, MenuTab::About, "Sobre");
                });
                ui.separator();
                egui::ScrollArea::vertical().max_height(available_height).show(ui, |ui| {
                    match self.menu_tab {
                        MenuTab::About => {
                            ui.heading("Spotify Dropdown");
                            ui.label(egui::RichText::new(format!("Versão {}", env!("CARGO_PKG_VERSION"))).color(GREEN));
                            ui.add_space(6.0);
                            ui.label("Um controle discreto para o Spotify Desktop, feito em Rust com egui.");
                            ui.label("Reprodução e metadados pela sessão de mídia do Windows, sem OAuth.");
                            ui.separator();
                            ui.label("Favorito, volume e playlists ainda estão indisponíveis.");
                            ui.small("Não afiliado ao Spotify. Preferências salvas em settings.ini, ao lado do executável.");
                        },
                        MenuTab::Settings => {
                            ui.strong("Modo de exibição");
                            let mut dropdown = preferences.mode == Mode::Dropdown;
                            if ui.checkbox(&mut dropdown, "Ativar modo dropdown").changed() {
                                preferences.mode = if dropdown { Mode::Dropdown } else { Mode::Free };
                            }
                            ui.small("Arraste o cabeçalho para mover. Arrastar o dropdown transforma-o em janela livre.");
                            ui.separator();
                            ui.strong("Dropdown no topo");
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut preferences.alignment, Alignment::Left, "Esquerda");
                                ui.radio_value(&mut preferences.alignment, Alignment::Center, "Centro");
                                ui.radio_value(&mut preferences.alignment, Alignment::Right, "Direita");
                            });
                            let label = self.window.monitors.iter().enumerate().find(|(_, monitor)| monitor.device == preferences.monitor).map(|(index, monitor)| monitor.label(index)).unwrap_or_else(|| "Monitor atual".into());
                            egui::ComboBox::from_id_salt("monitor").width(252.0).selected_text(label).show_ui(ui, |ui| {
                                for (index, monitor) in self.window.monitors.iter().enumerate() {
                                    ui.selectable_value(&mut preferences.monitor, monitor.device.clone(), monitor.label(index));
                                }
                            });
                            ui.checkbox(&mut preferences.auto_hide, "Recolher quando o cursor sair");
                            let delay = ui.add_enabled(preferences.auto_hide, egui::Slider::new(&mut self.delay_draft, 0.1..=5.0).step_by(0.1).suffix(" s").text("Recolher após"));
                            if delay.drag_stopped() || (delay.changed() && !delay.dragged()) {
                                preferences.hide_delay_ms = (self.delay_draft * 1000.0).round() as u64;
                            }
                            ui.add_enabled_ui(preferences.mode == Mode::Free, |ui| { ui.checkbox(&mut preferences.always_on_top, "Manter janela sempre no topo"); });
                            ui.separator();
                            ui.strong("Tamanho da janela");
                            ui.add(egui::Slider::new(&mut self.size_draft[0], 360.0..=1400.0).integer().text("Largura"));
                            ui.add(egui::Slider::new(&mut self.size_draft[1], 340.0..=900.0).integer().text("Altura"));
                            if ui.button("Aplicar tamanho").clicked() { preferences.set_size(self.size_draft); }
                            ui.horizontal(|ui| {
                                for (name, size) in [("Vertical", [430.0, 584.0]), ("Largo", [760.0, 340.0]), ("Enxuto", [360.0, 420.0])] {
                                    if ui.button(name).clicked() { self.size_draft = size; preferences.set_size(size); }
                                }
                            });
                            ui.small(if preferences.mode == Mode::Dropdown { "Redimensione pelo canto inferior direito. Cada modo guarda seu tamanho." } else { "Redimensione pelas bordas ou cantos. Cada modo guarda seu tamanho." });
                            if preferences.mode == Mode::Dropdown && ui.button("Recolher agora").clicked() { collapse = true; }
                            if let Some(error) = &self.window.error { ui.colored_label(Color32::from_rgb(224, 177, 103), error); }
                        },
                    }
                });
            });
        self.menu_open = open && !collapse && old_mode == preferences.mode;
        self.window.apply(preferences);
        if collapse {
            self.window.collapse();
        }
        self.window.set_hold_open(self.menu_open);
    }

    fn cover(&self, ui: &mut egui::Ui, cover: Rect) {
        let painter = ui.painter().clone();
        painter.rect_filled(cover, 0, Color32::from_rgb(24, 37, 29));
        if let Some(texture) = &self.texture {
            // Preserve the entire original image, including non-square images. No rounded crop.
            let source = texture.size_vec2();
            let image_size = source * (cover.width() / source.x).min(cover.height() / source.y);
            let image_rect = Rect::from_center_size(cover.center(), image_size);
            ui.put(
                image_rect,
                egui::Image::new(texture)
                    .fit_to_exact_size(image_size)
                    .corner_radius(0),
            );
        } else {
            painter.circle_filled(
                cover.center(),
                cover.width() * 0.28,
                Color32::from_rgb(32, 54, 39),
            );
            draw_icon(
                &painter,
                cover.center(),
                Icon::Note,
                Color32::from_rgb(99, 156, 115),
            );
        }
    }

    fn track_labels(
        &self,
        painter: &egui::Painter,
        position: Pos2,
        width: f32,
        snapshot: &Snapshot,
    ) {
        let album = if snapshot.connected {
            if snapshot.album.trim().is_empty() {
                "Álbum não informado"
            } else {
                &snapshot.album
            }
        } else {
            "SEU SPOTIFY, POR PERTO"
        };
        text_block(painter, position, width, 1, album, 11.0, MUTED);
        let title = if !snapshot.connected {
            "Sua música,\npor perto."
        } else if snapshot.title.trim().is_empty() {
            "Faixa sem título"
        } else {
            &snapshot.title
        };
        let mut job = egui::text::LayoutJob::simple(
            title.into(),
            FontId::proportional(if width < 220.0 { 20.0 } else { 22.0 }),
            TEXT,
            width,
        );
        job.wrap.max_rows = 2;
        job.wrap.break_anywhere = true;
        job.wrap.overflow_character = Some('…');
        let galley = painter.layout_job(job);
        let title_position = position + vec2(0.0, 21.0);
        let artist_y = title_position.y + galley.size().y + 5.0;
        painter.galley(title_position, galley, TEXT);
        let artist = if !snapshot.connected {
            "Aguardando o Spotify Desktop"
        } else if snapshot.artist.trim().is_empty() {
            "Artista não informado"
        } else {
            &snapshot.artist
        };
        text_block(
            painter,
            pos2(position.x, artist_y),
            width,
            1,
            artist,
            12.0,
            MUTED,
        );
    }

    fn playback_card(&self, ui: &mut egui::Ui, card: Rect, state: &crate::media::BackendState) {
        let snapshot = &state.snapshot;
        let painter = ui.painter().with_clip_rect(card.intersect(ui.clip_rect()));
        painter.rect_filled(card, 10, Color32::from_rgb(20, 26, 22));
        let color = if snapshot.connected { GREEN } else { MUTED };
        let problem = self.window.error.as_ref().or(snapshot.problem.as_ref());
        let status = if problem.is_some() {
            "INTEGRAÇÃO INDISPONÍVEL"
        } else if state.busy {
            "ENVIANDO…"
        } else if !snapshot.connected {
            "AGUARDANDO SPOTIFY"
        } else {
            match snapshot.playback {
                Playback::Playing => "REPRODUZINDO",
                Playback::Paused => "PAUSADO",
                Playback::Stopped => "PARADO",
            }
        };
        text(
            &painter,
            card.right_top() + vec2(-14.0, 17.0),
            Align2::RIGHT_CENTER,
            status,
            9.0,
            color,
        );
        let row_y = card.top() + 44.0;
        let bar = Rect::from_min_size(
            pos2(card.left() + 65.0, row_y - 2.0),
            vec2(card.width() - 130.0, 4.0),
        );
        painter.rect_filled(bar, 2, Color32::from_rgb(39, 49, 42));
        let mut current = "—:—".to_string();
        let mut total = "—:—".to_string();
        if let Some(timeline) = &snapshot.timeline {
            let position = timeline.position_at(Instant::now());
            let fraction = if timeline.duration.is_zero() {
                0.0
            } else {
                (position.as_secs_f64() / timeline.duration.as_secs_f64()).clamp(0.0, 1.0) as f32
            };
            if fraction > 0.0 {
                painter.rect_filled(
                    Rect::from_min_size(bar.min, vec2(bar.width() * fraction, bar.height())),
                    2,
                    GREEN,
                );
            }
            current = format_time(position);
            total = format_time(timeline.duration);
        }
        text(
            &painter,
            pos2(card.left() + 14.0, row_y),
            Align2::LEFT_CENTER,
            &current,
            11.0,
            MUTED,
        );
        text(
            &painter,
            pos2(card.right() - 14.0, row_y),
            Align2::RIGHT_CENTER,
            &total,
            11.0,
            MUTED,
        );
        let notice = state
            .notice
            .as_ref()
            .filter(|(_, time)| time.elapsed() < Duration::from_secs(7));
        let help = problem
            .map(String::as_str)
            .or_else(|| notice.map(|(message, _)| message.as_str()))
            .unwrap_or("Progresso somente leitura, informado pela sessão de mídia do Windows.");
        ui.interact(card, ui.id().with("progress-info"), egui::Sense::hover())
            .on_hover_text(help);
    }

    fn transport(&self, ui: &mut egui::Ui, center: Pos2, snapshot: &Snapshot, busy: bool) {
        let available = snapshot.connected && !busy;
        let playing = snapshot.playback == Playback::Playing;
        for (offset, id, label, enabled, primary, icon, command) in [
            (
                -72.0,
                "previous",
                "Música anterior",
                available && snapshot.capabilities.previous,
                false,
                Icon::Previous,
                Command::Previous,
            ),
            (
                0.0,
                "play-pause",
                if playing { "Pausar" } else { "Reproduzir" },
                available && snapshot.capabilities.can_toggle(snapshot.playback),
                true,
                if playing { Icon::Pause } else { Icon::Play },
                Command::Toggle,
            ),
            (
                72.0,
                "next",
                "Próxima música",
                available && snapshot.capabilities.next,
                false,
                Icon::Next,
                Command::Next,
            ),
        ] {
            let size = if primary { 58.0 } else { 44.0 };
            if icon_button(
                ui,
                Rect::from_center_size(center + vec2(offset, 0.0), vec2(size, size)),
                id,
                label,
                enabled,
                primary,
                icon,
            )
            .clicked()
            {
                self.backend.send(command);
            }
        }
    }

    fn disabled_card(&self, ui: &mut egui::Ui, card: Rect) {
        let painter = ui.painter().clone();
        let p = |x, y| card.min + vec2(x, y);
        painter.rect_filled(card, 12, Color32::from_rgb(20, 26, 22));
        text(
            &painter,
            p(16.0, 18.0),
            Align2::LEFT_CENTER,
            "VOLUME",
            9.0,
            DIM,
        );
        text(
            &painter,
            p(card.width() - 16.0, 18.0),
            Align2::RIGHT_CENTER,
            "INDISPONÍVEL",
            8.0,
            DIM,
        );
        draw_icon(&painter, p(25.0, 39.0), Icon::Volume, DIM);
        painter.line_segment(
            [p(52.0, 39.0), p(card.width() - 52.0, 39.0)],
            Stroke::new(3.0_f32, BORDER),
        );
        text(
            &painter,
            p(card.width() - 16.0, 39.0),
            Align2::RIGHT_CENTER,
            "—",
            11.0,
            DIM,
        );
        let playlist = Rect::from_min_size(p(14.0, 58.0), vec2(card.width() - 28.0, 33.0));
        painter.rect_filled(playlist, 7, Color32::from_rgb(26, 33, 28));
        draw_icon(
            &painter,
            playlist.left_center() + vec2(16.0, 0.0),
            Icon::Playlist,
            DIM,
        );
        text(
            &painter,
            playlist.left_center() + vec2(34.0, 0.0),
            Align2::LEFT_CENTER,
            "Playlists",
            11.0,
            DIM,
        );
        text(
            &painter,
            playlist.right_center() - vec2(14.0, 0.0),
            Align2::RIGHT_CENTER,
            "Em breve",
            10.0,
            DIM,
        );
        ui.interact(card, ui.id().with("unavailable"), egui::Sense::hover()).on_hover_text("Volume e playlists indisponíveis nesta versão. Ajuste pelo Spotify ou pelo Windows.");
    }
}

impl eframe::App for SpotifyDropdown {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.window.update(self.menu_open);
        if let Some([x, y]) = self.window.context_click() {
            self.open_menu(pos2(x, y));
        }
        if ctx.input(|input| input.pointer.button_clicked(egui::PointerButton::Secondary)) {
            self.open_menu(
                ctx.input(|input| input.pointer.interact_pos())
                    .unwrap_or(pos2(40.0, 60.0)),
            );
        }
        if self.window.collapsed() {
            return;
        }
        let state = self.backend.state();
        let snapshot = &state.snapshot;
        self.update_artwork(&snapshot.artwork, ctx);
        if snapshot.playback == Playback::Playing && snapshot.timeline.is_some() {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let painter = ui.painter().clone();
                let viewport = ui.max_rect();
                let height = self.window.scene_height(viewport.height());
                // Slide a fixed-size scene through the growing native window, without reflow.
                let panel = Rect::from_min_size(
                    viewport.min + vec2(0.0, viewport.height() - height),
                    vec2(viewport.width(), height),
                );
                let width = panel.width();
                let p = |x, y| panel.min + vec2(x, y);
                let r = |x, y, w, h| Rect::from_min_size(p(x, y), vec2(w, h));
                let corners = if self.window.preferences.mode == Mode::Dropdown {
                    egui::CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: 16,
                        se: 16,
                    }
                } else {
                    egui::CornerRadius::same(16)
                };
                let background = if self.window.preferences.mode == Mode::Dropdown {
                    Rect::from_min_max(panel.min, panel.max - vec2(1.0, 1.0))
                } else {
                    panel.shrink(1.0)
                };
                painter.rect_filled(background, corners, Color32::from_rgb(12, 16, 14));
                painter.rect_stroke(
                    background,
                    corners,
                    Stroke::new(1.0_f32, BORDER),
                    StrokeKind::Inside,
                );
                painter.circle_filled(p(28.0, 28.0), 4.0, GREEN);
                text(
                    &painter,
                    p(41.0, 28.0),
                    Align2::LEFT_CENTER,
                    "SPOTIFY DROPDOWN",
                    12.0,
                    TEXT,
                );
                if icon_button(
                    ui,
                    r(width - 112.0, 11.0, 28.0, 32.0),
                    "menu",
                    "Ajustes e Sobre",
                    true,
                    false,
                    Icon::Menu,
                )
                .clicked()
                {
                    if self.menu_open {
                        self.menu_open = false;
                    } else {
                        self.open_menu(p((width - 324.0).max(10.0), 48.0));
                    }
                }
                if icon_button(
                    ui,
                    r(width - 79.0, 11.0, 28.0, 32.0),
                    "minimize",
                    "Minimizar",
                    true,
                    false,
                    Icon::Minimize,
                )
                .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
                if icon_button(
                    ui,
                    r(width - 46.0, 11.0, 28.0, 32.0),
                    "close",
                    "Fechar",
                    true,
                    false,
                    Icon::Close,
                )
                .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                painter.line_segment(
                    [p(25.0, 56.0), p(width - 25.0, 56.0)],
                    Stroke::new(1.0_f32, BORDER),
                );
                let layout = PlayerLayout::new(width, height);
                self.cover(ui, layout.cover.translate(panel.min.to_vec2()));
                let labels = layout.labels.translate(panel.min.to_vec2());
                self.track_labels(
                    &painter.with_clip_rect(labels.intersect(ui.clip_rect())),
                    labels.min,
                    labels.width(),
                    snapshot,
                );
                self.transport(
                    ui,
                    layout.transport + panel.min.to_vec2(),
                    snapshot,
                    state.busy,
                );
                self.playback_card(ui, layout.playback.translate(panel.min.to_vec2()), &state);
                if let Some(extras) = layout.extras {
                    self.disabled_card(ui, extras.translate(panel.min.to_vec2()));
                    icon_button(
                        ui,
                        r(width - 56.0, layout.transport.y - 13.0, 32.0, 26.0),
                        "favorite",
                        "Favorito · indisponível nesta versão",
                        false,
                        false,
                        Icon::Heart,
                    );
                }
                for offset in [7.0, 12.0] {
                    painter.line_segment(
                        [
                            p(width - offset - 3.0, height - 5.0),
                            p(width - 5.0, height - offset - 3.0),
                        ],
                        Stroke::new(1.0_f32, DIM),
                    );
                }
            });
        if self.menu_open {
            self.menu(ctx);
        }
        self.window.set_hold_open(self.menu_open);
    }
}

use crate::{
    layout::PlayerLayout,
    media::{Artwork, Backend, BackendState, Command, Snapshot},
    model::{Playback, format_time},
    preferences::{Alignment, Mode, Preferences},
    ui::{
        BORDER, DIM, GREEN, Icon, MUTED, SLIDER_KNOB, SLIDER_TRACK, TEXT, draw_icon, green_button,
        green_checkbox, green_radio, icon_button, spotify_slider, text, transport_button,
    },
    volume::VolumeBackend,
    window::WindowManager,
};
use eframe::egui::{
    self, Align2, Color32, FontId, Rect, Stroke, StrokeKind, TextureHandle, pos2, vec2,
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
    volume: VolumeBackend,
    window: WindowManager,
    artwork: Option<Arc<Artwork>>,
    texture: Option<TextureHandle>,
    menu_open: bool,
    menu_tab: MenuTab,
    delay_draft: f32,
    compact: bool,
    marquee_started: Instant,
    track_key: String,
    volume_open: bool,
    volume_draft: f32,
    volume_changed: Instant,
}

impl SpotifyDropdown {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        preferences: Preferences,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.window_fill = Color32::from_rgb(13, 13, 13);
        visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
        visuals.selection.bg_fill = GREEN.gamma_multiply(0.25);
        visuals.selection.stroke = Stroke::new(1.0_f32, GREEN);
        cc.egui_ctx.set_visuals(visuals);
        cc.egui_ctx.style_mut(|s| {
            s.animation_time = 0.08;
            s.spacing.item_spacing = vec2(8.0, 8.0);
        });
        let delay_draft = preferences.hide_delay_ms as f32 / 1000.0;
        let compact = preferences.size()[1] < 312.0;
        Ok(Self {
            backend: Backend::start(cc.egui_ctx.clone()),
            volume: VolumeBackend::start(cc.egui_ctx.clone()),
            window: WindowManager::new(cc, preferences)?,
            artwork: None,
            texture: None,
            menu_open: false,
            menu_tab: MenuTab::Settings,
            delay_draft,
            compact,
            marquee_started: Instant::now(),
            track_key: String::new(),
            volume_open: false,
            volume_draft: 1.0,
            volume_changed: Instant::now(),
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

    fn menu(&mut self, ui: &mut egui::Ui, panel: Rect) {
        let mut preferences = self.window.preferences.clone();
        let tabs = Rect::from_min_size(
            panel.min + vec2(20.0, 66.0),
            vec2(panel.width() - 40.0, 26.0),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(tabs), |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.menu_tab, MenuTab::Settings, "Ajustes");
                ui.selectable_value(&mut self.menu_tab, MenuTab::About, "Sobre");
            });
        });
        let content =
            Rect::from_min_max(panel.min + vec2(20.0, 102.0), panel.max - vec2(20.0, 58.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
            ui.set_clip_rect(content.intersect(ui.clip_rect()));
            egui::ScrollArea::vertical().id_salt("settings-scroll").max_height(content.height()).show(ui, |ui| {
                ui.set_width(content.width() - 12.0);
                match self.menu_tab {
                    MenuTab::About => {
                        ui.heading("Spotify Dropdown");
                        ui.colored_label(GREEN, format!("Versão {} · Alpha", env!("CARGO_PKG_VERSION")));
                        ui.label("Rust / egui · integração local com as sessões de mídia do Windows.");
                        ui.label("Volume exclusivo das sessões de áudio do Spotify. Nenhum ajuste do volume global.");
                        ui.label("Adicionar à biblioteca e playlists permanecem indisponíveis.");
                        ui.small("Preferências em settings.ini, ao lado do executável. Não afiliado ao Spotify.");
                    }
                    MenuTab::Settings => {
                        ui.strong("Exibição");
                        let mut dropdown = preferences.mode == Mode::Dropdown;
                        if green_checkbox(ui, &mut dropdown, "Ativar modo dropdown").changed() {
                            preferences.mode = if dropdown { Mode::Dropdown } else { Mode::Free };
                        }
                        ui.add_space(6.0);
                        ui.strong("Posição no topo");
                        ui.horizontal(|ui| {
                            green_radio(ui, &mut preferences.alignment, Alignment::Left, "Esquerda");
                            green_radio(ui, &mut preferences.alignment, Alignment::Center, "Centro");
                            green_radio(ui, &mut preferences.alignment, Alignment::Right, "Direita");
                        });
                        let label = self.window.monitors.iter().enumerate().find(|(_,m)| m.device == preferences.monitor).map(|(i,m)| m.label(i)).unwrap_or_else(|| "Monitor atual".into());
                        egui::ComboBox::from_id_salt("monitor").width((content.width()-20.0).min(340.0)).selected_text(label).show_ui(ui, |ui| {
                            for (i,m) in self.window.monitors.iter().enumerate() {
                                ui.selectable_value(&mut preferences.monitor, m.device.clone(), m.label(i));
                            }
                        });
                        ui.separator();
                        green_checkbox(ui, &mut preferences.auto_hide, "Recolher quando o cursor sair");
                        ui.horizontal(|ui| {
                            let delay = spotify_slider(ui, "hide-delay", &mut self.delay_draft, 0.1..=5.0, false, 100.0, preferences.auto_hide, Some(0.1));
                            if delay.drag_stopped() || (delay.changed() && !delay.dragged()) {
                                preferences.hide_delay_ms = (self.delay_draft * 1000.0).round() as u64;
                            }
                            ui.label(format!("{:.1} s", self.delay_draft));
                            ui.label("Recolher após");
                        });
                        ui.separator();
                        ui.small("Arraste o cabeçalho para mover. No dropdown, redimensione pelos cantos inferiores. Esconder desativa a abertura pelo topo até o duplo clique na bandeja.");
                        if let Some(error) = &self.window.error { ui.colored_label(Color32::from_rgb(224,177,103),error); }
                    }
                }
            });
        });
        self.window.apply(preferences);
        let y = panel.bottom() - 45.0;
        if green_button(
            ui,
            Rect::from_min_size(pos2(panel.left() + 20.0, y), vec2(118.0, 31.0)),
            "hide",
            "Esconder",
        )
        .clicked()
        {
            self.menu_open = false;
            self.volume_open = false;
            self.window.hide();
        }
        if green_button(
            ui,
            Rect::from_min_size(pos2(panel.right() - 138.0, y), vec2(118.0, 31.0)),
            "quit",
            "Fechar",
        )
        .clicked()
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn cover(&self, ui: &mut egui::Ui, cover: Rect) {
        ui.painter()
            .rect_filled(cover, 2, Color32::from_rgb(21, 23, 21));
        if let Some(texture) = &self.texture {
            let source = texture.size_vec2();
            let available = cover.shrink(1.0);
            let size = source * (available.width() / source.x).min(available.height() / source.y);
            ui.put(
                Rect::from_center_size(cover.center(), size),
                egui::Image::new(texture)
                    .fit_to_exact_size(size)
                    .corner_radius(2),
            );
        } else {
            draw_icon(ui.painter(), cover.center(), Icon::Note, DIM);
        }
    }

    fn marquee(&self, ui: &egui::Ui, area: Rect, value: &str, size: f32, color: Color32) {
        let painter = ui.painter().with_clip_rect(area.intersect(ui.clip_rect()));
        let value = value.replace(['\r', '\n'], " ");
        let galley = painter.layout_no_wrap(value, FontId::proportional(size), color);
        let y = area.center().y - galley.size().y / 2.0;
        if galley.size().x <= area.width() {
            painter.galley(pos2(area.left(), y), galley, color);
        } else {
            // Positive offsets scroll right; a second copy keeps the loop continuous.
            let period = galley.size().x + 36.0;
            let elapsed = self.marquee_started.elapsed().as_secs_f32();
            let offset = ((elapsed - 1.0).max(0.0) * 24.0) % period;
            painter.galley(pos2(area.left() + offset, y), Arc::clone(&galley), color);
            painter.galley(pos2(area.left() + offset - period, y), galley, color);
            ui.ctx().request_repaint_after(Duration::from_millis(33));
        }
    }

    fn track_labels(&self, ui: &egui::Ui, area: Rect, snapshot: &Snapshot, compactness: f32) {
        let title = if !snapshot.connected {
            "Aguardando Spotify"
        } else if snapshot.title.is_empty() {
            "Faixa sem título"
        } else {
            &snapshot.title
        };
        let artist = if !snapshot.connected {
            "Spotify Desktop"
        } else if snapshot.artist.is_empty() {
            "Artista não informado"
        } else {
            &snapshot.artist
        };
        let album = if snapshot.album.is_empty() {
            "Álbum não informado"
        } else {
            &snapshot.album
        };
        let title_size = egui::lerp((area.width() / 16.0).clamp(20.0, 26.0)..=18.0, compactness);
        let row_height = title_size + 4.0;
        let total = row_height + 30.0;
        let y = area.center().y - total / 2.0;
        let row = |top, h| Rect::from_min_size(pos2(area.left(), top), vec2(area.width(), h));
        self.marquee(ui, row(y, row_height), title, title_size, TEXT);
        self.marquee(ui, row(y + row_height + 1.0, 14.0), artist, 11.5, MUTED);
        self.marquee(ui, row(y + row_height + 16.0, 13.0), album, 10.0, DIM);
    }

    fn header(&self, ui: &egui::Ui, panel: Rect, state: &BackendState) {
        let painter = ui.painter();
        let origin = panel.min + vec2(25.0, 28.0);
        let snapshot = &state.snapshot;
        let status = if state.busy {
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
        if snapshot.connected && snapshot.playback == Playback::Paused {
            for dx in [-2.5, 2.5] {
                painter.rect_filled(
                    Rect::from_center_size(origin + vec2(dx, 0.0), vec2(2.0, 9.0)),
                    0,
                    GREEN,
                );
            }
        } else {
            painter.circle_filled(origin, 3.5, if snapshot.connected { GREEN } else { DIM });
        }
        text(
            painter,
            origin + vec2(13.0, 0.0),
            Align2::LEFT_CENTER,
            status,
            11.0,
            TEXT,
        );
    }

    fn playback_card(
        &self,
        ui: &mut egui::Ui,
        layout: &PlayerLayout,
        offset: egui::Vec2,
        state: &BackendState,
    ) {
        let painter = ui.painter().clone();
        self.transport(ui, layout, offset, &state.snapshot, state.busy);
        icon_button(
            ui,
            layout.add.translate(offset),
            "add",
            "Adicionar à biblioteca · indisponível",
            false,
            false,
            Icon::Add,
        );
        let bar = layout.timeline.translate(offset);
        painter.rect_filled(bar, 2, SLIDER_TRACK);
        let mut current = "—:—".to_string();
        let mut total = "—:—".to_string();
        if let Some(timeline) = &state.snapshot.timeline {
            let position = timeline.position_at(Instant::now());
            let fraction = if timeline.duration.is_zero() {
                0.0
            } else {
                (position.as_secs_f64() / timeline.duration.as_secs_f64()).clamp(0.0, 1.0) as f32
            };
            painter.rect_filled(
                Rect::from_min_size(bar.min, vec2(bar.width() * fraction, bar.height())),
                2,
                GREEN,
            );
            painter.circle_filled(
                pos2(bar.left() + bar.width() * fraction, bar.center().y),
                4.0,
                SLIDER_KNOB,
            );
            current = format_time(position);
            total = format_time(timeline.duration);
        }
        text(
            &painter.with_clip_rect(
                Rect::from_min_size(
                    layout.current_time + offset - vec2(0.0, 8.0),
                    vec2(36.0, 16.0),
                )
                .intersect(ui.clip_rect()),
            ),
            layout.current_time + offset,
            Align2::LEFT_CENTER,
            &current,
            10.0,
            MUTED,
        );
        text(
            &painter.with_clip_rect(
                Rect::from_min_size(
                    layout.total_time + offset - vec2(36.0, 8.0),
                    vec2(36.0, 16.0),
                )
                .intersect(ui.clip_rect()),
            ),
            layout.total_time + offset,
            Align2::RIGHT_CENTER,
            &total,
            10.0,
            MUTED,
        );
        let help = self
            .window
            .error
            .as_ref()
            .or(state.snapshot.problem.as_ref())
            .map(String::as_str)
            .or_else(|| {
                state
                    .notice
                    .as_ref()
                    .filter(|(_, t)| t.elapsed() < Duration::from_secs(7))
                    .map(|(s, _)| s.as_str())
            })
            .unwrap_or("Progresso somente leitura.");
        ui.interact(
            bar.expand(6.0),
            ui.id().with("timeline-info"),
            egui::Sense::hover(),
        )
        .on_hover_text(help);
    }

    fn transport(
        &self,
        ui: &mut egui::Ui,
        layout: &PlayerLayout,
        offset: egui::Vec2,
        snapshot: &Snapshot,
        busy: bool,
    ) {
        let center = layout.transport + offset;
        let step = 50.0 - 14.0 * layout.compactness;
        let available = snapshot.connected && !busy;
        let playing = snapshot.playback == Playback::Playing;
        for (offset, id, label, enabled, primary, icon, command) in [
            (
                -step,
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
                step,
                "next",
                "Próxima música",
                available && snapshot.capabilities.next,
                false,
                Icon::Next,
                Command::Next,
            ),
        ] {
            let size = if primary {
                44.0 - 4.0 * layout.compactness
            } else {
                32.0
            };
            let visual_size = if primary {
                36.0 - 6.0 * layout.compactness
            } else {
                26.0
            };
            if transport_button(
                ui,
                Rect::from_center_size(center + vec2(offset, 0.0), vec2(size, size)),
                visual_size,
                0.78 - 0.10 * layout.compactness,
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

    fn volume_control(&mut self, ui: &mut egui::Ui, button: Rect, panel: Rect) {
        // A falling/retracting viewport may expose less space than the popup needs.
        if panel.height() < 136.0 || panel.width() < 64.0 {
            self.volume_open = false;
            return;
        }
        let state = self.volume.state();
        let help = state.error.as_deref().unwrap_or(if state.muted {
            "Clique para ativar o som do Spotify"
        } else {
            "Clique para mutar o Spotify; passe o cursor para ajustar o volume"
        });
        let response = icon_button(
            ui,
            button,
            "volume",
            help,
            state.available,
            false,
            if state.muted {
                Icon::VolumeMuted
            } else {
                Icon::Volume
            },
        );
        if response.clicked() {
            self.volume.toggle_mute();
        }
        if response.hovered() || response.clicked() {
            if !self.volume_open {
                self.volume_draft = state.level;
            }
            self.volume_open = state.available;
        }
        if !state.available {
            self.volume_open = false;
        }
        if !self.volume_open {
            return;
        }
        if !ui.ctx().input(|i| i.pointer.any_down())
            && self.volume_changed.elapsed() > Duration::from_millis(500)
        {
            self.volume_draft = state.level;
        }
        let popup_size = vec2(56.0, 128.0);
        let x = (button.center().x - 28.0).clamp(panel.left() + 4.0, panel.right() - 60.0);
        let y = (button.top() - 132.0).clamp(panel.top() + 4.0, panel.bottom() - 132.0);
        let popup = Rect::from_min_size(pos2(x, y), popup_size);
        let corridor = popup.union(button).expand(3.0);
        let inside = ui
            .ctx()
            .input(|i| i.pointer.hover_pos().is_some_and(|p| corridor.contains(p)));
        if !inside {
            self.volume_open = false;
            return;
        }
        egui::Area::new(egui::Id::new("spotify-volume-panel"))
            .order(egui::Order::Foreground)
            .fixed_pos(popup.min)
            .constrain_to(panel)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(Color32::from_rgb(25, 25, 25))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_width(40.0);
                        ui.set_min_size(vec2(40.0, 112.0));
                        ui.spacing_mut().slider_width = 88.0;
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.vertical_centered(|ui| {
                            let slider = spotify_slider(
                                ui,
                                "spotify-volume",
                                &mut self.volume_draft,
                                0.0..=1.0,
                                true,
                                88.0,
                                true,
                                None,
                            );
                            if slider.changed() {
                                self.volume_changed = Instant::now();
                                self.volume.set_level(self.volume_draft);
                            }
                        });
                    });
            });
    }
}

impl eframe::App for SpotifyDropdown {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.window.update(self.menu_open || self.volume_open);
        if self.window.take_dismiss_menu() {
            self.menu_open = false;
            self.volume_open = false;
            self.window.set_hold_open(false);
        }
        if self.window.collapsed() {
            self.menu_open = false;
            self.volume_open = false;
            self.marquee_started = Instant::now();
            return;
        }
        let state = self.backend.state();
        self.update_artwork(&state.snapshot.artwork, ctx);
        let key = format!(
            "{}\0{}\0{}",
            state.snapshot.title, state.snapshot.artist, state.snapshot.album
        );
        if self.track_key != key {
            self.track_key = key;
            self.marquee_started = Instant::now();
        }
        if state.snapshot.playback == Playback::Playing && state.snapshot.timeline.is_some() {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let visible = ui.max_rect();
                let height = self.window.scene_height(visible.height());
                let panel = Rect::from_min_size(
                    visible.min + vec2(0.0, visible.height() - height),
                    vec2(visible.width(), height),
                );
                if height < 312.0 {
                    self.compact = true;
                } else if height >= 328.0 {
                    self.compact = false;
                }
                let corners = if self.window.preferences.mode == Mode::Dropdown {
                    egui::CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: 14,
                        se: 14,
                    }
                } else {
                    egui::CornerRadius::same(14)
                };
                ui.painter()
                    .rect_filled(panel, corners, Color32::from_rgb(8, 8, 8));
                ui.painter().rect_stroke(
                    panel.shrink(0.5),
                    corners,
                    Stroke::new(1.0_f32, BORDER),
                    StrokeKind::Inside,
                );
                self.header(ui, panel, &state);
                let button = Rect::from_min_size(
                    panel.min + vec2(panel.width() - 49.0, 11.0),
                    vec2(32.0, 32.0),
                );
                if icon_button(
                    ui,
                    button,
                    "settings",
                    if self.menu_open {
                        "Voltar ao controlador"
                    } else {
                        "Ajustes e comandos"
                    },
                    true,
                    false,
                    if self.menu_open {
                        Icon::Close
                    } else {
                        Icon::Menu
                    },
                )
                .clicked()
                {
                    self.menu_open = !self.menu_open;
                    self.volume_open = false;
                    self.delay_draft = self.window.preferences.hide_delay_ms as f32 / 1000.0;
                }
                ui.painter().line_segment(
                    [
                        panel.min + vec2(20.0, 55.0),
                        panel.min + vec2(panel.width() - 20.0, 55.0),
                    ],
                    Stroke::new(1.0_f32, BORDER),
                );
                if self.menu_open {
                    self.menu(ui, panel);
                } else {
                    let blend = ctx.animate_bool_with_time(
                        egui::Id::new("compact-layout"),
                        self.compact,
                        0.20,
                    );
                    let layout = PlayerLayout::new(panel.width(), height, blend);
                    ui.painter().rect_filled(
                        layout.playback.translate(panel.min.to_vec2()),
                        10,
                        Color32::from_rgb(17, 17, 17),
                    );
                    self.cover(ui, layout.cover.translate(panel.min.to_vec2()));
                    self.track_labels(
                        ui,
                        layout.labels.translate(panel.min.to_vec2()),
                        &state.snapshot,
                        layout.compactness,
                    );
                    self.playback_card(ui, &layout, panel.min.to_vec2(), &state);
                    self.volume_control(
                        ui,
                        layout.volume.translate(panel.min.to_vec2()),
                        panel.intersect(visible),
                    );
                    if let Some(extras) = layout.extras {
                        let card = extras.translate(panel.min.to_vec2());
                        ui.painter()
                            .rect_filled(card, 8, Color32::from_rgb(14, 14, 14));
                        draw_icon(
                            ui.painter(),
                            card.left_center() + vec2(20.0, 0.0),
                            Icon::Playlist,
                            DIM,
                        );
                        text(
                            ui.painter(),
                            card.left_center() + vec2(40.0, 0.0),
                            Align2::LEFT_CENTER,
                            "Playlists",
                            11.0,
                            DIM,
                        );
                        text(
                            ui.painter(),
                            card.right_center() - vec2(14.0, 0.0),
                            Align2::RIGHT_CENTER,
                            "Indisponível",
                            10.0,
                            DIM,
                        );
                    }
                }
                for d in [6.0, 11.0] {
                    ui.painter().line_segment(
                        [
                            panel.max - vec2(d + 3.0, 4.0),
                            panel.max - vec2(4.0, d + 3.0),
                        ],
                        Stroke::new(1.0_f32, DIM),
                    );
                }
            });
        self.window
            .set_hold_open(self.menu_open || self.volume_open);
    }
}

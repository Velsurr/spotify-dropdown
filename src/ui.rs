//! Shared colors and small vector icons; no raster assets are cropped.
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, vec2};
pub(crate) const GREEN: Color32 = Color32::from_rgb(30, 215, 96);
pub(crate) const TEXT: Color32 = Color32::from_rgb(239, 244, 240);
pub(crate) const MUTED: Color32 = Color32::from_rgb(146, 158, 150);
pub(crate) const DIM: Color32 = Color32::from_rgb(92, 105, 97);
pub(crate) const BORDER: Color32 = Color32::from_rgb(39, 48, 42);
pub(crate) const SLIDER_TRACK: Color32 = Color32::from_rgb(43, 43, 43);
pub(crate) const SLIDER_KNOB: Color32 = Color32::from_rgb(245, 245, 245);
pub(crate) fn text(
    painter: &egui::Painter,
    position: Pos2,
    anchor: Align2,
    value: &str,
    size: f32,
    color: Color32,
) {
    painter.text(position, anchor, value, FontId::proportional(size), color);
}

#[derive(Clone, Copy)]
pub(crate) enum Icon {
    Menu,
    Previous,
    Play,
    Pause,
    Next,
    Add,
    Volume,
    VolumeMuted,
    Playlist,
    Note,
    Close,
}

pub(crate) fn icon_button(
    ui: &mut egui::Ui,
    rect: Rect,
    id: &str,
    tooltip: &str,
    enabled: bool,
    primary: bool,
    icon: Icon,
) -> egui::Response {
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            ui.interact(rect, ui.id().with(id), Sense::click())
        })
        .inner;
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, tooltip));
    let hover = enabled && response.hovered();
    let fill = if primary {
        if !enabled {
            Color32::from_rgb(33, 54, 40)
        } else if hover {
            Color32::from_rgb(60, 230, 120)
        } else {
            GREEN
        }
    } else if hover {
        Color32::from_rgb(31, 41, 34)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, if primary { 30 } else { 12 }, fill);
    let color = if !enabled {
        DIM
    } else if primary {
        Color32::from_rgb(8, 25, 14)
    } else {
        TEXT
    };
    draw_icon(ui.painter(), rect.center(), icon, color);
    if enabled {
        response
            .on_hover_text(tooltip)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response.on_disabled_hover_text(tooltip)
    }
}

/// A smaller drawing inside the original hit rectangle keeps the controls comfortable to click.
pub(crate) fn transport_button(
    ui: &mut egui::Ui,
    hit_rect: Rect,
    visual_diameter: f32,
    icon_scale: f32,
    id: &str,
    tooltip: &str,
    enabled: bool,
    primary: bool,
    icon: Icon,
) -> egui::Response {
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            ui.interact(hit_rect, ui.id().with(id), Sense::click())
        })
        .inner;
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, tooltip));
    let hover = enabled && response.hovered();
    let pressed = enabled && response.is_pointer_button_down_on();
    let fill = if !enabled {
        if primary {
            Color32::from_rgb(32, 37, 34)
        } else {
            Color32::TRANSPARENT
        }
    } else if pressed {
        if primary {
            Color32::from_rgb(23, 182, 80)
        } else {
            Color32::from_rgb(35, 42, 38)
        }
    } else if primary {
        if hover {
            Color32::from_rgb(54, 229, 113)
        } else {
            GREEN
        }
    } else if hover {
        Color32::from_rgb(28, 34, 30)
    } else {
        Color32::TRANSPARENT
    };
    let diameter = visual_diameter
        .min(hit_rect.width())
        .min(hit_rect.height())
        .max(1.0);
    ui.painter()
        .circle_filled(hit_rect.center(), diameter / 2.0, fill);
    let color = if !enabled {
        DIM
    } else if primary {
        Color32::from_rgb(5, 23, 12)
    } else {
        TEXT
    };
    draw_icon_scaled(ui.painter(), hit_rect.center(), icon, color, icon_scale);
    if enabled {
        response
            .on_hover_text(tooltip)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response.on_disabled_hover_text(tooltip)
    }
}

pub(crate) fn green_button(ui: &mut egui::Ui, rect: Rect, id: &str, label: &str) -> egui::Response {
    let response = ui.interact(rect, ui.id().with(id), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    let fill = if response.is_pointer_button_down_on() {
        Color32::from_rgb(23, 182, 80)
    } else if response.hovered() {
        Color32::from_rgb(54, 229, 113)
    } else {
        GREEN
    };
    ui.painter().rect_filled(rect, 8, fill);
    text(
        ui.painter(),
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        12.0,
        Color32::from_rgb(4, 22, 11),
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn green_selected_style(ui: &mut egui::Ui, selected: bool) {
    if !selected || !ui.is_enabled() {
        return;
    }
    let widgets = &mut ui.visuals_mut().widgets;
    for visuals in [
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
    ] {
        // Checkbox ticks, radio dots and their fallback label color use fg_stroke.
        visuals.fg_stroke = Stroke::new(1.5_f32, GREEN);
        visuals.bg_stroke = Stroke::new(1.0_f32, GREEN.gamma_multiply(0.7));
    }
    widgets.inactive.bg_fill = Color32::from_rgb(14, 37, 23);
    widgets.hovered.bg_fill = Color32::from_rgb(17, 49, 29);
    widgets.active.bg_fill = Color32::from_rgb(21, 57, 33);
}

pub(crate) fn green_checkbox(ui: &mut egui::Ui, value: &mut bool, label: &str) -> egui::Response {
    let selected = *value;
    ui.scope(|ui| {
        green_selected_style(ui, selected);
        ui.checkbox(value, label)
    })
    .inner
}

pub(crate) fn green_radio<T: PartialEq>(
    ui: &mut egui::Ui,
    current: &mut T,
    value: T,
    label: &str,
) -> egui::Response {
    let selected = *current == value;
    ui.scope(|ui| {
        green_selected_style(ui, selected);
        ui.radio_value(current, value, label)
    })
    .inner
}

/// Keep egui's drag, keyboard, clamping and step semantics; change only the drawing.
pub(crate) fn spotify_slider(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    vertical: bool,
    length: f32,
    enabled: bool,
    step: Option<f64>,
) -> egui::Response {
    let start = *range.start();
    let end = *range.end();
    let enabled = enabled && ui.is_enabled();
    let response = ui
        .push_id(id, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().slider_width = length.max(24.0);
                ui.spacing_mut().slider_rail_height = 4.0;
                ui.spacing_mut().interact_size.y = 20.0;
                let visuals = ui.visuals_mut();
                visuals.selection.bg_fill = if enabled { GREEN } else { DIM };
                visuals.widgets.inactive.bg_fill = SLIDER_TRACK;
                visuals.widgets.inactive.corner_radius = 2.into();
                visuals.widgets.inactive.fg_stroke = Stroke::NONE;
                visuals.widgets.inactive.expansion = 0.0;
                visuals.widgets.hovered.bg_fill = SLIDER_KNOB;
                visuals.widgets.hovered.fg_stroke = Stroke::NONE;
                visuals.widgets.hovered.expansion = 0.0;
                visuals.widgets.active.bg_fill = SLIDER_KNOB;
                visuals.widgets.active.fg_stroke = Stroke::NONE;
                visuals.widgets.active.expansion = 0.0;
                let mut slider = egui::Slider::new(&mut *value, range)
                    .show_value(false)
                    .trailing_fill(true)
                    .handle_shape(egui::style::HandleShape::Circle);
                if vertical {
                    slider = slider.vertical();
                }
                if let Some(step) = step {
                    slider = slider.step_by(step);
                }
                ui.add_enabled(enabled, slider)
            })
            .inner
        })
        .inner;
    // egui shares inactive.bg_fill between the rail and the handle. Overlay only
    // the latter, using the same geometry as the pinned egui 0.33.3 Circle handle.
    let rect = response.rect;
    let radius = if vertical {
        rect.width()
    } else {
        rect.height()
    } / 2.5;
    let fraction = if end > start {
        ((*value - start) / (end - start)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let center = if vertical {
        egui::pos2(
            rect.center().x,
            rect.bottom() - radius - (rect.height() - 2.0 * radius) * fraction,
        )
    } else {
        egui::pos2(
            rect.left() + radius + (rect.width() - 2.0 * radius) * fraction,
            rect.center().y,
        )
    };
    if ui.is_rect_visible(rect) {
        ui.painter()
            .circle_filled(center, radius, if enabled { SLIDER_KNOB } else { DIM });
    }
    response
}

pub(crate) fn draw_icon(painter: &egui::Painter, center: Pos2, icon: Icon, color: Color32) {
    draw_icon_scaled(painter, center, icon, color, 1.0);
}

pub(crate) fn draw_icon_scaled(
    painter: &egui::Painter,
    center: Pos2,
    icon: Icon,
    color: Color32,
    scale: f32,
) {
    let scale = scale.clamp(0.4, 1.6);
    let p = |x, y| center + vec2(x, y) * scale;
    let stroke = Stroke::new(1.7_f32 * scale, color);
    match icon {
        Icon::Menu => {
            for x in [-5.0, 0.0, 5.0] {
                painter.circle_filled(p(x, 0.0), 1.3 * scale, color);
            }
        }
        Icon::Play => {
            painter.add(Shape::convex_polygon(
                vec![p(-5.0, -9.0), p(9.0, 0.0), p(-5.0, 9.0)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Pause => {
            for x in [-7.0, 2.0] {
                painter.rect_filled(
                    Rect::from_min_size(p(x, -9.0), vec2(5.0, 18.0) * scale),
                    1,
                    color,
                );
            }
        }
        Icon::Previous | Icon::Next => {
            let sign = if matches!(icon, Icon::Previous) {
                -1.0
            } else {
                1.0
            };
            painter.add(Shape::convex_polygon(
                vec![
                    p(-6.0 * sign, -7.5),
                    p(5.0 * sign, 0.0),
                    p(-6.0 * sign, 7.5),
                ],
                color,
                Stroke::NONE,
            ));
            painter.line_segment(
                [p(8.0 * sign, -8.0), p(8.0 * sign, 8.0)],
                Stroke::new(2.5_f32 * scale, color),
            );
        }
        Icon::Add => {
            painter.circle_stroke(center, 8.0 * scale, Stroke::new(1.4_f32 * scale, color));
            painter.line_segment([p(-4.0, 0.0), p(4.0, 0.0)], stroke);
            painter.line_segment([p(0.0, -4.0), p(0.0, 4.0)], stroke);
        }
        Icon::Volume | Icon::VolumeMuted => {
            painter.add(Shape::closed_line(
                vec![
                    p(-7.0, -3.0),
                    p(-3.0, -3.0),
                    p(2.0, -7.0),
                    p(2.0, 7.0),
                    p(-3.0, 3.0),
                    p(-7.0, 3.0),
                ],
                stroke,
            ));
            if matches!(icon, Icon::VolumeMuted) {
                painter.line_segment(
                    [p(-9.0, 9.0), p(9.0, -9.0)],
                    Stroke::new(1.8_f32 * scale, color),
                );
            } else {
                painter.add(Shape::line(
                    vec![p(6.0, -4.0), p(8.0, 0.0), p(6.0, 4.0)],
                    stroke,
                ));
            }
        }
        Icon::Playlist => {
            for y in [-5.0, 0.0, 5.0] {
                painter.line_segment([p(-6.0, y), p(6.0, y)], Stroke::new(1.4_f32 * scale, color));
            }
        }
        Icon::Note => {
            painter.line_segment(
                [p(-3.0, 6.0), p(-3.0, -9.0)],
                Stroke::new(2.0_f32 * scale, color),
            );
            painter.line_segment(
                [p(-3.0, -9.0), p(8.0, -12.0)],
                Stroke::new(2.0_f32 * scale, color),
            );
            painter.line_segment(
                [p(8.0, -12.0), p(8.0, 3.0)],
                Stroke::new(2.0_f32 * scale, color),
            );
            painter.circle_filled(p(-6.0, 7.0), 4.0 * scale, color);
            painter.circle_filled(p(5.0, 4.0), 4.0 * scale, color);
        }
        Icon::Close => {
            painter.line_segment(
                [p(-4.0, -4.0), p(4.0, 4.0)],
                Stroke::new(1.2_f32 * scale, color),
            );
            painter.line_segment(
                [p(-4.0, 4.0), p(4.0, -4.0)],
                Stroke::new(1.2_f32 * scale, color),
            );
        }
    }
}

pub fn app_icon() -> egui::IconData {
    let mut rgba = vec![0; 64 * 64 * 4];
    for y in 0..64 {
        for x in 0..64 {
            let index = (y * 64 + x) * 4;
            if (x as f32 - 31.5).hypot(y as f32 - 31.5) < 29.0 {
                rgba[index..index + 4].copy_from_slice(&[14, 28, 18, 255]);
                if ((19..25).contains(&x) && (25..40).contains(&y))
                    || ((29..35).contains(&x) && (17..47).contains(&y))
                    || ((39..45).contains(&x) && (22..42).contains(&y))
                {
                    rgba[index..index + 4].copy_from_slice(&[30, 215, 96, 255]);
                }
            }
        }
    }
    egui::IconData {
        rgba,
        width: 64,
        height: 64,
    }
}

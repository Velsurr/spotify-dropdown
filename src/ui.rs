//! Shared colors and small vector icons; no raster assets are cropped.
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, vec2};
pub(crate) const GREEN: Color32 = Color32::from_rgb(30, 215, 96);
pub(crate) const TEXT: Color32 = Color32::from_rgb(239, 244, 240);
pub(crate) const MUTED: Color32 = Color32::from_rgb(146, 158, 150);
pub(crate) const DIM: Color32 = Color32::from_rgb(92, 105, 97);
pub(crate) const BORDER: Color32 = Color32::from_rgb(39, 48, 42);
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

pub(crate) fn text_block(
    painter: &egui::Painter,
    position: Pos2,
    width: f32,
    rows: usize,
    value: &str,
    size: f32,
    color: Color32,
) {
    let mut job =
        egui::text::LayoutJob::simple(value.into(), FontId::proportional(size), color, width);
    job.wrap.max_rows = rows;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    let galley = painter.layout_job(job);
    painter.galley(position, galley, color);
}

#[derive(Clone, Copy)]
pub(crate) enum Icon {
    Menu,
    Previous,
    Play,
    Pause,
    Next,
    Heart,
    Volume,
    Playlist,
    Note,
    Minimize,
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

pub(crate) fn draw_icon(painter: &egui::Painter, center: Pos2, icon: Icon, color: Color32) {
    let p = |x, y| center + vec2(x, y);
    let stroke = Stroke::new(1.7_f32, color);
    match icon {
        Icon::Menu => {
            for x in [-5.0, 0.0, 5.0] {
                painter.circle_filled(p(x, 0.0), 1.3, color);
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
                painter.rect_filled(Rect::from_min_size(p(x, -9.0), vec2(5.0, 18.0)), 1, color);
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
                Stroke::new(2.5_f32, color),
            );
        }
        Icon::Heart => {
            painter.add(Shape::closed_line(
                vec![
                    p(0.0, 7.0),
                    p(-8.0, -1.0),
                    p(-8.0, -5.0),
                    p(-5.0, -8.0),
                    p(-2.0, -8.0),
                    p(0.0, -5.0),
                    p(2.0, -8.0),
                    p(5.0, -8.0),
                    p(8.0, -5.0),
                    p(8.0, -1.0),
                ],
                Stroke::new(1.4_f32, color),
            ));
        }
        Icon::Volume => {
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
            painter.add(Shape::line(
                vec![p(6.0, -4.0), p(8.0, 0.0), p(6.0, 4.0)],
                stroke,
            ));
        }
        Icon::Playlist => {
            for y in [-5.0, 0.0, 5.0] {
                painter.line_segment([p(-6.0, y), p(6.0, y)], Stroke::new(1.4_f32, color));
            }
        }
        Icon::Note => {
            painter.line_segment([p(-3.0, 6.0), p(-3.0, -9.0)], Stroke::new(2.0_f32, color));
            painter.line_segment([p(-3.0, -9.0), p(8.0, -12.0)], Stroke::new(2.0_f32, color));
            painter.line_segment([p(8.0, -12.0), p(8.0, 3.0)], Stroke::new(2.0_f32, color));
            painter.circle_filled(p(-6.0, 7.0), 4.0, color);
            painter.circle_filled(p(5.0, 4.0), 4.0, color);
        }
        Icon::Minimize => {
            painter.line_segment([p(-4.0, 0.0), p(4.0, 0.0)], Stroke::new(1.2_f32, color));
        }
        Icon::Close => {
            painter.line_segment([p(-4.0, -4.0), p(4.0, 4.0)], Stroke::new(1.2_f32, color));
            painter.line_segment([p(-4.0, 4.0), p(4.0, -4.0)], Stroke::new(1.2_f32, color));
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

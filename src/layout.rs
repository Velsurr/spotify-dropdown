//! Reserved regions keep transport, add and volume controls inside one playback card.
use eframe::egui::{Pos2, Rect, pos2, vec2};

pub struct PlayerLayout {
    pub cover: Option<Rect>,
    pub labels: Rect,
    pub transport: Pos2,
    pub playback: Rect,
    pub add: Rect,
    pub volume: Rect,
    pub extras: Option<Rect>,
}

impl PlayerLayout {
    pub fn new(width: f32, height: f32, compact: bool) -> Self {
        let extras = (height >= 520.0)
            .then(|| Rect::from_min_size(pos2(16.0, height - 50.0), vec2(width - 32.0, 38.0)));
        let bottom = extras.map_or(height - 12.0, |r| r.top() - 12.0);
        let playback = Rect::from_min_max(pos2(16.0, bottom - 92.0), pos2(width - 16.0, bottom));
        let body = Rect::from_min_max(pos2(20.0, 68.0), pos2(width - 20.0, playback.top() - 12.0));
        let cover = (!compact).then(|| {
            let side = (width * 0.23).clamp(78.0, 144.0).min(body.height());
            Rect::from_center_size(
                pos2(body.left() + side / 2.0, body.center().y),
                vec2(side, side),
            )
        });
        let labels = Rect::from_min_max(
            pos2(cover.map_or(body.left(), |r| r.right() + 18.0), body.top()),
            body.max,
        );
        let row_y = playback.top() + 29.0;
        Self {
            cover,
            labels,
            playback,
            extras,
            transport: pos2(width / 2.0, row_y),
            add: Rect::from_center_size(pos2(playback.left() + 23.0, row_y), vec2(30.0, 30.0)),
            volume: Rect::from_center_size(pos2(playback.right() - 23.0, row_y), vec2(30.0, 30.0)),
        }
    }
}

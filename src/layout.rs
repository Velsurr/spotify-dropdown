//! Non-overlapping regions, shared by all window sizes and animation frames.
use eframe::egui::{Pos2, Rect, pos2, vec2};

pub struct PlayerLayout {
    pub cover: Rect,
    pub labels: Rect,
    pub transport: Pos2,
    pub playback: Rect,
    pub extras: Option<Rect>,
}

impl PlayerLayout {
    pub fn new(width: f32, height: f32) -> Self {
        let wide = width >= 700.0;
        let extras = ((!wide && height >= 550.0) || (wide && height >= 650.0))
            .then(|| Rect::from_min_size(pos2(24.0, height - 124.0), vec2(width - 48.0, 106.0)));
        let bottom = extras.map_or(height - 18.0, |r| r.top() - 14.0);
        let playback = Rect::from_min_max(pos2(24.0, bottom - 68.0), pos2(width - 24.0, bottom));
        let body = Rect::from_min_max(pos2(24.0, 74.0), pos2(width - 24.0, playback.top() - 16.0));
        let (cover, labels, transport) = if wide {
            let side = body.height().min(width * 0.25).min(220.0);
            let cover = Rect::from_min_size(body.min, vec2(side, side));
            let transport = pos2(body.right() - 101.0, body.center().y);
            let labels = Rect::from_min_max(
                pos2(cover.right() + 18.0, body.top() + 2.0),
                pos2(transport.x - 119.0, body.bottom()),
            );
            (cover, labels, transport)
        } else {
            let transport = pos2(width / 2.0, body.bottom() - 29.0);
            let metadata_bottom = transport.y - 29.0 - 12.0;
            let side = (width * 0.27)
                .clamp(88.0, 150.0)
                .min(metadata_bottom - body.top());
            let cover = Rect::from_min_size(body.min, vec2(side, side));
            let labels = Rect::from_min_max(
                pos2(cover.right() + 18.0, body.top() + 2.0),
                pos2(body.right(), metadata_bottom),
            );
            (cover, labels, transport)
        };
        Self {
            cover,
            labels,
            transport,
            playback,
            extras,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_sizes_keep_art_metadata_transport_and_timeline_apart() {
        for width in (360..=1400).step_by(13) {
            for height in (340..=900).step_by(11) {
                let l = PlayerLayout::new(width as f32, height as f32);
                let controls = Rect::from_center_size(l.transport, vec2(202.0, 58.0));
                for rect in [l.cover, l.labels, controls, l.playback] {
                    assert!(rect.is_positive(), "{width} × {height}: {rect:?}");
                    assert!(rect.left() >= 24.0 && rect.right() <= width as f32 - 24.0);
                    assert!(rect.top() >= 74.0 && rect.bottom() <= height as f32 - 18.0);
                }
                assert!(!l.cover.intersects(l.playback));
                assert!(!l.labels.intersects(l.playback));
                assert!(!controls.intersects(l.playback));
                assert!(!l.cover.intersects(controls));
                assert!(!l.labels.intersects(controls));
                if let Some(extras) = l.extras {
                    assert!(extras.top() > l.playback.bottom());
                }
            }
        }
    }
}

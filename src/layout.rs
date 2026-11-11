//! Layout only: native resize, centering, tray and dropdown geometry remain unchanged.
use eframe::egui::{self, Pos2, Rect, pos2, vec2};

pub struct PlayerLayout {
    pub cover: Rect,
    pub labels: Rect,
    pub transport: Pos2,
    pub playback: Rect,
    pub add: Rect,
    pub volume: Rect,
    pub timeline: Rect,
    pub current_time: Pos2,
    pub total_time: Pos2,
    pub compactness: f32,
    pub extras: Option<Rect>,
}

impl PlayerLayout {
    pub fn new(width: f32, height: f32, compactness: f32) -> Self {
        let mix = compactness.clamp(0.0, 1.0);
        if mix == 0.0 || mix == 1.0 {
            return Self::base(width, height, mix == 1.0);
        }
        let mut normal = Self::base(width, height, false);
        let compact = Self::base(width, height, true);
        let rect_mix = |a: Rect, b: Rect, t: f32| {
            Rect::from_min_max(a.min + (b.min - a.min) * t, a.max + (b.max - a.max) * t)
        };
        normal.cover = rect_mix(normal.cover, compact.cover, mix);
        // Reserve the text column and move the controls right before moving them up.
        // Reversing the same path puts controls below the text before expanding it.
        normal.labels.min = normal.labels.min + (compact.labels.min - normal.labels.min) * mix;
        normal.labels.max.x = egui::lerp(
            normal.labels.max.x..=compact.labels.max.x,
            (mix * 2.0).min(1.0),
        );
        normal.labels.max.y = normal.cover.bottom();
        normal.transport.x = egui::lerp(
            normal.transport.x..=compact.transport.x,
            (mix * 2.0).min(1.0),
        );
        normal.transport.y = egui::lerp(
            normal.transport.y..=compact.transport.y,
            ((mix - 0.5) * 2.0).max(0.0),
        );
        let row_y = egui::lerp(
            normal.timeline.center().y..=compact.timeline.center().y,
            mix,
        )
        .max(normal.transport.y + 38.0);
        normal.playback = rect_mix(normal.playback, compact.playback, mix);
        normal.playback.max.y = normal.playback.bottom().max(row_y + 18.0);
        for point in [&mut normal.current_time, &mut normal.total_time] {
            point.y = row_y;
        }
        normal.timeline = normal
            .timeline
            .translate(vec2(0.0, row_y - normal.timeline.center().y));
        normal.add = normal
            .add
            .translate(vec2(0.0, row_y - normal.add.center().y));
        normal.volume = normal
            .volume
            .translate(vec2(0.0, row_y - normal.volume.center().y));
        normal.compactness = mix;
        normal.extras = None;
        normal
    }

    fn base(width: f32, height: f32, compact: bool) -> Self {
        let span = (width - 32.0).min(840.0);
        let left = (width - span) / 2.0;
        let (cover, labels, playback, transport, extras) = if compact {
            let side = (56.0 + ((height - 240.0) / 88.0).clamp(0.0, 1.0) * 12.0).min(span * 0.20);
            let block_height = side + 64.0;
            let top = 68.0 + ((height - 84.0 - block_height) / 2.0).max(0.0);
            let playback = Rect::from_min_size(pos2(left, top), vec2(span, block_height));
            let cover = Rect::from_min_size(pos2(left + 12.0, top + 12.0), vec2(side, side));
            let transport = pos2(playback.right() - 68.0, cover.center().y);
            let labels = Rect::from_min_max(
                pos2(cover.right() + 12.0, cover.top()),
                pos2(transport.x - 64.0, cover.bottom()),
            );
            (cover, labels, playback, transport, None)
        } else {
            let side = (width * 0.22)
                .clamp(78.0, 132.0)
                .min((height - 176.0).max(56.0));
            let extras_height = (height - 470.0).clamp(0.0, 50.0);
            let block_height = side + 12.0 + 80.0 + extras_height;
            let top = 68.0 + ((height - 84.0 - block_height) / 2.0).max(0.0);
            let cover = Rect::from_min_size(pos2(left + 4.0, top), vec2(side, side));
            let labels = Rect::from_min_max(
                pos2(cover.right() + 16.0, top),
                pos2(left + span - 4.0, top + side),
            );
            let playback = Rect::from_min_size(pos2(left, cover.bottom() + 12.0), vec2(span, 80.0));
            let transport = pos2(width / 2.0, playback.top() + 24.0);
            let extras = (extras_height >= 50.0).then(|| {
                Rect::from_min_size(pos2(left, playback.bottom() + 12.0), vec2(span, 38.0))
            });
            (cover, labels, playback, transport, extras)
        };
        let row_y = playback.bottom() - 18.0;
        let add = Rect::from_center_size(pos2(playback.left() + 20.0, row_y), vec2(30.0, 30.0));
        let volume = Rect::from_center_size(pos2(playback.right() - 20.0, row_y), vec2(30.0, 30.0));
        let (rail_inset, time_inset) = (84.0, 44.0);
        Self {
            cover,
            labels,
            playback,
            transport,
            add,
            volume,
            compactness: if compact { 1.0 } else { 0.0 },
            extras,
            timeline: Rect::from_min_size(
                pos2(playback.left() + rail_inset, row_y - 2.0),
                vec2(span - rail_inset * 2.0, 4.0),
            ),
            current_time: pos2(playback.left() + time_inset, row_y),
            total_time: pos2(playback.right() - time_inset, row_y),
        }
    }
}

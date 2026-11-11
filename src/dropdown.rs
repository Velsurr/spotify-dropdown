//! Pure animation and native hit-testing rules; no background work here.
use std::time::{Duration, Instant};

const SLIDE_TIME: Duration = Duration::from_millis(230);

pub struct Slide {
    from: f32,
    target: f32,
    started: Instant,
}

impl Slide {
    pub fn open() -> Self {
        Self {
            from: 1.0,
            target: 1.0,
            started: Instant::now(),
        }
    }

    pub fn value(&self, now: Instant) -> f32 {
        let t = (now.saturating_duration_since(self.started).as_secs_f32()
            / SLIDE_TIME.as_secs_f32())
        .min(1.0);
        self.from + (self.target - self.from) * (1.0 - (1.0 - t).powi(3))
    }

    pub fn target(&mut self, open: bool, now: Instant) {
        let target = if open { 1.0 } else { 0.0 };
        if self.target != target {
            self.from = self.value(now);
            self.target = target;
            self.started = now;
        }
    }

    pub fn moving(&self, now: Instant) -> bool {
        self.from != self.target && now.saturating_duration_since(self.started) < SLIDE_TIME
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Client,
    Caption,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub fn hit_test(x: f32, y: f32, width: f32, height: f32, dropdown: bool) -> Hit {
    if dropdown {
        if x < 14.0 && y >= height - 14.0 {
            return Hit::BottomLeft;
        }
        if x >= width - 14.0 && y >= height - 14.0 {
            return Hit::BottomRight;
        }
    } else {
        match (x < 7.0, x >= width - 7.0, y < 7.0, y >= height - 7.0) {
            (true, _, true, _) => return Hit::TopLeft,
            (_, true, true, _) => return Hit::TopRight,
            (true, _, _, true) => return Hit::BottomLeft,
            (_, true, _, true) => return Hit::BottomRight,
            (true, _, _, _) => return Hit::Left,
            (_, true, _, _) => return Hit::Right,
            (_, _, true, _) => return Hit::Top,
            (_, _, _, true) => return Hit::Bottom,
            _ => {}
        }
    }
    if (8.0..54.0).contains(&y) && x < width - 60.0 {
        Hit::Caption
    } else {
        Hit::Client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropdown_only_resizes_from_bottom_corners() {
        for x in [0.0, 100.0, 379.0, 759.0] {
            assert_eq!(hit_test(x, 0.0, 760.0, 340.0, true), Hit::Client);
        }
        assert_eq!(hit_test(0.0, 160.0, 760.0, 340.0, true), Hit::Client);
        assert_eq!(hit_test(759.0, 160.0, 760.0, 340.0, true), Hit::Client);
        assert_eq!(hit_test(200.0, 339.0, 760.0, 340.0, true), Hit::Client);
        assert_eq!(hit_test(0.0, 339.0, 760.0, 340.0, true), Hit::BottomLeft);
        assert_eq!(hit_test(759.0, 339.0, 760.0, 340.0, true), Hit::BottomRight);
        assert_eq!(hit_test(80.0, 26.0, 760.0, 340.0, true), Hit::Caption);
    }

    #[test]
    fn free_window_keeps_all_resize_edges() {
        for (x, y, expected) in [
            (0.0, 0.0, Hit::TopLeft),
            (200.0, 0.0, Hit::Top),
            (429.0, 0.0, Hit::TopRight),
            (0.0, 200.0, Hit::Left),
            (429.0, 200.0, Hit::Right),
            (0.0, 583.0, Hit::BottomLeft),
            (200.0, 583.0, Hit::Bottom),
            (429.0, 583.0, Hit::BottomRight),
        ] {
            assert_eq!(hit_test(x, y, 430.0, 584.0, false), expected);
        }
    }

    #[test]
    fn slide_reverses_without_jumping_and_finishes() {
        let now = Instant::now();
        let mut slide = Slide::open();
        slide.target(false, now);
        let midway = now + Duration::from_millis(90);
        let value = slide.value(midway);
        assert!(value > 0.0 && value < 1.0);
        slide.target(true, midway);
        assert!((slide.value(midway) - value).abs() < f32::EPSILON);
        assert_eq!(slide.value(midway + SLIDE_TIME), 1.0);
        assert!(!slide.moving(midway + SLIDE_TIME));
        slide.target(false, midway + SLIDE_TIME);
        assert_eq!(slide.value(midway + SLIDE_TIME * 2), 0.0);
    }
}

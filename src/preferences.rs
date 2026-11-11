//! Portable settings and window geometry. Coordinates below are physical Windows pixels.
use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};

pub const MIN_WIDTH: f32 = 360.0;
pub const MIN_HEIGHT: f32 = 240.0;
pub const TRIGGER_HEIGHT: i32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Free,
    Dropdown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Alignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Preferences {
    pub mode: Mode,
    pub alignment: Alignment,
    pub monitor: String,
    pub free_size: [f32; 2],
    pub dropdown_size: [f32; 2],
    pub free_position: Option<[i32; 2]>,
    pub auto_hide: bool,
    pub hide_delay_ms: u64,
    pub always_on_top: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            mode: Mode::Free,
            alignment: Alignment::Center,
            monitor: String::new(),
            free_size: [430.0, 584.0],
            dropdown_size: [760.0, 340.0],
            free_position: None,
            auto_hide: true,
            hide_delay_ms: 800,
            always_on_top: false,
        }
    }
}

impl Preferences {
    pub fn path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.join("settings.ini")))
            .unwrap_or_else(|| PathBuf::from("settings.ini"))
    }

    pub fn load() -> Self {
        fs::read_to_string(Self::path())
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }

    pub fn size(&self) -> [f32; 2] {
        match self.mode {
            Mode::Free => self.free_size,
            Mode::Dropdown => self.dropdown_size,
        }
    }

    pub fn set_size(&mut self, size: [f32; 2]) {
        let size = [
            size[0].clamp(MIN_WIDTH, 1400.0),
            size[1].clamp(MIN_HEIGHT, 900.0),
        ];
        match self.mode {
            Mode::Free => self.free_size = size,
            Mode::Dropdown => self.dropdown_size = size,
        }
    }

    pub fn parse(text: &str) -> Self {
        let mut prefs = Self::default();
        let mut x = None;
        let mut y = None;
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            let dimension = |fallback: f32, min: f32, max: f32| {
                value
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite())
                    .map(|v| v.clamp(min, max))
                    .unwrap_or(fallback)
            };
            match key.trim() {
                "mode" => {
                    prefs.mode = if value == "dropdown" {
                        Mode::Dropdown
                    } else {
                        Mode::Free
                    }
                }
                "alignment" => {
                    prefs.alignment = match value {
                        "left" => Alignment::Left,
                        "right" => Alignment::Right,
                        _ => Alignment::Center,
                    }
                }
                "monitor" => prefs.monitor = value.chars().take(80).collect(),
                "free_width" => prefs.free_size[0] = dimension(430.0, MIN_WIDTH, 1400.0),
                "free_height" => prefs.free_size[1] = dimension(584.0, MIN_HEIGHT, 900.0),
                "dropdown_width" => prefs.dropdown_size[0] = dimension(760.0, MIN_WIDTH, 1400.0),
                "dropdown_height" => prefs.dropdown_size[1] = dimension(340.0, MIN_HEIGHT, 900.0),
                "free_x" => {
                    x = value
                        .parse::<i32>()
                        .ok()
                        .filter(|v| v.abs_diff(0) < 100_000)
                }
                "free_y" => {
                    y = value
                        .parse::<i32>()
                        .ok()
                        .filter(|v| v.abs_diff(0) < 100_000)
                }
                "auto_hide" => prefs.auto_hide = value != "false",
                "hide_delay_ms" => {
                    prefs.hide_delay_ms = value
                        .parse::<u64>()
                        .map(|v| v.clamp(100, 5000))
                        .unwrap_or(800)
                }
                "always_on_top" => prefs.always_on_top = value == "true",
                _ => {}
            }
        }
        prefs.free_position = x.zip(y).map(|(x, y)| [x, y]);
        prefs
    }

    pub fn serialize(&self) -> String {
        let mode = match self.mode {
            Mode::Free => "free",
            Mode::Dropdown => "dropdown",
        };
        let alignment = match self.alignment {
            Alignment::Left => "left",
            Alignment::Center => "center",
            Alignment::Right => "right",
        };
        let mut text = format!(
            "# Spotify Dropdown — preferencias locais\nmode={mode}\nalignment={alignment}\nmonitor={}\nfree_width={}\nfree_height={}\ndropdown_width={}\ndropdown_height={}\nauto_hide={}\nhide_delay_ms={}\nalways_on_top={}\n",
            self.monitor,
            self.free_size[0],
            self.free_size[1],
            self.dropdown_size[0],
            self.dropdown_size[1],
            self.auto_hide,
            self.hide_delay_ms,
            self.always_on_top
        );
        if let Some([x, y]) = self.free_position {
            text.push_str(&format!("free_x={x}\nfree_y={y}\n"));
        }
        text
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        let temporary = path.with_extension("ini.tmp");
        fs::write(&temporary, self.serialize())?;
        fs::rename(temporary, path)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Bounds {
    pub fn contains(self, point: [i32; 2]) -> bool {
        point[0] >= self.x
            && point[0] < self.x + self.width
            && point[1] >= self.y
            && point[1] < self.y + self.height
    }

    pub fn anchored(screen: Self, size: [i32; 2], alignment: Alignment, collapsed: bool) -> Self {
        let width = size[0].clamp(1, screen.width.max(1));
        let height = if collapsed {
            TRIGGER_HEIGHT
        } else {
            size[1].clamp(1, screen.height.max(1))
        };
        let x = match alignment {
            Alignment::Left => screen.x,
            Alignment::Center => screen.x + (screen.width - width) / 2,
            Alignment::Right => screen.x + screen.width - width,
        };
        Self {
            x,
            y: screen.y,
            width,
            height,
        }
    }

    pub fn fit_inside(self, work: Self) -> Self {
        let width = self.width.clamp(1, work.width.max(1));
        let height = self.height.clamp(1, work.height.max(1));
        Self {
            x: self.x.clamp(work.x, work.x + work.width - width),
            y: self.y.clamp(work.y, work.y + work.height - height),
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeAction {
    Reveal,
    Collapse,
}

pub struct EdgeWatcher {
    pub enabled: bool,
    pub expanded: bool,
    pub auto_hide: bool,
    pub hold_open: bool,
    pub bounds: Bounds,
    pub hide_delay: Duration,
    outside_since: Option<Instant>,
}

impl Default for EdgeWatcher {
    fn default() -> Self {
        Self {
            enabled: false,
            expanded: false,
            auto_hide: false,
            hold_open: false,
            bounds: Bounds::default(),
            hide_delay: Duration::from_millis(800),
            outside_since: None,
        }
    }
}

impl EdgeWatcher {
    pub fn reset(&mut self) {
        self.outside_since = None;
    }

    pub fn poll(&mut self, point: [i32; 2], mouse_down: bool, now: Instant) -> Option<EdgeAction> {
        if !self.enabled || self.hold_open || (mouse_down && self.expanded) {
            self.reset();
            return None;
        }
        if self.bounds.contains(point) {
            self.reset();
            return (!self.expanded).then_some(EdgeAction::Reveal);
        }
        if self.expanded && self.auto_hide {
            let since = self.outside_since.get_or_insert(now);
            if now.saturating_duration_since(*since) >= self.hide_delay {
                return Some(EdgeAction::Collapse);
            }
        } else {
            self.reset();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_preserves_modes_sizes_and_negative_monitor_coordinates() {
        let prefs = Preferences {
            mode: Mode::Dropdown,
            alignment: Alignment::Right,
            monitor: "\\\\.\\DISPLAY2".into(),
            free_position: Some([-1600, -200]),
            always_on_top: true,
            ..Default::default()
        };
        assert_eq!(Preferences::parse(&prefs.serialize()), prefs);
    }

    #[test]
    fn invalid_settings_cannot_create_infinite_or_tiny_windows() {
        let prefs = Preferences::parse(
            "free_width=NaN\nfree_height=-20\ndropdown_width=99999\nfree_x=-2147483648\nfree_y=2\nmode=unknown",
        );
        assert_eq!(prefs.free_size, [430.0, MIN_HEIGHT]);
        assert_eq!(prefs.dropdown_size[0], 1400.0);
        assert!(prefs.free_position.is_none());
        assert_eq!(prefs.mode, Mode::Free);
    }

    #[test]
    fn dropdown_alignment_is_relative_to_the_selected_monitor() {
        let monitor = Bounds {
            x: -1920,
            y: -1080,
            width: 1920,
            height: 1080,
        };
        assert_eq!(
            Bounds::anchored(monitor, [760, 340], Alignment::Left, false).x,
            -1920
        );
        assert_eq!(
            Bounds::anchored(monitor, [760, 340], Alignment::Center, false).x,
            -1340
        );
        assert_eq!(
            Bounds::anchored(monitor, [760, 340], Alignment::Right, false).x,
            -760
        );
        let collapsed = Bounds::anchored(monitor, [3000, 340], Alignment::Right, true);
        assert_eq!(
            collapsed,
            Bounds {
                x: -1920,
                y: -1080,
                width: 1920,
                height: TRIGGER_HEIGHT
            }
        );
    }

    #[test]
    fn removed_monitor_position_is_clamped_back_into_a_work_area() {
        let window = Bounds {
            x: -3000,
            y: -1500,
            width: 760,
            height: 900,
        };
        let work = Bounds {
            x: 0,
            y: 0,
            width: 1366,
            height: 728,
        };
        assert_eq!(
            window.fit_inside(work),
            Bounds {
                x: 0,
                y: 0,
                width: 760,
                height: 728
            }
        );
    }

    #[test]
    fn collapsed_trigger_is_limited_to_its_monitor_and_width() {
        let now = Instant::now();
        let mut watcher = EdgeWatcher {
            enabled: true,
            bounds: Bounds {
                x: 500,
                y: 0,
                width: 760,
                height: TRIGGER_HEIGHT,
            },
            ..Default::default()
        };
        assert_eq!(watcher.poll([499, 0], false, now), None);
        assert_eq!(watcher.poll([600, TRIGGER_HEIGHT], false, now), None);
        assert_eq!(watcher.poll([600, 0], false, now), Some(EdgeAction::Reveal));
    }

    #[test]
    fn delay_is_backward_compatible_bounded_and_used_by_the_watcher() {
        assert_eq!(Preferences::parse("mode=dropdown").hide_delay_ms, 800);
        assert_eq!(Preferences::parse("hide_delay_ms=0").hide_delay_ms, 100);
        assert_eq!(
            Preferences::parse("hide_delay_ms=999999").hide_delay_ms,
            5000
        );
        assert_eq!(
            Preferences::parse("hide_delay_ms=invalid").hide_delay_ms,
            800
        );
        let prefs = Preferences::parse("hide_delay_ms=1500");
        assert_eq!(Preferences::parse(&prefs.serialize()).hide_delay_ms, 1500);
        let now = Instant::now();
        let mut watcher = EdgeWatcher {
            enabled: true,
            expanded: true,
            auto_hide: true,
            hide_delay: Duration::from_millis(prefs.hide_delay_ms),
            ..Default::default()
        };
        assert_eq!(watcher.poll([900, 500], false, now), None);
        assert_eq!(
            watcher.poll([900, 500], false, now + Duration::from_millis(1499)),
            None
        );
        assert_eq!(
            watcher.poll([900, 500], false, now + Duration::from_millis(1500)),
            Some(EdgeAction::Collapse)
        );
    }

    #[test]
    fn dropdown_waits_before_hiding_and_stays_open_for_menus_or_dragging() {
        let now = Instant::now();
        let mut watcher = EdgeWatcher {
            enabled: true,
            expanded: true,
            auto_hide: true,
            bounds: Bounds {
                x: 0,
                y: 0,
                width: 760,
                height: 340,
            },
            ..Default::default()
        };
        assert_eq!(watcher.poll([900, 500], false, now), None);
        assert_eq!(
            watcher.poll([900, 500], false, now + Duration::from_millis(799)),
            None
        );
        assert_eq!(
            watcher.poll([900, 500], false, now + Duration::from_millis(801)),
            Some(EdgeAction::Collapse)
        );
        watcher.hold_open = true;
        assert_eq!(
            watcher.poll([900, 500], false, now + Duration::from_secs(3)),
            None
        );
        watcher.hold_open = false;
        assert_eq!(
            watcher.poll([900, 500], true, now + Duration::from_secs(5)),
            None
        );
        assert_eq!(
            watcher.poll([900, 500], false, now + Duration::from_secs(5)),
            None
        );
    }
}

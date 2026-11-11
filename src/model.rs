use std::time::{Duration, Instant};

pub const TICKS_PER_SECOND: i64 = 10_000_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Playback {
    Playing,
    Paused,
    #[default]
    Stopped,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Capabilities {
    pub previous: bool,
    pub play: bool,
    pub pause: bool,
    pub toggle: bool,
    pub next: bool,
}

impl Capabilities {
    pub fn can_toggle(self, playback: Playback) -> bool {
        self.toggle
            || match playback {
                Playback::Playing => self.pause,
                Playback::Paused | Playback::Stopped => self.play,
            }
    }
}

#[derive(Clone, Debug)]
pub struct Timeline {
    pub duration: Duration,
    pub position: Duration,
    pub sampled_at: Instant,
    pub advancing: bool,
}

impl Timeline {
    /// GSMTC timestamps are 100 ns ticks. A zero/invalid range is not a real timeline.
    pub fn from_ticks(start: i64, end: i64, position: i64, advancing: bool) -> Option<Self> {
        let length = end.checked_sub(start)?;
        if length <= 0 {
            return None;
        }
        let relative = position.saturating_sub(start).clamp(0, length);
        Some(Self {
            duration: Duration::from_secs_f64(length as f64 / TICKS_PER_SECOND as f64),
            position: Duration::from_secs_f64(relative as f64 / TICKS_PER_SECOND as f64),
            sampled_at: Instant::now(),
            advancing,
        })
    }

    pub fn position_at(&self, now: Instant) -> Duration {
        let elapsed = if self.advancing {
            now.saturating_duration_since(self.sampled_at)
        } else {
            Duration::ZERO
        };
        self.position.saturating_add(elapsed).min(self.duration)
    }
}

pub fn is_spotify_source(source: &str) -> bool {
    let source = source.to_ascii_lowercase();
    matches!(source.as_str(), "spotify.exe" | "spotify")
        || (source.starts_with("spotifyab.spotifymusic_") && source.ends_with("!spotify"))
        || source == "com.squirrel.spotify.spotify"
}

pub fn format_time(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_desktop_and_store_without_matching_other_apps() {
        for source in [
            "Spotify.exe",
            "SPOTIFY",
            "SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify",
        ] {
            assert!(is_spotify_source(source), "{source}");
        }
        for source in [
            "chrome.exe",
            "msedge.exe",
            "notspotify.exe",
            "Spotify Helper.exe",
            "SpotifyAB.Other_123!Spotify",
            "",
        ] {
            assert!(!is_spotify_source(source), "{source}");
        }
    }

    #[test]
    fn invalid_or_overflowing_timeline_is_unavailable() {
        assert!(Timeline::from_ticks(0, 0, 0, true).is_none());
        assert!(Timeline::from_ticks(20, 10, 15, false).is_none());
        assert!(Timeline::from_ticks(i64::MIN, i64::MAX, 0, false).is_none());
    }

    #[test]
    fn timeline_respects_origin_pause_and_track_end() {
        let tick = TICKS_PER_SECOND;
        let mut timeline = Timeline::from_ticks(10 * tick, 190 * tick, 30 * tick, true).unwrap();
        assert_eq!(timeline.duration, Duration::from_secs(180));
        assert_eq!(timeline.position, Duration::from_secs(20));
        let later = timeline.sampled_at + Duration::from_secs(200);
        assert_eq!(timeline.position_at(later), Duration::from_secs(180));
        timeline.advancing = false;
        assert_eq!(timeline.position_at(later), Duration::from_secs(20));
    }

    #[test]
    fn out_of_range_positions_are_clamped() {
        assert_eq!(
            Timeline::from_ticks(100, 200, 0, false).unwrap().position,
            Duration::ZERO
        );
        let timeline = Timeline::from_ticks(0, TICKS_PER_SECOND, i64::MAX, false).unwrap();
        assert_eq!(timeline.position, timeline.duration);
    }

    #[test]
    fn play_pause_fallback_obeys_capabilities_and_state() {
        let play_only = Capabilities {
            play: true,
            ..Default::default()
        };
        assert!(play_only.can_toggle(Playback::Paused));
        assert!(!play_only.can_toggle(Playback::Playing));
        assert!(!Capabilities::default().can_toggle(Playback::Stopped));
    }

    #[test]
    fn time_format_supports_long_tracks() {
        assert_eq!(format_time(Duration::ZERO), "0:00");
        assert_eq!(format_time(Duration::from_secs(192)), "3:12");
        assert_eq!(format_time(Duration::from_secs(3661)), "61:01");
    }
}

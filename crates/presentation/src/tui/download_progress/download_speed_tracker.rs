use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use localnar_domain::ByteLength;

/// Tracks download transfer rates by sampling byte progress over time.
///
/// Keeps a bounded history of recent per-sample rates so a smoothed rate can be
/// derived. The smoothed rate absorbs the burst-and-pause jitter that a raw
/// instantaneous rate exhibits on real networks, which is what keeps a derived
/// ETA from swinging wildly between refreshes.
#[derive(Debug, Clone)]
pub struct DownloadSpeedTracker {
    last_sample_instant: Option<Instant>,
    last_transferred_bytes: u64,
    current_speed_bytes_per_second: u64,
    recent_speeds: VecDeque<u64>,
    recent_speeds_capacity: usize,
}

impl DownloadSpeedTracker {
    pub const MINIMUM_SAMPLE_INTERVAL: Duration = Duration::from_millis(250);
    pub const DEFAULT_SMOOTHING_WINDOW: usize = 16;

    /// Builds a new speed tracker with zero initial rate and the default
    /// smoothing window.
    pub fn new() -> Self {
        Self::with_smoothing_window(Self::DEFAULT_SMOOTHING_WINDOW)
    }

    /// Builds a new speed tracker that averages the last `window` sampled rates.
    ///
    /// A larger window smooths more aggressively at the cost of reacting more
    /// slowly to a genuine sustained change in transfer rate. The window is
    /// clamped to at least one so a smoothed rate is always defined.
    pub fn with_smoothing_window(window: usize) -> Self {
        let capacity = window.max(1);
        Self {
            last_sample_instant: None,
            last_transferred_bytes: 0,
            current_speed_bytes_per_second: 0,
            recent_speeds: VecDeque::with_capacity(capacity),
            recent_speeds_capacity: capacity,
        }
    }

    /// Resets the tracker state for a new transfer.
    pub fn reset(&mut self) {
        self.last_sample_instant = None;
        self.last_transferred_bytes = 0;
        self.current_speed_bytes_per_second = 0;
        self.recent_speeds.clear();
    }

    /// Records a progress sample and updates the computed speed if the sample window elapsed.
    pub fn record_sample(&mut self, transferred_bytes: u64, now: Instant) -> ByteLength {
        let Some(last_instant) = self.last_sample_instant else {
            self.last_sample_instant = Some(now);
            self.last_transferred_bytes = transferred_bytes;
            return ByteLength::new(self.current_speed_bytes_per_second);
        };

        let elapsed = now.saturating_duration_since(last_instant);
        if elapsed < Self::MINIMUM_SAMPLE_INTERVAL {
            return ByteLength::new(self.current_speed_bytes_per_second);
        }

        let delta_bytes = transferred_bytes.saturating_sub(self.last_transferred_bytes);
        let elapsed_seconds = elapsed.as_secs_f64();
        if elapsed_seconds > 0.0 {
            self.current_speed_bytes_per_second = (delta_bytes as f64 / elapsed_seconds) as u64;
            self.push_recent_speed(self.current_speed_bytes_per_second);
        }

        self.last_sample_instant = Some(now);
        self.last_transferred_bytes = transferred_bytes;

        ByteLength::new(self.current_speed_bytes_per_second)
    }

    /// Returns the most recently computed instantaneous transfer rate.
    pub fn current_speed(&self) -> ByteLength {
        ByteLength::new(self.current_speed_bytes_per_second)
    }

    /// Returns the smoothed transfer rate averaged over the recent sample window.
    ///
    /// Falls back to the instantaneous rate until at least one rate has been
    /// sampled, so an ETA can still be derived from the very first update.
    pub fn smoothed_speed(&self) -> ByteLength {
        if self.recent_speeds.is_empty() {
            return ByteLength::new(self.current_speed_bytes_per_second);
        }

        let total: u128 = self
            .recent_speeds
            .iter()
            .map(|rate| u128::from(*rate))
            .sum();
        let average = total / self.recent_speeds.len() as u128;
        ByteLength::new(average as u64)
    }

    fn push_recent_speed(&mut self, speed_bytes_per_second: u64) {
        if self.recent_speeds.len() == self.recent_speeds_capacity {
            self.recent_speeds.pop_front();
        }
        self.recent_speeds.push_back(speed_bytes_per_second);
    }
}

impl Default for DownloadSpeedTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_has_zero_speed() {
        let tracker = DownloadSpeedTracker::new();
        assert_eq!(tracker.current_speed(), ByteLength::ZERO);
    }

    #[test]
    fn first_sample_establishes_baseline_without_speed() {
        let mut tracker = DownloadSpeedTracker::new();
        let start = Instant::now();

        let speed = tracker.record_sample(1024, start);
        assert_eq!(speed, ByteLength::ZERO);
    }

    #[test]
    fn sample_within_minimum_interval_preserves_previous_speed() {
        let mut tracker = DownloadSpeedTracker::new();
        let start = Instant::now();

        tracker.record_sample(0, start);
        let intermediate = start + Duration::from_millis(100);
        let speed = tracker.record_sample(500_000, intermediate);

        assert_eq!(speed, ByteLength::ZERO);
    }

    #[test]
    fn sample_after_interval_calculates_rate_correctly() {
        let mut tracker = DownloadSpeedTracker::new();
        let start = Instant::now();

        tracker.record_sample(0, start);
        let one_second_later = start + Duration::from_secs(1);
        let speed = tracker.record_sample(10 * 1024 * 1024, one_second_later);

        assert_eq!(speed, ByteLength::new(10 * 1024 * 1024));
    }

    #[test]
    fn reset_clears_tracked_speed_and_state() {
        let mut tracker = DownloadSpeedTracker::new();
        let start = Instant::now();

        tracker.record_sample(0, start);
        tracker.record_sample(10 * 1024 * 1024, start + Duration::from_secs(1));
        assert_eq!(tracker.current_speed(), ByteLength::new(10 * 1024 * 1024));

        tracker.reset();
        assert_eq!(tracker.current_speed(), ByteLength::ZERO);
    }

    #[test]
    fn smoothed_speed_defaults_to_instantaneous_before_any_sample() {
        let tracker = DownloadSpeedTracker::new();

        assert_eq!(tracker.smoothed_speed(), ByteLength::ZERO);
    }

    #[test]
    fn smoothed_speed_averages_recent_samples() {
        let mut tracker = DownloadSpeedTracker::with_smoothing_window(4);
        let start = Instant::now();

        tracker.record_sample(0, start);
        tracker.record_sample(1_000_000, start + Duration::from_secs(1));
        tracker.record_sample(4_000_000, start + Duration::from_secs(2));

        assert_eq!(tracker.smoothed_speed(), ByteLength::new(2_000_000));
    }

    #[test]
    fn smoothed_speed_absorbs_a_single_spike() {
        let mut tracker = DownloadSpeedTracker::with_smoothing_window(4);
        let start = Instant::now();

        tracker.record_sample(0, start);
        tracker.record_sample(1_000_000, start + Duration::from_secs(1));
        tracker.record_sample(2_000_000, start + Duration::from_secs(2));
        tracker.record_sample(21_000_000, start + Duration::from_secs(3));

        assert_eq!(tracker.current_speed(), ByteLength::new(19_000_000));
        assert_eq!(tracker.smoothed_speed(), ByteLength::new(7_000_000));
    }

    #[test]
    fn smoothed_speed_window_drops_oldest_samples() {
        let mut tracker = DownloadSpeedTracker::with_smoothing_window(2);
        let start = Instant::now();

        tracker.record_sample(0, start);
        tracker.record_sample(1_000_000, start + Duration::from_secs(1));
        tracker.record_sample(3_000_000, start + Duration::from_secs(2));
        tracker.record_sample(9_000_000, start + Duration::from_secs(3));

        assert_eq!(tracker.smoothed_speed(), ByteLength::new(4_000_000));
    }

    #[test]
    fn reset_clears_smoothed_history() {
        let mut tracker = DownloadSpeedTracker::new();
        let start = Instant::now();

        tracker.record_sample(0, start);
        tracker.record_sample(5_000_000, start + Duration::from_secs(1));
        assert_eq!(tracker.smoothed_speed(), ByteLength::new(5_000_000));

        tracker.reset();
        assert_eq!(tracker.smoothed_speed(), ByteLength::ZERO);
    }
}

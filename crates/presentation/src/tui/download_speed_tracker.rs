use std::time::{Duration, Instant};

use localnar_domain::ByteLength;

/// Tracks download transfer rates by sampling byte progress over time.
#[derive(Debug, Clone)]
pub struct DownloadSpeedTracker {
    last_sample_instant: Option<Instant>,
    last_transferred_bytes: u64,
    current_speed_bytes_per_second: u64,
}

impl DownloadSpeedTracker {
    pub const MINIMUM_SAMPLE_INTERVAL: Duration = Duration::from_millis(250);

    /// Builds a new speed tracker with zero initial rate.
    pub fn new() -> Self {
        Self {
            last_sample_instant: None,
            last_transferred_bytes: 0,
            current_speed_bytes_per_second: 0,
        }
    }

    /// Resets the tracker state for a new transfer.
    pub fn reset(&mut self) {
        self.last_sample_instant = None;
        self.last_transferred_bytes = 0;
        self.current_speed_bytes_per_second = 0;
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
        }

        self.last_sample_instant = Some(now);
        self.last_transferred_bytes = transferred_bytes;

        ByteLength::new(self.current_speed_bytes_per_second)
    }

    /// Returns the most recently computed transfer rate.
    pub fn current_speed(&self) -> ByteLength {
        ByteLength::new(self.current_speed_bytes_per_second)
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
}

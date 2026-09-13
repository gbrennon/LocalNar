use std::time::Instant;

use localnar_domain::ByteLength;
use localnar_infrastructure::adapters::{ProgressBus, ProgressEvent};
use tokio::sync::mpsc;

use crate::tui::{AppEvent, DownloadSpeedTracker};

/// Bridge between infrastructure progress bus and TUI event channel.
/// Subscribes to infrastructure ProgressEvent and converts to TUI AppEvent.
pub struct ProgressReporterBridge {
    _receiver_handle: tokio::task::JoinHandle<()>,
}

impl ProgressReporterBridge {
    /// Create a new bridge connecting the infrastructure progress bus to the TUI event channel.
    pub fn new(bus: &ProgressBus, sender: mpsc::UnboundedSender<AppEvent>) -> Self {
        let mut receiver = bus.subscribe();

        let receiver_handle = tokio::spawn(async move {
            let mut speed_tracker = DownloadSpeedTracker::new();
            while let Ok(event) = receiver.recv().await {
                let app_event = Self::convert_event(event, &mut speed_tracker, Instant::now());
                if sender.send(app_event).is_err() {
                    break;
                }
            }
        });

        Self {
            _receiver_handle: receiver_handle,
        }
    }

    /// Converts an infrastructure progress event to an application event.
    pub fn convert_event(
        event: ProgressEvent,
        tracker: &mut DownloadSpeedTracker,
        now: Instant,
    ) -> AppEvent {
        match event {
            ProgressEvent::Started { total } => {
                tracker.reset();
                let message = format!("Downloading: 0 B / {} (0.0%)", ByteLength::new(total));
                AppEvent::InstallProgress(0.0, message)
            }
            ProgressEvent::Advanced { transferred, total } => {
                let speed = tracker.record_sample(transferred, now);
                let percentage = if total > 0 {
                    (transferred as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                let ratio = percentage / 100.0;
                let message = if speed.bytes() > 0 {
                    format!(
                        "Downloading: {} / {} ({:.1}%) @ {}/s · ETA {}",
                        ByteLength::new(transferred),
                        ByteLength::new(total),
                        percentage,
                        speed,
                        Self::format_eta(transferred, total, speed.bytes())
                    )
                } else {
                    format!(
                        "Downloading: {} / {} ({:.1}%) · ETA calculating...",
                        ByteLength::new(transferred),
                        ByteLength::new(total),
                        percentage
                    )
                };
                AppEvent::InstallProgress(ratio, message)
            }
            ProgressEvent::Finished => {
                tracker.reset();
                AppEvent::InstallProgress(1.0, "Download completed".to_owned())
            }
        }
    }

    fn format_eta(transferred: u64, total: u64, speed: u64) -> String {
        if speed == 0 {
            return "calculating...".to_owned();
        }

        let remaining = total.saturating_sub(transferred);
        let seconds = remaining / speed + u64::from(!remaining.is_multiple_of(speed));
        let hours = seconds / 3_600;
        let minutes = (seconds % 3_600) / 60;
        let seconds = seconds % 60;

        if hours > 0 {
            return format!("{hours}h {minutes}m {seconds}s");
        }
        if minutes > 0 {
            return format!("{minutes}m {seconds}s");
        }
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use localnar_infrastructure::adapters::ProgressEvent;

    use super::*;

    #[test]
    fn started_event_reports_zero_progress_with_total_size() {
        let mut tracker = DownloadSpeedTracker::new();
        let event = ProgressEvent::Started { total: 10_000_000 };
        let app_event = ProgressReporterBridge::convert_event(event, &mut tracker, Instant::now());

        match app_event {
            AppEvent::InstallProgress(ratio, message) => {
                assert_eq!(ratio, 0.0);
                assert!(message.contains("0 B / 9.5 MiB (0.0%)"));
            }
            _ => panic!("expected InstallProgress event"),
        }
    }

    #[test]
    fn advanced_event_with_elapsed_time_reports_download_speed_rate() {
        let mut tracker = DownloadSpeedTracker::new();
        let start = Instant::now();

        ProgressReporterBridge::convert_event(
            ProgressEvent::Advanced {
                transferred: 0,
                total: 100_000_000,
            },
            &mut tracker,
            start,
        );

        let one_second_later = start + Duration::from_secs(1);
        let app_event = ProgressReporterBridge::convert_event(
            ProgressEvent::Advanced {
                transferred: 20_000_000,
                total: 100_000_000,
            },
            &mut tracker,
            one_second_later,
        );

        match app_event {
            AppEvent::InstallProgress(ratio, message) => {
                assert_eq!(ratio, 0.2);
                assert!(message.contains("19.1 MiB / 95.4 MiB (20.0%) @ 19.1 MiB/s · ETA 4s"));
            }
            _ => panic!("expected InstallProgress event with speed"),
        }
    }

    #[test]
    fn advanced_event_without_speed_reports_calculating_eta() {
        let mut tracker = DownloadSpeedTracker::new();
        let app_event = ProgressReporterBridge::convert_event(
            ProgressEvent::Advanced {
                transferred: 0,
                total: 100_000_000,
            },
            &mut tracker,
            Instant::now(),
        );

        match app_event {
            AppEvent::InstallProgress(_, message) => {
                assert!(message.contains("ETA calculating..."));
            }
            _ => panic!("expected InstallProgress event"),
        }
    }

    #[test]
    fn eta_uses_compact_duration_units() {
        assert_eq!(
            ProgressReporterBridge::format_eta(0, 0, 0),
            "calculating..."
        );
        assert_eq!(ProgressReporterBridge::format_eta(0, 5, 1), "5s");
        assert_eq!(ProgressReporterBridge::format_eta(0, 125, 1), "2m 5s");
        assert_eq!(ProgressReporterBridge::format_eta(0, 3_725, 1), "1h 2m 5s");
    }

    #[test]
    fn finished_event_reports_completion() {
        let mut tracker = DownloadSpeedTracker::new();
        let event = ProgressEvent::Finished;
        let app_event = ProgressReporterBridge::convert_event(event, &mut tracker, Instant::now());

        match app_event {
            AppEvent::InstallProgress(ratio, message) => {
                assert_eq!(ratio, 1.0);
                assert_eq!(message, "Download completed");
            }
            _ => panic!("expected InstallProgress event"),
        }
    }
}

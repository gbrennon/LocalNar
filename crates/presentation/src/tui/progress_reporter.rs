use domain::ByteLength;
use infrastructure::adapters::{ProgressBus, ProgressEvent};
use tokio::sync::mpsc;

use crate::tui::app_event::AppEvent;

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
            while let Ok(event) = receiver.recv().await {
                let app_event = Self::convert_event(event);
                if sender.send(app_event).is_err() {
                    break;
                }
            }
        });

        Self {
            _receiver_handle: receiver_handle,
        }
    }

    /// Convert infrastructure ProgressEvent to TUI AppEvent.
    fn convert_event(event: ProgressEvent) -> AppEvent {
        let (transferred, total, percentage) = match event {
            ProgressEvent::Started { total } => (0, total, 0.0),
            ProgressEvent::Advanced { transferred, total } => {
                let pct = if total > 0 {
                    (transferred as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                (transferred, total, pct)
            }
            ProgressEvent::Finished => (0, 0, 100.0),
        };

        let ratio = percentage / 100.0;
        let message = format!(
            "Downloading: {} / {} ({:.1}%)",
            ByteLength::new(transferred),
            ByteLength::new(total),
            percentage
        );
        AppEvent::InstallProgress(ratio, message)
    }
}

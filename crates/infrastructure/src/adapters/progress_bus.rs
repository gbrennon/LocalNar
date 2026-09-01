use localnar_application::ports::outbound::download_progress::DownloadProgress;
use tokio::sync::broadcast;

/// Progress event emitted by the progress bus.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// Download started with total size
    Started { total: u64 },
    /// Download advanced with transferred and total bytes
    Advanced { transferred: u64, total: u64 },
    /// Download finished
    Finished,
}

/// A message bus for broadcasting progress events to multiple subscribers.
/// Uses a broadcast channel for fan-out to multiple receivers.
pub struct ProgressBus {
    sender: broadcast::Sender<ProgressEvent>,
}

impl ProgressBus {
    /// Create a new progress bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _receiver) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to progress events.
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressEvent> {
        self.sender.subscribe()
    }

    /// Emit a progress event to all subscribers.
    pub fn emit(&self, event: ProgressEvent) {
        let _ = self.sender.send(event);
    }

    /// Get a sender for use with DownloadProgressPort.
    pub fn sender(&self) -> ProgressBusSender {
        ProgressBusSender {
            sender: self.sender.clone(),
        }
    }
}

/// Sender type for implementing DownloadProgressPort.
#[derive(Clone)]
pub struct ProgressBusSender {
    sender: broadcast::Sender<ProgressEvent>,
}

impl ProgressBusSender {
    /// Create a new sender from a broadcast sender.
    pub fn new(sender: broadcast::Sender<ProgressEvent>) -> Self {
        Self { sender }
    }
}

impl localnar_application::ports::outbound::download_progress_port::DownloadProgressPort
    for ProgressBusSender
{
    fn report(&self, progress: DownloadProgress) {
        let event = match progress {
            DownloadProgress::Started { total } => ProgressEvent::Started {
                total: total.bytes(),
            },
            DownloadProgress::Advanced { transferred, total } => ProgressEvent::Advanced {
                transferred: transferred.bytes(),
                total: total.bytes(),
            },
            DownloadProgress::Finished => ProgressEvent::Finished,
        };
        let _ = self.sender.send(event);
    }
}

#[cfg(test)]
mod tests {
    use localnar_application::ports::outbound::{
        download_progress::DownloadProgress, download_progress_port::DownloadProgressPort,
    };
    use localnar_domain::ByteLength;

    use super::*;

    #[tokio::test]
    async fn test_progress_bus_broadcast() {
        let bus = ProgressBus::new(16);
        let mut receiver = bus.subscribe();

        bus.emit(ProgressEvent::Started { total: 1000 });
        let event = receiver.recv().await.expect("should receive event");
        assert!(matches!(event, ProgressEvent::Started { total: 1000 }));

        bus.emit(ProgressEvent::Advanced {
            transferred: 500,
            total: 1000,
        });
        let event = receiver.recv().await.expect("should receive event");
        assert!(matches!(
            event,
            ProgressEvent::Advanced {
                transferred: 500,
                total: 1000
            }
        ));
    }

    #[tokio::test]
    async fn test_sender_port_implementation() {
        let bus = ProgressBus::new(16);
        let sender = bus.sender();
        let mut receiver = bus.subscribe();

        sender.report(DownloadProgress::Started {
            total: ByteLength::new(1000),
        });
        let event = receiver.recv().await.expect("should receive event");
        assert!(matches!(event, ProgressEvent::Started { total: 1000 }));

        sender.report(DownloadProgress::Advanced {
            transferred: ByteLength::new(500),
            total: ByteLength::new(1000),
        });
        let event = receiver.recv().await.expect("should receive event");
        assert!(matches!(
            event,
            ProgressEvent::Advanced {
                transferred: 500,
                total: 1000
            }
        ));
    }
}

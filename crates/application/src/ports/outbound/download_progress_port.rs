use crate::ports::outbound::download_progress::DownloadProgress;

/// Outbound contract for observing how a transfer is advancing.
///
/// The method is deliberately synchronous and returns nothing: reporting must
/// never block the transfer, so an adapter is expected to overwrite a shared
/// snapshot or push into an unbounded channel rather than await anything.
pub trait DownloadProgressPort: Send + Sync {
    /// Records the latest state of the transfer in flight.
    fn report(&self, progress: DownloadProgress);
}

impl<Port> DownloadProgressPort for &Port
where
    Port: DownloadProgressPort + ?Sized,
{
    fn report(&self, progress: DownloadProgress) {
        (**self).report(progress);
    }
}

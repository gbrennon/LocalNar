use domain::{ModelArtifact, RemoteModelFile};

use crate::errors::model_download_error::ModelDownloadError;
use crate::ports::outbound::download_progress_port::DownloadProgressPort;

/// Outbound contract for transmitting the bytes of a resolved remote file.
///
/// The outcome is a staged `ModelArtifact` whose bytes have not yet been
/// committed to the durable library; verification and storage are separate
/// ports so infrastructure stays swappable.
pub trait ModelDownloaderPort: Send + Sync {
    /// Transfers `remote` into a freshly created, staged artifact.
    ///
    /// The adapter reports to `progress` as bytes land. It may report as often
    /// as it likes; throttling is the consumer's concern.
    async fn fetch<Progress>(
        &self,
        remote: &RemoteModelFile,
        progress: &Progress,
    ) -> Result<ModelArtifact, ModelDownloadError>
    where
        Progress: DownloadProgressPort;
}

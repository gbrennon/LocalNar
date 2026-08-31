use domain::{ModelArtifact, RemoteModelFile};

use crate::{
    errors::model_download_error::ModelDownloadError,
    ports::outbound::download_progress_port::DownloadProgressPort,
};

/// Outbound contract for transmitting the bytes of a resolved remote file.
///
/// The outcome is a staged `ModelArtifact` whose bytes have not yet been
/// committed to the durable library; verification and storage are separate
/// ports so infrastructure stays swappable.
pub trait ModelDownloaderPort: Send + Sync {
    async fn fetch(
        &self,
        remote: &RemoteModelFile,
        progress: &dyn DownloadProgressPort,
    ) -> Result<ModelArtifact, ModelDownloadError>;
}

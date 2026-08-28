use async_trait::async_trait;

use crate::model_artifact::ModelArtifact;
use crate::ports::model_download_error::ModelDownloadError;
use crate::remote_model_file::RemoteModelFile;

/// Contract for transmitting the bytes of a resolved remote file.
///
/// The outcome is a staged `ModelArtifact` whose bytes have not yet been
/// committed to the durable library; verification and storage are separate
/// ports so infrastructure stays swappable.
#[async_trait]
pub trait ModelDownloader: Send + Sync {
    /// Transfers `remote` into a freshly created, staged artifact.
    async fn fetch(&self, remote: &RemoteModelFile) -> Result<ModelArtifact, ModelDownloadError>;
}

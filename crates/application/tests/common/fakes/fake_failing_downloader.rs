#![allow(dead_code)]
use localnar_application::{
    errors::ModelDownloadError,
    ports::outbound::{DownloadProgressPort, ModelDownloaderPort},
};
use localnar_domain::{ModelArtifact, RemoteModelFile};

/// A downloader whose transport always breaks mid transfer.
pub struct FakeFailingDownloader;

impl FakeFailingDownloader {
    /// The error every transfer attempt produces.
    pub fn error() -> ModelDownloadError {
        ModelDownloadError::Transport {
            file: "Qwen3-8B-Q4_K_M.gguf".to_string(),
            cause: "stream closed".to_string(),
        }
    }
}

impl ModelDownloaderPort for FakeFailingDownloader {
    async fn fetch(
        &self,
        _remote: &RemoteModelFile,
        _progress: &dyn DownloadProgressPort,
    ) -> Result<ModelArtifact, ModelDownloadError> {
        Err(Self::error())
    }
}

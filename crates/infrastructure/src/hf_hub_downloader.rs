use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use application::errors::ModelDownloadError;
use application::ports::outbound::{DownloadProgress, DownloadProgressPort, ModelDownloaderPort};
use domain::{ByteLength, ModelArtifact, RemoteModelFile};
use hf_hub::api::tokio::{ApiBuilder, Progress as HfProgress};
use hf_hub::{Repo, RepoType};
use tokio::sync::mpsc;

const DEFAULT_ENDPOINT: &str = "https://huggingface.co";

/// Model downloader using Hugging Face Hub tokio client.
#[derive(Debug, Clone)]
pub struct HfHubDownloader {
    staging_dir: PathBuf,
    endpoint: String,
    token: Option<String>,
}

impl HfHubDownloader {
    /// Builds a downloader that stages files under `staging_dir`.
    pub fn new(staging_dir: impl Into<PathBuf>) -> Self {
        Self {
            staging_dir: staging_dir.into(),
            endpoint: DEFAULT_ENDPOINT.to_string(),
            token: None,
        }
    }

    /// Resolves configuration from environment variables.
    pub fn from_env() -> Self {
        let staging_dir = std::env::var("BARE_AI_STAGING_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_staging_dir());
        let endpoint =
            std::env::var("HF_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        let token = std::env::var("HF_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty());

        Self {
            staging_dir,
            endpoint,
            token,
        }
    }

    /// Overrides the staging directory.
    pub fn with_staging_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.staging_dir = dir.into();
        self
    }

    /// Overrides the Hub endpoint URL.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Sets an optional authorization token.
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token.filter(|t| !t.trim().is_empty());
        self
    }

    /// Returns the configured staging directory.
    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }
}

impl Default for HfHubDownloader {
    fn default() -> Self {
        Self::from_env()
    }
}

#[derive(Clone)]
struct ProgressBridge {
    sender: mpsc::UnboundedSender<DownloadProgress>,
    transferred: Arc<AtomicU64>,
    total: Arc<AtomicU64>,
}

impl ProgressBridge {
    fn new(sender: mpsc::UnboundedSender<DownloadProgress>) -> Self {
        Self {
            sender,
            transferred: Arc::new(AtomicU64::new(0)),
            total: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl HfProgress for ProgressBridge {
    async fn init(&mut self, size: usize, _filename: &str) {
        self.total.store(size as u64, Ordering::SeqCst);
        self.transferred.store(0, Ordering::SeqCst);
        let _ = self.sender.send(DownloadProgress::Started {
            total: ByteLength::new(size as u64),
        });
    }

    async fn update(&mut self, size: usize) {
        let transferred = self.transferred.fetch_add(size as u64, Ordering::SeqCst) + (size as u64);
        let total = self.total.load(Ordering::SeqCst);
        let _ = self.sender.send(DownloadProgress::Advanced {
            transferred: ByteLength::new(transferred),
            total: ByteLength::new(total),
        });
    }

    async fn finish(&mut self) {
        let _ = self.sender.send(DownloadProgress::Finished);
    }
}

impl ModelDownloaderPort for HfHubDownloader {
    async fn fetch<Progress>(
        &self,
        remote: &RemoteModelFile,
        progress: &Progress,
    ) -> Result<ModelArtifact, ModelDownloadError>
    where
        Progress: DownloadProgressPort,
    {
        tokio::fs::create_dir_all(&self.staging_dir)
            .await
            .map_err(|err| ModelDownloadError::Transport {
                file: remote.file().to_string(),
                cause: err.to_string(),
            })?;

        let mut builder = ApiBuilder::new()
            .with_cache_dir(self.staging_dir.clone())
            .with_endpoint(self.endpoint.clone())
            .with_progress(false);

        if let Some(token) = &self.token {
            builder = builder.with_token(Some(token.clone()));
        }

        let api = builder
            .build()
            .map_err(|err| ModelDownloadError::Transport {
                file: remote.file().to_string(),
                cause: err.to_string(),
            })?;

        let repo_id = remote.repository().identifier().as_str().to_string();
        let revision = remote.repository().revision().as_str().to_string();
        let repo = Repo::with_revision(repo_id, RepoType::Model, revision);
        let api_repo = api.repo(repo);

        let (tx, mut rx) = mpsc::unbounded_channel::<DownloadProgress>();
        let bridge = ProgressBridge::new(tx);

        let file_name = remote.file().as_str().to_string();
        let download_handle =
            tokio::spawn(async move { api_repo.download_with_progress(&file_name, bridge).await });

        while let Some(event) = rx.recv().await {
            progress.report(event);
        }

        let download_result = download_handle
            .await
            .map_err(|err| ModelDownloadError::Transport {
                file: remote.file().to_string(),
                cause: err.to_string(),
            })?
            .map_err(|err| map_api_error(&err, remote.file().as_str()))?;

        let metadata = tokio::fs::metadata(&download_result).await.map_err(|err| {
            ModelDownloadError::Transport {
                file: remote.file().to_string(),
                cause: err.to_string(),
            }
        })?;

        let received_size = ByteLength::new(metadata.len());
        if remote.size() != ByteLength::ZERO && received_size != remote.size() {
            return Err(ModelDownloadError::SizeMismatch {
                file: remote.file().to_string(),
                expected: remote.size(),
                received: received_size,
            });
        }

        Ok(ModelArtifact::new(download_result, received_size))
    }
}

fn map_api_error(err: &hf_hub::api::tokio::ApiError, file_name: &str) -> ModelDownloadError {
    match err {
        hf_hub::api::tokio::ApiError::RequestError(reqwest_err) => {
            if reqwest_err.is_connect() || reqwest_err.is_timeout() {
                ModelDownloadError::Unreachable {
                    file: file_name.to_string(),
                    cause: reqwest_err.to_string(),
                }
            } else {
                ModelDownloadError::Transport {
                    file: file_name.to_string(),
                    cause: reqwest_err.to_string(),
                }
            }
        }
        other => ModelDownloadError::Transport {
            file: file_name.to_string(),
            cause: other.to_string(),
        },
    }
}

fn default_staging_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(".cache")
        .join("bare-ai-server")
        .join("staging")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downloader_initializes_with_staging_dir() {
        let downloader = HfHubDownloader::new("/tmp/staging");
        assert_eq!(downloader.staging_dir(), Path::new("/tmp/staging"));
        assert_eq!(downloader.endpoint, DEFAULT_ENDPOINT);
    }

    #[test]
    fn custom_endpoint_overrides_in_downloader() {
        let downloader =
            HfHubDownloader::new("/tmp/staging").with_endpoint("http://custom-hub:8080");
        assert_eq!(downloader.endpoint, "http://custom-hub:8080");
    }
}

use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use hf_hub::{
    Repo, RepoType,
    api::tokio::{ApiBuilder, ApiRepo, Progress as HfProgress},
};
use localnar_application::{
    errors::ModelDownloadError,
    ports::outbound::{DownloadProgress, DownloadProgressPort, ModelDownloaderPort},
};
use localnar_domain::{ByteLength, ModelArtifact, RemoteModelFile};
use tokio::sync::mpsc;

const DEFAULT_ENDPOINT: &str = "https://huggingface.co";

/// Transport contract for fetching files from Hugging Face Hub.
pub trait HubDownloadTransport: Send + Sync {
    async fn download_file(
        &self,
        remote: &RemoteModelFile,
        progress: &dyn DownloadProgressPort,
    ) -> Result<ModelArtifact, ModelDownloadError>;
}

/// Production downloader transport backed by `hf-hub`.
#[derive(Debug, Clone)]
pub struct HfHubTokioTransport {
    staging_dir: PathBuf,
    endpoint: String,
    token: Option<String>,
}

impl HfHubTokioTransport {
    /// Builds a transport that stages files under `staging_dir`.
    pub fn new(
        staging_dir: impl Into<PathBuf>,
        endpoint: impl Into<String>,
        token: Option<String>,
    ) -> Self {
        Self {
            staging_dir: staging_dir.into(),
            endpoint: endpoint.into(),
            token: token.filter(|t| !t.trim().is_empty()),
        }
    }

    /// Resolves configuration from environment variables.
    pub fn from_env() -> Self {
        let staging_dir = std::env::var("LOCALNAR_STAGING_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_staging_dir());
        let endpoint =
            std::env::var("HF_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        let token = std::env::var("HF_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty());

        Self::new(staging_dir, endpoint, token)
    }

    /// Returns the configured staging directory path.
    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }
}

impl Default for HfHubTokioTransport {
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

impl HubDownloadTransport for HfHubTokioTransport {
    async fn download_file(
        &self,
        remote: &RemoteModelFile,
        progress: &dyn DownloadProgressPort,
    ) -> Result<ModelArtifact, ModelDownloadError> {
        self.ensure_staging_dir(remote).await?;

        let api_repo = build_api_repo(
            &self.endpoint,
            self.token.as_deref(),
            &self.staging_dir,
            remote,
        )?;

        let downloaded = run_download(api_repo, remote, progress).await?;

        validate_download_size(&downloaded, remote.size(), remote.file().as_str()).await
    }
}

impl HfHubTokioTransport {
    /// Creates the staging directory downloads land in before they are
    /// committed.
    async fn ensure_staging_dir(&self, remote: &RemoteModelFile) -> Result<(), ModelDownloadError> {
        tokio::fs::create_dir_all(&self.staging_dir)
            .await
            .map_err(|err| ModelDownloadError::Transport {
                file: remote.file().to_string(),
                cause: err.to_string(),
            })
    }
}

/// Runs the download on a task, pumping its progress to `progress`, and reports
/// where the bytes landed.
async fn run_download(
    api_repo: ApiRepo,
    remote: &RemoteModelFile,
    progress: &dyn DownloadProgressPort,
) -> Result<PathBuf, ModelDownloadError> {
    let (tx, mut rx) = mpsc::unbounded_channel::<DownloadProgress>();
    let bridge = ProgressBridge::new(tx);

    let file_name = remote.file().as_str().to_string();
    let download_handle =
        tokio::spawn(async move { api_repo.download_with_progress(&file_name, bridge).await });

    while let Some(event) = rx.recv().await {
        progress.report(event);
    }

    download_handle
        .await
        .map_err(|err| ModelDownloadError::Transport {
            file: remote.file().to_string(),
            cause: err.to_string(),
        })?
        .map_err(|err| map_api_error(&err, remote.file().as_str()))
}

fn build_api_repo(
    endpoint: &str,
    token: Option<&str>,
    staging_dir: &Path,
    remote: &RemoteModelFile,
) -> Result<ApiRepo, ModelDownloadError> {
    let mut builder = ApiBuilder::new()
        .with_cache_dir(staging_dir.to_path_buf())
        .with_endpoint(endpoint.to_string())
        .with_progress(false);

    if let Some(token_val) = token {
        builder = builder.with_token(Some(token_val.to_string()));
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
    Ok(api.repo(repo))
}

async fn validate_download_size(
    downloaded_path: &Path,
    expected_size: ByteLength,
    file_name: &str,
) -> Result<ModelArtifact, ModelDownloadError> {
    let metadata = tokio::fs::metadata(downloaded_path).await.map_err(|err| {
        ModelDownloadError::Transport {
            file: file_name.to_string(),
            cause: err.to_string(),
        }
    })?;

    let received_size = ByteLength::new(metadata.len());
    if expected_size != ByteLength::ZERO && received_size != expected_size {
        return Err(ModelDownloadError::SizeMismatch {
            file: file_name.to_string(),
            expected: expected_size,
            received: received_size,
        });
    }

    Ok(ModelArtifact::new(
        downloaded_path.to_path_buf(),
        received_size,
    ))
}

/// Model downloader using Hugging Face Hub tokio client.
#[derive(Debug, Clone)]
pub struct HfHubDownloader<Transport = HfHubTokioTransport> {
    transport: Transport,
}

impl<Transport: HubDownloadTransport> HfHubDownloader<Transport> {
    /// Builds a downloader with an injected transport.
    pub fn new(transport: Transport) -> Self {
        Self { transport }
    }

    /// Returns a reference to the inner transport.
    pub fn transport(&self) -> &Transport {
        &self.transport
    }
}

impl HfHubDownloader<HfHubTokioTransport> {
    /// Resolves configuration from environment variables.
    pub fn from_env() -> Self {
        Self::new(HfHubTokioTransport::from_env())
    }
}

impl Default for HfHubDownloader<HfHubTokioTransport> {
    fn default() -> Self {
        Self::from_env()
    }
}

impl<Transport: HubDownloadTransport> ModelDownloaderPort for HfHubDownloader<Transport> {
    async fn fetch(
        &self,
        remote: &RemoteModelFile,
        progress: &dyn DownloadProgressPort,
    ) -> Result<ModelArtifact, ModelDownloadError> {
        self.transport.download_file(remote, progress).await
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
        .join("localnar")
        .join("staging")
}

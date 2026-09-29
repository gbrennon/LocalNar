use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use futures::{StreamExt, TryStreamExt, stream};
use hf_hub::{
    Repo, RepoType,
    api::tokio::{ApiBuilder, ApiRepo, Progress as HfProgress},
};
use localnar_application::{
    errors::ModelDownloadError,
    ports::outbound::{DownloadProgress, DownloadProgressPort, ModelDownloaderPort},
};
use localnar_domain::{
    ByteLength, ModelArtifact, ModelFileName, MultiPartShard, RemoteModelFile, Settings,
};
use tokio::sync::mpsc;

use super::settings::HuggingFaceSettings;

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

    /// Resolves configuration from persisted settings, falling back to the
    /// environment and built-in defaults for any value left unset.
    pub fn from_settings(settings: &Settings) -> Self {
        let hf = HuggingFaceSettings::from_settings(settings);
        let staging_dir = hf
            .cache_directory()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("LOCALNAR_STAGING_DIR")
                    .ok()
                    .map(PathBuf::from)
            })
            .unwrap_or_else(default_staging_dir);
        let endpoint = hf
            .endpoint()
            .map(str::to_owned)
            .or_else(|| std::env::var("HF_ENDPOINT").ok())
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
        let token = hf
            .api_token()
            .map(str::to_owned)
            .or_else(|| std::env::var("HF_TOKEN").ok());
        Self::new(staging_dir, endpoint, token)
    }

    /// Returns the configured staging directory path.
    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }

    /// Returns the configured Hugging Face endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
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

        let part_names = MultiPartShard
            .all_parts(remote.file())
            .unwrap_or_else(|| vec![remote.file().clone()]);
        let mut downloaded = self.download_parts(remote, &part_names, progress).await?;
        let total_size = downloaded.iter().map(|(_, size)| size.bytes()).sum();
        let (primary, _) = downloaded.remove(0);
        let companions = downloaded.into_iter().map(|(path, _)| path).collect();

        Ok(ModelArtifact::new(primary, ByteLength::new(total_size)).with_companions(companions))
    }
}

impl HfHubTokioTransport {
    async fn download_parts(
        &self,
        remote: &RemoteModelFile,
        parts: &[ModelFileName],
        progress: &dyn DownloadProgressPort,
    ) -> Result<Vec<(PathBuf, ByteLength)>, ModelDownloadError> {
        stream::iter(parts.iter().enumerate())
            .then(|(index, part)| self.download_part(remote, part, index == 0, progress))
            .try_collect()
            .await
    }

    async fn download_part(
        &self,
        remote: &RemoteModelFile,
        part: &ModelFileName,
        primary: bool,
        progress: &dyn DownloadProgressPort,
    ) -> Result<(PathBuf, ByteLength), ModelDownloadError> {
        let api_repo = build_api_repo(
            &self.endpoint,
            self.token.as_deref(),
            &self.staging_dir,
            remote,
        )?;
        let path = run_download(api_repo, part.as_str(), progress).await?;
        let size = downloaded_size(&path, part.as_str()).await?;
        validate_primary_size(primary, remote.size(), size, part.as_str())?;
        Ok((path, size))
    }

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
    file_name: &str,
    progress: &dyn DownloadProgressPort,
) -> Result<PathBuf, ModelDownloadError> {
    let (tx, mut rx) = mpsc::unbounded_channel::<DownloadProgress>();
    let bridge = ProgressBridge::new(tx);

    let file_name = file_name.to_string();
    let requested_file = file_name.clone();
    let download_handle = tokio::spawn(async move {
        api_repo
            .download_with_progress(&requested_file, bridge)
            .await
    });

    while let Some(event) = rx.recv().await {
        progress.report(event);
    }

    download_handle
        .await
        .map_err(|err| ModelDownloadError::Transport {
            file: file_name.clone(),
            cause: err.to_string(),
        })?
        .map_err(|err| map_api_error(&err, file_name.as_str()))
}

fn validate_primary_size(
    primary: bool,
    expected: ByteLength,
    received: ByteLength,
    file_name: &str,
) -> Result<(), ModelDownloadError> {
    match (primary, expected != ByteLength::ZERO, received != expected) {
        (true, true, true) => Err(ModelDownloadError::SizeMismatch {
            file: file_name.to_string(),
            expected,
            received,
        }),
        _ => Ok(()),
    }
}

async fn downloaded_size(path: &Path, file_name: &str) -> Result<ByteLength, ModelDownloadError> {
    let metadata =
        tokio::fs::metadata(path)
            .await
            .map_err(|err| ModelDownloadError::Transport {
                file: file_name.to_string(),
                cause: err.to_string(),
            })?;
    Ok(ByteLength::new(metadata.len()))
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

    /// Builds a downloader from persisted settings with env/default fallback.
    pub fn from_settings(settings: &Settings) -> Self {
        Self::new(HfHubTokioTransport::from_settings(settings))
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
            match reqwest_err.is_connect() || reqwest_err.is_timeout() {
                true => ModelDownloadError::Unreachable {
                    file: file_name.to_string(),
                    cause: reqwest_err.to_string(),
                },
                false => ModelDownloadError::Transport {
                    file: file_name.to_string(),
                    cause: reqwest_err.to_string(),
                },
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

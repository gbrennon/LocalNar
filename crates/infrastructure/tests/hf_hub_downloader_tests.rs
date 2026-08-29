use std::sync::Mutex;

use application::errors::ModelDownloadError;
use application::ports::outbound::{DownloadProgress, DownloadProgressPort, ModelDownloaderPort};
use domain::{
    ByteLength, Checksum, ModelArtifact, ModelFileName, ModelRepository, ModelRepositoryId,
    ModelRevision, RemoteModelFile,
};
use infrastructure::{HfHubDownloader, HubDownloadTransport};
use tempfile::TempDir;

#[derive(Default)]
struct ProgressRecorder {
    events: Mutex<Vec<DownloadProgress>>,
}

impl DownloadProgressPort for ProgressRecorder {
    fn report(&self, progress: DownloadProgress) {
        self.events.lock().unwrap().push(progress);
    }
}

struct FakeHubDownloadTransport {
    payload: Vec<u8>,
    staged_dir: TempDir,
    should_fail: Option<ModelDownloadError>,
}

impl FakeHubDownloadTransport {
    fn with_payload(payload: &[u8]) -> Self {
        Self {
            payload: payload.to_vec(),
            staged_dir: TempDir::new().expect("temp dir"),
            should_fail: None,
        }
    }

    fn failing_with(error: ModelDownloadError) -> Self {
        Self {
            payload: Vec::new(),
            staged_dir: TempDir::new().expect("temp dir"),
            should_fail: Some(error),
        }
    }
}

impl HubDownloadTransport for FakeHubDownloadTransport {
    async fn download_file<Progress>(
        &self,
        remote: &RemoteModelFile,
        progress: &Progress,
    ) -> Result<ModelArtifact, ModelDownloadError>
    where
        Progress: DownloadProgressPort,
    {
        if let Some(err) = &self.should_fail {
            return Err(err.clone());
        }

        let staged_file = self.staged_dir.path().join(remote.file().as_str());
        tokio::fs::write(&staged_file, &self.payload)
            .await
            .map_err(|err| ModelDownloadError::Transport {
                file: remote.file().to_string(),
                cause: err.to_string(),
            })?;

        let total = ByteLength::new(self.payload.len() as u64);
        progress.report(DownloadProgress::Started { total });
        progress.report(DownloadProgress::Advanced {
            transferred: total,
            total,
        });
        progress.report(DownloadProgress::Finished);

        if remote.size() != ByteLength::ZERO && total != remote.size() {
            return Err(ModelDownloadError::SizeMismatch {
                file: remote.file().to_string(),
                expected: remote.size(),
                received: total,
            });
        }

        Ok(ModelArtifact::new(staged_file, total))
    }
}

#[tokio::test]
async fn fetch_downloads_artifact_and_reports_progress() {
    let payload = b"fake-model-weights";
    let transport = FakeHubDownloadTransport::with_payload(payload);
    let downloader = HfHubDownloader::new(transport);

    let repository = ModelRepository::new(
        ModelRepositoryId::parse("test-org/test-model").expect("valid id"),
        ModelRevision::new("main").expect("valid rev"),
    );
    let file = ModelFileName::new("model.gguf").expect("valid file");
    let remote_file = RemoteModelFile::new(
        repository,
        file,
        ByteLength::new(payload.len() as u64),
        Some(Checksum::from_bytes([0x12; 32])),
    );

    let recorder = ProgressRecorder::default();
    let artifact = downloader
        .fetch(&remote_file, &recorder)
        .await
        .expect("fetch must succeed");

    assert_eq!(artifact.size(), ByteLength::new(payload.len() as u64));
    assert!(artifact.staged_at().exists());

    let recorded = recorder.events.lock().unwrap().clone();
    assert_eq!(
        recorded,
        vec![
            DownloadProgress::Started {
                total: ByteLength::new(payload.len() as u64),
            },
            DownloadProgress::Advanced {
                transferred: ByteLength::new(payload.len() as u64),
                total: ByteLength::new(payload.len() as u64),
            },
            DownloadProgress::Finished,
        ]
    );
}

#[tokio::test]
async fn fetch_propagates_injected_download_error() {
    let transport = FakeHubDownloadTransport::failing_with(ModelDownloadError::Unreachable {
        file: "model.gguf".to_string(),
        cause: "host unreachable".to_string(),
    });
    let downloader = HfHubDownloader::new(transport);

    let repository = ModelRepository::new(
        ModelRepositoryId::parse("test-org/test-model").expect("valid id"),
        ModelRevision::new("main").expect("valid rev"),
    );
    let file = ModelFileName::new("model.gguf").expect("valid file");
    let remote_file = RemoteModelFile::new(repository, file, ByteLength::new(100), None);

    let recorder = ProgressRecorder::default();
    let result = downloader.fetch(&remote_file, &recorder).await;

    assert!(matches!(
        result,
        Err(ModelDownloadError::Unreachable { .. })
    ));
}

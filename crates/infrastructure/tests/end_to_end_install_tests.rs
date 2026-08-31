use std::sync::Mutex;

use application::{
    ports::{
        inbound::InstallModelPort,
        outbound::{DownloadProgress, DownloadProgressPort},
    },
    services::InstallModelService,
};
use domain::{
    ByteLength, Checksum, ModelArtifact, ModelFileName, ModelRepository, ModelRepositoryId,
    ModelRevision, ModelSpec, RemoteModelFile,
};
use infrastructure::{
    DiskModelLibrary, HfApiRegistry, HfHubDownloader, HubDownloadTransport, HubTransport,
};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

#[derive(Default)]
struct ProgressSpy {
    events: Mutex<Vec<DownloadProgress>>,
}

impl DownloadProgressPort for ProgressSpy {
    fn report(&self, progress: DownloadProgress) {
        self.events.lock().unwrap().push(progress);
    }
}

fn compute_sha256(bytes: &[u8]) -> Checksum {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Checksum::from_bytes(hasher.finalize().into())
}

struct FakeHubTransport {
    response_json: String,
}

impl HubTransport for FakeHubTransport {
    async fn get_json<T: DeserializeOwned>(
        &self,
        _path: &str,
    ) -> Result<T, application::errors::RegistryReadError> {
        serde_json::from_str(&self.response_json).map_err(|_| {
            application::errors::RegistryReadError::Malformed {
                repository: "fake".to_string(),
            }
        })
    }
}

struct FakeDownloadTransport {
    payload: Vec<u8>,
    staged_dir: TempDir,
}

impl HubDownloadTransport for FakeDownloadTransport {
    async fn download_file(
        &self,
        remote: &RemoteModelFile,
        progress: &dyn DownloadProgressPort,
    ) -> Result<ModelArtifact, application::errors::ModelDownloadError> {
        let staged_file = self.staged_dir.path().join(remote.file().as_str());
        tokio::fs::write(&staged_file, &self.payload)
            .await
            .map_err(|err| application::errors::ModelDownloadError::Transport {
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

        Ok(ModelArtifact::new(staged_file, total))
    }
}

#[tokio::test]
async fn install_model_service_end_to_end_with_injected_fakes() {
    let payload = b"complete-e2e-model-weights-gguf-format";
    let expected_checksum = compute_sha256(payload);

    let registry_json = format!(
        r#"{{
            "id": "e2e-org/test-llm",
            "siblings": [
                {{
                    "rfilename": "model-q4.gguf",
                    "size": {len},
                    "lfs": {{
                        "sha256": "{sha}",
                        "size": {len}
                    }}
                }}
            ]
        }}"#,
        len = payload.len(),
        sha = expected_checksum.to_hex()
    );

    let models_dir = TempDir::new().expect("models dir");
    let staged_dir = TempDir::new().expect("staged dir");

    let registry_transport = FakeHubTransport {
        response_json: registry_json,
    };
    let download_transport = FakeDownloadTransport {
        payload: payload.to_vec(),
        staged_dir,
    };

    let registry = HfApiRegistry::new(registry_transport);
    let downloader = HfHubDownloader::new(download_transport);
    let library = DiskModelLibrary::new(models_dir.path());
    let progress = ProgressSpy::default();

    let service = InstallModelService::new(registry, downloader, library, progress);

    let spec = ModelSpec::new(
        ModelRepository::new(
            ModelRepositoryId::parse("e2e-org/test-llm").expect("valid id"),
            ModelRevision::new("main").expect("valid rev"),
        ),
        ModelFileName::new("model-q4.gguf").expect("valid file"),
        vec![],
    );

    let installed = service.execute(&spec).await.expect("installation succeeds");

    assert_eq!(installed.spec(), &spec);
    assert_eq!(installed.size().bytes(), payload.len() as u64);
    assert_eq!(installed.digest(), Some(expected_checksum));
    assert!(installed.is_verified());
    assert!(installed.path().exists());
}

/// A transport that stages the way `hf-hub` really does.
///
/// The bytes land in a content-addressed `blobs` directory and the artifact
/// points at a snapshot entry that is only a symlink, whose target is relative
/// to the snapshot directory. Staging this shape is what makes an install that
/// relocates the link, rather than the bytes, leave an unusable reference.
struct FakeCacheStagingTransport {
    payload: Vec<u8>,
    cache_dir: TempDir,
}

impl FakeCacheStagingTransport {
    const COMMIT: &'static str = "c8b5954a88c2775c546b92593eda40ea041d3176";

    fn with_payload(payload: &[u8]) -> Self {
        Self {
            payload: payload.to_vec(),
            cache_dir: TempDir::new().expect("cache dir"),
        }
    }

    fn blob_name(&self) -> String {
        compute_sha256(&self.payload).to_hex()
    }
}

impl HubDownloadTransport for FakeCacheStagingTransport {
    async fn download_file(
        &self,
        remote: &RemoteModelFile,
        progress: &dyn DownloadProgressPort,
    ) -> Result<ModelArtifact, application::errors::ModelDownloadError> {
        let blobs = self.cache_dir.path().join("blobs");
        let snapshot = self.cache_dir.path().join("snapshots").join(Self::COMMIT);
        tokio::fs::create_dir_all(&blobs).await.expect("blobs dir");
        tokio::fs::create_dir_all(&snapshot)
            .await
            .expect("snapshot dir");

        let blob = blobs.join(self.blob_name());
        tokio::fs::write(&blob, &self.payload)
            .await
            .expect("write blob");

        let staged = snapshot.join(remote.file().as_str());
        std::os::unix::fs::symlink(format!("../../blobs/{}", self.blob_name()), &staged)
            .expect("snapshot symlink");

        let total = ByteLength::new(self.payload.len() as u64);
        progress.report(DownloadProgress::Started { total });
        progress.report(DownloadProgress::Finished);

        Ok(ModelArtifact::new(staged, total))
    }
}

fn revision_json(file: &str, payload: &[u8]) -> String {
    format!(
        r#"{{
            "id": "e2e-org/test-llm",
            "siblings": [
                {{
                    "rfilename": "{file}",
                    "size": {len},
                    "lfs": {{ "sha256": "{sha}", "size": {len} }}
                }}
            ]
        }}"#,
        len = payload.len(),
        sha = compute_sha256(payload).to_hex()
    )
}

fn test_spec(file: &str) -> ModelSpec {
    ModelSpec::new(
        ModelRepository::new(
            ModelRepositoryId::parse("e2e-org/test-llm").expect("valid id"),
            ModelRevision::new("main").expect("valid rev"),
        ),
        ModelFileName::new(file).expect("valid file"),
        vec![],
    )
}

fn destination_of(root: &std::path::Path, spec: &ModelSpec) -> std::path::PathBuf {
    root.join(spec.repository().identifier().owner())
        .join(spec.repository().identifier().name())
        .join(spec.repository().revision().as_str())
        .join(spec.file().as_str())
}

/// The reported failure: an install that relocated the downloader's staged
/// symlink left the library holding a relative target that does not resolve
/// from there, so the model read as missing and the service settled on blaming
/// upstream for bytes it had already received. Installing again must take the
/// destination over and land servable bytes.
#[tokio::test]
async fn install_recovers_a_library_holding_a_dangling_entry() {
    let payload = b"replacement-model-weights-gguf";
    let models_dir = TempDir::new().expect("models dir");
    let spec = test_spec("model-q4.gguf");

    let destination = destination_of(models_dir.path(), &spec);
    std::fs::create_dir_all(destination.parent().expect("parent")).expect("create parent");
    std::os::unix::fs::symlink("../../blobs/vanished", &destination).expect("stale entry");

    let service = InstallModelService::new(
        HfApiRegistry::new(FakeHubTransport {
            response_json: revision_json("model-q4.gguf", payload),
        }),
        HfHubDownloader::new(FakeCacheStagingTransport::with_payload(payload)),
        DiskModelLibrary::new(models_dir.path()),
        ProgressSpy::default(),
    );

    let installed = service.execute(&spec).await.expect("installation recovers");

    assert!(installed.is_verified());
    assert_eq!(installed.digest(), Some(compute_sha256(payload)));
    assert!(
        std::fs::symlink_metadata(&destination)
            .expect("committed entry")
            .is_file()
    );
    assert_eq!(
        tokio::fs::read(&destination).await.expect("read model"),
        payload
    );
}

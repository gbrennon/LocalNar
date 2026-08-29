use std::sync::Mutex;

use application::ports::inbound::InstallModelPort;
use application::ports::outbound::{DownloadProgress, DownloadProgressPort};
use application::services::InstallModelService;
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
    async fn download_file<Progress>(
        &self,
        remote: &RemoteModelFile,
        progress: &Progress,
    ) -> Result<ModelArtifact, application::errors::ModelDownloadError>
    where
        Progress: DownloadProgressPort,
    {
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
    );

    let installed = service.execute(&spec).await.expect("installation succeeds");

    assert_eq!(installed.spec(), &spec);
    assert_eq!(installed.size().bytes(), payload.len() as u64);
    assert_eq!(installed.digest(), Some(expected_checksum));
    assert!(installed.is_verified());
    assert!(installed.path().exists());
}

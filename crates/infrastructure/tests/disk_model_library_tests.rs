use localnar_application::{errors::LibraryError, ports::outbound::ModelLibraryPort};
use localnar_domain::{
    ByteLength, Checksum, ModelArtifact, ModelFileName, ModelRepository, ModelRepositoryId,
    ModelRevision, ModelSpec, ModelState,
};
use localnar_infrastructure::DiskModelLibrary;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn sample_spec() -> ModelSpec {
    let repo_id = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
    let revision = ModelRevision::new("main").expect("valid revision");
    let repository = ModelRepository::new(repo_id, revision);
    let file = ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file");
    ModelSpec::new(repository, file, vec![])
}

fn compute_digest(bytes: &[u8]) -> Checksum {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Checksum::from_bytes(hasher.finalize().into())
}

#[tokio::test]
async fn installed_state_returns_missing_when_no_file_exists() {
    let temp_dir = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp_dir.path());
    let spec = sample_spec();

    let state = library.installed_state(&spec).await.expect("query state");
    assert_eq!(state, ModelState::Missing);
}

#[tokio::test]
async fn commit_artifact_places_model_and_reports_downloaded_state() {
    let temp_dir = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp_dir.path());
    let spec = sample_spec();

    let staged_dir = TempDir::new().expect("staged dir");
    let staged_path = staged_dir.path().join("model.bin");
    let content = b"binary-model-weights-content";
    tokio::fs::write(&staged_path, content)
        .await
        .expect("write staged");

    let artifact = ModelArtifact::new(&staged_path, ByteLength::new(content.len() as u64));
    let state = library
        .commit_artifact(&spec, &artifact)
        .await
        .expect("commit");
    assert_eq!(state, ModelState::Downloaded);

    let current_state = library
        .installed_state(&spec)
        .await
        .expect("installed state");
    assert_eq!(current_state, ModelState::Downloaded);

    let expected_installed_path = temp_dir
        .path()
        .join("unsloth")
        .join("Qwen3-8B-GGUF")
        .join("main")
        .join("Qwen3-8B-Q4_K_M.gguf");
    assert!(expected_installed_path.exists());
}

#[tokio::test]
async fn verify_integrity_persists_verified_checksum_and_updates_state() {
    let temp_dir = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp_dir.path());
    let spec = sample_spec();

    let content = b"authentic-gguf-model-data";
    let staged_dir = TempDir::new().expect("staged dir");
    let staged_path = staged_dir.path().join("model.bin");
    tokio::fs::write(&staged_path, content)
        .await
        .expect("write staged");

    let expected_checksum = compute_digest(content);
    let artifact = ModelArtifact::new(&staged_path, ByteLength::new(content.len() as u64));
    library
        .commit_artifact(&spec, &artifact)
        .await
        .expect("commit");

    let verify_state = library
        .verify_integrity(&spec, Some(expected_checksum))
        .await
        .expect("verify");
    assert_eq!(verify_state, ModelState::Verified);

    let state_after = library.installed_state(&spec).await.expect("query state");
    assert_eq!(state_after, ModelState::Verified);

    let installed_model = library.locate(&spec).await.expect("locate");
    assert_eq!(installed_model.digest(), Some(expected_checksum));
    assert_eq!(
        installed_model.size(),
        ByteLength::new(content.len() as u64)
    );
    assert!(installed_model.is_verified());
}

#[tokio::test]
async fn verify_integrity_with_corrupted_content_reports_mismatch() {
    let temp_dir = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp_dir.path());
    let spec = sample_spec();

    let content = b"corrupted-model-data";
    let staged_dir = TempDir::new().expect("staged dir");
    let staged_path = staged_dir.path().join("model.bin");
    tokio::fs::write(&staged_path, content)
        .await
        .expect("write staged");

    let actual_checksum = compute_digest(content);
    let expected_checksum = Checksum::from_bytes([0x44; 32]);

    let artifact = ModelArtifact::new(&staged_path, ByteLength::new(content.len() as u64));
    library
        .commit_artifact(&spec, &artifact)
        .await
        .expect("commit");

    let verify_state = library
        .verify_integrity(&spec, Some(expected_checksum))
        .await
        .expect("verify");
    assert_eq!(
        verify_state,
        ModelState::IntegrityMismatch {
            expected: expected_checksum,
            actual: actual_checksum,
        }
    );
}

#[tokio::test]
async fn locate_fails_when_model_is_not_installed() {
    let temp_dir = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp_dir.path());
    let spec = sample_spec();

    let result = library.locate(&spec).await;
    assert!(matches!(result, Err(LibraryError::Unreadable { .. })));
}

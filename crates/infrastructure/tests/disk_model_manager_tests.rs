use application::ports::outbound::{
    LibraryMaintenancePort, ModelEvictionPort, ModelInventoryPort, ModelLibraryPort,
};
use domain::{
    ByteLength, Checksum, ModelArtifact, ModelFileName, ModelRepository, ModelRepositoryId,
    ModelRevision, ModelSpec, ModelState,
};
use infrastructure::DiskModelLibrary;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn spec(owner: &str, name: &str, revision: &str, file: &str) -> ModelSpec {
    let repository = ModelRepository::new(
        ModelRepositoryId::parse(format!("{owner}/{name}")).expect("valid id"),
        ModelRevision::new(revision).expect("valid revision"),
    );
    let file = ModelFileName::new(file).expect("valid file");

    ModelSpec::new(repository, file, vec![])
}

fn digest_of(bytes: &[u8]) -> Checksum {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Checksum::from_bytes(hasher.finalize().into())
}

async fn install(library: &DiskModelLibrary, model: &ModelSpec, content: &[u8]) {
    let staged = TempDir::new().expect("staged dir");
    let staged_path = staged.path().join("replica");
    tokio::fs::write(&staged_path, content)
        .await
        .expect("write staged");

    let artifact = ModelArtifact::new(&staged_path, ByteLength::new(content.len() as u64));
    library
        .commit_artifact(model, &artifact)
        .await
        .expect("commit");
}

async fn install_proven(library: &DiskModelLibrary, model: &ModelSpec, content: &[u8]) {
    install(library, model, content).await;
    library
        .verify_integrity(model, Some(digest_of(content)))
        .await
        .expect("verify");
}

#[tokio::test]
async fn an_absent_root_enumerates_an_empty_inventory() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path().join("never-created"));

    let inventory = library.enumerate().await.expect("enumerate");

    assert!(inventory.is_empty());
    assert_eq!(inventory.count(), 0);
}

#[tokio::test]
async fn a_committed_model_enumerates_as_a_single_downloaded_entry_with_its_size_and_path() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let model = spec("owner", "Model-1", "main", "model.gguf");

    install(&library, &model, b"downloadable-weight-bytes").await;

    let inventory = library.enumerate().await.expect("enumerate");

    assert_eq!(inventory.count(), 1);
    let entry = inventory.find(&model).expect("the installed model");
    assert!(!entry.is_verified(), "a commit alone is not proof");
    assert_eq!(
        entry.size(),
        ByteLength::new(b"downloadable-weight-bytes".len() as u64)
    );
    assert!(entry.path().is_file(), "path: {}", entry.path().display());
    assert_eq!(entry.digest(), None);
}

#[tokio::test]
async fn a_model_with_a_recorded_digest_enumerates_as_verified_carrying_that_digest() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let model = spec("owner", "Model-1", "main", "model.gguf");
    let content = b"authentic-proved-bytes";

    install_proven(&library, &model, content).await;

    let inventory = library.enumerate().await.expect("enumerate");

    let entry = inventory.find(&model).expect("the installed model");
    assert!(entry.is_verified());
    assert_eq!(entry.digest(), Some(digest_of(content)));
}

#[tokio::test]
async fn models_from_different_repositories_all_enumerate_and_their_sizes_sum() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let first = spec("alice", "Tiny", "main", "tiny.gguf");
    let second = spec("bob", "Large", "main", "large.gguf");

    install(&library, &first, b"short").await;
    install(&library, &second, b"a-considerably-longer-replica").await;

    let inventory = library.enumerate().await.expect("enumerate");

    assert_eq!(inventory.count(), 2);
    assert_eq!(
        inventory.total_size(),
        ByteLength::new((b"short".len() + b"a-considerably-longer-replica".len()) as u64)
    );
    assert!(inventory.find(&first).is_some());
    assert!(inventory.find(&second).is_some());
}

#[tokio::test]
async fn a_stray_file_directly_under_the_root_never_appears_as_an_entry() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    tokio::fs::write(temp.path().join("operator-notes.txt"), b"notes")
        .await
        .expect("write stray");

    let inventory = library.enumerate().await.expect("enumerate");

    assert!(inventory.is_empty());
}

#[tokio::test]
async fn a_checksum_sidecar_never_appears_as_an_entry() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let model = spec("owner", "Model-1", "main", "model.gguf");

    install_proven(&library, &model, b"proved-bytes").await;

    let inventory = library.enumerate().await.expect("enumerate");

    assert_eq!(inventory.count(), 1);
    assert!(inventory.find(&model).is_some());
}

#[tokio::test]
async fn enumeration_returns_entries_in_a_stable_order_across_calls() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    for name in ["zebra", "aardvark", "marmoset"] {
        let model = spec("menagerie", name, "main", "model.gguf");
        install(&library, &model, b"bytes").await;
    }

    let first = library.enumerate().await.expect("first enumerate");
    let second = library.enumerate().await.expect("second enumerate");

    let first_names: Vec<String> = first
        .entries()
        .iter()
        .map(|entry| entry.spec().to_string())
        .collect();
    let second_names: Vec<String> = second
        .entries()
        .iter()
        .map(|entry| entry.spec().to_string())
        .collect();
    assert_eq!(first_names, second_names);
}

#[tokio::test]
async fn evicting_an_installed_model_makes_the_library_report_it_missing() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let model = spec("owner", "Model-1", "main", "model.gguf");

    install_proven(&library, &model, b"proved-bytes").await;
    library.evict(&model).await.expect("evict");

    let inventory = library.enumerate().await.expect("enumerate");
    assert!(inventory.find(&model).is_none());
    assert_eq!(
        library
            .installed_state(&model)
            .await
            .expect("installed state"),
        ModelState::Missing
    );
}

#[tokio::test]
async fn evicting_an_installed_model_reports_the_reclaimed_size_and_removes_its_sidecar() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let model = spec("owner", "Model-1", "main", "model.gguf");
    let content = b"bytes-of-a-replica";

    install_proven(&library, &model, content).await;

    let removed = library.evict(&model).await.expect("evict");

    assert_eq!(removed.reclaimed(), ByteLength::new(content.len() as u64));
    assert_eq!(removed.spec(), &model);
    let sidecar = temp
        .path()
        .join("owner")
        .join("Model-1")
        .join("main")
        .join("model.gguf.sha256");
    assert!(!sidecar.exists(), "the sidecar must be removed");
}

#[tokio::test]
async fn evicting_an_installed_model_leaves_no_empty_directory_behind() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let model = spec("owner", "Model-1", "main", "model.gguf");

    install(&library, &model, b"bytes").await;
    library.evict(&model).await.expect("evict");

    assert!(!temp.path().join("owner").exists(), "the hierarchy is gone");
    assert!(temp.path().exists(), "the root survives");
}

#[tokio::test]
async fn evicting_a_model_the_library_never_held_reclaims_nothing() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let model = spec("owner", "Never-Had", "main", "model.gguf");

    let removed = library.evict(&model).await.expect("evict");

    assert_eq!(removed.reclaimed(), ByteLength::ZERO);
}

#[tokio::test]
async fn a_sweep_discards_an_orphan_sidecar() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let orphan = temp
        .path()
        .join("owner")
        .join("Model-1")
        .join("main")
        .join("vanished.gguf.sha256");
    tokio::fs::create_dir_all(orphan.parent().expect("parent dir"))
        .await
        .expect("make dir");
    tokio::fs::write(&orphan, digest_of(b"gone").to_hex())
        .await
        .expect("write note");

    let discarded = library.discard_strays().await.expect("sweep");

    assert!(
        discarded.iter().any(|stray| stray.path() == orphan),
        "the orphan note must be discarded"
    );
    assert!(!orphan.exists());
}

#[tokio::test]
async fn a_sweep_discards_an_empty_directory() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let empty = temp.path().join("owner").join("Gone").join("main");
    tokio::fs::create_dir_all(&empty).await.expect("make dir");

    let discarded = library.discard_strays().await.expect("sweep");

    assert_eq!(discarded.len(), 3, "each emptied level is its own leftover");
    assert!(!temp.path().join("owner").exists());
}

#[tokio::test]
async fn a_sweep_leaves_an_installed_model_reachable() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let proven = spec("owner", "Proven", "main", "proven.gguf");
    let unproven = spec("owner", "Unproven", "main", "unproven.gguf");

    install_proven(&library, &proven, b"proven-bytes").await;
    install(&library, &unproven, b"unproven-bytes").await;
    tokio::fs::write(
        temp.path()
            .join("owner")
            .join("Proven")
            .join("main")
            .join("stray.gguf.sha256"),
        digest_of(b"gone").to_hex(),
    )
    .await
    .expect("write stray note");

    library.discard_strays().await.expect("sweep");

    let inventory = library.enumerate().await.expect("enumerate");
    assert!(inventory.find(&proven).is_some());
    assert!(inventory.find(&unproven).is_some());
    assert!(
        temp.path().exists(),
        "the library root must survive a sweep"
    );
}

#[tokio::test]
async fn a_sweep_of_a_clean_library_discards_nothing() {
    let temp = TempDir::new().expect("temp dir");
    let library = DiskModelLibrary::new(temp.path());
    let model = spec("owner", "Model-1", "main", "model.gguf");

    install_proven(&library, &model, b"proved-bytes").await;

    let discarded = library.discard_strays().await.expect("sweep");

    assert!(discarded.is_empty());
    assert!(
        library
            .enumerate()
            .await
            .expect("enumerate")
            .find(&model)
            .is_some()
    );
}

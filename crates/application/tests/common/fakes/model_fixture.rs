use std::path::{Path, PathBuf};

use domain::{
    ByteLength, Checksum, ContextLength, DiscardedStray, InstalledModel, ManagedModel,
    ModelArtifact, ModelFileName, ModelInfo, ModelProfile, ModelRepository, ModelRepositoryId,
    ModelSpec, ModelState, ParameterCount, RemoteModelFile, RemovedModel, SearchQuery,
};

/// Canonical values every install and management scenario is written against.
pub struct ModelFixture;

impl ModelFixture {
    /// The root every fixture location hangs off.
    ///
    /// Relative and self-describing, so a fixture path can never name a real
    /// staging directory or model library even by accident.
    const FIXTURE_ROOT: &'static str = "fixture-only";

    /// A fixture location that no filesystem is expected to hold.
    fn nowhere(relative: &str) -> PathBuf {
        Path::new(Self::FIXTURE_ROOT).join(relative)
    }

    /// The install intent under test.
    pub fn spec() -> ModelSpec {
        let identifier = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
        ModelSpec::new(
            ModelRepository::at_default_revision(identifier),
            ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name"),
        )
    }

    /// The digest an upstream registry advertises for the fixture file.
    pub fn expected_digest() -> Checksum {
        Checksum::from_bytes([0x11; 32])
    }

    /// A digest that deliberately disagrees with the advertised one.
    pub fn actual_digest() -> Checksum {
        Checksum::from_bytes([0x22; 32])
    }

    /// The upstream listing an advertising registry answers with.
    pub fn remote_file() -> RemoteModelFile {
        let spec = Self::spec();
        RemoteModelFile::new(
            spec.repository().clone(),
            spec.file().clone(),
            ByteLength::new(4_096),
            Some(Self::expected_digest()),
        )
    }

    /// What the catalog discloses about serving the fixture model.
    pub fn profile() -> ModelProfile {
        ModelProfile::new(
            Some(ParameterCount::new(8_190_735_360)),
            Some(ContextLength::new(40_960)),
        )
    }

    /// The single candidate a searching registry describes the fixture as.
    pub fn model_info() -> ModelInfo {
        ModelInfo::describing(&Self::remote_file(), Self::profile())
    }

    /// The staged bytes a downloader hands back for the fixture file.
    ///
    /// The location is a label the fakes only ever compare, never open, so it
    /// names a place that cannot exist rather than a real staging directory.
    pub fn artifact() -> ModelArtifact {
        ModelArtifact::new(Self::nowhere("staged.gguf"), ByteLength::new(4_096))
    }

    /// The replica a library reports once the fixture file is on disk.
    ///
    /// `digest` carries the proof of integrity, which is absent when upstream
    /// advertised no checksum to compare against. As with the staged artifact,
    /// the location is only ever compared.
    pub fn installed(digest: Option<Checksum>) -> InstalledModel {
        InstalledModel::new(
            Self::spec(),
            Self::nowhere("installed/Qwen3-8B-Q4_K_M.gguf"),
            ByteLength::new(4_096),
            digest,
        )
    }

    /// The second install intent, so a stocked library holds more than one model.
    pub fn companion_spec() -> ModelSpec {
        let identifier = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
        ModelSpec::new(
            ModelRepository::at_default_revision(identifier),
            ModelFileName::new("Qwen3-8B-Q8_0.gguf").expect("valid file name"),
        )
    }

    /// The replica a library reports for the companion model.
    ///
    /// It carries no digest, so it stands for a replica nothing ever proved,
    /// and it occupies a different amount of space than the fixture model.
    pub fn companion_installed() -> InstalledModel {
        InstalledModel::new(
            Self::companion_spec(),
            Self::nowhere("installed/Qwen3-8B-Q8_0.gguf"),
            ByteLength::new(8_192),
            None,
        )
    }

    /// The directory a library keeps its fixture models under.
    pub fn library_root() -> PathBuf {
        Self::nowhere("installed")
    }

    /// The reading a replica whose bytes disagree with its digest comes back in.
    pub fn mismatched_state() -> ModelState {
        ModelState::IntegrityMismatch {
            expected: Self::expected_digest(),
            actual: Self::actual_digest(),
        }
    }

    /// The entry a library holds for a replica proven against its digest.
    pub fn verified_entry() -> ManagedModel {
        ManagedModel::new(
            Self::installed(Some(Self::expected_digest())),
            ModelState::Verified,
        )
    }

    /// The entry a library holds for a replica nothing ever proved.
    pub fn unproven_entry() -> ManagedModel {
        ManagedModel::new(Self::companion_installed(), ModelState::Downloaded)
    }

    /// The entry a library holds for a replica that no longer hashes as recorded.
    pub fn broken_entry() -> ManagedModel {
        ManagedModel::new(
            Self::installed(Some(Self::expected_digest())),
            Self::mismatched_state(),
        )
    }

    /// The record of discarding the fixture replica from the library.
    ///
    /// It reports the place the bytes used to occupy and the space they gave
    /// back, which is the whole size of the fixture replica.
    pub fn removed() -> RemovedModel {
        RemovedModel::new(
            Self::spec(),
            Self::nowhere("installed/Qwen3-8B-Q4_K_M.gguf"),
            ByteLength::new(4_096),
        )
    }

    /// A recorded digest whose model file went away by other means.
    pub fn discarded_digest_record() -> DiscardedStray {
        DiscardedStray::new(
            Self::nowhere("installed/Qwen3-8B-Q4_K_M.gguf.sha256"),
            ByteLength::new(64),
        )
    }

    /// The same leftover kept for the companion model.
    pub fn discarded_companion_digest_record() -> DiscardedStray {
        DiscardedStray::new(
            Self::nowhere("installed/Qwen3-8B-Q8_0.gguf.sha256"),
            ByteLength::new(64),
        )
    }

    /// A directory the library was left with once its last model went.
    pub fn discarded_emptied_directory() -> DiscardedStray {
        DiscardedStray::new(Self::nowhere("installed/emptied"), ByteLength::ZERO)
    }

    /// The phrase an operator types to find the fixture model.
    pub fn query() -> SearchQuery {
        SearchQuery::new("qwen3 gguf").expect("the fixture phrase carries text")
    }
}

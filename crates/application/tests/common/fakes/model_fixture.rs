use domain::{
    ByteLength, Checksum, ContextLength, InstalledModel, ModelArtifact, ModelFileName, ModelInfo,
    ModelProfile, ModelRepository, ModelRepositoryId, ModelSpec, ParameterCount, RemoteModelFile,
    SearchQuery,
};

/// Canonical values every install scenario is written against.
pub struct ModelFixture;

impl ModelFixture {
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
    pub fn artifact() -> ModelArtifact {
        ModelArtifact::new("/tmp/bare-ai-server/staged.gguf", ByteLength::new(4_096))
    }

    /// The replica a library reports once the fixture file is on disk.
    ///
    /// `digest` carries the proof of integrity, which is absent when upstream
    /// advertised no checksum to compare against.
    pub fn installed(digest: Option<Checksum>) -> InstalledModel {
        InstalledModel::new(
            Self::spec(),
            "/var/lib/bare-ai-server/models/Qwen3-8B-Q4_K_M.gguf",
            ByteLength::new(4_096),
            digest,
        )
    }

    /// The phrase an operator types to find the fixture model.
    pub fn query() -> SearchQuery {
        SearchQuery::new("qwen3 gguf").expect("the fixture phrase carries text")
    }
}

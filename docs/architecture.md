# Model downloader automation - current state

This document reflects the code as committed on this branch. The public API shown below is copied from the actual sources (one type per file, filename = snake_case(type)). When in doubt, trust the code over this prose: it can drift, the signatures in `crates/domain/src` cannot.

## 1. Workspace layout

`Cargo.toml` (root) declares four members:

```toml
[workspace]
members = [
    "crates/domain",
    "crates/application",
    "crates/infrastructure",
    "crates/presentation",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
```

Shared dependencies (`async-trait`, `thiserror`, `serde`, `serde_json`, `tokio`, `hf-hub`, `sha2`, `tempfile`, `clap`, `tracing`) are declared once under `[workspace.dependencies]` in the root manifest.

Status of each layer:

| Crate | Role | Implemented |
|---|---|---|
| `domain` | value objects, state machine, ports | yes |
| `application` | use cases against the ports | scaffold only |
| `infrastructure` | adapters (hf-hub, fs, sha2) | scaffold only |
| `presentation` | CLI entry point | scaffold only |

The three non-domain crates currently contain only a one-line crate doc so the workspace resolves; see [roadmap.md](roadmap.md) for the planned work.

## 2. Domain crate (`crates/domain`)

The domain is deliberately free of I/O. It owns what a model is and what "correctly installed" means, and it declares the contracts infrastructure must implement.

### 2.1 Module map (file -> public type)

```
crates/domain/src
├── lib.rs                      (crate root: modules + re-exports)
├── byte_length.rs              -> ByteLength
├── domain_error.rs             -> DomainError
├── model_artifact.rs           -> ModelArtifact
├── model_file_name.rs          -> ModelFileName
├── model_id.rs                 -> ModelId
├── model_plan.rs               -> ModelPlan
├── model_repository_id.rs      -> ModelRepositoryId
├── model_repository.rs         -> ModelRepository
├── model_revision.rs           -> ModelRevision
├── model_spec.rs               -> ModelSpec
├── model_state.rs              -> ModelState
├── remote_model_file.rs        -> RemoteModelFile
├── sha256.rs                   -> Sha256
└── ports
    ├── model_downloader.rs     -> ModelDownloader (trait)
    ├── model_download_error.rs -> ModelDownloadError
    ├── model_library.rs        -> ModelLibrary (trait)
    ├── library_error.rs        -> LibraryError
    ├── remote_model_registry.rs-> RemoteModelRegistry (trait)
    ├── registry_read_error.rs  -> RegistryReadError
    └── mod.rs                  (module root: re-exports)
```

### 2.2 Value objects (concrete signatures)

The most important constructors and accessors, copied verbatim from source.

#### `Sha256` - `crates/domain/src/sha256.rs`

```rust
pub struct Sha256([u8; 32]);

impl Sha256 {
    pub const fn from_bytes(raw: [u8; 32]) -> Self;
    pub fn parse(literal: &str) -> Result<Self, DomainError>;  // 64 hex chars
    pub fn to_hex(self) -> String;
    pub fn as_bytes(&self) -> &[u8; 32];
    pub fn into_bytes(self) -> [u8; 32];
}
```

#### `ModelRepositoryId` - `crates/domain/src/model_repository_id.rs`

```rust
pub struct ModelRepositoryId(String);   // "owner/name", two non-empty segments

impl ModelRepositoryId {
    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError>;
    pub fn owner(&self) -> &str;
    pub fn name(&self) -> &str;
    pub fn as_str(&self) -> &str;
}
```

#### `ModelRevision` - `crates/domain/src/model_revision.rs`

```rust
pub struct ModelRevision(String);       // branch/tag/commit, default "main"

impl ModelRevision {
    pub const DEFAULT_REVISION: &'static str = "main";
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError>;
    pub fn main() -> Self;
    pub fn is_default(&self) -> bool;
    pub fn as_str(&self) -> &str;
}
```

#### `ModelRepository` - `crates/domain/src/model_repository.rs`

```rust
pub struct ModelRepository {
    identifier: ModelRepositoryId,
    revision: ModelRevision,
}

impl ModelRepository {
    pub fn new(identifier: ModelRepositoryId, revision: ModelRevision) -> Self;
    pub fn at_default_revision(identifier: ModelRepositoryId) -> Self;
    pub fn identifier(&self) -> &ModelRepositoryId;
    pub fn revision(&self) -> &ModelRevision;
}
```

#### `ModelId` - `crates/domain/src/model_id.rs`

```rust
pub struct ModelId(String);              // local install name, e.g. "qwen3-8b"

impl ModelId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError>;  // rejects blank
    pub fn as_str(&self) -> &str;
}
```

#### `ModelFileName` - `crates/domain/src/model_file_name.rs`

```rust
pub struct ModelFileName(String);        // one plain relative file name

impl ModelFileName {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError>;
    pub fn as_str(&self) -> &str;
}
```

#### `RemoteModelFile` - `crates/domain/src/remote_model_file.rs`

```rust
pub struct RemoteModelFile {
    repository: ModelRepository,
    file: ModelFileName,
    size: ByteLength,
    sha256: Option<Sha256>,
}

impl RemoteModelFile {
    pub fn new(repository: ModelRepository, file: ModelFileName,
               size: ByteLength, sha256: Option<Sha256>) -> Self;
    pub fn repository(&self) -> &ModelRepository;
    pub fn file(&self) -> &ModelFileName;
    pub fn size(&self) -> ByteLength;
    pub fn has_checksum(&self) -> bool;
    pub fn verify_against(&self, actual: Sha256) -> Result<(), DomainError>;
}
```

#### `ModelSpec` - `crates/domain/src/model_spec.rs`

```rust
pub struct ModelSpec { id, repository, file }

impl ModelSpec {
    pub fn new(id: ModelId, repository: ModelRepository, file: ModelFileName) -> Self;
    pub fn id(&self) -> &ModelId;
    pub fn repository(&self) -> &ModelRepository;
    pub fn file(&self) -> &ModelFileName;
}
```

#### `ModelArtifact` - `crates/domain/src/model_artifact.rs`

```rust
pub struct ModelArtifact { staged_at: PathBuf, size: ByteLength, origin: ModelRepository }

impl ModelArtifact {
    pub fn new(staged_at: impl Into<PathBuf>, size: ByteLength, origin: ModelRepository) -> Self;
    pub fn staged_at(&self) -> &Path;
    pub fn size(&self) -> ByteLength;
    pub fn origin(&self) -> &ModelRepository;
}
```

### 2.3 Install lifecycle (state machine + plan)

`ModelState` - `crates/domain/src/model_state.rs`:

```rust
pub enum ModelState {
    Missing,
    Downloaded,
    Verified,
    IntegrityMismatch { expected: Sha256, actual: Sha256 },
}

impl ModelState {
    pub fn is_ready(self) -> bool;
    pub fn needs_fetch(self) -> bool;
}
```

`ModelPlan` - `crates/domain/src/model_plan.rs` is derived from the state via `From<ModelState>`:

```rust
pub enum ModelPlan {
    Ignore,   // state Verified
    Fetch,    // state Missing
    Verify,   // state Downloaded
    Repair,   // state IntegrityMismatch
}
```

```rust
impl From<ModelState> for ModelPlan {
    fn from(state: ModelState) -> Self {
        match state {
            ModelState::Verified => Self::Ignore,
            ModelState::Missing => Self::Fetch,
            ModelState::Downloaded => Self::Verify,
            ModelState::IntegrityMismatch { .. } => Self::Repair,
        }
    }
}
```

### 2.4 Domain errors - `crates/domain/src/domain_error.rs`

```rust
pub enum DomainError {
    EmptyModelId,
    EmptyRevision,
    MalformedRepository(String),
    InvalidFileName(String),
    InvalidSha256Literal(String),
    IntegrityMismatch { expected: String, actual: String },
}
```

### 2.5 Ports (contracts for infrastructure)

All ports are `#[async_trait]` traits, `Send + Sync`, defined in the domain so the inner layer owns the abstraction.

#### `RemoteModelRegistry` - `crates/domain/src/ports/remote_model_registry.rs`

```rust
#[async_trait]
pub trait RemoteModelRegistry: Send + Sync {
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError>;

    async fn list_repository_files(
        &self,
        repository: &ModelRepository,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError> {
        let _ = repository;
        Err(RegistryReadError::EnumerationUnsupported)
    }
}
```

#### `ModelDownloader` - `crates/domain/src/ports/model_downloader.rs`

```rust
#[async_trait]
pub trait ModelDownloader: Send + Sync {
    async fn fetch(&self, remote: &RemoteModelFile) -> Result<ModelArtifact, ModelDownloadError>;
}
```

#### `ModelLibrary` - `crates/domain/src/ports/model_library.rs`

```rust
#[async_trait]
pub trait ModelLibrary: Send + Sync {
    async fn installed_state(&self, model: &ModelSpec) -> Result<ModelState, LibraryError>;
    async fn commit_artifact(
        &self,
        model: &ModelSpec,
        artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError>;
    async fn verify_integrity(
        &self,
        model: &ModelSpec,
        expected: Option<Sha256>,
    ) -> Result<ModelState, LibraryError>;
}
```

Error enums: `RegistryReadError`, `ModelDownloadError`, `LibraryError` (files, respectively, `registry_read_error.rs`, `model_download_error.rs`, `library_error.rs`).

## 3. Running and validating

All commands run from the repository root:

```sh
cargo build --workspace
cargo test -p domain          # 24 unit tests (state-based, no mocks, no I/O)
cargo clippy --workspace --all-targets
cargo fmt --all
cargo doc -p domain --no-deps
```

Current test count can be checked with `cargo test -p domain -- --list`.

See [roadmap.md](roadmap.md) for what is planned next.

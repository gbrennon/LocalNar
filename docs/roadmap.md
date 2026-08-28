# Model downloader automation - roadmap

Ordered by dependency. Everything below follows the existing domain contracts in
`crates/domain/src`; items are intentionally small and testable. The `application`,
`infrastructure`, and `presentation` crates currently compile as empty scaffolds.

## Done

- [x] Workspace scaffold: four crates under `crates/`, shared deps in root `Cargo.toml`.
- [x] Domain crate: value objects, install state machine, and the three ports with
      their error types (24 unit tests).

## Planned

### 1. Infrastructure adapters (`crates/infrastructure`)

Implement each domain port behind the existing traits. Adapters stay out of domain
code; domain purity is preserved.

- [ ] `HfHubRegistry` implements `RemoteModelRegistry` using `hf-hub` to enumerate
      and resolve files in a repository.
- [ ] `HfHubDownloader` implements `ModelDownloader`, streaming a resolved
      `RemoteModelFile` into a `tempfile`-backed staging area and producing a
      `ModelArtifact`.
- [ ] `FileSystemLibrary` implements `ModelLibrary`, mapping a `ModelSpec` to an
      on-disk path, hashing committed files with `sha2`, and returning `ModelState`.

Each adapter gets an integration test against a stub/real backing, plus unit tests
for its pure helpers. Error map:

- registry failures -> `RegistryReadError`
- transfer failures -> `ModelDownloadError`
- filesystem/hash failures -> `LibraryError`

### 2. Application layer (`crates/application`)

A small use-case/orchestration crate that wires the three ports together and
returns a `ModelPlan` decision. Planned API:

```rust
pub struct InstallModel {
    registry: Arc<dyn RemoteModelRegistry>,
    downloader: Arc<dyn ModelDownloader>,
    library: Arc<dyn ModelLibrary>,
}

impl InstallModel {
    pub async fn inspect(&self, spec: &ModelSpec) -> Result<ModelPlan, ?Error>;
    pub async fn ensure_installed(&self, spec: &ModelSpec) -> Result<ModelState, ?Error>;
}
```

Behavior follows the state machine already in the domain:
`Missing -> Fetch`, `Downloaded -> Verify`, `Verified -> Ignore`,
`IntegrityMismatch -> Repair`. The application layer holds no I/O; it composes the
injected ports.

### 3. Presentation layer (`crates/presentation`)

- [ ] A `clap`-based CLI: a subcommand (e.g.
      `install --id qwen3-8b --repo unsloth/Qwen3-8B-GGUF --file Qwen3-8B-Q4_K_M.gguf`)
      that constructs the adapters, runs the application use case, and prints the
      resulting `ModelPlan`/`ModelState`.
- [ ] Optional: a `check` subcommand that only inspects installed state.

### 4. Root binary

Once `presentation` exposes an entry point, wire it into the root package so
`cargo run` starts the CLI. The placeholder `src/main.rs` prints "Hello, world!"
and is not part of the workspace; decide whether the root package delegates to
`crates/presentation` or the bin lives there.

## Cross-cutting conventions

- **One type per file**, filename = snake_case of the type.
- **Domain purity**: no I/O or SDK inside `domain`; all external work is a port/adapter.
- **Errors** are typed per port; adapters translate dependency failures into the
  domain error types.
- **"downloader" naming** is always `ModelDownloader` / `ModelDownloadError`.
- **Docstrings, no inline comments**; docs stay close to the code.

## Test conventions

- Domain tests are state-based unit tests: no mocking framework, no I/O.
- Adapter tests are integration tests against a real dependency where possible
  (a tiny public `hf-hub` repo or a local fixture server); otherwise a fake/stub
  that implements the port for exactly one scenario.
- Do not mock what you own; inject fakes through the port.

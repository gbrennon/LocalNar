# LocalNar model manager - current state

LocalNar is a local model manager: it discovers models in a remote catalog,
installs them into a durable library on disk, and then gives the operator total
control over what that library holds - listing, inspecting, proving, deleting,
and sweeping it.

This document reflects the code as committed. When in doubt, trust the code over
this prose: the signatures under `crates/*/src` cannot drift, this file can.

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
```

Every layer is implemented:

| Crate | Role |
|---|---|
| `domain` | value objects, install state machine, what the library holds |
| `application` | inbound/outbound ports, typed errors, use-case services |
| `infrastructure` | adapters: Hugging Face registry, downloader, disk library |
| `presentation` | the TUI that drives the use cases |

The dependency rule points inwards. `domain` depends on nothing;
`application` depends only on `domain`; `infrastructure` and `presentation`
depend on the layers inside them and never the reverse.

Ports are plain traits using native `async fn` in trait position, enabled by
`#![allow(async_fn_in_trait)]` in the `application` crate root. There is no
`#[async_trait]` anywhere in the workspace.

## 2. Domain crate (`crates/domain`)

Free of I/O. It owns what a model is, what "correctly installed" means, and what
the library holds. One type per file, filename = `snake_case(type)`.

```
crates/domain/src
├── byte_length.rs          ByteLength
├── checksum.rs             Checksum
├── context_length.rs       ContextLength
├── discarded_stray.rs      DiscardedStray
├── domain_error.rs         DomainError
├── installed_model.rs      InstalledModel
├── managed_model.rs        ManagedModel
├── model_artifact.rs       ModelArtifact
├── model_file_name.rs      ModelFileName
├── model_info.rs           ModelInfo
├── model_inventory.rs      ModelInventory
├── model_profile.rs        ModelProfile
├── model_repository.rs     ModelRepository
├── model_repository_id.rs  ModelRepositoryId
├── model_revision.rs       ModelRevision
├── model_spec.rs           ModelSpec
├── model_state.rs          ModelState
├── model_weight_choice.rs  ModelWeightChoice
├── parameter_count.rs      ParameterCount
├── quantization.rs         Quantization
├── remote_model_file.rs    RemoteModelFile
├── removed_model.rs        RemovedModel
└── search_query.rs         SearchQuery
```

### 2.1 Naming one model

`ModelSpec` names a model: a `ModelRepository` (a `ModelRepositoryId` of
`<owner>/<name>` plus a `ModelRevision`, default `main`) and a `ModelFileName`.
A spec is the identity every use case takes.

`Checksum` is a SHA-256 digest, built from 32 raw bytes or parsed from a
64-character hex literal. `ByteLength` is a byte count that renders itself
(`4.7 GiB`).

### 2.2 Install lifecycle

`ModelState` - `crates/domain/src/model_state.rs`:

```rust
pub enum ModelState {
    Missing,
    Downloaded,
    Verified,
    IntegrityMismatch { expected: Checksum, actual: Checksum },
}
```

`Downloaded` means bytes are on disk but nothing has proved them. `Verified`
means the library recorded a digest it proved. The distinction is load-bearing:
an upstream that advertises no checksum yields an installed but unproven
replica, and the manager reports that honestly rather than claiming success.

### 2.3 What the library holds

- `InstalledModel` - a replica that exists: its spec, path, size, and optional
  proven digest.
- `ManagedModel` - an `InstalledModel` paired with its `ModelState`, answering
  `is_verified`, `is_unproven`, `is_broken`.
- `ModelInventory` - the library's root path plus every `ManagedModel` in it,
  with `count`, `total_size`, `verified_count`, `broken_count`, `find`.
- `RemovedModel` - what a deletion reclaimed: spec, path, `ByteLength`.
- `DiscardedStray` - one leftover a sweep discarded, plus `total_reclaimed`.

### 2.4 Domain errors - `crates/domain/src/domain_error.rs`

`BlankSearchQuery`, `EmptyRevision`, `MalformedRepository`, `InvalidFileName`,
`InvalidChecksumLiteral`. Construction is fallible where a value has a rule;
nothing else validates on the operator's behalf.

## 3. Application crate (`crates/application`)

Owns the boundary: the ports, the typed errors, and the services that
orchestrate them. No I/O lives here.

### 3.1 Inbound ports - what a driver may ask for

| Port | Question it answers |
|---|---|
| `SearchModelsPort` | which models does the catalog offer for this text |
| `InstallModelPort` | bring this model to its verified state |
| `ListInstalledModelsPort` | what does this machine hold |
| `InspectModelPort` | everything known about this one replica |
| `VerifyModelPort` | do this replica's bytes still match its digest |
| `RemoveModelPort` | discard this replica and reclaim its space |
| `PruneLibraryPort` | discard what the library keeps that is no model |

### 3.2 Outbound ports - what adapters must provide

| Port | Responsibility |
|---|---|
| `RemoteModelRegistryPort` | read the remote catalog |
| `ModelDownloaderPort` | fetch bytes to a staging place |
| `ModelLibraryPort` | durable store: state, commit, verify, locate |
| `ModelInventoryPort` | enumerate the whole library |
| `ModelEvictionPort` | discard one replica |
| `LibraryMaintenancePort` | discard the library's leftovers |
| `DownloadProgressPort` | observe a transfer in flight |

### 3.3 Services

One service per inbound port, each taking exactly the outbound ports its use
case needs as generic parameters, so calls stay statically dispatched:
`SearchModelsService`, `InstallModelService`, `ListInstalledModelsService`,
`InspectModelService`, `VerifyModelService`, `RemoveModelService`,
`PruneLibraryService`.

`InstallModelService` drives the state machine: `Verified` is a no-op,
`Downloaded` verifies, `Missing` fetches then commits then verifies, and
`IntegrityMismatch` repairs once before returning
`InstallModelError::UnresolvedIntegrity`.

`VerifyModelService` re-hashes against the digest the library itself recorded -
no network call. A replica with no recorded digest cannot be proven, and the
service says so rather than inventing a verdict.

### 3.4 Errors

One error type per use case, so a failing boundary is never flattened into an
opaque string: `SearchModelsError`, `InstallModelError`,
`ListInstalledModelsError`, `InspectModelError`, `VerifyModelError`,
`RemoveModelError`, `PruneLibraryError`, plus the outbound `LibraryError`,
`RegistryReadError`, `ModelDownloadError`.

## 4. Infrastructure crate (`crates/infrastructure`)

```
crates/infrastructure/src
├── adapters/
│   ├── progress_bus.rs        broadcast bus for progress events
│   └── progress_reporter.rs   DownloadProgressPort implementation
├── persistence/disk/
│   ├── model_library.rs       ModelLibraryPort + the path layout
│   ├── model_inventory.rs     ModelInventoryPort
│   ├── model_eviction.rs      ModelEvictionPort
│   ├── library_maintenance.rs LibraryMaintenancePort
│   ├── library_tree.rs        reading and pruning the directory tree
│   ├── inventory_walk.rs      reading replicas out of the hierarchy
│   ├── library_sweep.rs       finding and discarding leftovers
│   └── library_fault.rs       building path-based LibraryError values
└── remote/huggingface/
    ├── registry.rs            RemoteModelRegistryPort
    └── downloader.rs          ModelDownloaderPort
```

### 4.1 Disk layout

`DiskModelLibrary` stores each replica at:

```
<root>/<owner>/<name>/<revision>/<file>
<root>/<owner>/<name>/<revision>/<file>.sha256   <- the digest note
```

The root comes from `$LOCALNAR_MODELS_DIR`, defaulting to
`~/.cache/localnar/models`. `model_file_path` is the single place that knows
this layout; the sibling adapters read it rather than restating it.

### 4.2 Reading the library cheaply

`ModelInventoryPort::enumerate` walks owner -> name -> revision -> file and
trusts the digest note to decide `Verified` versus `Downloaded`. It never hashes
a file, which is what keeps listing a library of multi-gigabyte models cheap.
Proving bytes is `VerifyModelPort`'s job, on demand, for one model.

Anything in the tree that names no model is left out rather than reported as a
broken entry: a digest note, an entry at the wrong depth, and a path segment the
domain refuses all stand for no model.

### 4.3 Deleting and sweeping

Eviction reads the size before removing anything, discards the replica and its
digest note together, then discards the directories that model alone needed -
stopping short of the root.

A sweep discards only two things: digest notes whose replica is gone, and
directories left holding nothing. A replica the operator installed is never a
leftover, proven or not, and neither is a file the library did not put there.

## 5. Presentation crate (`crates/presentation`)

A `ratatui` + `crossterm` TUI with five modes - search, model table, install
progress, library, and help - cycled with `Tab`/`Shift+Tab`.

`TuiApp` is the composition root; `LibraryManager` owns the five management use
cases and turns each outcome into an `AppEvent`, so the widgets never call a
service directly. Key bindings are documented in the README.

## 6. Running and validating

All commands run from the repository root:

```sh
cargo run                    # start the TUI
./scripts/verify.sh          # fmt --check, build, test, clippy -D warnings
cargo test --workspace       # every suite
```

`scripts/verify.sh` is the gate; it must exit 0 before a change lands.

## 7. Conventions

- **One type per file**, filename = `snake_case(type)`. Package initializers
  (`mod.rs`) only re-export.
- **No inline comments.** Names carry the meaning; doc comments state the
  contract on a port and the behavior on an implementation.
- **No setters.** A method is named for what it does, not for the field it
  writes.
- **Ports before adapters.** The inner layer owns the abstraction.
- Domain and application tests are state-based with hand-written fakes, one fake
  per file, each standing for exactly one scenario. Infrastructure tests are
  integration tests against a real filesystem via `tempfile`.

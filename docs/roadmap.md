# LocalNar model manager - roadmap

LocalNar is a local model manager. Automating `llama.cpp` is no longer the point
of this application; running a server against a managed model is a separate
concern, documented as a runbook in the README.

Ordered by dependency. Items are intentionally small and testable.

## Done

- [x] Workspace: four crates under `crates/`, shared deps in root `Cargo.toml`,
      dependency rule pointing inwards.
- [x] Domain: value objects, the install state machine (`ModelState`), and the
      types describing what the library holds - `ManagedModel`,
      `ModelInventory`, `RemovedModel`, `DiscardedStray`.
- [x] Application: seven inbound ports, seven outbound ports, one typed error
      per use case, and one service per port.
- [x] Infrastructure: Hugging Face registry and downloader; `DiskModelLibrary`
      implementing the library, inventory, eviction, and maintenance ports.
- [x] Presentation: a `ratatui` TUI with search, model table, install progress,
      library, and help modes.
- [x] Search a remote catalog and install a model, verifying its checksum.
- [x] Total control over the local library: list, inspect, verify, delete with
      confirmation, and prune leftovers.

## Planned

### 1. A non-interactive command surface

The TUI is currently the only driver. A `clap`-based CLI over the same inbound
ports would make the manager scriptable and testable end to end without a
terminal.

- [ ] `localnar list` printing the inventory.
- [ ] `localnar install <repo> <file>`, `localnar verify`, `localnar remove`,
      `localnar prune`.
- [ ] Reuse the existing services untouched; the CLI is a second driving adapter,
      not a second implementation.

### 2. Richer library reporting

- [ ] Report a replica's age and last verification time, which needs the library
      to record when it proved a digest.
- [ ] Group the inventory by repository so several revisions of one model read as
      one entry with its variants.

### 3. Relocating the library

- [ ] Move the library root, migrating existing replicas and their digest notes
      rather than orphaning them.

### 4. Local inference

Serving a managed model is the intended next capability, kept deliberately
behind the manager: the manager owns what is on disk, and any runtime consumes
that through a port rather than reaching into the filesystem itself.

- [ ] A port describing "serve this installed model", with the runtime as an
      adapter behind it.

## Conventions

- **One type per file**, filename = `snake_case(type)`. `mod.rs` only re-exports.
- **Ports before adapters**: the inner layer owns the abstraction.
- **Domain purity**: no I/O or SDK inside `domain`.
- **No inline comments**; expressive names, with doc comments stating the
  contract on ports and the behavior on implementations.
- **No setters**; methods are named for what they do.

## Test conventions

- Domain and application tests are state-based unit tests: no mocking framework,
  no I/O, one hand-written fake per file standing for exactly one scenario.
- Infrastructure tests are integration tests against a real filesystem via
  `tempfile`.
- Presentation tests render widgets against `ratatui`'s `TestBackend` and assert
  on the rendered cells.
- `./verify.sh` (fmt, build, test, clippy `-D warnings`) must exit 0.

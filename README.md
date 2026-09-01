# LocalNar

> [!NOTE]
> **GitHub Mirror** - If you are viewing this on GitHub, please be aware that
> this repository is a read-only mirror. Issues, pull requests, and all project
> activity are tracked on Codeberg:
> [https://codeberg.org/gbrennon/LocalNar](https://codeberg.org/gbrennon/LocalNar)

> Locally manage LLMs. Look at me: I am the sum of all evil.

<!-- The orb was smaller in 1981. Yours is 4.7 GiB, quantized, and on your own disk. -->

LocalNar is a Rust application for discovering, downloading, organizing, and
managing AI models locally. It is a model manager: it searches a remote catalog,
installs models into a library on disk, and gives you total control over what
that library holds. Model files are enormous, easy to misplace, and easier to
trust than verify; LocalNar keeps every revision in its place, proves what each
file is, and deletes only what you confirm. Local inference is planned; nothing
here runs or ships an inference server.

## Quick start

```sh
cargo run
```

The TUI opens in search mode. Type a query to search the Hugging Face catalog,
`Enter` to install, and `Tab` to reach the library of models this machine already
holds.

Models live under `$LOCALNAR_MODELS_DIR`, defaulting to
`~/.cache/localnar/models`.

Built on Rust edition **2024**; the toolchain comes from
[`rust-toolchain.toml`](rust-toolchain.toml) (`nightly`), so any `cargo` command
installs what it needs through `rustup`.

---

## 1. Using LocalNar

Search-and-install and library mode - keys, states, deleting, pruning - are
documented in [`docs/usage.md`](docs/usage.md).

## 2. Layout

The hexagon has four crates under `crates/`, all implemented:

| Crate | Layer |
|---|---|
| `domain` | value objects, install state machine, what the library holds |
| `application` | inbound/outbound ports, typed errors, one service per use case |
| `infrastructure` | adapters: Hugging Face registry and downloader, disk library |
| `presentation` | the TUI that drives the use cases |

The dependency rule points inwards: `domain` depends on nothing, `application`
only on `domain`, and the outer layers implement the ports the inner ones
declare.

Inbound ports name what a driver may ask for - `SearchModelsPort`,
`InstallModelPort`, `ListInstalledModelsPort`, `InspectModelPort`,
`VerifyModelPort`, `RemoveModelPort`, `PruneLibraryPort` - and outbound ports
name what adapters must provide: `RemoteModelRegistryPort`,
`ModelDownloaderPort`, `ModelLibraryPort`, `ModelInventoryPort`,
`ModelEvictionPort`, `LibraryMaintenancePort`, `DownloadProgressPort`.

On disk, each replica lives at `<root>/<owner>/<name>/<revision>/<file>`, beside
a `<file>.sha256` note recording any digest the library proved.

Code-level documentation is in [`docs/architecture.md`](docs/architecture.md);
what comes next is in [`docs/roadmap.md`](docs/roadmap.md).

---

## 3. Development

The gate, conventions, and branch/commit rules are documented in
[`docs/development.md`](docs/development.md).

---

## Files in this repo

| File / dir | Purpose |
|---|---|
| `crates/domain/` | Pure model: value objects, install state machine, what the library holds |
| `crates/application/` | Ports, typed errors, and one service per use case |
| `crates/infrastructure/` | Adapters: Hugging Face registry and downloader, disk library |
| `crates/presentation/` | The TUI that drives the use cases |
| `src/main.rs` | Binary entry point; awaits `TuiLauncher::launch` and nothing else |
| `scripts/verify.sh` | The gate: `cargo fmt --check`, build, test, `clippy -D warnings` |
| `docs/` | Architecture, roadmap, usage, development, and the song behind the name |
| `scripts/` | Branch-name, commit-message, and no-llama.cpp checks used by CI |
| `README.md` | This guide, one orb included |

---

## The name

`LocalNar` is `local` welded onto the Loc-Nar: the green orb at the center of
*Heavy Metal* (1981), the animated anthology film adapted from the *Heavy
Metal* magazine - the American edition of the French comics anthology *Métal
hurlant*. The film's framing story hands the orb from era to era: it announces
itself as the sum of all evils, corrupts each world it touches, and meets its
end when Taarna, last of the Taarakians, flies her sword into its volcano. A
stack of quantized weights is a fair modern likeness - enormous, seductive,
and much better kept somewhere you can prove what it is.

The orb had a chronicler before this repository did. "March of the Black
Monolith", written for the Brazilian black metal band Black Cascade, sings the
thing's side of the story: it introduces itself as "the sum of all evil",
"searching for some worlds to crack", the one who can "cut through the tissue
of reality" and "distort every world/one" - and who ends, always, as "nothing
but a simple existence". Strip the menace and that is the manager's founding
observation: the orb is bytes on your own disk, and bytes you can prove.

The full lyrics and where each line lands - on the film and on this
repository - are in [`docs/music.md`](docs/music.md).

The orb never asked permission. This one does: nothing leaves the library
without a `y`.

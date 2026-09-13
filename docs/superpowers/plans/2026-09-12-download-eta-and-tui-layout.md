# Download ETA and TUI Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Organize the presentation TUI by responsibility and display a readable download ETA.

**Architecture:** Keep `TuiApp` as the TUI controller and move its user-facing rendering into
explicit screen modules. Keep reusable ratatui widgets separate from screen composition, and group
input, integrations, download progress, layout, terminal execution, and startup assembly by their behavior. ETA
remains in the presentation progress integration and does not alter domain or infrastructure
contracts.

**Tech Stack:** Rust, Tokio, Ratatui, Crossterm, Cargo tests, Clippy.

## Global Constraints

- Preserve existing public TUI behavior and event flow.
- Keep domain and infrastructure contracts unchanged.
- Calculate ETA as remaining bytes divided by measured bytes per second.
- Show `ETA calculating...` when no usable speed exists.
- Use compact seconds, minutes, and hours for ETA output.
- Keep each file focused on one responsibility.
- Add tests before production changes for the ETA behavior.

---

## File map

- `crates/presentation/src/tui/application/`: TUI controller, state, modes, tabs, and events.
- `crates/presentation/src/tui/screens/`: complete user-facing screen compositions.
- `crates/presentation/src/tui/widgets/`: existing reusable visual widgets, moved from `components`.
- `crates/presentation/src/tui/interactions/`: keyboard/event polling input handling.
- `crates/presentation/src/tui/integrations/`: application/infrastructure-to-TUI coordination.
- `crates/presentation/src/tui/download_progress/`: transfer-rate sampling and progress/ETA formatting.
- `crates/presentation/src/tui/layout/`: reusable screen geometry helpers.
- `crates/presentation/src/tui/event_loop/`: input polling, redraws, and application ticks.
- `crates/presentation/src/tui/terminal/`: terminal ownership and restoration.
- `crates/presentation/src/tui/composition/`: dependency assembly and startup errors.

## Task 1: Move TUI files into responsibility-based modules

**Files:**
- Create: `crates/presentation/src/tui/application/`
- Create: `crates/presentation/src/tui/screens/`
- Create: `crates/presentation/src/tui/widgets/`
- Create: `crates/presentation/src/tui/interactions/`
- Create: `crates/presentation/src/tui/integrations/`
- Create: `crates/presentation/src/tui/terminal/`
- Create: `crates/presentation/src/tui/composition/`
- Modify: `crates/presentation/src/tui/mod.rs`
- Modify: module imports throughout `crates/presentation/src/tui/`

- [ ] **Step 1: Move files without changing behavior.**

Move the existing files according to this map:

```text
application: app_event.rs, app_mode.rs, app_tab.rs, tui_app.rs
widgets: all files currently under components/
interactions: events.rs
integrations: library_manager.rs
download_progress: download_speed_tracker.rs, progress_reporter.rs
layout: layout_helper.rs
terminal: terminal_session.rs
event_loop: ui.rs renamed to app_runner.rs
composition: tui_launcher.rs, tui_launch_error.rs
```

Create screen modules for the existing screen branches and move only screen-specific rendering
out of `TuiApp`:

```text
screens: search_screen.rs, library_screen.rs, help_screen.rs,
         settings_screen.rs, install_progress_screen.rs, overlays.rs
```

Each screen must receive the state or widget references it needs and preserve the current rendered
output. `TuiApp` retains event handling, state ownership, and use-case invocation.

- [ ] **Step 2: Update declarations and imports.**

Make each directory’s `mod.rs` declare only its immediate files and re-export only its immediate
public contract. Update internal imports to use the new module paths. Keep the existing `tui`
public re-exports available to current consumers.

- [ ] **Step 3: Run formatting and compilation.**

Run:

```bash
cargo fmt --check
cargo check --workspace
```

Expected: both commands succeed with no diagnostics.

## Task 2: Add ETA formatting tests first

**Files:**
- Modify: `crates/presentation/src/tui/download_progress/progress_reporter.rs`

- [ ] **Step 1: Add a failing test for a measured ETA.**

Add a test that establishes a zero-byte baseline, advances to 20,000,000 bytes of a
100,000,000-byte download one second later, and asserts the message contains:

```text
@ 19.1 MiB/s · ETA 4s
```

- [ ] **Step 2: Add a failing test for an unavailable ETA.**

Assert that the first advanced event with zero measured speed contains `ETA calculating...`.

- [ ] **Step 3: Run the focused tests.**

Run:

```bash
cargo test -p localnar-presentation progress_reporter
```

Expected: the new assertions fail because ETA is not present.

## Task 3: Implement and verify ETA formatting

**Files:**
- Modify: `crates/presentation/src/tui/download_progress/progress_reporter.rs`

- [ ] **Step 1: Implement minimal ETA calculation.**

For an advanced event, compute remaining bytes with saturating subtraction. When speed is zero,
append `ETA calculating...`; otherwise compute ceiling remaining seconds from the byte count and
speed and append the compact duration. Preserve the existing speed and progress text.

- [ ] **Step 2: Add duration formatting coverage.**

Test seconds, minutes, and hours using the progress reporter’s formatting behavior.

- [ ] **Step 3: Run focused tests.**

Run:

```bash
cargo test -p localnar-presentation progress_reporter
```

Expected: all progress reporter tests pass.

## Task 4: Verify TUI rendering and repository quality

**Files:**
- Modify: `crates/presentation/tests/tui_app_tests.rs`
- Modify: `docs/usage.md` if the progress wording needs documentation.

- [ ] **Step 1: Extend the TUI rendering test.**

Assert that a progress event containing a measured rate renders both the speed and `ETA` in the
progress screen.

- [ ] **Step 2: Run presentation tests.**

Run:

```bash
cargo test -p localnar-presentation
```

Expected: all presentation tests pass.

- [ ] **Step 3: Run the repository verification command.**

Run:

```bash
./scripts/verify.sh
```

Expected: formatting, build, tests, and Clippy pass.

- [ ] **Step 4: Run available structural quality checks.**

Run `check-code-quality` when available. Otherwise run the repository’s configured equivalent.
Expected: zero violations in changed files.

## Self-review

Every specification requirement maps to a task: module organization is covered by Task 1, ETA
states and formatting by Tasks 2 and 3, visible rendering by Task 4, and full verification by
Task 4. No placeholders or unresolved design choices remain.

# Download ETA and TUI Layout Design

This specification adds a readable download ETA and organizes the TUI into named modules.

## Scope

- Show an ETA beside download progress and speed.
- Keep ETA unavailable until a usable transfer rate exists.
- Reorganize files currently in `crates/presentation/src/tui/` into focused directories.
- Add a `screens` module so screen rendering has an explicit home.
- Preserve public behavior and existing application event flow.

## User-visible behavior

Progress messages use the existing status string rendered by `ProgressWidget`.

- A started transfer remains `Downloading: 0 B / <total> (0.0%)`.
- An advanced transfer without a measured rate shows `ETA calculating...`.
- An advanced transfer with a measured rate shows `@ <speed>/s · ETA <duration>`.
- ETA is the remaining byte count divided by the current bytes-per-second rate.
- Durations use compact human-readable units: seconds, minutes and hours.
- A completed transfer remains `Download completed`.

The ETA is calculated only in the presentation progress adapter. Domain and infrastructure
contracts remain unchanged.

## TUI module layout

The TUI root will expose focused submodules rather than individual implementation files:

```text
crates/presentation/src/tui/
├── application/
│   ├── app_event.rs
│   ├── app_mode.rs
│   ├── app_tab.rs
│   └── tui_app.rs
├── components/
├── download_progress/
│   ├── download_speed_tracker.rs
│   └── progress_reporter.rs
├── event_loop/
│   ├── app_runner.rs
│   └── events.rs
├── layout/
│   └── layout_helper.rs
├── launching/
│   ├── tui_launch_error.rs
│   └── tui_launcher.rs
├── library_management/
│   └── library_manager.rs
├── screens/
│   ├── help_screen.rs
│   ├── library_screen.rs
│   ├── model_details_screen.rs
│   ├── progress_screen.rs
│   ├── search_screen.rs
│   └── settings_screen.rs
├── terminal/
│   └── terminal_session.rs
└── mod.rs
```

`components/` remains the home for reusable widgets. `screens/` owns the user-facing compositions
for help, library, model details, download progress, search, and settings. Each screen combines
widgets and screen-specific layout; it does not own application state or infrastructure access.
The `application/` package owns state, input coordination, and use-case invocation, then delegates
screen rendering to the active screen. `download_progress/` owns transfer-rate sampling and
progress-message formatting. `integrations/` owns asynchronous library action coordination.
`event_loop/`, `launching/`, and `terminal/` separate recurring execution, startup assembly, and
terminal resource management. `layout/` owns reusable screen geometry rather than display text.

Submodule `mod.rs` files only declare modules and re-export symbols belonging to that immediate
package. The TUI root re-exports the public types currently consumed by the binary and tests.

## Data flow

The infrastructure progress bus continues to emit `ProgressEvent`. `ProgressReporterBridge`
in `download_progress/` continues to sample speed and convert events to `AppEvent::InstallProgress`.
During conversion,
it computes remaining bytes and formats ETA when speed is positive. `TuiApp` continues to pass
the message to `ProgressWidget`, which renders it without parsing or recalculating the value.

## Testing

- Extend progress reporter unit tests for calculating and formatting ETA.
- Cover the zero-speed calculating state.
- Cover completed transfers preserving their existing message.
- Extend the TUI rendering test to assert that ETA is visible with speed and progress.
- Run formatting, compilation, tests, clippy, and the repository verification script.

## Non-goals

- No downloader timing or sampling algorithm changes.
- No new progress event fields.
- No changes to domain entities or application ports.
- No redesign of widget visuals or screen content beyond the ETA text.

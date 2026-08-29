use application::ports::outbound::download_progress_port::DownloadProgressPort;
use crate::adapters::progress_bus::ProgressBusSender;

/// Infrastructure adapter for progress reporting using the event bus pattern.
/// This replaces the presentation-layer TuiProgressReporter.
pub type ProgressReporter = ProgressBusSender;

// DownloadProgressPort is already implemented for ProgressBusSender in progress_bus.rs
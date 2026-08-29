use crate::adapters::progress_bus::ProgressBusSender;

/// The progress reporter every download adapter reports through.
///
/// Reporting is a broadcast on the progress bus, so any number of observers can
/// follow one transfer without the download knowing about them.
pub type ProgressReporter = ProgressBusSender;

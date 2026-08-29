#![allow(dead_code)]
use application::ports::outbound::{DownloadProgress, DownloadProgressPort};

/// A progress sink for scenarios that do not assert on reporting.
pub struct FakeSilentProgress;

impl DownloadProgressPort for FakeSilentProgress {
    fn report(&self, _progress: DownloadProgress) {}
}

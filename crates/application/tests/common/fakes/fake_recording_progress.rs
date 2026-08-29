#![allow(dead_code)]
use std::sync::Mutex;

use application::ports::outbound::{DownloadProgress, DownloadProgressPort};

/// A progress sink that remembers every report in order.
#[derive(Default)]
pub struct FakeRecordingProgress {
    reports: Mutex<Vec<DownloadProgress>>,
}

impl FakeRecordingProgress {
    /// Builds an empty recorder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Every report received so far, oldest first.
    pub fn reports(&self) -> Vec<DownloadProgress> {
        self.reports
            .lock()
            .expect("the recorder must not be poisoned")
            .clone()
    }
}

impl DownloadProgressPort for FakeRecordingProgress {
    fn report(&self, progress: DownloadProgress) {
        self.reports
            .lock()
            .expect("the recorder must not be poisoned")
            .push(progress);
    }
}

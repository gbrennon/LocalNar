use std::path::{Path, PathBuf};

use crate::ByteLength;

/// An entry the library kept that stood for no model, and was discarded.
///
/// A library accumulates leftovers: the record of a digest whose model file was
/// removed by other means, or a directory left behind once its last model went.
/// None of them is a replica, so none of them appears in the inventory, yet they
/// occupy the operator's machine and hide the true shape of the library.
/// Discarding one is reported by its place and the space it gave back, because
/// that is all a leftover ever was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscardedStray {
    path: PathBuf,
    reclaimed: ByteLength,
}

impl DiscardedStray {
    /// Records that the leftover at `path` gave back `reclaimed` bytes.
    pub fn new(path: impl Into<PathBuf>, reclaimed: ByteLength) -> Self {
        Self {
            path: path.into(),
            reclaimed,
        }
    }

    /// The place the leftover used to occupy.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// How much space discarding it gave back.
    pub fn reclaimed(&self) -> ByteLength {
        self.reclaimed
    }

    /// The space a whole sweep of discarded leftovers gave back.
    pub fn total_reclaimed(strays: &[Self]) -> ByteLength {
        ByteLength::new(
            strays
                .iter()
                .map(|stray| stray.reclaimed.bytes())
                .sum::<u64>(),
        )
    }
}

#[cfg(test)]
mod discarded_stray_tests {
    use super::DiscardedStray;
    use crate::ByteLength;

    #[test]
    fn a_discarded_leftover_names_its_place_and_reclaimed_space() {
        let stray = DiscardedStray::new(
            "/models/org/name/main/gone.gguf.sha256",
            ByteLength::new(64),
        );

        assert_eq!(
            stray.path().to_str(),
            Some("/models/org/name/main/gone.gguf.sha256")
        );
        assert_eq!(stray.reclaimed(), ByteLength::new(64));
    }

    #[test]
    fn a_sweep_reports_the_space_of_every_leftover_together() {
        let strays = vec![
            DiscardedStray::new("/models/a.sha256", ByteLength::new(64)),
            DiscardedStray::new("/models/org/name", ByteLength::new(0)),
            DiscardedStray::new("/models/b.sha256", ByteLength::new(128)),
        ];

        assert_eq!(
            DiscardedStray::total_reclaimed(&strays),
            ByteLength::new(192)
        );
    }

    #[test]
    fn a_sweep_that_discarded_nothing_reclaimed_nothing() {
        assert_eq!(DiscardedStray::total_reclaimed(&[]), ByteLength::ZERO);
    }
}

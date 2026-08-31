//! Clearing the leftovers a library's directory tree accumulates.

use std::{cmp::Reverse, path::Path};

use application::errors::LibraryError;
use domain::{ByteLength, DiscardedStray};

use super::{library_tree::LibraryTree, model_library::DiskModelLibrary};

/// Discards everything beneath a root that stands for no model.
///
/// A replica is a model the operator installed, proven or not, and any other
/// file is something the library did not put there, so neither is ever a
/// leftover. What remains are digest notes whose replica is gone and
/// directories left holding nothing.
pub(super) struct LibrarySweep<'root> {
    root: &'root Path,
}

impl<'root> LibrarySweep<'root> {
    /// Prepares to sweep the tree beneath `root`.
    pub(super) fn rooted_at(root: &'root Path) -> Self {
        Self { root }
    }

    /// Discards every leftover the tree holds and names each one discarded.
    ///
    /// Notes go first, because a note is the last thing a departed model can
    /// leave in a directory and discarding it is what makes that directory a
    /// leftover in turn. The answer is ordered by path, so two sweeps of alike
    /// libraries read alike.
    pub(super) async fn discarded_strays(&self) -> Result<Vec<DiscardedStray>, LibraryError> {
        let mut discarded = self.discard_orphaned_notes().await?;
        discarded.extend(self.discard_emptied_directories().await?);
        discarded.sort_by(|discarded, other| discarded.path().cmp(other.path()));

        Ok(discarded)
    }

    /// Discards every digest note whose replica is gone.
    ///
    /// A note records bytes the library proved, so a note whose replica is
    /// absent records nothing and only takes up room. A note whose replica is
    /// present belongs to that replica and is left where it is.
    async fn discard_orphaned_notes(&self) -> Result<Vec<DiscardedStray>, LibraryError> {
        let mut discarded = Vec::new();

        for note in LibraryTree::files_below(self.root).await? {
            let Some(replica) = DiskModelLibrary::companion_of(&note) else {
                continue;
            };

            if LibraryTree::something_occupies(&replica).await? {
                continue;
            }

            let reclaimed = LibraryTree::occupied_space(&note).await?;
            LibraryTree::discard_file(&note).await?;
            discarded.push(DiscardedStray::new(note, reclaimed));
        }

        Ok(discarded)
    }

    /// Discards every directory that holds nothing.
    ///
    /// The deepest directories are offered first, so a directory left holding
    /// nothing by the discarding of its own last sub-directory is discarded in
    /// the same sweep rather than surviving until the next one. A directory
    /// occupies no space of its own, so discarding one reclaims none. The root
    /// is never offered: it is where the library lives, not something a model
    /// left behind.
    async fn discard_emptied_directories(&self) -> Result<Vec<DiscardedStray>, LibraryError> {
        let mut directories = LibraryTree::directories_below(self.root).await?;
        directories.sort_by_key(|directory| Reverse(directory.components().count()));

        let mut discarded = Vec::new();

        for directory in directories {
            if LibraryTree::discard_directory_if_empty(&directory).await? {
                discarded.push(DiscardedStray::new(directory, ByteLength::ZERO));
            }
        }

        Ok(discarded)
    }
}

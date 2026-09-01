//! Reading and pruning the directory tree the disk library keeps.

use std::{
    fs::Metadata,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use localnar_application::errors::LibraryError;
use localnar_domain::ByteLength;
use tokio::fs::DirEntry;

use super::library_fault::LibraryFault;

/// Reads and prunes the directory tree beneath a library root.
///
/// Every adapter that manages the library walks the same tree, so the walking
/// is decided once here and the adapters are left to decide only what their own
/// job makes of what they find.
pub(super) struct LibraryTree;

impl LibraryTree {
    /// Lists what `directory` holds, treating an absent directory as holding
    /// nothing.
    ///
    /// The library's root comes into being with the first install, so an
    /// operator who has installed nothing has no root at all: that is an empty
    /// library rather than a fault, and the same reading covers a branch that
    /// vanishes while a walk is under way. A directory that exists and still
    /// refuses to be read is a genuine fault and is reported as one.
    pub(super) async fn entries_of(directory: &Path) -> Result<Vec<DirEntry>, LibraryError> {
        let mut reader = match tokio::fs::read_dir(directory).await {
            Ok(reader) => reader,
            Err(cause) if cause.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(cause) => return Err(LibraryFault::unreadable_at(directory, cause)),
        };

        let mut entries = Vec::new();
        while let Some(entry) = reader
            .next_entry()
            .await
            .map_err(|cause| LibraryFault::unreadable_at(directory, cause))?
        {
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Reads what the filesystem says `path` is, or nothing when it says `path`
    /// is gone.
    ///
    /// A walk of a live library races the operator using it, and an entry
    /// listed a moment ago that has since gone stands for nothing now.
    /// Following a link to what it names matches how the library reads a
    /// replica it was asked for, so a walk and a lookup agree on both the kind
    /// and the size of what is there.
    ///
    /// Contrast with leads_deeper: that predicate uses entry.file_type() and does not
    /// follow symlinks, so a pre-placed symlink-to-directory is walked as a file and
    /// never descended into. describe follows the link because the library reads a
    /// replica it was asked for through the same link; the two must agree on kind.
    pub(super) async fn describe(path: &Path) -> Result<Option<Metadata>, LibraryError> {
        match tokio::fs::metadata(path).await {
            Ok(described) => Ok(Some(described)),
            Err(cause) if cause.kind() == ErrorKind::NotFound => Ok(None),
            Err(cause) => Err(LibraryFault::unreadable_at(path, cause)),
        }
    }

    /// Answers whether the filesystem holds anything at `path`.
    pub(super) async fn something_occupies(path: &Path) -> Result<bool, LibraryError> {
        tokio::fs::try_exists(path)
            .await
            .map_err(|cause| LibraryFault::unreadable_at(path, cause))
    }

    /// Measures the space the file at `path` occupies, or none when nothing
    /// occupies it.
    ///
    /// A file that is not there occupies nothing, which is exactly the space a
    /// caller reclaims by discarding it.
    pub(super) async fn occupied_space(path: &Path) -> Result<ByteLength, LibraryError> {
        match Self::describe(path).await? {
            Some(occupant) => Ok(ByteLength::new(occupant.len())),
            None => Ok(ByteLength::ZERO),
        }
    }

    /// Lists every file the tree under `root` holds, at whatever depth it holds
    /// it.
    ///
    /// A link is listed as the file it is rather than followed, so a walk of
    /// the library stays inside the library.
    pub(super) async fn files_below(root: &Path) -> Result<Vec<PathBuf>, LibraryError> {
        let mut files = Vec::new();
        let mut unwalked = vec![root.to_path_buf()];

        while let Some(directory) = unwalked.pop() {
            for entry in Self::entries_of(&directory).await? {
                let path = entry.path();
                if Self::leads_deeper(&entry).await? {
                    unwalked.push(path);
                } else {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    /// Lists every directory the tree under `root` holds, excluding the root
    /// itself.
    ///
    /// A link naming a directory elsewhere is neither walked into nor listed:
    /// what lies outside the library is not the library's to prune.
    pub(super) async fn directories_below(root: &Path) -> Result<Vec<PathBuf>, LibraryError> {
        let mut directories = Vec::new();
        let mut unwalked = vec![root.to_path_buf()];

        while let Some(directory) = unwalked.pop() {
            for entry in Self::entries_of(&directory).await? {
                if Self::leads_deeper(&entry).await? {
                    unwalked.push(entry.path());
                    directories.push(entry.path());
                }
            }
        }

        Ok(directories)
    }

    /// Discards the file at `path`, accepting a path nothing occupies.
    ///
    /// Callers discard a file to establish that nothing of it is left behind,
    /// and a file that was never there already satisfies that, so its absence
    /// is the outcome asked for rather than a failure to reach it.
    pub(super) async fn discard_file(path: &Path) -> Result<(), LibraryError> {
        match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(cause) if cause.kind() == ErrorKind::NotFound => Ok(()),
            Err(cause) => Err(LibraryFault::unwritable_at(path, cause)),
        }
    }

    /// Discards `path` when it is a directory holding nothing, and answers
    /// whether it went.
    ///
    /// A directory still holding something is something the library keeps, so
    /// it survives and the caller learns it did. Letting the filesystem decide
    /// emptiness during the removal itself, rather than reading the directory
    /// first and removing it after, keeps the answer true of the moment it is
    /// given: no directory that gained an entry in between is ever reported as
    /// discarded.
    pub(super) async fn discard_directory_if_empty(path: &Path) -> Result<bool, LibraryError> {
        match tokio::fs::remove_dir(path).await {
            Ok(()) => Ok(true),
            Err(cause) if cause.kind() == ErrorKind::DirectoryNotEmpty => Ok(false),
            Err(cause) if cause.kind() == ErrorKind::NotFound => Ok(false),
            Err(cause) => Err(LibraryFault::unwritable_at(path, cause)),
        }
    }

    /// Discards each directory above `replica` that its removal left holding
    /// nothing, stopping short of `root`.
    ///
    /// A model is the only reason its revision, name, and owner directories
    /// exist, so once the last of them goes those directories stand for nothing
    /// and would otherwise be listed forever as a repository holding no model.
    /// The walk stops at the first directory that still holds something, since
    /// everything above that one holds it too, and it never reaches the root:
    /// the library's own location is no leftover of any model, and an operator
    /// who discards their last model still has a library.
    pub(super) async fn discard_emptied_ancestors(
        root: &Path,
        replica: &Path,
    ) -> Result<(), LibraryError> {
        let mut ancestor = replica.parent();

        while let Some(directory) = ancestor {
            if directory == root {
                break;
            }

            if !Self::discard_directory_if_empty(directory).await? {
                break;
            }

            ancestor = directory.parent();
        }

        Ok(())
    }

    /// Answers whether `entry` is a directory a walk should descend into.
    async fn leads_deeper(entry: &DirEntry) -> Result<bool, LibraryError> {
        let kind = entry
            .file_type()
            .await
            .map_err(|cause| LibraryFault::unreadable_at(&entry.path(), cause))?;

        Ok(kind.is_dir())
    }
}

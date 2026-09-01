//! Reporting filesystem faults raised against a library path.

use std::{fmt, path::Path};

use localnar_application::errors::LibraryError;

/// Builds the library failures that a path, rather than a named model, raises.
///
/// Managing a library means touching paths no caller named a model for, so the
/// faulting path stands in for the model the error type asks about: it is the
/// only thing known about what failed, and it is what an operator needs in
/// order to fix it.
pub(super) struct LibraryFault;

impl LibraryFault {
    /// Reports that `path` could not be read.
    pub(super) fn unreadable_at(path: &Path, cause: impl fmt::Display) -> LibraryError {
        LibraryError::Unreadable {
            model: path.display().to_string(),
            cause: cause.to_string(),
        }
    }

    /// Reports that `path` could not be discarded.
    ///
    /// Removal is a write, so a removal the filesystem refuses is reported as
    /// the write failure it is rather than as an absent entry.
    pub(super) fn unwritable_at(path: &Path, cause: impl fmt::Display) -> LibraryError {
        LibraryError::Unwritable {
            model: path.display().to_string(),
            cause: cause.to_string(),
        }
    }
}

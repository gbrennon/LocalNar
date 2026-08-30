use std::error::Error;
use std::fmt;

use crate::errors::library_error::LibraryError;

/// Failures that can end a sweep of the library's leftovers.
///
/// A sweep that found nothing to discard is an answer, not a failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PruneLibraryError {
    /// The durable library could not be read or swept.
    Library(LibraryError),
}

impl fmt::Display for PruneLibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Library(cause) => {
                write!(formatter, "the library could not be pruned: {cause}")
            }
        }
    }
}

impl Error for PruneLibraryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Library(cause) => Some(cause),
        }
    }
}

impl From<LibraryError> for PruneLibraryError {
    fn from(cause: LibraryError) -> Self {
        Self::Library(cause)
    }
}

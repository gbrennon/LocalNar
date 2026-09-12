use std::{error::Error, fmt};

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

#[cfg(test)]
mod prune_library_error_tests {
    use super::*;
    use crate::errors::LibraryError;

    #[test]
    fn library_variant_constructs_and_displays() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = PruneLibraryError::Library(cause);
        assert_eq!(
            error.to_string(),
            "the library could not be pruned: could not read the library for model `m`: io"
        );
    }

    #[test]
    fn prune_library_error_source_returns_inner() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = PruneLibraryError::Library(cause);
        assert!(error.source().is_some());
    }

    #[test]
    fn from_library_error_constructs_library_variant() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error: PruneLibraryError = cause.into();
        assert!(matches!(error, PruneLibraryError::Library(_)));
    }

    #[test]
    fn prune_library_error_implements_std_error() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = PruneLibraryError::Library(cause);
        let _: &dyn std::error::Error = &error;
    }
}

use std::{error::Error, fmt};

use crate::errors::library_error::LibraryError;

/// Failures that can end a listing of the local library.
///
/// Listing only reads the durable store, so the library is the single boundary
/// that can fail; the wrapper keeps the outbound error type out of the inbound
/// contract. A library that holds no model is an answer, never a failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListInstalledModelsError {
    /// The durable library could not be read.
    Library(LibraryError),
}

impl fmt::Display for ListInstalledModelsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Library(cause) => {
                write!(
                    formatter,
                    "the installed models could not be listed: {cause}"
                )
            }
        }
    }
}

impl Error for ListInstalledModelsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Library(cause) => Some(cause),
        }
    }
}

impl From<LibraryError> for ListInstalledModelsError {
    fn from(cause: LibraryError) -> Self {
        Self::Library(cause)
    }
}

#[cfg(test)]
mod list_installed_models_error_tests {
    use super::*;
    use crate::errors::LibraryError;

    #[test]
    fn library_variant_constructs_and_displays() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = ListInstalledModelsError::Library(cause);
        assert_eq!(
            error.to_string(),
            "the installed models could not be listed: could not read the library for model `m`: io"
        );
    }

    #[test]
    fn list_installed_models_error_source_returns_inner() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = ListInstalledModelsError::Library(cause);
        assert!(error.source().is_some());
    }

    #[test]
    fn from_library_error_constructs_library_variant() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error: ListInstalledModelsError = cause.into();
        assert!(matches!(error, ListInstalledModelsError::Library(_)));
    }

    #[test]
    fn list_installed_models_error_implements_std_error() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = ListInstalledModelsError::Library(cause);
        let _: &dyn std::error::Error = &error;
    }
}

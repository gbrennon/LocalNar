use std::{error::Error, fmt};

use crate::errors::library_error::LibraryError;

/// Failures that can end the removal of one locally installed model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoveModelError {
    /// The durable library could not be read or the replica could not be
    /// discarded.
    Library(LibraryError),

    /// The library holds no replica of the requested model to remove.
    ///
    /// Reported rather than silently succeeding: an operator reclaiming space
    /// is told the space was never occupied instead of being led to believe a
    /// model was just discarded.
    NotInstalled { model: String },

    /// The replica survived its own removal.
    ///
    /// A removal that answers with reclaimed space promises the bytes are gone.
    /// A library that still reports a replica afterwards has not kept that
    /// promise, and saying so is better than leaving the operator with a model
    /// they believe they discarded.
    StillInstalled { model: String },
}

impl fmt::Display for RemoveModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Library(cause) => {
                write!(formatter, "the model could not be removed: {cause}")
            }
            Self::NotInstalled { model } => {
                write!(
                    formatter,
                    "model `{model}` is not installed locally, so there is nothing to remove"
                )
            }
            Self::StillInstalled { model } => {
                write!(
                    formatter,
                    "model `{model}` is still installed after being removed"
                )
            }
        }
    }
}

impl Error for RemoveModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Library(cause) => Some(cause),
            Self::NotInstalled { .. } => None,
            Self::StillInstalled { .. } => None,
        }
    }
}

impl From<LibraryError> for RemoveModelError {
    fn from(cause: LibraryError) -> Self {
        Self::Library(cause)
    }
}

#[cfg(test)]
mod remove_model_error_tests {
    use super::*;
    use crate::errors::LibraryError;

    #[test]
    fn library_variant_constructs_and_displays() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = RemoveModelError::Library(cause);
        assert_eq!(
            error.to_string(),
            "the model could not be removed: could not read the library for model `m`: io"
        );
    }

    #[test]
    fn not_installed_variant_constructs_and_displays() {
        let error = RemoveModelError::NotInstalled { model: "m".into() };
        assert_eq!(
            error.to_string(),
            "model `m` is not installed locally, so there is nothing to remove"
        );
    }

    #[test]
    fn still_installed_variant_constructs_and_displays() {
        let error = RemoveModelError::StillInstalled { model: "m".into() };
        assert_eq!(
            error.to_string(),
            "model `m` is still installed after being removed"
        );
    }

    #[test]
    fn remove_model_error_source_returns_inner_for_wrapped() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = RemoveModelError::Library(cause);
        assert!(error.source().is_some());
    }

    #[test]
    fn remove_model_error_source_returns_none_for_leaf() {
        let error = RemoveModelError::NotInstalled { model: "m".into() };
        assert!(error.source().is_none());

        let error = RemoveModelError::StillInstalled { model: "m".into() };
        assert!(error.source().is_none());
    }

    #[test]
    fn from_library_error_constructs_library_variant() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error: RemoveModelError = cause.into();
        assert!(matches!(error, RemoveModelError::Library(_)));
    }

    #[test]
    fn remove_model_error_implements_std_error() {
        let error = RemoveModelError::NotInstalled { model: "m".into() };
        let _: &dyn std::error::Error = &error;
    }
}

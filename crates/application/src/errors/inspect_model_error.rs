use std::{error::Error, fmt};

use crate::errors::library_error::LibraryError;

/// Failures that can end an inspection of one locally installed model.
///
/// Inspecting reads the durable library alone; nothing upstream is consulted
/// and nothing on disk is changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectModelError {
    /// The durable library could not be read.
    Library(LibraryError),

    /// The library holds no replica of the requested model.
    ///
    /// The operator asked about a model this machine never installed, which is
    /// a different answer from a replica that is present but unproven.
    NotInstalled { model: String },
}

impl fmt::Display for InspectModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Library(cause) => {
                write!(formatter, "the model could not be inspected: {cause}")
            }
            Self::NotInstalled { model } => {
                write!(formatter, "model `{model}` is not installed locally")
            }
        }
    }
}

impl Error for InspectModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Library(cause) => Some(cause),
            Self::NotInstalled { .. } => None,
        }
    }
}

impl From<LibraryError> for InspectModelError {
    fn from(cause: LibraryError) -> Self {
        Self::Library(cause)
    }
}

#[cfg(test)]
mod inspect_model_error_tests {
    use super::*;
    use crate::errors::LibraryError;

    #[test]
    fn library_variant_constructs_and_displays() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = InspectModelError::Library(cause);
        assert_eq!(
            error.to_string(),
            "the model could not be inspected: could not read the library for model `m`: io"
        );
    }

    #[test]
    fn not_installed_variant_constructs_and_displays() {
        let error = InspectModelError::NotInstalled { model: "m".into() };
        assert_eq!(error.to_string(), "model `m` is not installed locally");
    }

    #[test]
    fn inspect_model_error_source_returns_inner_for_wrapped() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = InspectModelError::Library(cause);
        assert!(error.source().is_some());
    }

    #[test]
    fn inspect_model_error_source_returns_none_for_leaf() {
        let error = InspectModelError::NotInstalled { model: "m".into() };
        assert!(error.source().is_none());
    }

    #[test]
    fn from_library_error_constructs_library_variant() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error: InspectModelError = cause.into();
        assert!(matches!(error, InspectModelError::Library(_)));
    }

    #[test]
    fn inspect_model_error_implements_std_error() {
        let error = InspectModelError::NotInstalled { model: "m".into() };
        let _: &dyn std::error::Error = &error;
    }
}

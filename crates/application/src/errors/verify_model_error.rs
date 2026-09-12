use std::{error::Error, fmt};

use crate::errors::library_error::LibraryError;

/// Failures that can end a verification of one locally installed model.
///
/// A replica whose bytes disagree with the digest recorded for them is not a
/// failure of this use case: the disagreement is the verdict the operator asked
/// for, and it comes back as the model's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyModelError {
    /// The durable library could not be read, written, or hashed.
    Library(LibraryError),

    /// The library holds no replica of the requested model to verify.
    NotInstalled { model: String },
}

impl fmt::Display for VerifyModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Library(cause) => {
                write!(formatter, "the model could not be verified: {cause}")
            }
            Self::NotInstalled { model } => {
                write!(
                    formatter,
                    "model `{model}` is not installed locally, so there is nothing to verify"
                )
            }
        }
    }
}

impl Error for VerifyModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Library(cause) => Some(cause),
            Self::NotInstalled { .. } => None,
        }
    }
}

impl From<LibraryError> for VerifyModelError {
    fn from(cause: LibraryError) -> Self {
        Self::Library(cause)
    }
}

#[cfg(test)]
mod verify_model_error_tests {
    use super::*;
    use crate::errors::LibraryError;

    #[test]
    fn library_variant_constructs_and_displays() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = VerifyModelError::Library(cause);
        assert_eq!(
            error.to_string(),
            "the model could not be verified: could not read the library for model `m`: io"
        );
    }

    #[test]
    fn not_installed_variant_constructs_and_displays() {
        let error = VerifyModelError::NotInstalled { model: "m".into() };
        assert_eq!(
            error.to_string(),
            "model `m` is not installed locally, so there is nothing to verify"
        );
    }

    #[test]
    fn verify_model_error_source_returns_inner_for_wrapped() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = VerifyModelError::Library(cause);
        assert!(error.source().is_some());
    }

    #[test]
    fn verify_model_error_source_returns_none_for_leaf() {
        let error = VerifyModelError::NotInstalled { model: "m".into() };
        assert!(error.source().is_none());
    }

    #[test]
    fn from_library_error_constructs_library_variant() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error: VerifyModelError = cause.into();
        assert!(matches!(error, VerifyModelError::Library(_)));
    }

    #[test]
    fn verify_model_error_implements_std_error() {
        let error = VerifyModelError::NotInstalled { model: "m".into() };
        let _: &dyn std::error::Error = &error;
    }
}

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

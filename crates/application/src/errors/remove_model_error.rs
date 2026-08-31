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

use std::error::Error;
use std::fmt;

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

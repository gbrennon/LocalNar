use std::error::Error;
use std::fmt;

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

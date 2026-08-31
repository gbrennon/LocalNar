use std::{error::Error, fmt};

use crate::errors::registry_read_error::RegistryReadError;

/// Failures that can end a model search.
///
/// Searching reads only the upstream catalog, so the registry is the single
/// boundary that can fail; the wrapper keeps the outbound error type out of
/// the inbound contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchModelsError {
    /// The upstream registry could not answer the search.
    Registry(RegistryReadError),
}

impl fmt::Display for SearchModelsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(cause) => {
                write!(formatter, "the catalog could not be searched: {cause}")
            }
        }
    }
}

impl Error for SearchModelsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registry(cause) => Some(cause),
        }
    }
}

impl From<RegistryReadError> for SearchModelsError {
    fn from(cause: RegistryReadError) -> Self {
        Self::Registry(cause)
    }
}

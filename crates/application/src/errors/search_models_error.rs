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

#[cfg(test)]
mod search_models_error_tests {
    use super::*;
    use crate::errors::RegistryReadError;

    #[test]
    fn registry_variant_constructs_and_displays() {
        let cause = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        let error = SearchModelsError::Registry(cause);
        assert_eq!(
            error.to_string(),
            "the catalog could not be searched: repository `repo` could not be reached: network"
        );
    }

    #[test]
    fn search_models_error_source_returns_inner() {
        let cause = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        let error = SearchModelsError::Registry(cause);
        assert!(error.source().is_some());
    }

    #[test]
    fn from_registry_read_error_constructs_registry_variant() {
        let cause = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        let error: SearchModelsError = cause.into();
        assert!(matches!(error, SearchModelsError::Registry(_)));
    }

    #[test]
    fn search_models_error_implements_std_error() {
        let cause = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        let error = SearchModelsError::Registry(cause);
        let _: &dyn std::error::Error = &error;
    }
}

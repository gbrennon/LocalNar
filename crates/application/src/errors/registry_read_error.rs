use std::{error::Error, fmt};

/// Failures that can occur while reading the upstream catalog of a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryReadError {
    /// The registry could not be reached for the given repository.
    Unreachable { repository: String, cause: String },
    /// The registry does not expose the requested file under the repository.
    FileNotFound { repository: String, file: String },
    /// The registry answered with data that could not be interpreted.
    Malformed { repository: String },
    /// The adapter does not offer the requested operation.
    EnumerationUnsupported,
}

impl fmt::Display for RegistryReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable { repository, cause } => write!(
                formatter,
                "repository `{repository}` could not be reached: {cause}"
            ),
            Self::FileNotFound { repository, file } => write!(
                formatter,
                "file `{file}` was not found in repository `{repository}`"
            ),
            Self::Malformed { repository } => write!(
                formatter,
                "the response for repository `{repository}` was malformed"
            ),
            Self::EnumerationUnsupported => {
                formatter.write_str("this registry does not support enumerating files")
            }
        }
    }
}

impl Error for RegistryReadError {}

#[cfg(test)]
mod registry_read_error_tests {
    use super::*;

    #[test]
    fn unreachable_variant_constructs_and_displays() {
        let error = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        assert_eq!(
            error.to_string(),
            "repository `repo` could not be reached: network"
        );
    }

    #[test]
    fn file_not_found_variant_constructs_and_displays() {
        let error = RegistryReadError::FileNotFound {
            repository: "repo".into(),
            file: "file".into(),
        };
        assert_eq!(
            error.to_string(),
            "file `file` was not found in repository `repo`"
        );
    }

    #[test]
    fn malformed_variant_constructs_and_displays() {
        let error = RegistryReadError::Malformed {
            repository: "repo".into(),
        };
        assert_eq!(
            error.to_string(),
            "the response for repository `repo` was malformed"
        );
    }

    #[test]
    fn enumeration_unsupported_variant_constructs_and_displays() {
        let error = RegistryReadError::EnumerationUnsupported;
        assert_eq!(
            error.to_string(),
            "this registry does not support enumerating files"
        );
    }

    #[test]
    fn registry_read_error_implements_std_error() {
        let error = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        let _: &dyn std::error::Error = &error;
    }
}

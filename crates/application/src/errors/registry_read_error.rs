use std::error::Error;
use std::fmt;

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

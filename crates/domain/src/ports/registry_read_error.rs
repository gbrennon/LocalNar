/// Failures that can occur while reading the upstream catalog of a repository.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryReadError {
    /// The registry could not be reached for the given repository.
    #[error("repository `{repository}` could not be reached: {cause}")]
    Unreachable { repository: String, cause: String },
    /// The registry does not expose the requested file under the repository.
    #[error("file `{file}` was not found in repository `{repository}`")]
    FileNotFound { repository: String, file: String },
    /// The registry answered with data that could not be interpreted.
    #[error("the response for repository `{repository}` was malformed")]
    Malformed { repository: String },
    /// The adapter does not offer the requested operation.
    #[error("this registry does not support enumerating files")]
    EnumerationUnsupported,
}

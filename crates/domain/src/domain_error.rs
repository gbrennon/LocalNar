use thiserror::Error;

/// Mistakes raised by domain rules while building or interpreting model values.
///
/// The variants carry only plain data so they stay serializable and cheap to
/// clone; infrastructure adapters translate their own failures into these or
/// into the port-specific error types.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    /// The `ModelId` was built from a blank string.
    #[error("a model identifier must not be blank")]
    EmptyModelId,

    /// The repository revision was built from a blank string.
    #[error("a repository revision must not be blank")]
    EmptyRevision,

    /// A repository identifier did not look like `<owner>/<name>`.
    #[error("repository identifier `{0}` must follow `<owner>/<name>`")]
    MalformedRepository(String),

    /// A repository file name was rejected because it is unsafe or empty.
    #[error("file name `{0}` is not a valid single repository file")]
    InvalidFileName(String),

    /// A SHA-256 literal did not parse as 64 hexadecimal characters.
    #[error("`{0}` is not a valid 64-character hexadecimal SHA-256 digest")]
    InvalidSha256Literal(String),

    /// A computed checksum disagreed with the one the remote advertised.
    #[error("checksum mismatch: expected `{expected}`, computed `{actual}`")]
    IntegrityMismatch { expected: String, actual: String },
}

/// Failures that can occur while reading or writing the durable model library.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LibraryError {
    /// The library location could not be read to answer a state query.
    #[error("could not read the library for model `{model}`: {cause}")]
    Unreadable { model: String, cause: String },
    /// A staged or installed artifact could not be written.
    #[error("could not write the library for model `{model}`: {cause}")]
    Unwritable { model: String, cause: String },
    /// A committed file could not be hashed for integrity verification.
    #[error("could not verify model `{model}`: {cause}")]
    Unverifiable { model: String, cause: String },
}

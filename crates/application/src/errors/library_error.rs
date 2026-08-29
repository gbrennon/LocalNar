use std::error::Error;
use std::fmt;

/// Failures that can occur while reading or writing the durable model library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryError {
    /// The library location could not be read to answer a state query.
    Unreadable { model: String, cause: String },
    /// A staged or installed artifact could not be written.
    Unwritable { model: String, cause: String },
    /// A committed file could not be hashed for integrity verification.
    Unverifiable { model: String, cause: String },
}

impl fmt::Display for LibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { model, cause } => write!(
                formatter,
                "could not read the library for model `{model}`: {cause}"
            ),
            Self::Unwritable { model, cause } => write!(
                formatter,
                "could not write the library for model `{model}`: {cause}"
            ),
            Self::Unverifiable { model, cause } => {
                write!(formatter, "could not verify model `{model}`: {cause}")
            }
        }
    }
}

impl Error for LibraryError {}

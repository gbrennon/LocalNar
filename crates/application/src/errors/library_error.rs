use std::{error::Error, fmt};

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

#[cfg(test)]
mod library_error_tests {
    use super::*;

    #[test]
    fn unreadable_variant_constructs_and_displays() {
        let error = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        assert_eq!(
            error.to_string(),
            "could not read the library for model `m`: io"
        );
    }

    #[test]
    fn unwritable_variant_constructs_and_displays() {
        let error = LibraryError::Unwritable {
            model: "m".into(),
            cause: "io".into(),
        };
        assert_eq!(
            error.to_string(),
            "could not write the library for model `m`: io"
        );
    }

    #[test]
    fn unverifiable_variant_constructs_and_displays() {
        let error = LibraryError::Unverifiable {
            model: "m".into(),
            cause: "io".into(),
        };
        assert_eq!(error.to_string(), "could not verify model `m`: io");
    }

    #[test]
    fn library_error_implements_std_error() {
        let error = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let _: &dyn std::error::Error = &error;
    }
}

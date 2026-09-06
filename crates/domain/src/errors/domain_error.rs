use std::{error::Error, fmt};

/// Mistakes raised by domain rules while building or interpreting model values.
///
/// The variants carry only plain data so they stay serializable and cheap to
/// clone; infrastructure adapters translate their own failures into these or
/// into the port-specific error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// A search was requested with a phrase that carries no text.
    BlankSearchQuery,

    /// The repository revision was built from a blank string.
    EmptyRevision,

    /// A repository identifier did not look like `<owner>/<name>`.
    MalformedRepository(String),

    /// A repository file name was rejected because it is unsafe or empty.
    InvalidFileName(String),

    /// A literal did not parse as 64 hexadecimal characters.
    InvalidChecksumLiteral(String),

    /// A model tag was built from a blank label.
    InvalidModelTag,

    /// A computed checksum disagreed with the one the remote advertised.
    IntegrityMismatch { expected: String, actual: String },
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSearchQuery => formatter.write_str("a search query must not be blank"),
            Self::EmptyRevision => formatter.write_str("a repository revision must not be blank"),
            Self::MalformedRepository(identifier) => write!(
                formatter,
                "repository identifier `{identifier}` must follow `<owner>/<name>`"
            ),
            Self::InvalidFileName(name) => write!(
                formatter,
                "file name `{name}` is not a valid single repository file"
            ),
            Self::InvalidChecksumLiteral(literal) => write!(
                formatter,
                "`{literal}` is not a valid 64-character hexadecimal digest"
            ),
            Self::InvalidModelTag => formatter.write_str("a model tag must not be blank"),
            Self::IntegrityMismatch { expected, actual } => write!(
                formatter,
                "checksum mismatch: expected `{expected}`, computed `{actual}`"
            ),
        }
    }
}

impl Error for DomainError {}

#[cfg(test)]
mod domain_error_tests {
    use std::error::Error;

    use crate::errors::DomainError;

    #[test]
    fn each_variant_renders_a_distinct_message() {
        assert_eq!(
            DomainError::BlankSearchQuery.to_string(),
            "a search query must not be blank"
        );
        assert_eq!(
            DomainError::EmptyRevision.to_string(),
            "a repository revision must not be blank"
        );
        assert_eq!(
            DomainError::MalformedRepository("owner".to_owned()).to_string(),
            "repository identifier `owner` must follow `<owner>/<name>`"
        );
        assert_eq!(
            DomainError::InvalidFileName("../escape".to_owned()).to_string(),
            "file name `../escape` is not a valid single repository file"
        );
        assert_eq!(
            DomainError::InvalidChecksumLiteral("zz".to_owned()).to_string(),
            "`zz` is not a valid 64-character hexadecimal digest"
        );
        assert_eq!(
            DomainError::InvalidModelTag.to_string(),
            "a model tag must not be blank"
        );
        assert_eq!(
            DomainError::IntegrityMismatch {
                expected: "aa".to_owned(),
                actual: "bb".to_owned(),
            }
            .to_string(),
            "checksum mismatch: expected `aa`, computed `bb`"
        );
    }

    #[test]
    fn a_domain_error_is_a_standard_error_without_a_source() {
        let error = DomainError::BlankSearchQuery;

        assert!(error.source().is_none());
    }
}

use std::{error::Error, fmt};

use localnar_domain::ByteLength;

/// Failures that can occur while transmitting a remote file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelDownloadError {
    /// The upstream host could not be reached.
    Unreachable { file: String, cause: String },
    /// The received byte count disagreed with the announced size.
    SizeMismatch {
        file: String,
        expected: ByteLength,
        received: ByteLength,
    },
    /// The transport subsystem failed for an unexpected reason.
    Transport { file: String, cause: String },
}

impl fmt::Display for ModelDownloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable { file, cause } => write!(
                formatter,
                "could not reach the host while downloading `{file}`: {cause}"
            ),
            Self::SizeMismatch {
                file,
                expected,
                received,
            } => write!(
                formatter,
                "download of `{file}` was incomplete: received {received} of {expected} bytes"
            ),
            Self::Transport { file, cause } => {
                write!(formatter, "download of `{file}` failed: {cause}")
            }
        }
    }
}

impl Error for ModelDownloadError {}

#[cfg(test)]
mod model_download_error_tests {
    use super::*;

    #[test]
    fn unreachable_variant_constructs_and_displays() {
        let error = ModelDownloadError::Unreachable {
            file: "f".into(),
            cause: "network".into(),
        };
        assert_eq!(
            error.to_string(),
            "could not reach the host while downloading `f`: network"
        );
    }

    #[test]
    fn size_mismatch_variant_constructs_and_displays() {
        use localnar_domain::ByteLength;
        let error = ModelDownloadError::SizeMismatch {
            file: "f".into(),
            expected: ByteLength::new(100),
            received: ByteLength::new(50),
        };
        assert_eq!(
            error.to_string(),
            "download of `f` was incomplete: received 50 B of 100 B bytes"
        );
    }

    #[test]
    fn transport_variant_constructs_and_displays() {
        let error = ModelDownloadError::Transport {
            file: "f".into(),
            cause: "timeout".into(),
        };
        assert_eq!(error.to_string(), "download of `f` failed: timeout");
    }

    #[test]
    fn model_download_error_implements_std_error() {
        let error = ModelDownloadError::Unreachable {
            file: "f".into(),
            cause: "network".into(),
        };
        let _: &dyn std::error::Error = &error;
    }
}

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

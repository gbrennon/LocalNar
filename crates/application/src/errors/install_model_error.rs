use std::error::Error;
use std::fmt;

use crate::errors::library_error::LibraryError;
use crate::errors::model_download_error::ModelDownloadError;
use crate::errors::registry_read_error::RegistryReadError;

/// Failures that can end an install use case run.
///
/// Each variant preserves the port error that produced it so the presentation
/// layer can report the failing boundary instead of a flattened message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallModelError {
    /// The upstream registry could not describe the requested file.
    Registry(RegistryReadError),

    /// The file could not be transmitted.
    Download(ModelDownloadError),

    /// The durable library could not be read or written.
    Library(LibraryError),

    /// Upstream never supplied the bytes: the replica is still absent after a
    /// fetch was performed and committed.
    UpstreamUnavailable,

    /// The repaired replica still mismatches the advertised checksum.
    UnresolvedIntegrity {
        /// Checksum the upstream registry advertised for the file.
        expected: String,
        /// Checksum observed on disk after the repair attempt.
        actual: String,
    },
}

impl fmt::Display for InstallModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(cause) => {
                write!(
                    formatter,
                    "the registry could not describe the model: {cause}"
                )
            }
            Self::Download(cause) => {
                write!(formatter, "the model could not be downloaded: {cause}")
            }
            Self::Library(cause) => {
                write!(formatter, "the model library could not be used: {cause}")
            }
            Self::UpstreamUnavailable => write!(
                formatter,
                "the model is still missing after a download attempt: upstream supplied no bytes"
            ),
            Self::UnresolvedIntegrity { expected, actual } => write!(
                formatter,
                "model repair failed: expected checksum `{expected}` but got `{actual}`"
            ),
        }
    }
}

impl Error for InstallModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registry(cause) => Some(cause),
            Self::Download(cause) => Some(cause),
            Self::Library(cause) => Some(cause),
            Self::UpstreamUnavailable => None,
            Self::UnresolvedIntegrity { .. } => None,
        }
    }
}

impl From<RegistryReadError> for InstallModelError {
    fn from(cause: RegistryReadError) -> Self {
        Self::Registry(cause)
    }
}

impl From<ModelDownloadError> for InstallModelError {
    fn from(cause: ModelDownloadError) -> Self {
        Self::Download(cause)
    }
}

impl From<LibraryError> for InstallModelError {
    fn from(cause: LibraryError) -> Self {
        Self::Library(cause)
    }
}

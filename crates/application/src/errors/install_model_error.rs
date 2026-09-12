use std::{error::Error, fmt};

use crate::errors::{
    library_error::LibraryError, model_download_error::ModelDownloadError,
    registry_read_error::RegistryReadError,
};

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

#[cfg(test)]
mod install_model_error_tests {
    use super::*;
    use crate::errors::{LibraryError, ModelDownloadError, RegistryReadError};

    #[test]
    fn registry_variant_constructs_and_displays() {
        let cause = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        let error = InstallModelError::Registry(cause);
        assert_eq!(
            error.to_string(),
            "the registry could not describe the model: repository `repo` could not be reached: network"
        );
    }

    #[test]
    fn download_variant_constructs_and_displays() {
        let cause = ModelDownloadError::Unreachable {
            file: "f".into(),
            cause: "network".into(),
        };
        let error = InstallModelError::Download(cause);
        assert_eq!(
            error.to_string(),
            "the model could not be downloaded: could not reach the host while downloading `f`: network"
        );
    }

    #[test]
    fn library_variant_constructs_and_displays() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = InstallModelError::Library(cause);
        assert_eq!(
            error.to_string(),
            "the model library could not be used: could not read the library for model `m`: io"
        );
    }

    #[test]
    fn upstream_unavailable_variant_constructs_and_displays() {
        let error = InstallModelError::UpstreamUnavailable;
        assert_eq!(
            error.to_string(),
            "the model is still missing after a download attempt: upstream supplied no bytes"
        );
    }

    #[test]
    fn unresolved_integrity_variant_constructs_and_displays() {
        let error = InstallModelError::UnresolvedIntegrity {
            expected: "abc".into(),
            actual: "def".into(),
        };
        assert_eq!(
            error.to_string(),
            "model repair failed: expected checksum `abc` but got `def`"
        );
    }

    #[test]
    fn install_model_error_source_returns_inner_for_wrapped() {
        let cause = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        let error = InstallModelError::Registry(cause);
        assert!(error.source().is_some());

        let cause = ModelDownloadError::Unreachable {
            file: "f".into(),
            cause: "network".into(),
        };
        let error = InstallModelError::Download(cause);
        assert!(error.source().is_some());

        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error = InstallModelError::Library(cause);
        assert!(error.source().is_some());
    }

    #[test]
    fn install_model_error_source_returns_none_for_leaf() {
        let error = InstallModelError::UpstreamUnavailable;
        assert!(error.source().is_none());

        let error = InstallModelError::UnresolvedIntegrity {
            expected: "abc".into(),
            actual: "def".into(),
        };
        assert!(error.source().is_none());
    }

    #[test]
    fn from_registry_read_error_constructs_registry_variant() {
        let cause = RegistryReadError::Unreachable {
            repository: "repo".into(),
            cause: "network".into(),
        };
        let error: InstallModelError = cause.into();
        assert!(matches!(error, InstallModelError::Registry(_)));
    }

    #[test]
    fn from_model_download_error_constructs_download_variant() {
        let cause = ModelDownloadError::Unreachable {
            file: "f".into(),
            cause: "network".into(),
        };
        let error: InstallModelError = cause.into();
        assert!(matches!(error, InstallModelError::Download(_)));
    }

    #[test]
    fn from_library_error_constructs_library_variant() {
        let cause = LibraryError::Unreadable {
            model: "m".into(),
            cause: "io".into(),
        };
        let error: InstallModelError = cause.into();
        assert!(matches!(error, InstallModelError::Library(_)));
    }

    #[test]
    fn install_model_error_implements_std_error() {
        let error = InstallModelError::UpstreamUnavailable;
        let _: &dyn std::error::Error = &error;
    }
}

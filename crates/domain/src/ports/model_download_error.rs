use crate::byte_length::ByteLength;

/// Failures that can occur while transmitting a remote file.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelDownloadError {
    /// The upstream host could not be reached.
    #[error("could not reach the host while downloading `{file}`: {cause}")]
    Unreachable { file: String, cause: String },
    /// The received byte count disagreed with the announced size.
    #[error("download of `{file}` was incomplete: received {received} of {expected} bytes")]
    SizeMismatch {
        file: String,
        expected: ByteLength,
        received: ByteLength,
    },
    /// The transport subsystem failed for an unexpected reason.
    #[error("download of `{file}` failed: {cause}")]
    Transport { file: String, cause: String },
}

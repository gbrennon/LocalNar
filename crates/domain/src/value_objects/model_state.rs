use crate::value_objects::Checksum;

/// The observer state of one model with respect to the durable local library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelState {
    /// No replica exists on disk at all.
    Missing,
    /// A file exists on disk but its integrity has not yet been verified.
    Downloaded,
    /// The on-disk file is complete and its checksum matches the remote one.
    Verified,
    /// The remote advertised a checksum that the local file does not match.
    IntegrityMismatch {
        expected: Checksum,
        actual: Checksum,
    },
}

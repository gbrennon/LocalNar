use crate::sha256::Sha256;

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
    IntegrityMismatch { expected: Sha256, actual: Sha256 },
}

impl ModelState {
    /// Whether the local replica is complete and correct right now.
    pub fn is_ready(self) -> bool {
        matches!(self, Self::Verified)
    }

    /// Whether the next automated step must (re)transmit the file.
    pub fn needs_fetch(self) -> bool {
        matches!(self, Self::Missing | Self::IntegrityMismatch { .. })
    }
}

#[cfg(test)]
mod model_state_tests {
    use crate::model_state::ModelState;
    use crate::sha256::Sha256;

    #[test]
    fn only_verified_is_ready() {
        assert!(ModelState::Verified.is_ready());
        assert!(!ModelState::Missing.is_ready());
        assert!(!ModelState::Downloaded.is_ready());
        assert!(!corrupt().is_ready());
    }

    #[test]
    fn missing_and_corrupt_need_a_fetch() {
        assert!(ModelState::Missing.needs_fetch());
        assert!(corrupt().needs_fetch());
        assert!(!ModelState::Downloaded.needs_fetch());
        assert!(!ModelState::Verified.needs_fetch());
    }

    fn corrupt() -> ModelState {
        ModelState::IntegrityMismatch {
            expected: Sha256::from_bytes([1u8; 32]),
            actual: Sha256::from_bytes([2u8; 32]),
        }
    }
}

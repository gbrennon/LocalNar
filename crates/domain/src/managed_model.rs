use std::path::Path;

use crate::byte_length::ByteLength;
use crate::checksum::Checksum;
use crate::installed_model::InstalledModel;
use crate::model_spec::ModelSpec;
use crate::model_state::ModelState;

/// A replica the local library holds, read together with its current state.
///
/// `InstalledModel` answers where the bytes are; `ModelState` answers how much
/// they can be trusted. Managing a local model needs both at once: an operator
/// deciding whether to keep, re-verify, or discard a replica is choosing on the
/// pair, so the two are carried together rather than fetched separately and
/// risked drifting apart.
///
/// The state is the reading taken when the entry was observed. A cheap
/// enumeration answers `Downloaded` or `Verified` from what the library
/// recorded; only an explicit verification re-reads the bytes and can therefore
/// report `IntegrityMismatch`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedModel {
    replica: InstalledModel,
    state: ModelState,
}

impl ManagedModel {
    /// Describes `replica` as observed in `state`.
    pub fn new(replica: InstalledModel, state: ModelState) -> Self {
        Self { replica, state }
    }

    /// The model this entry holds a replica of.
    pub fn spec(&self) -> &ModelSpec {
        self.replica.spec()
    }

    /// Where the replica's bytes live.
    pub fn path(&self) -> &Path {
        self.replica.path()
    }

    /// How much space the replica occupies.
    pub fn size(&self) -> ByteLength {
        self.replica.size()
    }

    /// The digest the library recorded for the replica, if integrity was proven.
    pub fn digest(&self) -> Option<Checksum> {
        self.replica.digest()
    }

    /// The state the replica was observed in.
    pub fn state(&self) -> &ModelState {
        &self.state
    }

    /// The replica this entry describes.
    pub fn replica(&self) -> &InstalledModel {
        &self.replica
    }

    /// Whether the replica's bytes were proven against a recorded digest.
    pub fn is_verified(&self) -> bool {
        matches!(self.state, ModelState::Verified)
    }

    /// Whether the replica is installed but has never been proven.
    pub fn is_unproven(&self) -> bool {
        matches!(self.state, ModelState::Downloaded)
    }

    /// Whether the replica's bytes disagree with the digest recorded for them.
    ///
    /// A broken replica is the one case where keeping the file serves nobody:
    /// it occupies the space of a model that cannot be trusted to load, so the
    /// operator is expected to repair or discard it.
    pub fn is_broken(&self) -> bool {
        matches!(self.state, ModelState::IntegrityMismatch { .. })
    }
}

#[cfg(test)]
mod managed_model_tests {
    use super::ManagedModel;
    use crate::byte_length::ByteLength;
    use crate::checksum::Checksum;
    use crate::installed_model::InstalledModel;
    use crate::model_file_name::ModelFileName;
    use crate::model_repository::ModelRepository;
    use crate::model_repository_id::ModelRepositoryId;
    use crate::model_spec::ModelSpec;
    use crate::model_state::ModelState;

    fn spec() -> ModelSpec {
        ModelSpec::new(
            ModelRepository::at_default_revision(
                ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id"),
            ),
            ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name"),
        )
    }

    fn replica(digest: Option<Checksum>) -> InstalledModel {
        InstalledModel::new(spec(), "/models/qwen.gguf", ByteLength::new(2_048), digest)
    }

    #[test]
    fn a_verified_entry_reads_as_proven_alone() {
        let digest = Checksum::from_bytes([0x33; 32]);
        let entry = ManagedModel::new(replica(Some(digest)), ModelState::Verified);

        assert!(entry.is_verified());
        assert!(!entry.is_unproven());
        assert!(!entry.is_broken());
        assert_eq!(entry.digest(), Some(digest));
        assert_eq!(entry.size(), ByteLength::new(2_048));
        assert_eq!(entry.spec(), &spec());
    }

    #[test]
    fn a_downloaded_entry_reads_as_unproven_alone() {
        let entry = ManagedModel::new(replica(None), ModelState::Downloaded);

        assert!(entry.is_unproven());
        assert!(!entry.is_verified());
        assert!(!entry.is_broken());
        assert_eq!(entry.digest(), None);
    }

    #[test]
    fn a_mismatching_entry_reads_as_broken_alone() {
        let entry = ManagedModel::new(
            replica(Some(Checksum::from_bytes([0x44; 32]))),
            ModelState::IntegrityMismatch {
                expected: Checksum::from_bytes([0x44; 32]),
                actual: Checksum::from_bytes([0x55; 32]),
            },
        );

        assert!(entry.is_broken());
        assert!(!entry.is_verified());
        assert!(!entry.is_unproven());
        assert_eq!(entry.path().to_str(), Some("/models/qwen.gguf"));
    }
}

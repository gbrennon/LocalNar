use std::path::{Path, PathBuf};

use crate::ByteLength;
use crate::ManagedModel;
use crate::ModelSpec;

/// Everything the local library holds, read as one answer.
///
/// The inventory is the operator's view of the machine: the place the library
/// keeps its models and every replica found there. It is a snapshot, so the
/// readings it exposes are consistent with each other; recomputing a total from
/// entries gathered at different moments is what this type exists to prevent.
///
/// The entries are held in the order the library reported them, which lets a
/// caller present a stable listing without sorting a copy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInventory {
    root: PathBuf,
    entries: Vec<ManagedModel>,
}

impl ModelInventory {
    /// Describes the library rooted at `root` as holding exactly `entries`.
    pub fn new(root: impl Into<PathBuf>, entries: Vec<ManagedModel>) -> Self {
        Self {
            root: root.into(),
            entries,
        }
    }

    /// The directory the library keeps its models under.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Every replica the library holds.
    pub fn entries(&self) -> &[ManagedModel] {
        &self.entries
    }

    /// How many replicas the library holds.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Whether the library holds no replica at all.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The disk space every replica occupies together.
    pub fn total_size(&self) -> ByteLength {
        ByteLength::new(
            self.entries
                .iter()
                .map(|entry| entry.size().bytes())
                .sum::<u64>(),
        )
    }

    /// How many replicas were proven against a recorded digest.
    pub fn verified_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.is_verified())
            .count()
    }

    /// How many replicas disagree with the digest recorded for them.
    pub fn broken_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.is_broken())
            .count()
    }

    /// The entry holding a replica of `spec`, when the library holds one.
    pub fn find(&self, spec: &ModelSpec) -> Option<&ManagedModel> {
        self.entries.iter().find(|entry| entry.spec() == spec)
    }
}

#[cfg(test)]
mod model_inventory_tests {
    use super::ModelInventory;
    use crate::ByteLength;
    use crate::Checksum;
    use crate::InstalledModel;
    use crate::ManagedModel;
    use crate::ModelFileName;
    use crate::ModelRepository;
    use crate::ModelRepositoryId;
    use crate::ModelSpec;
    use crate::ModelState;

    fn spec(file: &str) -> ModelSpec {
        ModelSpec::new(
            ModelRepository::at_default_revision(
                ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id"),
            ),
            ModelFileName::new(file).expect("valid file name"),
            vec![],
        )
    }

    fn entry(file: &str, bytes: u64, state: ModelState) -> ManagedModel {
        ManagedModel::new(
            InstalledModel::new(
                spec(file),
                format!("/models/{file}"),
                ByteLength::new(bytes),
                Some(Checksum::from_bytes([0x66; 32])),
            ),
            state,
        )
    }

    #[test]
    fn an_empty_library_reports_no_entries_and_no_space() {
        let inventory = ModelInventory::new("/models", Vec::new());

        assert!(inventory.is_empty());
        assert_eq!(inventory.count(), 0);
        assert_eq!(inventory.total_size(), ByteLength::ZERO);
        assert_eq!(inventory.verified_count(), 0);
        assert_eq!(inventory.broken_count(), 0);
        assert_eq!(inventory.root(), std::path::Path::new("/models"));
    }

    #[test]
    fn the_total_size_sums_every_replica() {
        let inventory = ModelInventory::new(
            "/models",
            vec![
                entry("a.gguf", 1_000, ModelState::Verified),
                entry("b.gguf", 2_500, ModelState::Downloaded),
            ],
        );

        assert_eq!(inventory.total_size(), ByteLength::new(3_500));
        assert_eq!(inventory.count(), 2);
    }

    #[test]
    fn entries_are_counted_by_the_state_they_were_observed_in() {
        let inventory = ModelInventory::new(
            "/models",
            vec![
                entry("a.gguf", 1, ModelState::Verified),
                entry("b.gguf", 1, ModelState::Downloaded),
                entry(
                    "c.gguf",
                    1,
                    ModelState::IntegrityMismatch {
                        expected: Checksum::from_bytes([0x01; 32]),
                        actual: Checksum::from_bytes([0x02; 32]),
                    },
                ),
            ],
        );

        assert_eq!(inventory.verified_count(), 1);
        assert_eq!(inventory.broken_count(), 1);
    }

    #[test]
    fn a_held_model_is_found_by_its_spec() {
        let inventory = ModelInventory::new(
            "/models",
            vec![entry("a.gguf", 1_024, ModelState::Verified)],
        );

        assert_eq!(
            inventory.find(&spec("a.gguf")).map(ManagedModel::size),
            Some(ByteLength::new(1_024))
        );
        assert_eq!(inventory.find(&spec("absent.gguf")), None);
    }
}

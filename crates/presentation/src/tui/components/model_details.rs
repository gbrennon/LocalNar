use localnar_domain::{ManagedModel, ModelState};

/// One installed model rendered as the labeled facts of a details view.
///
/// A row in the library table trades completeness for width; this is the other
/// half of that trade, holding the facts a row cannot fit: the exact place on
/// disk, the full digest, and, when the replica failed its own digest, both the
/// digest that was recorded and the one its bytes now produce. Reading them side
/// by side is what tells an operator whether a broken replica is worth
/// repairing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelDetails {
    facts: Vec<(&'static str, String)>,
}

impl ModelDetails {
    /// The label of the repository the replica was drawn from.
    pub const REPOSITORY: &'static str = "Repository";

    /// The label of the revision the replica was drawn at.
    pub const REVISION: &'static str = "Revision";

    /// The label of the weight file the replica holds.
    pub const FILE: &'static str = "File";

    /// The label of the verdict the library holds the replica under.
    pub const STATE: &'static str = "State";

    /// The label of the space the replica occupies.
    pub const SIZE: &'static str = "Size";

    /// The label of the digest recorded for the replica.
    pub const DIGEST: &'static str = "Digest";

    /// The label of the digest the library expected of a broken replica.
    pub const EXPECTED_DIGEST: &'static str = "Expected";

    /// The label of the digest a broken replica's bytes actually produce.
    pub const ACTUAL_DIGEST: &'static str = "Actual";

    /// The label of the place the replica's bytes occupy.
    pub const PATH: &'static str = "Path";

    /// What a fact shows when the library recorded no value for it.
    pub const UNRECORDED: &'static str = "-";

    /// What the state fact shows for a replica proven against its digest.
    pub const VERIFIED: &'static str = "verified";

    /// What the state fact shows for a replica that was never proven.
    pub const UNPROVEN: &'static str = "unproven, no digest recorded";

    /// What the state fact shows for a replica that failed its own digest.
    pub const BROKEN: &'static str = "BROKEN, bytes disagree with the digest";

    /// What the state fact shows for a replica that is no longer there.
    pub const ABSENT: &'static str = "absent";

    /// Renders the facts of `entry`.
    pub fn describing(entry: &ManagedModel) -> Self {
        let mut facts = vec![
            (
                Self::REPOSITORY,
                entry.spec().repository().identifier().to_string(),
            ),
            (
                Self::REVISION,
                entry.spec().repository().revision().as_str().to_owned(),
            ),
            (Self::FILE, entry.spec().file().to_string()),
            (Self::STATE, Self::state_of(entry).to_owned()),
            (Self::SIZE, entry.size().to_string()),
            (
                Self::DIGEST,
                entry
                    .digest()
                    .map(|digest| digest.to_hex())
                    .unwrap_or_else(|| Self::UNRECORDED.to_owned()),
            ),
        ];

        if let ModelState::IntegrityMismatch { expected, actual } = entry.state() {
            facts.push((Self::EXPECTED_DIGEST, expected.to_hex()));
            facts.push((Self::ACTUAL_DIGEST, actual.to_hex()));
        }

        facts.push((Self::PATH, entry.path().display().to_string()));

        Self { facts }
    }

    /// Each fact as its label paired with its value, in reading order.
    pub fn facts(&self) -> &[(&'static str, String)] {
        &self.facts
    }

    /// Each fact rendered as a single line of text, in reading order.
    pub fn to_lines(&self) -> Vec<String> {
        self.facts
            .iter()
            .map(|(label, value)| format!("{label:<11}{value}"))
            .collect()
    }

    fn state_of(entry: &ManagedModel) -> &'static str {
        match entry.state() {
            ModelState::Verified => Self::VERIFIED,
            ModelState::Downloaded => Self::UNPROVEN,
            ModelState::IntegrityMismatch { .. } => Self::BROKEN,
            ModelState::Missing => Self::ABSENT,
        }
    }
}

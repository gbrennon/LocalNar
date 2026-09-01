use localnar_domain::ManagedModel;

/// One locally installed model rendered as the cells of a single table row.
///
/// Every cell is filled from the same entry, so a row can never show the state
/// of one replica next to the size of another. The state is spelled out rather
/// than left to the digest cell to imply: an operator deciding what to discard
/// reads the verdict, not the absence of a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryRow {
    repository: String,
    file: String,
    state: String,
    size: String,
    digest: String,
}

impl LibraryRow {
    /// The heading of each cell, in the order the cells are rendered.
    pub const HEADINGS: [&'static str; 5] = ["Repository", "File", "State", "Size", "Digest"];

    /// What the state cell shows for a replica proven against its digest.
    pub const VERIFIED: &'static str = "verified";

    /// What the state cell shows for a replica that was never proven.
    /// The detail view (model_details.rs) uses the longer form "unproven, no digest recorded"
    /// because it has room; the row is constrained to STATE_WIDTH = 8.
    pub const UNPROVEN: &'static str = "unproven";

    /// What the state cell shows for a replica that failed its own digest.
    pub const BROKEN: &'static str = "BROKEN";

    /// What the state cell shows for a replica that is no longer there.
    pub const ABSENT: &'static str = "absent";

    /// What the digest cell shows for a replica holding no recorded digest.
    pub const UNRECORDED: &'static str = "-";

    /// How many leading hexadecimal digits of a digest the row shows.
    pub const DIGEST_PREFIX_LENGTH: usize = 12;

    /// Renders the cells of the row that stands for `entry`.
    pub fn describing(entry: &ManagedModel) -> Self {
        Self {
            repository: entry.spec().repository().to_string(),
            file: entry.spec().file().to_string(),
            state: Self::state_of(entry).to_owned(),
            size: entry.size().to_string(),
            digest: Self::abbreviated_digest(entry),
        }
    }

    /// The upstream repository the replica was drawn from.
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// The weight file the replica holds.
    pub fn file(&self) -> &str {
        &self.file
    }

    /// The verdict the library holds the replica under.
    pub fn state(&self) -> &str {
        &self.state
    }

    /// The space the replica occupies.
    pub fn size(&self) -> &str {
        &self.size
    }

    /// The recorded digest, abbreviated to fit a row.
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Whether the row stands for a replica the operator should act on.
    pub fn is_broken(&self) -> bool {
        self.state == Self::BROKEN
    }

    /// The cells in the order the headings name them.
    pub fn into_cells(self) -> [String; 5] {
        [
            self.repository,
            self.file,
            self.state,
            self.size,
            self.digest,
        ]
    }

    fn state_of(entry: &ManagedModel) -> &'static str {
        if entry.is_verified() {
            Self::VERIFIED
        } else if entry.is_broken() {
            Self::BROKEN
        } else if entry.is_unproven() {
            Self::UNPROVEN
        } else {
            Self::ABSENT
        }
    }

    fn abbreviated_digest(entry: &ManagedModel) -> String {
        entry
            .digest()
            .map(|digest| digest.to_hex()[..Self::DIGEST_PREFIX_LENGTH].to_owned())
            .unwrap_or_else(|| Self::UNRECORDED.to_owned())
    }
}

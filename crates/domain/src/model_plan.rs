use crate::model_state::ModelState;

/// The next automated step for one model, derived purely from its current state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelPlan {
    /// The replica is complete and correct; nothing needs to happen.
    Ignore,
    /// No replica exists; the file must be fetched from upstream.
    Fetch,
    /// A replica exists but is unverified; its checksum must be computed.
    Verify,
    /// The replica is corrupt; the file must be fetched again.
    Repair,
}

impl ModelPlan {
    /// A human-oriented label describing the action.
    pub fn describe(self) -> &'static str {
        match self {
            Self::Ignore => "already present",
            Self::Fetch => "needs download",
            Self::Verify => "needs verification",
            Self::Repair => "needs redownload",
        }
    }
}

impl From<ModelState> for ModelPlan {
    fn from(state: ModelState) -> Self {
        match state {
            ModelState::Verified => Self::Ignore,
            ModelState::Missing => Self::Fetch,
            ModelState::Downloaded => Self::Verify,
            ModelState::IntegrityMismatch { .. } => Self::Repair,
        }
    }
}

#[cfg(test)]
mod model_plan_tests {
    use crate::model_plan::ModelPlan;
    use crate::model_state::ModelState;
    use crate::sha256::Sha256;

    #[test]
    fn every_state_maps_to_the_expected_plan() {
        assert_eq!(ModelPlan::from(ModelState::Verified), ModelPlan::Ignore);
        assert_eq!(ModelPlan::from(ModelState::Missing), ModelPlan::Fetch);
        assert_eq!(ModelPlan::from(ModelState::Downloaded), ModelPlan::Verify);
        assert_eq!(corrupt_plan(), ModelPlan::Repair);
    }

    #[test]
    fn descriptions_are_human_oriented() {
        assert_eq!(ModelPlan::Ignore.describe(), "already present");
        assert_eq!(ModelPlan::Fetch.describe(), "needs download");
        assert_eq!(ModelPlan::Verify.describe(), "needs verification");
        assert_eq!(ModelPlan::Repair.describe(), "needs redownload");
    }

    fn corrupt_plan() -> ModelPlan {
        ModelState::IntegrityMismatch {
            expected: Sha256::from_bytes([1u8; 32]),
            actual: Sha256::from_bytes([2u8; 32]),
        }
        .into()
    }
}

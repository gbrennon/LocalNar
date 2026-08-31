use crate::value_objects::{ContextLength, ParameterCount};

/// What a catalog discloses about serving a model, beyond the weights themselves.
///
/// Publishers are inconsistent about metadata, so each fact is independently
/// optional and an absent fact is never substituted with a guess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelProfile {
    parameters: Option<ParameterCount>,
    context_length: Option<ContextLength>,
}

impl ModelProfile {
    /// The profile of a model whose catalog disclosed nothing about serving it.
    pub const UNDISCLOSED: Self = Self {
        parameters: None,
        context_length: None,
    };

    /// Builds a profile from the facts the catalog disclosed.
    pub const fn new(
        parameters: Option<ParameterCount>,
        context_length: Option<ContextLength>,
    ) -> Self {
        Self {
            parameters,
            context_length,
        }
    }

    /// The number of weights, when the catalog disclosed it.
    pub fn parameters(&self) -> Option<ParameterCount> {
        self.parameters
    }

    /// The context window, when the catalog disclosed it.
    pub fn context_length(&self) -> Option<ContextLength> {
        self.context_length
    }
}

#[cfg(test)]
mod model_profile_tests {
    use crate::value_objects::{ContextLength, ModelProfile, ParameterCount};

    #[test]
    fn a_disclosed_profile_reports_both_facts() {
        let profile = ModelProfile::new(
            Some(ParameterCount::new(8_190_735_360)),
            Some(ContextLength::new(40_960)),
        );

        assert_eq!(
            profile.parameters(),
            Some(ParameterCount::new(8_190_735_360))
        );
        assert_eq!(profile.context_length(), Some(ContextLength::new(40_960)));
    }

    #[test]
    fn an_undisclosed_profile_reports_neither_fact() {
        assert_eq!(ModelProfile::UNDISCLOSED.parameters(), None);
        assert_eq!(ModelProfile::UNDISCLOSED.context_length(), None);
    }

    #[test]
    fn facts_are_independently_optional() {
        let profile = ModelProfile::new(Some(ParameterCount::new(270_000_000)), None);

        assert_eq!(profile.parameters(), Some(ParameterCount::new(270_000_000)));
        assert_eq!(profile.context_length(), None);
    }
}

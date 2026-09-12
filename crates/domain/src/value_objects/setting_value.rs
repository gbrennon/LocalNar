use std::fmt;

/// The value of a single configuration setting.
///
/// A configuration value is an unconstrained string, empty included; no existing
/// value object models an arbitrary config value, so construction is infallible.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingValue(String);

impl SettingValue {
    /// Builds a value from any string-like value, verbatim.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrows the value as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SettingValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod setting_value_tests {
    use crate::value_objects::SettingValue;

    #[test]
    fn a_value_round_trips_through_its_accessors() {
        let value = SettingValue::new("/tmp/hf");

        assert_eq!(value.as_str(), "/tmp/hf");
        assert_eq!(value.to_string(), "/tmp/hf");
    }

    #[test]
    fn an_empty_value_is_accepted() {
        let value = SettingValue::new("");

        assert_eq!(value.as_str(), "");
    }
}

use std::fmt;

use crate::errors::DomainError;

/// The identifier of a single configuration setting.
///
/// A key is a non-blank string; construction trims surrounding whitespace and
/// rejects values that are empty once trimmed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingKey(String);

impl SettingKey {
    /// Builds a key from any string-like value.
    ///
    /// The value is trimmed; a blank value yields [`DomainError::BlankSettingKey`].
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(DomainError::BlankSettingKey);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Borrows the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SettingKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod setting_key_tests {
    use crate::{errors::DomainError, value_objects::SettingKey};

    #[test]
    fn a_non_blank_key_is_accepted_and_trimmed() {
        let key = SettingKey::new("  huggingface.api_token  ").expect("valid key");

        assert_eq!(key.as_str(), "huggingface.api_token");
        assert_eq!(key.to_string(), "huggingface.api_token");
    }

    #[test]
    fn blank_and_whitespace_only_keys_are_rejected() {
        assert_eq!(SettingKey::new(""), Err(DomainError::BlankSettingKey));
        assert_eq!(SettingKey::new("   "), Err(DomainError::BlankSettingKey));
        assert_eq!(SettingKey::new("\t\n"), Err(DomainError::BlankSettingKey));
    }
}

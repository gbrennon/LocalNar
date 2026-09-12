use std::{error::Error, fmt};

/// The reason a settings store could not read or write the persisted settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsStoreError {
    /// The persisted settings could not be read from their backing store.
    Unreadable(String),

    /// The settings could not be written to their backing store.
    Unwritable(String),

    /// The persisted settings existed but could not be parsed.
    Malformed(String),
}

impl fmt::Display for SettingsStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(cause) => {
                write!(formatter, "the settings could not be read: {cause}")
            }
            Self::Unwritable(cause) => {
                write!(formatter, "the settings could not be saved: {cause}")
            }
            Self::Malformed(cause) => {
                write!(formatter, "the stored settings are malformed: {cause}")
            }
        }
    }
}

impl Error for SettingsStoreError {}

#[cfg(test)]
mod settings_store_error_tests {
    use super::*;

    #[test]
    fn unreadable_variant_constructs_and_displays() {
        let error = SettingsStoreError::Unreadable("disk".into());

        assert_eq!(error.to_string(), "the settings could not be read: disk");
    }

    #[test]
    fn unwritable_variant_constructs_and_displays() {
        let error = SettingsStoreError::Unwritable("disk".into());

        assert_eq!(error.to_string(), "the settings could not be saved: disk");
    }

    #[test]
    fn malformed_variant_constructs_and_displays() {
        let error = SettingsStoreError::Malformed("syntax".into());

        assert_eq!(
            error.to_string(),
            "the stored settings are malformed: syntax"
        );
    }

    #[test]
    fn settings_store_error_implements_std_error() {
        let error = SettingsStoreError::Unreadable("disk".into());

        let _: &dyn std::error::Error = &error;
    }
}

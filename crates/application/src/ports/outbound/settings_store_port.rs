use localnar_domain::Settings;

use crate::errors::settings_store_error::SettingsStoreError;

/// Outbound contract for persisting and retrieving operator settings.
///
/// Implementations own where and how settings are stored. Loading an absent
/// store MUST yield an empty [`Settings`] rather than an error, so a first run
/// starts from a clean slate. Saving MUST make the given settings durable.
pub trait SettingsStorePort: Send + Sync {
    /// Reads the persisted settings, or an empty set when none were ever saved.
    fn load(&self) -> Result<Settings, SettingsStoreError>;

    /// Persists `settings`, replacing any previously stored set.
    fn save(&self, settings: &Settings) -> Result<(), SettingsStoreError>;
}

use localnar_domain::Settings;

use crate::errors::settings_store_error::SettingsStoreError;

/// Inbound contract for persisting operator settings.
pub trait SaveSettingsPort: Send + Sync {
    /// Persists `settings`, replacing any previously stored set.
    fn execute(&self, settings: &Settings) -> Result<(), SettingsStoreError>;
}

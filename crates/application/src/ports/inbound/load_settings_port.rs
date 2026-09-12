use localnar_domain::Settings;

use crate::errors::settings_store_error::SettingsStoreError;

/// Inbound contract for retrieving the operator's current settings.
pub trait LoadSettingsPort: Send + Sync {
    /// Returns the currently persisted settings, empty when none were saved.
    fn execute(&self) -> Result<Settings, SettingsStoreError>;
}

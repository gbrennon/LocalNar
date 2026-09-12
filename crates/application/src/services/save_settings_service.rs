use localnar_domain::Settings;

use crate::{
    errors::settings_store_error::SettingsStoreError,
    ports::{inbound::save_settings_port::SaveSettingsPort, outbound::SettingsStorePort},
};

/// Persists the operator's settings through a settings store.
pub struct SaveSettingsService<Store>
where
    Store: SettingsStorePort,
{
    store: Store,
}

impl<Store> SaveSettingsService<Store>
where
    Store: SettingsStorePort,
{
    /// Builds the service over an injected settings store.
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl<Store> SaveSettingsPort for SaveSettingsService<Store>
where
    Store: SettingsStorePort,
{
    fn execute(&self, settings: &Settings) -> Result<(), SettingsStoreError> {
        self.store.save(settings)
    }
}

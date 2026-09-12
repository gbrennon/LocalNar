use localnar_domain::Settings;

use crate::{
    errors::settings_store_error::SettingsStoreError,
    ports::{inbound::load_settings_port::LoadSettingsPort, outbound::SettingsStorePort},
};

/// Retrieves the operator's settings through a settings store.
pub struct LoadSettingsService<Store>
where
    Store: SettingsStorePort,
{
    store: Store,
}

impl<Store> LoadSettingsService<Store>
where
    Store: SettingsStorePort,
{
    /// Builds the service over an injected settings store.
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl<Store> LoadSettingsPort for LoadSettingsService<Store>
where
    Store: SettingsStorePort,
{
    fn execute(&self) -> Result<Settings, SettingsStoreError> {
        self.store.load()
    }
}

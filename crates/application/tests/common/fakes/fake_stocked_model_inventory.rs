#![allow(dead_code)]
use application::errors::LibraryError;
use application::ports::outbound::ModelInventoryPort;
use domain::ModelInventory;

use crate::common::fakes::model_fixture::ModelFixture;

/// An inventory of a library holding several replicas of differing states.
pub struct FakeStockedModelInventory;

impl ModelInventoryPort for FakeStockedModelInventory {
    async fn enumerate(&self) -> Result<ModelInventory, LibraryError> {
        Ok(ModelInventory::new(
            ModelFixture::library_root(),
            vec![
                ModelFixture::verified_entry(),
                ModelFixture::unproven_entry(),
            ],
        ))
    }
}

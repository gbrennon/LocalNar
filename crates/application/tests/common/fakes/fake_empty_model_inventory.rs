#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::ModelInventoryPort};
use localnar_domain::ModelInventory;

use crate::common::fakes::model_fixture::ModelFixture;

/// An inventory of a library that exists but holds no replica at all.
pub struct FakeEmptyModelInventory;

impl ModelInventoryPort for FakeEmptyModelInventory {
    async fn enumerate(&self) -> Result<ModelInventory, LibraryError> {
        Ok(ModelInventory::new(
            ModelFixture::library_root(),
            Vec::new(),
        ))
    }
}

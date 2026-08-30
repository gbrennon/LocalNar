use application::errors::ListInstalledModelsError;
use application::ports::inbound::ListInstalledModelsPort;
use application::ports::outbound::ModelInventoryPort;
use application::services::ListInstalledModelsService;
use domain::{ByteLength, ModelInventory};

use crate::common::block_on::BlockOn;
use crate::common::fakes::fake_empty_model_inventory::FakeEmptyModelInventory;
use crate::common::fakes::fake_stocked_model_inventory::FakeStockedModelInventory;
use crate::common::fakes::fake_unreadable_model_inventory::FakeUnreadableModelInventory;
use crate::common::fakes::model_fixture::ModelFixture;

/// Runs a listing against one inventory double.
struct ListHarness;

impl ListHarness {
    async fn outcome<Inventory: ModelInventoryPort>(
        inventory: Inventory,
    ) -> Result<ModelInventory, ListInstalledModelsError> {
        ListInstalledModelsService::new(inventory).execute().await
    }
}

#[test]
fn listing_when_the_library_is_stocked_then_every_replica_it_holds_is_answered() {
    BlockOn::run(async {
        let inventory = ListHarness::outcome(FakeStockedModelInventory)
            .await
            .expect("a stocked library must answer with what it holds");

        assert_eq!(
            inventory,
            ModelInventory::new(
                ModelFixture::library_root(),
                vec![
                    ModelFixture::verified_entry(),
                    ModelFixture::unproven_entry()
                ],
            )
        );
        assert_eq!(
            inventory.find(&ModelFixture::spec()),
            Some(&ModelFixture::verified_entry())
        );
        assert_eq!(
            inventory.find(&ModelFixture::companion_spec()),
            Some(&ModelFixture::unproven_entry())
        );
    });
}

#[test]
fn listing_when_the_library_is_stocked_then_its_place_and_summed_size_are_answered() {
    BlockOn::run(async {
        let inventory = ListHarness::outcome(FakeStockedModelInventory)
            .await
            .expect("a stocked library must answer with what it holds");

        assert_eq!(inventory.root(), ModelFixture::library_root().as_path());
        assert_eq!(inventory.total_size(), ByteLength::new(12_288));
        assert_eq!(inventory.count(), 2);
    });
}

#[test]
fn listing_when_the_library_holds_nothing_then_an_empty_inventory_is_answered() {
    BlockOn::run(async {
        let inventory = ListHarness::outcome(FakeEmptyModelInventory)
            .await
            .expect("a library holding nothing is an answer, not a failure");

        assert!(inventory.is_empty());
        assert_eq!(inventory.count(), 0);
        assert_eq!(inventory.total_size(), ByteLength::ZERO);
        assert_eq!(inventory.root(), ModelFixture::library_root().as_path());
    });
}

#[test]
fn listing_when_the_library_cannot_be_read_then_the_library_boundary_is_reported() {
    BlockOn::run(async {
        let failure = ListHarness::outcome(FakeUnreadableModelInventory)
            .await
            .expect_err("an unreadable library must not be listed as empty");

        assert_eq!(
            failure,
            ListInstalledModelsError::Library(FakeUnreadableModelInventory::error())
        );
    });
}

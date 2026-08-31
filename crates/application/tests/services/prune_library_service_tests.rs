use application::{
    errors::PruneLibraryError,
    ports::{inbound::PruneLibraryPort, outbound::LibraryMaintenancePort},
    services::PruneLibraryService,
};
use domain::{ByteLength, DiscardedStray};

use crate::common::{
    block_on::BlockOn,
    fakes::{
        fake_clean_maintenance::FakeCleanMaintenance,
        fake_sweeping_maintenance::FakeSweepingMaintenance,
        fake_unreadable_maintenance::FakeUnreadableMaintenance, model_fixture::ModelFixture,
    },
};

/// Runs a sweep against one maintenance double.
struct PruneHarness;

impl PruneHarness {
    async fn outcome<Maintenance: LibraryMaintenancePort>(
        maintenance: Maintenance,
    ) -> Result<Vec<DiscardedStray>, PruneLibraryError> {
        PruneLibraryService::new(maintenance).execute().await
    }
}

#[test]
fn pruning_when_the_library_carries_leftovers_then_each_one_discarded_is_answered() {
    BlockOn::run(async {
        let discarded = PruneHarness::outcome(FakeSweepingMaintenance)
            .await
            .expect("a sweep of a library carrying leftovers must report them");

        assert_eq!(
            discarded,
            vec![
                ModelFixture::discarded_digest_record(),
                ModelFixture::discarded_companion_digest_record(),
                ModelFixture::discarded_emptied_directory(),
            ]
        );
    });
}

#[test]
fn pruning_when_the_library_carries_leftovers_then_their_summed_space_is_reclaimed() {
    BlockOn::run(async {
        let discarded = PruneHarness::outcome(FakeSweepingMaintenance)
            .await
            .expect("a sweep of a library carrying leftovers must report them");

        assert_eq!(
            DiscardedStray::total_reclaimed(&discarded),
            ByteLength::new(128)
        );
    });
}

#[test]
fn pruning_when_the_library_keeps_only_models_then_nothing_is_discarded() {
    BlockOn::run(async {
        let discarded = PruneHarness::outcome(FakeCleanMaintenance)
            .await
            .expect("a library with nothing to sweep is an answer, not a failure");

        assert!(discarded.is_empty());
        assert_eq!(
            DiscardedStray::total_reclaimed(&discarded),
            ByteLength::ZERO
        );
    });
}

#[test]
fn pruning_when_the_library_cannot_be_read_then_the_library_boundary_is_reported() {
    BlockOn::run(async {
        let failure = PruneHarness::outcome(FakeUnreadableMaintenance)
            .await
            .expect_err("an unreadable library must not be reported as swept clean");

        assert_eq!(
            failure,
            PruneLibraryError::Library(FakeUnreadableMaintenance::error())
        );
    });
}

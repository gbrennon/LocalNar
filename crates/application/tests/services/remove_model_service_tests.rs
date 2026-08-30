use application::errors::RemoveModelError;
use application::ports::inbound::RemoveModelPort;
use application::ports::outbound::{ModelEvictionPort, ModelLibraryPort};
use application::services::RemoveModelService;
use domain::{ByteLength, RemovedModel};

use crate::common::block_on::BlockOn;
use crate::common::fakes::fake_evicting_library::FakeEvictingLibrary;
use crate::common::fakes::fake_missing_model_library::FakeMissingModelLibrary;
use crate::common::fakes::fake_refusing_eviction::FakeRefusingEviction;
use crate::common::fakes::fake_unreachable_eviction::FakeUnreachableEviction;
use crate::common::fakes::fake_vacating_model_library::FakeVacatingModelLibrary;
use crate::common::fakes::fake_verified_model_library::FakeVerifiedModelLibrary;
use crate::common::fakes::model_fixture::ModelFixture;

/// Runs a removal against one combination of library and eviction doubles.
struct RemoveHarness;

impl RemoveHarness {
    async fn outcome<Library, Eviction>(
        library: Library,
        eviction: Eviction,
    ) -> Result<RemovedModel, RemoveModelError>
    where
        Library: ModelLibraryPort,
        Eviction: ModelEvictionPort,
    {
        RemoveModelService::new(library, eviction)
            .execute(&ModelFixture::spec())
            .await
    }
}

#[test]
fn removing_when_the_library_holds_the_replica_then_the_reclaimed_space_is_reported() {
    BlockOn::run(async {
        let removal = RemoveHarness::outcome(FakeVacatingModelLibrary::new(), FakeEvictingLibrary)
            .await
            .expect("a held replica must be discarded");

        assert_eq!(removal, ModelFixture::removed());
        assert_eq!(removal.reclaimed(), ByteLength::new(4_096));
        assert_eq!(removal.spec(), &ModelFixture::spec());
    });
}

#[test]
fn removing_when_the_library_holds_no_replica_then_nothing_is_discarded() {
    BlockOn::run(async {
        let failure = RemoveHarness::outcome(FakeMissingModelLibrary, FakeUnreachableEviction)
            .await
            .expect_err("an operator reclaiming space must be told it was never occupied");

        assert_eq!(
            failure,
            RemoveModelError::NotInstalled {
                model: ModelFixture::spec().to_string(),
            }
        );
    });
}

#[test]
fn removing_when_the_replica_survives_its_own_removal_then_it_is_reported_still_installed() {
    BlockOn::run(async {
        let failure = RemoveHarness::outcome(FakeVerifiedModelLibrary, FakeEvictingLibrary)
            .await
            .expect_err("a replica the library still reports must not be reported as reclaimed");

        assert_eq!(
            failure,
            RemoveModelError::StillInstalled {
                model: ModelFixture::spec().to_string(),
            }
        );
    });
}

#[test]
fn removing_when_the_replica_cannot_be_discarded_then_the_library_boundary_is_reported() {
    BlockOn::run(async {
        let failure = RemoveHarness::outcome(FakeVerifiedModelLibrary, FakeRefusingEviction)
            .await
            .expect_err("a library that cannot discard must not answer with reclaimed space");

        assert_eq!(
            failure,
            RemoveModelError::Library(FakeRefusingEviction::error())
        );
    });
}

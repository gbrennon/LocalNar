use application::errors::InspectModelError;
use application::ports::inbound::InspectModelPort;
use application::ports::outbound::ModelLibraryPort;
use application::services::InspectModelService;
use domain::ManagedModel;

use crate::common::block_on::BlockOn;
use crate::common::fakes::fake_missing_model_library::FakeMissingModelLibrary;
use crate::common::fakes::fake_unproven_model_library::FakeUnprovenModelLibrary;
use crate::common::fakes::fake_unreadable_model_library::FakeUnreadableModelLibrary;
use crate::common::fakes::fake_verified_model_library::FakeVerifiedModelLibrary;
use crate::common::fakes::model_fixture::ModelFixture;

/// Runs an inspection against one library double.
struct InspectHarness;

impl InspectHarness {
    async fn outcome<Library: ModelLibraryPort>(
        library: Library,
    ) -> Result<ManagedModel, InspectModelError> {
        InspectModelService::new(library)
            .execute(&ModelFixture::spec())
            .await
    }
}

#[test]
fn inspecting_when_the_replica_was_proven_then_it_is_described_as_verified() {
    BlockOn::run(async {
        let entry = InspectHarness::outcome(FakeVerifiedModelLibrary)
            .await
            .expect("a proven replica must be describable");

        assert_eq!(entry, ModelFixture::verified_entry());
        assert!(entry.is_verified());
    });
}

#[test]
fn inspecting_when_the_replica_was_never_proven_then_it_is_described_as_unproven() {
    BlockOn::run(async {
        let entry = InspectHarness::outcome(FakeUnprovenModelLibrary)
            .await
            .expect("an unproven replica is still installed and describable");

        assert!(entry.is_unproven());
        assert_eq!(entry.replica(), &ModelFixture::installed(None));
        assert_eq!(entry.digest(), None);
    });
}

#[test]
fn inspecting_when_the_library_holds_no_replica_then_it_is_reported_not_installed() {
    BlockOn::run(async {
        let failure = InspectHarness::outcome(FakeMissingModelLibrary)
            .await
            .expect_err("a model this machine never installed must not be described");

        assert_eq!(
            failure,
            InspectModelError::NotInstalled {
                model: ModelFixture::spec().to_string(),
            }
        );
    });
}

#[test]
fn inspecting_when_the_library_cannot_be_read_then_the_library_boundary_is_reported() {
    BlockOn::run(async {
        let failure = InspectHarness::outcome(FakeUnreadableModelLibrary)
            .await
            .expect_err("an unreadable library must not be reported as holding nothing");

        assert_eq!(
            failure,
            InspectModelError::Library(FakeUnreadableModelLibrary::error())
        );
    });
}

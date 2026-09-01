use localnar_application::{
    errors::VerifyModelError,
    ports::{inbound::VerifyModelPort, outbound::ModelLibraryPort},
    services::VerifyModelService,
};
use localnar_domain::ManagedModel;

use crate::common::{
    block_on::BlockOn,
    fakes::{
        fake_broken_model_library::FakeBrokenModelLibrary,
        fake_missing_model_library::FakeMissingModelLibrary,
        fake_unhashable_model_library::FakeUnhashableModelLibrary,
        fake_unproven_model_library::FakeUnprovenModelLibrary,
        fake_vanishing_model_library::FakeVanishingModelLibrary,
        fake_verified_model_library::FakeVerifiedModelLibrary, model_fixture::ModelFixture,
    },
};

/// Runs a verification against one library double.
struct VerifyHarness;

impl VerifyHarness {
    async fn outcome<Library: ModelLibraryPort>(
        library: Library,
    ) -> Result<ManagedModel, VerifyModelError> {
        VerifyModelService::new(library)
            .execute(&ModelFixture::spec())
            .await
    }
}

#[test]
fn verifying_when_the_bytes_still_match_the_recorded_digest_then_the_replica_is_proven() {
    BlockOn::run(async {
        let entry = VerifyHarness::outcome(FakeVerifiedModelLibrary)
            .await
            .expect("a replica matching its digest must come back proven");

        assert_eq!(entry, ModelFixture::verified_entry());
        assert!(entry.is_verified());
    });
}

#[test]
fn verifying_when_the_bytes_no_longer_match_the_recorded_digest_then_the_replica_reads_as_broken() {
    BlockOn::run(async {
        let entry = VerifyHarness::outcome(FakeBrokenModelLibrary)
            .await
            .expect("a disagreement is the verdict the operator asked for, not a failure");

        assert!(entry.is_broken());
        assert_eq!(entry, ModelFixture::broken_entry());
        assert_eq!(entry.state(), &ModelFixture::mismatched_state());
    });
}

#[test]
fn verifying_when_the_replica_carries_no_recorded_digest_then_it_reads_as_unproven_unread() {
    BlockOn::run(async {
        let entry = VerifyHarness::outcome(FakeUnprovenModelLibrary)
            .await
            .expect("a replica with nothing to prove it against is still installed");

        assert!(entry.is_unproven());
        assert_eq!(entry.replica(), &ModelFixture::installed(None));
    });
}

#[test]
fn verifying_when_the_library_holds_no_replica_then_it_is_reported_not_installed() {
    BlockOn::run(async {
        let failure = VerifyHarness::outcome(FakeMissingModelLibrary)
            .await
            .expect_err("a model this machine never installed has nothing to verify");

        assert_eq!(
            failure,
            VerifyModelError::NotInstalled {
                model: ModelFixture::spec().to_string(),
            }
        );
    });
}

#[test]
fn verifying_when_the_replica_goes_away_midway_then_it_is_reported_not_installed() {
    BlockOn::run(async {
        let failure = VerifyHarness::outcome(FakeVanishingModelLibrary)
            .await
            .expect_err("a replica that is gone must not be answered as a state of itself");

        assert_eq!(
            failure,
            VerifyModelError::NotInstalled {
                model: ModelFixture::spec().to_string(),
            }
        );
    });
}

#[test]
fn verifying_when_the_replica_cannot_be_hashed_then_the_library_boundary_is_reported() {
    BlockOn::run(async {
        let failure = VerifyHarness::outcome(FakeUnhashableModelLibrary)
            .await
            .expect_err("a replica that cannot be hashed must not be reported as proven");

        assert_eq!(
            failure,
            VerifyModelError::Library(FakeUnhashableModelLibrary::error())
        );
    });
}

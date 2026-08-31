use application::{
    errors::InstallModelError,
    ports::{
        inbound::InstallModelPort,
        outbound::{
            DownloadProgress, ModelDownloaderPort, ModelLibraryPort, RemoteModelRegistryPort,
        },
    },
    services::InstallModelService,
};
use domain::InstalledModel;

use crate::common::{
    block_on::BlockOn,
    fakes::{
        fake_absent_model_library::FakeAbsentModelLibrary,
        fake_advertising_registry::FakeAdvertisingRegistry,
        fake_corrupt_model_library::FakeCorruptModelLibrary,
        fake_downloaded_model_library::FakeDownloadedModelLibrary,
        fake_failing_downloader::FakeFailingDownloader,
        fake_missing_model_library::FakeMissingModelLibrary,
        fake_recording_progress::FakeRecordingProgress, fake_silent_progress::FakeSilentProgress,
        fake_staging_downloader::FakeStagingDownloader,
        fake_stubborn_model_library::FakeStubbornModelLibrary,
        fake_unreachable_registry::FakeUnreachableRegistry,
        fake_unreadable_model_library::FakeUnreadableModelLibrary,
        fake_unverifiable_model_library::FakeUnverifiableModelLibrary,
        fake_verified_model_library::FakeVerifiedModelLibrary, model_fixture::ModelFixture,
    },
};

/// Runs an install against one combination of port doubles.
struct InstallHarness;

impl InstallHarness {
    async fn outcome<Registry, Downloader, Library>(
        registry: Registry,
        downloader: Downloader,
        library: Library,
    ) -> Result<InstalledModel, InstallModelError>
    where
        Registry: RemoteModelRegistryPort,
        Downloader: ModelDownloaderPort,
        Library: ModelLibraryPort,
    {
        InstallModelService::new(registry, downloader, library, FakeSilentProgress)
            .execute(&ModelFixture::spec())
            .await
    }

    /// Wires the ports that let a whole install run to completion.
    async fn with_working_ports<Library: ModelLibraryPort>(
        library: Library,
    ) -> Result<InstalledModel, InstallModelError> {
        Self::outcome(FakeAdvertisingRegistry, FakeStagingDownloader, library).await
    }
}

#[test]
fn install_when_replica_is_already_verified_then_nothing_is_installed() {
    BlockOn::run(async {
        let outcome = InstallHarness::with_working_ports(FakeVerifiedModelLibrary)
            .await
            .expect("an installed model needs no work");

        assert_eq!(
            outcome,
            ModelFixture::installed(Some(ModelFixture::expected_digest()))
        );
    });
}

#[test]
fn install_when_replica_is_absent_then_it_is_fetched_committed_and_verified() {
    BlockOn::run(async {
        let outcome = InstallHarness::with_working_ports(FakeMissingModelLibrary)
            .await
            .expect("a missing model must install end to end");

        assert_eq!(
            outcome,
            ModelFixture::installed(Some(ModelFixture::expected_digest()))
        );
    });
}

#[test]
fn install_when_replica_is_unverified_then_it_is_verified_in_place() {
    BlockOn::run(async {
        let outcome = InstallHarness::with_working_ports(FakeDownloadedModelLibrary)
            .await
            .expect("a staged replica must be verified without refetching");

        assert_eq!(
            outcome,
            ModelFixture::installed(Some(ModelFixture::expected_digest()))
        );
    });
}

#[test]
fn install_when_replica_fails_its_checksum_then_one_repair_resolves_it() {
    BlockOn::run(async {
        let outcome = InstallHarness::with_working_ports(FakeCorruptModelLibrary)
            .await
            .expect("a single repair must settle a recoverable mismatch");

        assert_eq!(
            outcome,
            ModelFixture::installed(Some(ModelFixture::expected_digest()))
        );
    });
}

#[test]
fn install_when_repair_does_not_settle_the_checksum_then_it_reports_unresolved_integrity() {
    BlockOn::run(async {
        let failure = InstallHarness::with_working_ports(FakeStubbornModelLibrary)
            .await
            .expect_err("a mismatch surviving repair must not be reported as success");

        assert_eq!(
            failure,
            InstallModelError::UnresolvedIntegrity {
                expected: ModelFixture::expected_digest().to_string(),
                actual: ModelFixture::actual_digest().to_string(),
            }
        );
    });
}

#[test]
fn install_when_the_registry_is_unreachable_then_the_registry_boundary_is_reported() {
    BlockOn::run(async {
        let failure = InstallHarness::outcome(
            FakeUnreachableRegistry,
            FakeStagingDownloader,
            FakeMissingModelLibrary,
        )
        .await
        .expect_err("an unreachable registry must fail the install");

        assert_eq!(
            failure,
            InstallModelError::Registry(FakeUnreachableRegistry::error())
        );
    });
}

#[test]
fn install_when_the_download_breaks_then_the_download_boundary_is_reported() {
    BlockOn::run(async {
        let failure = InstallHarness::outcome(
            FakeAdvertisingRegistry,
            FakeFailingDownloader,
            FakeMissingModelLibrary,
        )
        .await
        .expect_err("a broken transfer must fail the install");

        assert_eq!(
            failure,
            InstallModelError::Download(FakeFailingDownloader::error())
        );
    });
}

#[test]
fn install_when_the_library_cannot_be_read_then_the_library_boundary_is_reported() {
    BlockOn::run(async {
        let failure = InstallHarness::with_working_ports(FakeUnreadableModelLibrary)
            .await
            .expect_err("an unreadable library must fail the install");

        assert_eq!(
            failure,
            InstallModelError::Library(FakeUnreadableModelLibrary::error())
        );
    });
}

#[test]
fn install_when_bytes_are_transferred_then_progress_is_reported_start_to_finish() {
    BlockOn::run(async {
        let recorder = FakeRecordingProgress::new();
        let total = ModelFixture::remote_file().size();

        InstallModelService::new(
            FakeAdvertisingRegistry,
            FakeStagingDownloader,
            FakeMissingModelLibrary,
            &recorder,
        )
        .execute(&ModelFixture::spec())
        .await
        .expect("a missing model must install end to end");

        assert_eq!(
            recorder.reports(),
            vec![
                DownloadProgress::Started { total },
                DownloadProgress::Advanced {
                    transferred: total,
                    total,
                },
                DownloadProgress::Finished,
            ]
        );
    });
}

#[test]
fn install_when_nothing_is_transferred_then_no_progress_is_reported() {
    BlockOn::run(async {
        let recorder = FakeRecordingProgress::new();

        InstallModelService::new(
            FakeAdvertisingRegistry,
            FakeStagingDownloader,
            FakeVerifiedModelLibrary,
            &recorder,
        )
        .execute(&ModelFixture::spec())
        .await
        .expect("an installed model needs no work");

        assert!(recorder.reports().is_empty());
    });
}

#[test]
fn a_search_row_yields_the_intent_that_downloads_it() {
    let row = ModelFixture::remote_file();

    assert_eq!(row.to_spec(), ModelFixture::spec());
}

#[test]
fn install_when_upstream_advertises_no_checksum_then_it_is_installed_but_unverified() {
    BlockOn::run(async {
        let outcome = InstallHarness::with_working_ports(FakeUnverifiableModelLibrary)
            .await
            .expect("an unprovable replica is still installed");

        assert!(
            !outcome.is_verified(),
            "integrity must not be claimed without an advertised digest"
        );
        assert_eq!(outcome.path(), ModelFixture::installed(None).path());
    });
}

#[test]
fn install_when_a_fetch_leaves_the_replica_absent_then_upstream_is_reported_unavailable() {
    BlockOn::run(async {
        let failure = InstallHarness::outcome(
            FakeAdvertisingRegistry,
            FakeStagingDownloader,
            FakeAbsentModelLibrary,
        )
        .await
        .expect_err("a replica that never lands must not be reported as installed");

        assert_eq!(failure, InstallModelError::UpstreamUnavailable);
    });
}

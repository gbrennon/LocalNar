use application::errors::{RegistryReadError, SearchModelsError};
use application::ports::inbound::SearchModelsPort;
use application::services::SearchModelsService;

use crate::common::block_on::BlockOn;
use crate::common::fakes::fake_idle_registry::FakeIdleRegistry;
use crate::common::fakes::fake_searching_registry::FakeSearchingRegistry;
use crate::common::fakes::fake_unreachable_registry::FakeUnreachableRegistry;
use crate::common::fakes::model_fixture::ModelFixture;

#[test]
fn search_when_the_catalog_has_matches_then_each_downloadable_file_is_a_row() {
    BlockOn::run(async {
        let rows = SearchModelsService::new(FakeSearchingRegistry)
            .execute(&ModelFixture::query())
            .await
            .expect("a reachable catalog must be searchable");

        assert_eq!(rows, vec![ModelFixture::remote_file()]);
    });
}

#[test]
fn search_rows_carry_the_repository_file_and_size_a_candidate_needs() {
    BlockOn::run(async {
        let rows = SearchModelsService::new(FakeSearchingRegistry)
            .execute(&ModelFixture::query())
            .await
            .expect("a reachable catalog must be searchable");

        let row = rows.first().expect("the catalog matched one file");
        let expected = ModelFixture::spec();

        assert_eq!(row.repository(), expected.repository());
        assert_eq!(row.file(), expected.file());
        assert_eq!(row.size(), ModelFixture::remote_file().size());
    });
}

#[test]
fn search_when_the_registry_is_unreachable_then_the_registry_boundary_is_reported() {
    BlockOn::run(async {
        let failure = SearchModelsService::new(FakeUnreachableRegistry)
            .execute(&ModelFixture::query())
            .await
            .expect_err("an unreachable registry must fail the search");

        assert_eq!(
            failure,
            SearchModelsError::Registry(FakeUnreachableRegistry::error())
        );
    });
}

#[test]
fn search_when_the_adapter_cannot_search_then_it_says_so_without_resolving() {
    BlockOn::run(async {
        let failure = SearchModelsService::new(FakeIdleRegistry)
            .execute(&ModelFixture::query())
            .await
            .expect_err("an adapter without search must decline");

        assert_eq!(
            failure,
            SearchModelsError::Registry(RegistryReadError::EnumerationUnsupported)
        );
    });
}

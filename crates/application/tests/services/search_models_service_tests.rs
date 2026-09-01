use localnar_application::{
    errors::{RegistryReadError, SearchModelsError},
    ports::inbound::SearchModelsPort,
    services::SearchModelsService,
};

use crate::common::{
    block_on::BlockOn,
    fakes::{
        fake_idle_registry::FakeIdleRegistry, fake_searching_registry::FakeSearchingRegistry,
        fake_unreachable_registry::FakeUnreachableRegistry, model_fixture::ModelFixture,
    },
};

#[test]
fn search_when_the_catalog_has_matches_then_each_model_is_one_row() {
    BlockOn::run(async {
        let rows = SearchModelsService::new(FakeSearchingRegistry)
            .execute(&ModelFixture::query())
            .await
            .expect("a reachable catalog must be searchable");

        assert_eq!(rows, vec![ModelFixture::model_info()]);
    });
}

#[test]
fn search_rows_carry_the_name_size_and_precision_a_candidate_needs() {
    BlockOn::run(async {
        let rows = SearchModelsService::new(FakeSearchingRegistry)
            .execute(&ModelFixture::query())
            .await
            .expect("a reachable catalog must be searchable");

        let row = rows.first().expect("the catalog matched one model");

        assert_eq!(row.name(), ModelFixture::spec().repository().identifier());
        assert_eq!(row.size(), ModelFixture::remote_file().size());
        assert_eq!(
            row.quantization().map(|quantization| quantization.label()),
            Some("Q4_K_M")
        );
    });
}

#[test]
fn search_rows_are_actionable_as_install_intents() {
    BlockOn::run(async {
        let rows = SearchModelsService::new(FakeSearchingRegistry)
            .execute(&ModelFixture::query())
            .await
            .expect("a reachable catalog must be searchable");

        let row = rows.first().expect("the catalog matched one model");

        assert_eq!(row.spec(), &ModelFixture::spec());
    });
}

#[test]
fn search_rows_carry_the_serving_profile_the_catalog_disclosed() {
    BlockOn::run(async {
        let rows = SearchModelsService::new(FakeSearchingRegistry)
            .execute(&ModelFixture::query())
            .await
            .expect("a reachable catalog must be searchable");

        let row = rows.first().expect("the catalog matched one model");

        assert_eq!(row.profile(), &ModelFixture::profile());
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

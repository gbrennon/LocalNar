use application::errors::RegistryReadError;
use application::ports::outbound::RemoteModelRegistryPort;
use domain::{
    ByteLength, Checksum, ContextLength, ModelFileName, ModelRepository, ModelRepositoryId,
    ModelRevision, ParameterCount, SearchQuery,
};
use infrastructure::{HfApiRegistry, HubTransport};
use serde::de::DeserializeOwned;

const SEARCH_PATH: &str = "api/models?search=qwen3 gguf&limit=10&expand%5B%5D=gguf";
const QWEN_REVISION_PATH: &str = "api/models/Qwen/Qwen3-8B-GGUF/revision/main?blobs=true";
const UNSLOTH_REVISION_PATH: &str = "api/models/unsloth/Qwen3-8B-GGUF/revision/main?blobs=true";

const CATALOG_JSON: &str = r#"[
    {
        "id": "Qwen/Qwen3-8B-GGUF",
        "gguf": { "total": 8190735360, "architecture": "qwen3", "context_length": 40960 }
    },
    {
        "id": "unsloth/Qwen3-8B-GGUF",
        "gguf": { "total": 8190735360, "architecture": "qwen3", "context_length": 40960 }
    }
]"#;

const QWEN_REVISION_JSON: &str = r#"{
    "id": "Qwen/Qwen3-8B-GGUF",
    "siblings": [
        { "rfilename": ".gitattributes", "size": 3083 },
        { "rfilename": "README.md", "size": 12000 },
        { "rfilename": "LICENSE", "size": 11343 },
        {
            "rfilename": "Qwen3-8B-Q8_0.gguf",
            "size": 8710000000,
            "lfs": { "size": 8710000000, "sha256": "b94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8" }
        },
        {
            "rfilename": "Qwen3-8B-Q4_K_M.gguf",
            "size": 5027784064,
            "lfs": { "size": 5027784064, "sha256": "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8" }
        }
    ]
}"#;

const UNSLOTH_REVISION_JSON: &str = r#"{
    "id": "unsloth/Qwen3-8B-GGUF",
    "siblings": [
        { "rfilename": ".gitattributes", "size": 3083 },
        {
            "rfilename": "Qwen3-8B-BF16.gguf",
            "size": 16388044384,
            "lfs": { "size": 16388044384, "sha256": "c94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8" }
        },
        {
            "rfilename": "Qwen3-8B-Q2_K.gguf",
            "size": 3281733440,
            "lfs": { "size": 3281733440, "sha256": "d94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8" }
        }
    ]
}"#;

/// A catalog that answers exactly the paths it was given and nothing else.
///
/// A path left out stands for an entry whose revision cannot be read, which is
/// how the adapter's tolerance of partial catalogs is exercised.
struct FakeCatalogTransport {
    answers: Vec<(String, String)>,
}

impl FakeCatalogTransport {
    fn answering(answers: &[(&str, &str)]) -> Self {
        Self {
            answers: answers
                .iter()
                .map(|(path, body)| ((*path).to_string(), (*body).to_string()))
                .collect(),
        }
    }
}

impl HubTransport for FakeCatalogTransport {
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, RegistryReadError> {
        let body = self
            .answers
            .iter()
            .find(|(known, _)| known == path)
            .map(|(_, body)| body)
            .ok_or_else(|| RegistryReadError::FileNotFound {
                repository: path.to_string(),
                file: String::new(),
            })?;

        serde_json::from_str(body).map_err(|_| RegistryReadError::Malformed {
            repository: path.to_string(),
        })
    }
}

/// A catalog whose host never answers.
struct FakeUnreachableTransport;

impl FakeUnreachableTransport {
    fn error() -> RegistryReadError {
        RegistryReadError::Unreachable {
            repository: "unsloth/Qwen3-8B-GGUF".to_string(),
            cause: "connection refused".to_string(),
        }
    }
}

impl HubTransport for FakeUnreachableTransport {
    async fn get_json<T: DeserializeOwned>(&self, _path: &str) -> Result<T, RegistryReadError> {
        Err(Self::error())
    }
}

fn query() -> SearchQuery {
    SearchQuery::new("qwen3 gguf").expect("the phrase carries text")
}

fn unsloth_repository() -> ModelRepository {
    ModelRepository::new(
        ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id"),
        ModelRevision::new("main").expect("valid revision"),
    )
}

fn readable_catalog() -> FakeCatalogTransport {
    FakeCatalogTransport::answering(&[
        (SEARCH_PATH, CATALOG_JSON),
        (QWEN_REVISION_PATH, QWEN_REVISION_JSON),
        (UNSLOTH_REVISION_PATH, UNSLOTH_REVISION_JSON),
    ])
}

#[tokio::test]
async fn search_describes_each_catalog_entry_as_a_single_row() {
    let registry = HfApiRegistry::new(readable_catalog());

    let rows = registry.search_models(&query()).await.expect("search");

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows.iter()
            .map(|row| row.name().to_string())
            .collect::<Vec<_>>(),
        vec!["Qwen/Qwen3-8B-GGUF", "unsloth/Qwen3-8B-GGUF"]
    );
}

#[tokio::test]
async fn search_rows_stand_for_the_weight_the_domain_chooses() {
    let registry = HfApiRegistry::new(readable_catalog());

    let rows = registry.search_models(&query()).await.expect("search");

    assert_eq!(
        rows.iter()
            .map(|row| row.spec().file().to_string())
            .collect::<Vec<_>>(),
        vec!["Qwen3-8B-Q4_K_M.gguf", "Qwen3-8B-Q2_K.gguf"]
    );
}

#[tokio::test]
async fn search_rows_carry_the_size_of_the_chosen_weight() {
    let registry = HfApiRegistry::new(readable_catalog());

    let rows = registry.search_models(&query()).await.expect("search");

    assert_eq!(
        rows.iter().map(|row| row.size()).collect::<Vec<_>>(),
        vec![
            ByteLength::new(5_027_784_064),
            ByteLength::new(3_281_733_440)
        ]
    );
}

#[tokio::test]
async fn search_rows_carry_the_precision_of_the_chosen_weight() {
    let registry = HfApiRegistry::new(readable_catalog());

    let rows = registry.search_models(&query()).await.expect("search");

    assert_eq!(
        rows.iter()
            .map(|row| row.quantization().map(|value| value.label().to_owned()))
            .collect::<Vec<_>>(),
        vec![Some("Q4_K_M".to_owned()), Some("Q2_K".to_owned())]
    );
}

#[tokio::test]
async fn search_rows_carry_the_serving_profile_the_catalog_disclosed() {
    let registry = HfApiRegistry::new(readable_catalog());

    let rows = registry.search_models(&query()).await.expect("search");
    let profile = rows
        .first()
        .expect("the catalog matched two models")
        .profile();

    assert_eq!(
        profile.parameters(),
        Some(ParameterCount::new(8_190_735_360))
    );
    assert_eq!(profile.context_length(), Some(ContextLength::new(40_960)));
}

#[tokio::test]
async fn search_leaves_out_an_entry_whose_revision_cannot_be_read() {
    let registry = HfApiRegistry::new(FakeCatalogTransport::answering(&[
        (SEARCH_PATH, CATALOG_JSON),
        (UNSLOTH_REVISION_PATH, UNSLOTH_REVISION_JSON),
    ]));

    let rows = registry.search_models(&query()).await.expect("search");

    assert_eq!(
        rows.iter()
            .map(|row| row.name().to_string())
            .collect::<Vec<_>>(),
        vec!["unsloth/Qwen3-8B-GGUF"]
    );
}

#[tokio::test]
async fn search_leaves_out_an_entry_that_publishes_no_installable_weight() {
    let registry = HfApiRegistry::new(FakeCatalogTransport::answering(&[
        (SEARCH_PATH, CATALOG_JSON),
        (
            QWEN_REVISION_PATH,
            r#"{ "siblings": [ { "rfilename": "README.md", "size": 12000 } ] }"#,
        ),
        (UNSLOTH_REVISION_PATH, UNSLOTH_REVISION_JSON),
    ]));

    let rows = registry.search_models(&query()).await.expect("search");

    assert_eq!(
        rows.iter()
            .map(|row| row.name().to_string())
            .collect::<Vec<_>>(),
        vec!["unsloth/Qwen3-8B-GGUF"]
    );
}

#[tokio::test]
async fn search_leaves_out_an_entry_whose_identifier_is_malformed() {
    let registry = HfApiRegistry::new(FakeCatalogTransport::answering(&[(
        SEARCH_PATH,
        r#"[ { "id": "no-owner-segment" } ]"#,
    )]));

    let rows = registry.search_models(&query()).await.expect("search");

    assert!(rows.is_empty());
}

#[tokio::test]
async fn search_when_a_catalog_entry_discloses_no_gguf_metadata_then_no_profile_is_invented() {
    let registry = HfApiRegistry::new(FakeCatalogTransport::answering(&[
        (SEARCH_PATH, r#"[ { "id": "unsloth/Qwen3-8B-GGUF" } ]"#),
        (UNSLOTH_REVISION_PATH, UNSLOTH_REVISION_JSON),
    ]));

    let rows = registry.search_models(&query()).await.expect("search");
    let profile = rows
        .first()
        .expect("the catalog matched one model")
        .profile();

    assert_eq!(profile.parameters(), None);
    assert_eq!(profile.context_length(), None);
}

#[tokio::test]
async fn search_propagates_a_transport_failure() {
    let registry = HfApiRegistry::new(FakeUnreachableTransport);

    let failure = registry
        .search_models(&query())
        .await
        .expect_err("an unreachable host must fail the search");

    assert_eq!(failure, FakeUnreachableTransport::error());
}

#[tokio::test]
async fn resolve_model_file_reads_the_size_and_digest_of_the_named_file() {
    let registry = HfApiRegistry::new(FakeCatalogTransport::answering(&[(
        UNSLOTH_REVISION_PATH,
        UNSLOTH_REVISION_JSON,
    )]));
    let repository = unsloth_repository();
    let file = ModelFileName::new("Qwen3-8B-Q2_K.gguf").expect("valid file name");

    let remote_file = registry
        .resolve_model_file(&repository, &file)
        .await
        .expect("resolve");

    assert_eq!(remote_file.repository(), &repository);
    assert_eq!(remote_file.file(), &file);
    assert_eq!(remote_file.size(), ByteLength::new(3_281_733_440));
    assert_eq!(
        remote_file.checksum(),
        Some(
            Checksum::parse("d94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8")
                .expect("valid digest")
        )
    );
}

#[tokio::test]
async fn resolve_model_file_reports_the_file_as_missing_when_the_revision_omits_it() {
    let registry = HfApiRegistry::new(FakeCatalogTransport::answering(&[(
        UNSLOTH_REVISION_PATH,
        UNSLOTH_REVISION_JSON,
    )]));
    let file = ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name");

    let failure = registry
        .resolve_model_file(&unsloth_repository(), &file)
        .await
        .expect_err("an absent file must be reported");

    assert_eq!(
        failure,
        RegistryReadError::FileNotFound {
            repository: unsloth_repository().to_string(),
            file: file.to_string(),
        }
    );
}

#[tokio::test]
async fn resolve_model_file_propagates_a_transport_failure() {
    let registry = HfApiRegistry::new(FakeUnreachableTransport);
    let file = ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name");

    let failure = registry
        .resolve_model_file(&unsloth_repository(), &file)
        .await
        .expect_err("an unreachable host must fail the resolution");

    assert_eq!(failure, FakeUnreachableTransport::error());
}

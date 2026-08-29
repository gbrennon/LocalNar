use application::errors::RegistryReadError;
use application::ports::outbound::RemoteModelRegistryPort;
use domain::{
    ByteLength, Checksum, ModelFileName, ModelRepository, ModelRepositoryId, ModelRevision,
    SearchQuery,
};
use infrastructure::{HfApiRegistry, HubTransport};
use serde::de::DeserializeOwned;

struct FakeHubTransport {
    response_json: String,
    should_fail: Option<RegistryReadError>,
}

impl FakeHubTransport {
    fn returning_json(json: &str) -> Self {
        Self {
            response_json: json.to_string(),
            should_fail: None,
        }
    }

    fn failing_with(error: RegistryReadError) -> Self {
        Self {
            response_json: String::new(),
            should_fail: Some(error),
        }
    }
}

impl HubTransport for FakeHubTransport {
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, RegistryReadError> {
        if let Some(err) = &self.should_fail {
            return Err(err.clone());
        }

        serde_json::from_str(&self.response_json).map_err(|_| RegistryReadError::Malformed {
            repository: path.to_string(),
        })
    }
}

#[tokio::test]
async fn resolve_model_file_extracts_lfs_checksum_and_size() {
    let mock_json = r#"{
        "id": "unsloth/Qwen3-8B-GGUF",
        "siblings": [
            {
                "rfilename": ".gitattributes",
                "size": 1175
            },
            {
                "rfilename": "Qwen3-8B-Q4_K_M.gguf",
                "size": 4200,
                "lfs": {
                    "sha256": "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8",
                    "size": 4096
                }
            }
        ]
    }"#;

    let fake_transport = FakeHubTransport::returning_json(mock_json);
    let registry = HfApiRegistry::new(fake_transport);

    let repository = ModelRepository::new(
        ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id"),
        ModelRevision::new("main").expect("valid rev"),
    );
    let file = ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file");

    let remote_file = registry
        .resolve_model_file(&repository, &file)
        .await
        .expect("resolve");

    assert_eq!(remote_file.repository(), &repository);
    assert_eq!(remote_file.file(), &file);
    assert_eq!(remote_file.size(), ByteLength::new(4096));
    assert_eq!(
        remote_file.checksum(),
        Some(
            Checksum::parse("a94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8")
                .unwrap()
        )
    );
}

#[tokio::test]
async fn resolve_model_file_reports_file_not_found_when_missing_from_siblings() {
    let mock_json = r#"{
        "id": "unsloth/Qwen3-8B-GGUF",
        "siblings": [
            { "rfilename": "other.gguf", "size": 100 }
        ]
    }"#;

    let fake_transport = FakeHubTransport::returning_json(mock_json);
    let registry = HfApiRegistry::new(fake_transport);

    let repository = ModelRepository::new(
        ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id"),
        ModelRevision::new("main").expect("valid rev"),
    );
    let file = ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file");

    let result = registry.resolve_model_file(&repository, &file).await;
    assert!(matches!(
        result,
        Err(RegistryReadError::FileNotFound { .. })
    ));
}

#[tokio::test]
async fn resolve_model_file_propagates_transport_error() {
    let fake_transport = FakeHubTransport::failing_with(RegistryReadError::Unreachable {
        repository: "unsloth/Qwen3-8B-GGUF".to_string(),
        cause: "connection refused".to_string(),
    });
    let registry = HfApiRegistry::new(fake_transport);

    let repository = ModelRepository::new(
        ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id"),
        ModelRevision::new("main").expect("valid rev"),
    );
    let file = ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file");

    let result = registry.resolve_model_file(&repository, &file).await;
    assert!(matches!(result, Err(RegistryReadError::Unreachable { .. })));
}

#[tokio::test]
async fn search_models_flattens_matching_files() {
    let mock_json = r#"[
        {
            "id": "unsloth/Qwen3-8B-GGUF",
            "siblings": [
                {
                    "rfilename": "Qwen3-8B-Q4_K_M.gguf",
                    "size": 5000,
                    "lfs": {
                        "size": 5000,
                        "sha256": "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8"
                    }
                }
            ]
        }
    ]"#;

    let fake_transport = FakeHubTransport::returning_json(mock_json);
    let registry = HfApiRegistry::new(fake_transport);

    let query = SearchQuery::new("qwen3").expect("valid query");
    let results = registry.search_models(&query).await.expect("search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].file().as_str(), "Qwen3-8B-Q4_K_M.gguf");
    assert_eq!(results[0].size(), ByteLength::new(5000));
}

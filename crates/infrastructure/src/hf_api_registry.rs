use application::errors::RegistryReadError;
use application::ports::outbound::RemoteModelRegistryPort;
use domain::{
    ByteLength, Checksum, ModelFileName, ModelRepository, ModelRepositoryId, ModelRevision,
    RemoteModelFile, SearchQuery,
};
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;

const DEFAULT_ENDPOINT: &str = "https://huggingface.co";

/// Transport contract for retrieving raw JSON from the Hugging Face Hub catalog.
pub trait HubTransport: Send + Sync {
    /// Performs a GET request against `path` and deserializes the JSON response.
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, RegistryReadError>;
}

/// Production HTTP transport for the Hugging Face Hub catalog.
#[derive(Debug, Clone)]
pub struct ReqwestHubTransport {
    endpoint: String,
    client: Client,
    token: Option<String>,
}

impl ReqwestHubTransport {
    /// Builds an HTTP transport for the given endpoint and optional authorization token.
    pub fn new(endpoint: impl Into<String>, token: Option<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: Client::builder().build().unwrap_or_default(),
            token: token.filter(|t| !t.trim().is_empty()),
        }
    }

    /// Resolves configuration from `HF_ENDPOINT` and `HF_TOKEN` environment variables.
    pub fn from_env() -> Self {
        let endpoint =
            std::env::var("HF_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        let token = std::env::var("HF_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty());
        Self::new(endpoint, token)
    }
}

impl Default for ReqwestHubTransport {
    fn default() -> Self {
        Self::from_env()
    }
}

impl HubTransport for ReqwestHubTransport {
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, RegistryReadError> {
        let url = format!(
            "{}/{}",
            self.endpoint.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let mut request = self.client.get(&url);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(|err| RegistryReadError::Unreachable {
                repository: path.to_string(),
                cause: err.to_string(),
            })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryReadError::FileNotFound {
                repository: path.to_string(),
                file: String::new(),
            });
        }

        if !response.status().is_success() {
            return Err(RegistryReadError::Unreachable {
                repository: path.to_string(),
                cause: format!("HTTP status {}", response.status()),
            });
        }

        response
            .json()
            .await
            .map_err(|_| RegistryReadError::Malformed {
                repository: path.to_string(),
            })
    }
}

/// Hugging Face Hub catalog adapter for resolving files and searching models.
#[derive(Debug, Clone)]
pub struct HfApiRegistry<Transport = ReqwestHubTransport> {
    transport: Transport,
}

impl<Transport: HubTransport> HfApiRegistry<Transport> {
    /// Builds a registry with an injected transport.
    pub fn new(transport: Transport) -> Self {
        Self { transport }
    }

    /// Returns a reference to the inner transport.
    pub fn transport(&self) -> &Transport {
        &self.transport
    }
}

impl HfApiRegistry<ReqwestHubTransport> {
    /// Resolves configuration from environment variables.
    pub fn from_env() -> Self {
        Self::new(ReqwestHubTransport::from_env())
    }
}

impl Default for HfApiRegistry<ReqwestHubTransport> {
    fn default() -> Self {
        Self::from_env()
    }
}

#[derive(Debug, Deserialize)]
struct RepoDetailsResponse {
    siblings: Option<Vec<RepoSiblingResponse>>,
}

#[derive(Debug, Deserialize)]
struct RepoSiblingResponse {
    rfilename: String,
    size: Option<u64>,
    lfs: Option<LfsDetailsResponse>,
}

#[derive(Debug, Deserialize)]
struct LfsDetailsResponse {
    size: Option<u64>,
    sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchModelItemResponse {
    id: String,
    siblings: Option<Vec<RepoSiblingResponse>>,
}

impl<Transport: HubTransport> RemoteModelRegistryPort for HfApiRegistry<Transport> {
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError> {
        let path = format!(
            "api/models/{}/{}/revision/{}?blobs=true",
            repository.identifier().owner(),
            repository.identifier().name(),
            repository.revision().as_str()
        );

        let repo_info: RepoDetailsResponse =
            self.transport
                .get_json(&path)
                .await
                .map_err(|err| match err {
                    RegistryReadError::FileNotFound { .. } => RegistryReadError::FileNotFound {
                        repository: repository.to_string(),
                        file: file.to_string(),
                    },
                    other => other,
                })?;

        extract_matching_file(repository, file, repo_info)
    }

    async fn search_models(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError> {
        let path = format!("api/models?search={}&full=true&limit=10", query.as_str());
        let results: Vec<SearchModelItemResponse> = self.transport.get_json(&path).await?;
        Ok(extract_search_files(results))
    }
}

fn extract_matching_file(
    repository: &ModelRepository,
    file: &ModelFileName,
    repo_info: RepoDetailsResponse,
) -> Result<RemoteModelFile, RegistryReadError> {
    let matching = repo_info
        .siblings
        .as_deref()
        .unwrap_or_default()
        .iter()
        .find(|sibling| sibling.rfilename == file.as_str())
        .ok_or_else(|| RegistryReadError::FileNotFound {
            repository: repository.to_string(),
            file: file.to_string(),
        })?;

    let size = matching
        .lfs
        .as_ref()
        .and_then(|lfs| lfs.size)
        .or(matching.size)
        .unwrap_or(0);

    let checksum = matching
        .lfs
        .as_ref()
        .and_then(|lfs| lfs.sha256.as_deref())
        .and_then(|digest_str| Checksum::parse(digest_str).ok());

    Ok(RemoteModelFile::new(
        repository.clone(),
        file.clone(),
        ByteLength::new(size),
        checksum,
    ))
}

fn extract_search_files(results: Vec<SearchModelItemResponse>) -> Vec<RemoteModelFile> {
    let mut files = Vec::new();
    for item in results {
        let Ok(repo_id) = ModelRepositoryId::parse(&item.id) else {
            continue;
        };
        let repository = ModelRepository::new(repo_id, ModelRevision::default());

        for sibling in item.siblings.unwrap_or_default() {
            let Ok(file_name) = ModelFileName::new(&sibling.rfilename) else {
                continue;
            };

            let size = sibling
                .lfs
                .as_ref()
                .and_then(|lfs| lfs.size)
                .or(sibling.size)
                .unwrap_or(0);

            let checksum = sibling
                .lfs
                .as_ref()
                .and_then(|lfs| lfs.sha256.as_deref())
                .and_then(|digest_str| Checksum::parse(digest_str).ok());

            files.push(RemoteModelFile::new(
                repository.clone(),
                file_name,
                ByteLength::new(size),
                checksum,
            ));
        }
    }
    files
}

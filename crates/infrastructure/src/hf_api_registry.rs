use application::errors::RegistryReadError;
use application::ports::outbound::RemoteModelRegistryPort;
use domain::{
    ByteLength, Checksum, ModelFileName, ModelRepository, ModelRepositoryId, ModelRevision,
    RemoteModelFile, SearchQuery,
};
use reqwest::Client;
use serde::Deserialize;

const DEFAULT_ENDPOINT: &str = "https://huggingface.co";

/// Hugging Face Hub catalog adapter for resolving files and searching models.
#[derive(Debug, Clone)]
pub struct HfApiRegistry {
    endpoint: String,
    client: Client,
    token: Option<String>,
}

impl HfApiRegistry {
    /// Builds a registry with default endpoint and environment credentials.
    pub fn new() -> Self {
        Self::from_env()
    }

    /// Resolves configuration from `HF_ENDPOINT` and `HF_TOKEN` environment variables.
    pub fn from_env() -> Self {
        let endpoint =
            std::env::var("HF_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        let token = std::env::var("HF_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty());
        Self {
            endpoint,
            client: Client::builder().build().unwrap_or_default(),
            token,
        }
    }

    /// Overrides the API endpoint URL.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Sets an optional authorization token.
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token.filter(|t| !t.trim().is_empty());
        self
    }

    /// Sets a custom reqwest client.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    fn auth_request(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.token {
            request.bearer_auth(token)
        } else {
            request
        }
    }
}

impl Default for HfApiRegistry {
    fn default() -> Self {
        Self::new()
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

impl RemoteModelRegistryPort for HfApiRegistry {
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError> {
        let url = format!(
            "{}/api/models/{}/{}/revision/{}?blobs=true",
            self.endpoint,
            repository.identifier().owner(),
            repository.identifier().name(),
            repository.revision().as_str()
        );

        let request = self.auth_request(self.client.get(&url));
        let response = request
            .send()
            .await
            .map_err(|err| RegistryReadError::Unreachable {
                repository: repository.to_string(),
                cause: err.to_string(),
            })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryReadError::FileNotFound {
                repository: repository.to_string(),
                file: file.to_string(),
            });
        }

        if !response.status().is_success() {
            return Err(RegistryReadError::Unreachable {
                repository: repository.to_string(),
                cause: format!("HTTP status {}", response.status()),
            });
        }

        let repo_info: RepoDetailsResponse =
            response
                .json()
                .await
                .map_err(|_| RegistryReadError::Malformed {
                    repository: repository.to_string(),
                })?;

        let matching_sibling = repo_info
            .siblings
            .as_deref()
            .unwrap_or_default()
            .iter()
            .find(|sibling| sibling.rfilename == file.as_str())
            .ok_or_else(|| RegistryReadError::FileNotFound {
                repository: repository.to_string(),
                file: file.to_string(),
            })?;

        let size = matching_sibling
            .lfs
            .as_ref()
            .and_then(|lfs| lfs.size)
            .or(matching_sibling.size)
            .unwrap_or(0);

        let checksum = matching_sibling
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

    async fn search_models(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError> {
        let url = format!("{}/api/models", self.endpoint);
        let request = self.auth_request(self.client.get(&url)).query(&[
            ("search", query.as_str()),
            ("full", "true"),
            ("limit", "10"),
        ]);

        let response = request
            .send()
            .await
            .map_err(|err| RegistryReadError::Unreachable {
                repository: query.as_str().to_string(),
                cause: err.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(RegistryReadError::Unreachable {
                repository: query.as_str().to_string(),
                cause: format!("HTTP status {}", response.status()),
            });
        }

        let results: Vec<SearchModelItemResponse> =
            response
                .json()
                .await
                .map_err(|_| RegistryReadError::Malformed {
                    repository: query.as_str().to_string(),
                })?;

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

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_points_to_huggingface() {
        let registry = HfApiRegistry::new();
        assert_eq!(registry.endpoint, DEFAULT_ENDPOINT);
    }

    #[test]
    fn custom_endpoint_overrides_default() {
        let registry = HfApiRegistry::new().with_endpoint("http://localhost:8080");
        assert_eq!(registry.endpoint, "http://localhost:8080");
    }
}

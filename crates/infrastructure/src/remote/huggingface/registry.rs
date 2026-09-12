use futures::future::join_all;
use localnar_application::{errors::RegistryReadError, ports::outbound::RemoteModelRegistryPort};
use localnar_domain::{
    ByteLength, Checksum, ContextLength, ModelFileName, ModelInfo, ModelProfile, ModelRepository,
    ModelRepositoryId, ModelTag, ModelWeightChoice, ParameterCount, RemoteModelFile, SearchQuery,
    Settings,
};
use reqwest::Client;
use serde::{Deserialize, de::DeserializeOwned};

use super::settings::HuggingFaceSettings;

pub trait HubTransport: Send + Sync {
    /// Performs a GET request against `path` and deserializes the JSON response.
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, RegistryReadError>;
}

const DEFAULT_ENDPOINT: &str = "https://huggingface.co";
const INSTALLABLE_EXTENSIONS: [&str; 2] = ["gguf", "safetensors"];

fn is_installable_format(file_name: &ModelFileName) -> bool {
    let name = file_name.as_str();
    let basename = name.rsplit('/').next().unwrap_or(name);
    if basename.to_ascii_lowercase().starts_with("mmproj") {
        return false;
    }
    name.rsplit_once('.')
        .is_some_and(|(_, extension)| INSTALLABLE_EXTENSIONS.contains(&extension))
}

#[derive(Debug, Deserialize)]
struct ModelRepoInfoResponse {
    tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct HfApiRegistry<Transport = ReqwestHubTransport> {
    transport: Transport,
}

impl<Transport: HubTransport> HfApiRegistry<Transport> {
    pub fn new(transport: Transport) -> Self {
        Self { transport }
    }

    async fn fetch_model_repo_info(
        &self,
        identifier: &ModelRepositoryId,
    ) -> Result<ModelRepoInfoResponse, RegistryReadError> {
        let path = format!("api/models/{}/{}", identifier.owner(), identifier.name());
        self.transport.get_json(&path).await
    }

    async fn offered_files(
        &self,
        repository: &ModelRepository,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError> {
        let path = format!(
            "api/models/{}/{}/revision/{}?blobs=true",
            repository.identifier().owner(),
            repository.identifier().name(),
            repository.revision().as_str()
        );

        let details: RepositoryRevisionResponse = self.transport.get_json(&path).await?;

        Ok(details
            .siblings
            .unwrap_or_default()
            .iter()
            .filter_map(|sibling| sibling.to_remote_file(repository))
            .filter(|file| is_installable_format(file.file()))
            .collect::<Vec<RemoteModelFile>>())
    }

    async fn describe(&self, entry: &CatalogEntryResponse) -> Option<ModelInfo> {
        let identifier = ModelRepositoryId::parse(&entry.id).ok()?;
        let repository = ModelRepository::at_default_revision(identifier);
        let offered = self.offered_files(&repository).await.ok()?;

        let tags = match self.fetch_model_repo_info(repository.identifier()).await {
            Ok(info) => info
                .tags
                .unwrap_or_default()
                .into_iter()
                .filter_map(|tag| ModelTag::new(tag).ok())
                .collect::<Vec<ModelTag>>(),
            Err(_) => Vec::new(),
        };

        let offered_with_tags = offered
            .into_iter()
            .map(|file| file.with_tags(tags.clone()))
            .collect::<Vec<RemoteModelFile>>();

        ModelWeightChoice::among(&offered_with_tags)
            .map(|weight| ModelInfo::describing(weight, entry.to_profile()))
    }
}

#[derive(Debug, Deserialize)]
struct RepositoryRevisionResponse {
    siblings: Option<Vec<RepositoryFileResponse>>,
}

#[derive(Debug, Deserialize)]
struct RepositoryFileResponse {
    rfilename: String,
    size: Option<u64>,
    lfs: Option<LfsInfo>,
}

impl RepositoryFileResponse {
    fn to_remote_file(&self, repository: &ModelRepository) -> Option<RemoteModelFile> {
        let file = ModelFileName::new(&self.rfilename).ok()?;
        Some(RemoteModelFile::new(
            repository.clone(),
            file,
            ByteLength::new(self.byte_size()),
            self.advertised_checksum(),
        ))
    }

    fn byte_size(&self) -> u64 {
        self.lfs
            .as_ref()
            .and_then(|lfs| lfs.size)
            .or(self.size)
            .unwrap_or_default()
    }

    fn advertised_checksum(&self) -> Option<Checksum> {
        self.lfs
            .as_ref()
            .and_then(|lfs| lfs.sha256.as_deref())
            .and_then(|digest| Checksum::parse(digest).ok())
    }
}

#[derive(Debug, Deserialize)]
struct LfsInfo {
    size: Option<u64>,
    sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntryResponse {
    id: String,
    #[serde(rename = "gguf")]
    model_metadata: Option<ModelMetadataResponse>,
}

impl CatalogEntryResponse {
    fn to_profile(&self) -> ModelProfile {
        self.model_metadata
            .as_ref()
            .map(ModelMetadataResponse::to_profile)
            .unwrap_or(ModelProfile::UNDISCLOSED)
    }
}

#[derive(Debug, Deserialize)]
struct ModelMetadataResponse {
    total: Option<u64>,
    context_length: Option<u32>,
}

impl ModelMetadataResponse {
    fn to_profile(&self) -> ModelProfile {
        ModelProfile::new(
            self.total.map(ParameterCount::new),
            self.context_length.map(ContextLength::new),
        )
    }
}

#[derive(Debug, Clone)]
pub struct ReqwestHubTransport {
    client: Client,
    endpoint: String,
    token: Option<String>,
}

impl ReqwestHubTransport {
    pub fn new(
        endpoint: impl Into<String>,
        token: Option<String>,
    ) -> Result<Self, RegistryReadError> {
        let client = Client::builder()
            .build()
            .map_err(|err| RegistryReadError::Unreachable {
                repository: String::new(),
                cause: format!("failed to build HTTP client: {err}"),
            })?;
        Ok(Self {
            endpoint: endpoint.into(),
            client,
            token: token.filter(|t| !t.trim().is_empty()),
        })
    }

    pub fn from_env() -> Result<Self, RegistryReadError> {
        let endpoint =
            std::env::var("HF_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        let token = std::env::var("HF_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty());
        Self::new(endpoint, token)
    }

    /// Resolves configuration from persisted settings, falling back to the
    /// environment and built-in defaults for any value left unset.
    pub fn from_settings(settings: &Settings) -> Result<Self, RegistryReadError> {
        let hf = HuggingFaceSettings::from_settings(settings);
        let endpoint = hf
            .endpoint()
            .map(str::to_owned)
            .or_else(|| std::env::var("HF_ENDPOINT").ok())
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
        let token = hf
            .api_token()
            .map(str::to_owned)
            .or_else(|| std::env::var("HF_TOKEN").ok());
        Self::new(endpoint, token)
    }

    /// Returns the base endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Formats the complete URL for a given relative API path.
    pub fn url_for(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.endpoint.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

impl HubTransport for ReqwestHubTransport {
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, RegistryReadError> {
        let url = self.url_for(path);
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

impl HfApiRegistry<ReqwestHubTransport> {
    pub fn from_env() -> Result<Self, RegistryReadError> {
        Ok(Self::new(ReqwestHubTransport::from_env()?))
    }

    /// Builds a registry from persisted settings with env/default fallback.
    pub fn from_settings(settings: &Settings) -> Result<Self, RegistryReadError> {
        Ok(Self::new(ReqwestHubTransport::from_settings(settings)?))
    }
}

impl<Transport: HubTransport> RemoteModelRegistryPort for HfApiRegistry<Transport> {
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError> {
        let offered = self
            .offered_files(repository)
            .await
            .map_err(|failure| Self::as_file_failure(failure, repository, file))?;

        let tags = match self.fetch_model_repo_info(repository.identifier()).await {
            Ok(info) => info
                .tags
                .unwrap_or_default()
                .into_iter()
                .filter_map(|tag| ModelTag::new(tag).ok())
                .collect::<Vec<ModelTag>>(),
            Err(_) => Vec::new(),
        };

        offered
            .into_iter()
            .map(|offer| offer.with_tags(tags.clone()))
            .find(|offer| offer.file() == file)
            .ok_or_else(|| RegistryReadError::FileNotFound {
                repository: repository.to_string(),
                file: file.to_string(),
            })
    }

    async fn search_models(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<ModelInfo>, RegistryReadError> {
        let entries: Vec<CatalogEntryResponse> =
            self.transport.get_json(&Self::search_path(query)).await?;

        let described = join_all(entries.iter().map(|entry| self.describe(entry))).await;

        Ok(described.into_iter().flatten().collect())
    }
}

impl<Transport: HubTransport> HfApiRegistry<Transport> {
    const SEARCH_RESULT_LIMIT: usize = 10;
    const GGUF_EXPANSION: &'static str = "expand%5B%5D=gguf";

    fn search_path(query: &SearchQuery) -> String {
        format!(
            "api/models?search={}&limit={}&{}",
            query.as_str(),
            Self::SEARCH_RESULT_LIMIT,
            Self::GGUF_EXPANSION
        )
    }

    fn as_file_failure(
        failure: RegistryReadError,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> RegistryReadError {
        match failure {
            RegistryReadError::FileNotFound { .. } => RegistryReadError::FileNotFound {
                repository: repository.to_string(),
                file: file.to_string(),
            },
            other => other,
        }
    }
}

impl<Transport: HubTransport + Default> Default for HfApiRegistry<Transport> {
    fn default() -> Self {
        Self::new(<Transport as Default>::default())
    }
}

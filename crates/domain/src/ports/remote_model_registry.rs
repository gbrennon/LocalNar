use async_trait::async_trait;

use crate::model_file_name::ModelFileName;
use crate::model_repository::ModelRepository;
use crate::ports::registry_read_error::RegistryReadError;
use crate::remote_model_file::RemoteModelFile;

/// Contract for reading what an upstream repository offers.
///
/// Implementations translate a Hugging Face Hub (or equivalent) listing into
/// domain `RemoteModelFile` values; the domain layer never performs network I/O
/// and only depends on this contract.
#[async_trait]
pub trait RemoteModelRegistry: Send + Sync {
    /// Resolves metadata for one requested file inside a repository.
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError>;

    /// Lists every candidate file the repository currently publishes.
    ///
    /// Adapters that cannot enumerate may return
    /// `RegistryReadError::EnumerationUnsupported`.
    async fn list_repository_files(
        &self,
        repository: &ModelRepository,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError> {
        let _ = repository;
        Err(RegistryReadError::EnumerationUnsupported)
    }
}

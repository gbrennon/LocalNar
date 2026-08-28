//! Pure domain layer for the local model downloader automation.
//!
//! This crate owns everything about a model that is true regardless of how it
//! is retrieved: what identifiers and files exist, what "correctly installed"
//! means, and the contracts (ports) that infrastructure adapters must satisfy.
//! There is deliberately no I/O here; the domain only reasons about values and
//! states, and connecting to Hugging Face, hashing files, or writing to disk is
//! delegated to the `infrastructure` crate through the traits in [`ports`].

mod byte_length;
mod domain_error;
mod model_artifact;
mod model_file_name;
mod model_id;
mod model_plan;
mod model_repository;
mod model_repository_id;
mod model_revision;
mod model_spec;
mod model_state;
mod remote_model_file;
mod sha256;

pub mod ports;

pub use byte_length::ByteLength;
pub use domain_error::DomainError;
pub use model_artifact::ModelArtifact;
pub use model_file_name::ModelFileName;
pub use model_id::ModelId;
pub use model_plan::ModelPlan;
pub use model_repository::ModelRepository;
pub use model_repository_id::ModelRepositoryId;
pub use model_revision::ModelRevision;
pub use model_spec::ModelSpec;
pub use model_state::ModelState;
pub use ports::{
    DownloadError, LibraryError, ModelDownloader, ModelLibrary, RegistryReadError,
    RemoteModelRegistry,
};
pub use remote_model_file::RemoteModelFile;
pub use sha256::Sha256;

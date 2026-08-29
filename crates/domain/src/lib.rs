//! Pure domain layer for the local model downloader automation.
//!
//! This crate owns everything about a model that is true regardless of how it
//! is retrieved: what identifiers and files exist, what "correctly installed"
//! means, and the value types that travel across the application boundary.
//! There is deliberately no I/O here; the domain only reasons about values and
//! states. Connecting to Hugging Face, hashing files, or writing to disk is
//! delegated to the infrastructure crate through the outbound ports defined in
//! the application layer.

mod byte_length;
mod checksum;
mod domain_error;
mod installed_model;
mod model_artifact;
mod model_file_name;
mod model_repository;
mod model_repository_id;
mod model_revision;
mod model_spec;
mod model_state;
mod remote_model_file;
mod search_query;

pub use byte_length::ByteLength;
pub use checksum::Checksum;
pub use domain_error::DomainError;
pub use installed_model::InstalledModel;
pub use model_artifact::ModelArtifact;
pub use model_file_name::ModelFileName;
pub use model_repository::ModelRepository;
pub use model_repository_id::ModelRepositoryId;
pub use model_revision::ModelRevision;
pub use model_spec::ModelSpec;
pub use model_state::ModelState;
pub use remote_model_file::RemoteModelFile;
pub use search_query::SearchQuery;

//! The model domain: what a model is, what makes one valid, and how one is chosen.
//!
//! Types are grouped by what they are -- values, entities, policies, errors --
//! and re-exported flat, so callers name a type without naming its stereotype.

mod entities;
mod errors;
mod policies;
mod specifications;
mod value_objects;

pub use entities::InstalledModel;
pub use errors::DomainError;
pub use policies::ModelWeightChoice;
pub use specifications::{MultiPartShard, Specification, WholeWeightFile};
pub use value_objects::ByteLength;
pub use value_objects::Checksum;
pub use value_objects::ContextLength;
pub use value_objects::ModelArtifact;
pub use value_objects::ModelFileName;
pub use value_objects::ModelInfo;
pub use value_objects::ModelProfile;
pub use value_objects::ModelRepository;
pub use value_objects::ModelRepositoryId;
pub use value_objects::ModelRevision;
pub use value_objects::ModelSpec;
pub use value_objects::ModelState;
pub use value_objects::ModelTag;
pub use value_objects::ParameterCount;
pub use value_objects::Quantization;
pub use value_objects::RemoteModelFile;
pub use value_objects::SearchQuery;

#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::ModelEvictionPort};
use localnar_domain::{ModelSpec, RemovedModel};

/// A library whose replicas cannot be discarded.
pub struct FakeRefusingEviction;

impl FakeRefusingEviction {
    /// The error every eviction produces.
    pub fn error() -> LibraryError {
        LibraryError::Unwritable {
            model: "qwen3-8b".to_string(),
            cause: "read-only file system".to_string(),
        }
    }
}

impl ModelEvictionPort for FakeRefusingEviction {
    async fn evict(&self, _model: &ModelSpec) -> Result<RemovedModel, LibraryError> {
        Err(Self::error())
    }
}

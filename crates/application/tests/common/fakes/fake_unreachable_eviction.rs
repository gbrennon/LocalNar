#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::ModelEvictionPort};
use localnar_domain::{ModelSpec, RemovedModel};

/// An eviction no scenario using it is allowed to reach.
///
/// It stands for the destructive authority a removal must not exercise once the
/// library has already answered that it holds no such replica.
pub struct FakeUnreachableEviction;

impl ModelEvictionPort for FakeUnreachableEviction {
    async fn evict(&self, _model: &ModelSpec) -> Result<RemovedModel, LibraryError> {
        panic!("a model the library does not hold must never be evicted")
    }
}

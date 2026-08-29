//! Adapters that persist model artifacts to a local store.
//!
//! Each sub-module corresponds to a concrete persistence technology. The
//! current implementation stores models on the local filesystem.

pub mod disk;

pub use disk::model_library::DiskModelLibrary;

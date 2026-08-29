//! Typed failures of the application boundary.
//!
//! Every port carries its own error so a failing boundary is never flattened
//! into an opaque message; adapters translate dependency failures into these.

pub mod install_model_error;
pub mod library_error;
pub mod model_download_error;
pub mod registry_read_error;
pub mod search_models_error;

pub use install_model_error::InstallModelError;
pub use library_error::LibraryError;
pub use model_download_error::ModelDownloadError;
pub use registry_read_error::RegistryReadError;
pub use search_models_error::SearchModelsError;

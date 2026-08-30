//! Typed failures of the application boundary.
//!
//! Every port carries its own error so a failing boundary is never flattened
//! into an opaque message; adapters translate dependency failures into these.

pub mod inspect_model_error;
pub mod install_model_error;
pub mod library_error;
pub mod list_installed_models_error;
pub mod model_download_error;
pub mod prune_library_error;
pub mod registry_read_error;
pub mod remove_model_error;
pub mod search_models_error;
pub mod verify_model_error;

pub use inspect_model_error::InspectModelError;
pub use install_model_error::InstallModelError;
pub use library_error::LibraryError;
pub use list_installed_models_error::ListInstalledModelsError;
pub use model_download_error::ModelDownloadError;
pub use prune_library_error::PruneLibraryError;
pub use registry_read_error::RegistryReadError;
pub use remove_model_error::RemoveModelError;
pub use search_models_error::SearchModelsError;
pub use verify_model_error::VerifyModelError;

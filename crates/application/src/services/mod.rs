//! Implementations of the inbound ports.
//!
//! A service composes outbound ports to fulfil one inbound contract; it holds
//! orchestration only, never I/O. Each service takes exactly the outbound
//! ports its own use case needs.

mod inspect_model_service;
mod install_model_service;
mod list_installed_models_service;
mod prune_library_service;
mod remove_model_service;
mod search_models_service;
mod verify_model_service;

pub use inspect_model_service::InspectModelService;
pub use install_model_service::InstallModelService;
pub use list_installed_models_service::ListInstalledModelsService;
pub use prune_library_service::PruneLibraryService;
pub use remove_model_service::RemoveModelService;
pub use search_models_service::SearchModelsService;
pub use verify_model_service::VerifyModelService;

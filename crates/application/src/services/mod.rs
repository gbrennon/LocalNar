//! Implementations of the inbound ports.
//!
//! A service composes outbound ports to fulfil one inbound contract; it holds
//! orchestration only, never I/O. Each service takes exactly the outbound
//! ports its own use case needs.

mod install_model_service;
mod search_models_service;

pub use install_model_service::InstallModelService;
pub use search_models_service::SearchModelsService;

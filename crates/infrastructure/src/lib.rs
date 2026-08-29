//! The infrastructure layer for bare-ai-server.
//!
//! Provides concrete outbound port adapters connecting the application layer
//! to external services including Hugging Face Hub and local filesystem storage.

#![allow(async_fn_in_trait)]

pub mod disk_model_library;
pub mod hf_api_registry;
pub mod hf_hub_downloader;

pub use disk_model_library::DiskModelLibrary;
pub use hf_api_registry::{HfApiRegistry, HubTransport, ReqwestHubTransport};
pub use hf_hub_downloader::{HfHubDownloader, HfHubTokioTransport, HubDownloadTransport};

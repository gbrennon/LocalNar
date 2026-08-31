//! Adapters that fetch model metadata and files from remote providers.
//!
//! Each sub-module corresponds to a concrete remote provider. The current
//! implementation targets Hugging Face Hub.

pub mod huggingface;

pub use huggingface::{
    downloader::{HfHubDownloader, HfHubTokioTransport, HubDownloadTransport},
    registry::{HfApiRegistry, HubTransport, ReqwestHubTransport},
};

//! The infrastructure layer for localnar.

//! Provides concrete outbound port adapters connecting the application layer
//! to external services. Adapters are grouped by implementation topic:
//! remote providers (`remote`) and local persistence (`persistence`).

#![allow(async_fn_in_trait)]

pub mod adapters;
pub mod persistence;
pub mod remote;

pub use persistence::DiskModelLibrary;
pub use remote::{
    HfApiRegistry, HfHubDownloader, HfHubTokioTransport, HubDownloadTransport, HubTransport,
    ReqwestHubTransport,
};

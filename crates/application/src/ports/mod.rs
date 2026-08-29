//! Inbound and outbound contracts of the application layer.
//!
//! Driving adapters depend on the inbound ports; driven adapters implement the
//! outbound ones. The application layer owns both sides of the boundary.

pub mod inbound;
pub mod outbound;

pub use inbound::InstallModelPort;
pub use outbound::{
    DownloadProgress, DownloadProgressPort, ModelDownloaderPort, ModelLibraryPort,
    RemoteModelRegistryPort,
};

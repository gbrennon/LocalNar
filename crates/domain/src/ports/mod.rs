//! Contracts that infrastructure adapters implement for the downloader.
//!
//! Each contract is defined here in the domain so the inner layer owns the
//! abstraction; concrete `hf-hub`, filesystem, and hashing adapters live in the
//! infrastructure crate and satisfy these traits.

mod download_error;
mod library_error;
mod model_downloader;
mod model_library;
mod registry_read_error;
mod remote_model_registry;

pub use download_error::DownloadError;
pub use library_error::LibraryError;
pub use model_downloader::ModelDownloader;
pub use model_library::ModelLibrary;
pub use registry_read_error::RegistryReadError;
pub use remote_model_registry::RemoteModelRegistry;

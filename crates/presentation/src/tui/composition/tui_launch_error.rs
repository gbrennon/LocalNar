use std::{error::Error, fmt};

use localnar_application::errors::{RegistryReadError, SettingsStoreError};

/// Reason a TUI launch could not reach its run loop.
#[derive(Debug)]
pub enum TuiLaunchError {
    /// The terminal could not be claimed or driven.
    Terminal(std::io::Error),

    /// The remote catalog could not be configured from the environment.
    Catalog(RegistryReadError),

    /// The persisted settings could not be read at startup.
    Settings(SettingsStoreError),
}

impl fmt::Display for TuiLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Terminal(cause) => write!(formatter, "the terminal could not be driven: {cause}"),
            Self::Catalog(cause) => write!(
                formatter,
                "the remote catalog could not be configured: {cause}"
            ),
            Self::Settings(cause) => {
                write!(formatter, "the settings could not be loaded: {cause}")
            }
        }
    }
}

impl Error for TuiLaunchError {}

impl From<std::io::Error> for TuiLaunchError {
    fn from(cause: std::io::Error) -> Self {
        Self::Terminal(cause)
    }
}

impl From<RegistryReadError> for TuiLaunchError {
    fn from(cause: RegistryReadError) -> Self {
        Self::Catalog(cause)
    }
}

impl From<SettingsStoreError> for TuiLaunchError {
    fn from(cause: SettingsStoreError) -> Self {
        Self::Settings(cause)
    }
}

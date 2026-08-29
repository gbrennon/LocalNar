//! Application layer for the local model downloader automation.
//!
//! This crate owns the boundary of the system: the inbound ports a driving
//! adapter calls, the outbound ports driven adapters implement, their typed
//! errors, and the services that orchestrate them. It reads the installed
//! state of a model, decides the next step from the domain state machine, and
//! drives the registry, downloader, and library to reach it. There is no I/O
//! here; every effect belongs to an injected port implementation.

#![allow(async_fn_in_trait)]

pub mod errors;
pub mod ports;
pub mod services;

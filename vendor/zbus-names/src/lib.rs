//! Compatibility layer for the standalone `zbus_names` crate.
//!
//! `zbus_names` is unmaintained (RUSTSEC-2026-0371): since zbus 6 its types live in
//! `zbus::names`. This crate keeps the old paths compiling on top of that maintained code.

pub use zbus::names::*;
pub use zbus::Error;

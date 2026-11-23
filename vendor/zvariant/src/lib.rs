//! Compatibility layer for the standalone `zvariant` crate.
//!
//! `zvariant` is unmaintained (RUSTSEC-2026-0370): since zbus 6 its wire format lives in
//! `zbus::wire`. This crate keeps the old paths compiling on top of that maintained code.

pub use zbus::wire::*;
pub use zbus::wire::{OwnedVariant as OwnedValue, Variant as Value};

pub mod as_value {
    pub use zbus::as_variant::*;
    pub use zbus::as_variant::{Deserialize as DeserializeValue, Serialize as SerializeValue};
}

pub mod dbus {
    pub use zbus::wire::dbus::*;
}

pub mod serialized {
    pub use zbus::wire::serialized::*;
}

pub mod signature {
    pub use zbus::wire::signature::*;
}

pub mod export {
    pub use zbus::wire::export::*;
}

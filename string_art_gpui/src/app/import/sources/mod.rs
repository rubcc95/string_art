//! Platform I/O for the import feature.
//!
//! These modules are the GUI edge of the feature: they open the platform file
//! picker, read the bytes and wire files dropped from the OS or the browser.
//! The decoding itself lives in `crate::import`.

#[cfg(not(target_family = "wasm"))]
mod desktop;
#[cfg(not(target_family = "wasm"))]
pub use desktop::{attach_drop, init, pick};

#[cfg(target_family = "wasm")]
mod web;
#[cfg(target_family = "wasm")]
pub use web::{attach_drop, init, pick};

//! Custom-drawn window chrome (client-side decorations).
//!
//! The application owns its title bar on every desktop platform. Platform
//! selection happens at compile time: exactly one of the platform modules below
//! is compiled and re-exported, and each one provides the same small API:
//!
//! - [`custom_chrome`] — whether the application has to draw the window chrome.
//! - [`left_controls`] / [`right_controls`] — window control buttons per side.
//! - [`resize_handles`] — edge strips that start a resize drag, when needed.
//! - [`LEFT_INSET`] / [`RIGHT_INSET`] — title bar content insets.
//!
//! The only runtime decisions left are the ones that depend on the environment
//! at startup: server versus client decorations, tiling and maximized state.

use gpui::{Pixels, px};

/// Height of the application title bar.
pub const HEIGHT: Pixels = px(44.);

/// Window controls shared by the platforms that draw them.
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "windows"))]
mod controls;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

/// Every other target (the web build among them) renders the title bar as a
/// plain toolbar, without window chrome.
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "windows",
    target_os = "macos"
)))]
mod web;
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "windows",
    target_os = "macos"
)))]
pub use web::*;

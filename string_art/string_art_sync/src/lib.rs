//! Distribution of the computation work across threads.
//!
//! The core knows nothing about threads, wasm, or interfaces: it only asks
//! for a backend that implements [`Parallel`].
//!
//! # Implementations
//!
//! - [`Serial`]: a single thread. Always available.
//! - [`RayonExec`]: rayon pool. Native and, with `wasm-bindgen-rayon`, web.
//! - `string_art_gpui::GpuiExec`: gpui multithreaded pool (web).
//!
//! # Two ways to distribute
//!
//! [`Parallel::for_each`] is blocking: when it returns, the work is done.
//! It is the cheap form and the one the core uses.
//!
//! [`Parallel::map`] returns a future. It exists because some backends cannot
//! block (the main thread of a browser cannot wait), and its default
//! implementation reuses `for_each`, so it only has to be overridden if the
//! backend needs it.

/// Conditional `Send`.
///
/// With the `threads` feature enabled it is `Send` for real: some backends
/// distribute the work across threads and the type has to be able to cross.
/// When disabled, there is no backend that crosses threads, so any type works
/// —`Rc` included— and the trait is left empty.
///
/// The condition is **which backends are in the build**, not the architecture:
/// in wasm with `gpui_web` there are threads, so tying it to `target_arch = wasm32`
/// (as was done before) allowed crossing threads with types that are not.
#[cfg(feature = "multithreaded")]
pub trait CondSend: Send {}
#[cfg(feature = "multithreaded")]
impl<T: Send + ?Sized> CondSend for T {}

/// See [`CondSend`].
#[cfg(not(feature = "multithreaded"))]
pub trait CondSend {}
#[cfg(not(feature = "multithreaded"))]
impl<T: ?Sized> CondSend for T {}

/// Conditional `Sync`. Same rule as [`CondSend`].
#[cfg(feature = "multithreaded")]
pub trait CondSync: Sync {}
#[cfg(feature = "multithreaded")]
impl<T: Sync + ?Sized> CondSync for T {}

/// See [`CondSync`].
#[cfg(not(feature = "multithreaded"))]
pub trait CondSync {}
#[cfg(not(feature = "multithreaded"))]
impl<T: ?Sized> CondSync for T {}

/// Distribution of work across threads.
pub trait Executor {
    /// Threads that this backend makes available.
    ///
    /// It is used to chunk the board and to size the buffers, so it must be
    /// stable for the whole lifetime of the backend.
    fn workers(&self) -> usize;

    /// Runs `f(i, element)` on each element of `bufs`.
    ///
    /// It may block: it is only valid to call it from a thread that can
    /// block. Never from the main thread of a browser.
    #[cfg(not(feature = "multithreaded"))]
    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        F: Fn(usize, &mut T);
    #[cfg(feature = "multithreaded")]
    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        T: Send,
        F: Fn(usize, &mut T) + Send + Sync;
}

//! Reparto del trabajo del cálculo entre hilos.
//!
//! El core no sabe nada de hilos, de wasm ni de interfaces: solo pide un
//! backend que implemente [`Parallel`].
//!
//! # Implementaciones
//!
//! - [`Serial`]: un solo hilo. Siempre disponible.
//! - [`RayonExec`]: pool de rayon. Nativo y, con `wasm-bindgen-rayon`, web.
//! - `string_art_gpui::GpuiExec`: pool multihilo de gpui (web).
//!
//! # Dos formas de repartir
//!
//! [`Parallel::for_each`] es bloqueante: cuando vuelve, el trabajo está hecho.
//! Es la forma barata y la que usa el core.
//!
//! [`Parallel::map`] devuelve un future. Existe porque algunos backends no
//! pueden bloquear (el hilo principal de un navegador no puede esperar), y su
//! implementación por defecto reutiliza `for_each`, así que solo hay que
//! reescribirla si el backend lo necesita.

/// `Send` condicional.
///
/// Con la feature `threads` encendida es `Send` de verdad: hay backends que
/// reparten el trabajo entre hilos y el tipo tiene que poder cruzar. Apagada no
/// hay ningún backend que cruce hilos, así que vale cualquier tipo —`Rc`
/// incluido— y el trait queda vacío.
///
/// La condición es **qué backends entran en el build**, no la arquitectura: en
/// wasm con `gpui_web` sí hay hilos, así que atarla a `target_arch = wasm32`
/// (como se hacía antes) dejaba cruzar hilos con tipos que no lo son.
#[cfg(feature = "multithreaded")]
pub trait CondSend: Send {}
#[cfg(feature = "multithreaded")]
impl<T: Send + ?Sized> CondSend for T {}

/// Ver [`CondSend`].
#[cfg(not(feature = "multithreaded"))]
pub trait CondSend {}
#[cfg(not(feature = "multithreaded"))]
impl<T: ?Sized> CondSend for T {}

/// `Sync` condicional. Misma regla que [`CondSend`].
#[cfg(feature = "multithreaded")]
pub trait CondSync: Sync {}
#[cfg(feature = "multithreaded")]
impl<T: Sync + ?Sized> CondSync for T {}

/// Ver [`CondSync`].
#[cfg(not(feature = "multithreaded"))]
pub trait CondSync {}
#[cfg(not(feature = "multithreaded"))]
impl<T: ?Sized> CondSync for T {}

/// Reparto del trabajo entre hilos.
pub trait Executor {
    /// Hilos que este backend pone a disposición.
    ///
    /// Se usa para trocear el tablero y para dimensionar los buffers, así que
    /// debe ser estable durante toda la vida del backend.
    fn workers(&self) -> usize;

    /// Ejecuta `f(i, elemento)` sobre cada elemento de `bufs`.
    ///
    /// Puede bloquear: solo es válido llamarlo desde un hilo que pueda
    /// bloquearse. Nunca desde el hilo principal de un navegador.
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

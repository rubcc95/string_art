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

use core::future::Future;

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
#[cfg(feature = "threads")]
pub trait CondSend: Send {}
#[cfg(feature = "threads")]
impl<T: Send + ?Sized> CondSend for T {}

/// Ver [`CondSend`].
#[cfg(not(feature = "threads"))]
pub trait CondSend {}
#[cfg(not(feature = "threads"))]
impl<T: ?Sized> CondSend for T {}

/// `Sync` condicional. Misma regla que [`CondSend`].
#[cfg(feature = "threads")]
pub trait CondSync: Sync {}
#[cfg(feature = "threads")]
impl<T: Sync + ?Sized> CondSync for T {}

/// Ver [`CondSync`].
#[cfg(not(feature = "threads"))]
pub trait CondSync {}
#[cfg(not(feature = "threads"))]
impl<T: ?Sized> CondSync for T {}

/// Reparto del trabajo entre hilos.
pub trait Parallel: Clone {
    /// Hilos que este backend pone a disposición.
    ///
    /// Se usa para trocear el tablero y para dimensionar los buffers, así que
    /// debe ser estable durante toda la vida del backend.
    fn workers(&self) -> usize;

    /// Ejecuta `f(i, elemento)` sobre cada elemento de `bufs`.
    ///
    /// Puede bloquear: solo es válido llamarlo desde un hilo que pueda
    /// bloquearse. Nunca desde el hilo principal de un navegador.
    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        T: CondSend,
        F: Fn(usize, &mut T) + CondSend + CondSync;

    /// Igual que [`Parallel::for_each`], pero sin bloquear al que llama.
    ///
    /// Por defecto delega en `for_each`: los backends que pueden bloquear no
    /// tienen que implementarlo.
    fn map<R, F>(&self, f: F) -> impl Future<Output = Vec<R>>
    where
        F: Fn(usize) -> R + CondSend + CondSync + 'static,
        R: CondSend + 'static,
    {
        async move {
            let mut slots: Vec<Option<R>> = (0..self.workers()).map(|_| None).collect();
            self.for_each(&mut slots, |index, slot| *slot = Some(f(index)));
            slots.into_iter().flatten().collect()
        }
    }
}

/// Un solo hilo.
///
/// Es el resultado de referencia de los tests y el fallback para plataformas
/// sin hilos.
#[derive(Clone, Copy, Debug, Default)]
pub struct Serial;

impl Parallel for Serial {
    fn workers(&self) -> usize {
        1
    }

    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        T: CondSend,
        F: Fn(usize, &mut T) + CondSend + CondSync,
    {
        for (index, buf) in bufs.iter_mut().enumerate() {
            f(index, buf);
        }
    }
}

#[cfg(all(feature = "threads", not(target_arch = "wasm32")))]
pub use from_rayon::RayonExec;

#[cfg(all(feature = "threads", not(target_arch = "wasm32")))]
mod from_rayon {
    use super::*;
    use rayon::prelude::*;
    use std::sync::Arc;

    /// Pool de rayon propio, con un número de hilos fijo.
    ///
    /// Tener el pool aquí (y no el global de rayon) permite elegir el número
    /// de hilos en runtime y medirlo sin tocar variables de entorno.
    #[derive(Clone)]
    pub struct RayonExec {
        pool: Arc<rayon::ThreadPool>,
        threads: usize,
    }

    impl RayonExec {
        /// Pool de `threads` hilos. `0` se trata como `1`.
        pub fn new(threads: usize) -> Self {
            let threads = threads.max(1);
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("no se pudo crear el pool de rayon");
            Self {
                pool: Arc::new(pool),
                threads,
            }
        }

        /// Un hilo por CPU disponible.
        pub fn auto() -> Self {
            Self::new(num_cpus::get())
        }
    }

    impl Default for RayonExec {
        fn default() -> Self {
            Self::auto()
        }
    }

    impl Parallel for RayonExec {
        fn workers(&self) -> usize {
            self.threads
        }

        fn for_each<T, F>(&self, bufs: &mut [T], f: F)
        where
            T: CondSend,
            F: Fn(usize, &mut T) + CondSend + CondSync,
        {
            self.pool.install(|| {
                bufs.par_iter_mut()
                    .enumerate()
                    .for_each(|(index, buf)| f(index, buf));
            });
        }
    }
}

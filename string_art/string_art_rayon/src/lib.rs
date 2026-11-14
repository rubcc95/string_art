use std::sync::Arc;

use rayon::iter::{
    IndexedParallelIterator as _, IntoParallelRefMutIterator as _, ParallelIterator as _,
};
use string_art_sync::Executor;

/// Pool de rayon propio, con un número de hilos fijo.
///
/// Tener el pool aquí (y no el global de rayon) permite elegir el número
/// de hilos en runtime y medirlo sin tocar variables de entorno. Además el
/// número de hilos queda fijado en el constructor, como exige [`Executor`]:
/// de él dependen el troceado del tablero y el tamaño de los buffers.
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
    ///
    /// Mismo criterio que el pool por defecto de rayon, pero sin pasar por
    /// `RAYON_NUM_THREADS` ni tocar el pool global.
    pub fn auto() -> Self {
        let threads = std::thread::available_parallelism()
            .map(|threads| threads.get())
            .unwrap_or(1);
        Self::new(threads)
    }
}

impl Default for RayonExec {
    fn default() -> Self {
        Self::auto()
    }
}

impl Executor for RayonExec {
    fn workers(&self) -> usize {
        self.threads
    }

    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        T: Send,
        F: Fn(usize, &mut T) + Send + Sync,
    {
        self.pool.install(|| {
            bufs.par_iter_mut()
                .enumerate()
                .for_each(|(index, buf)| f(index, buf));
        });
    }
}

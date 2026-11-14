use std::sync::Arc;

use rayon::iter::{
    IndexedParallelIterator as _, IntoParallelRefMutIterator as _, ParallelIterator as _,
};
use string_art_sync::Executor;

/// Its own rayon pool, with a fixed number of threads.
///
/// Having the pool here (and not rayon's global one) lets you choose the number
/// of threads at runtime and measure it without touching environment variables.
/// Besides, the number of threads is fixed in the constructor, as [`Executor`]
/// requires: board chunking and buffer sizes depend on it.
#[derive(Clone)]
pub struct RayonExec {
    pool: Arc<rayon::ThreadPool>,
    threads: usize,
}

impl RayonExec {
    /// Pool with `threads` threads. `0` is treated as `1`.
    pub fn new(threads: usize) -> Self {
        let threads = threads.max(1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("could not create the rayon pool");
        Self {
            pool: Arc::new(pool),
            threads,
        }
    }

    /// One thread per available CPU.
    ///
    /// Same criterion as rayon's default pool, but without going through
    /// `RAYON_NUM_THREADS` or touching the global pool.
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

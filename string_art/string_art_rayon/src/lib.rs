use std::sync::Arc;

use rayon::iter::{
    IndexedParallelIterator as _, IntoParallelRefMutIterator as _, ParallelIterator as _,
};
use string_art_executor::{Executor, Work};
/// Its own rayon pool, with a fixed number of threads.
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

impl<T, W> Executor<T, W> for RayonExec
where
    T: Send,
    W: Work<T> + Send + Sync,
{
    fn workers(&self) -> usize {
        self.threads
    }

    fn for_each(&self, buffers: &mut [T], work: W) {
        self.pool.install(|| {
            buffers
                .par_iter_mut()
                .enumerate()
                .for_each(|(index, buffer)| work.run(index, buffer));
        });
    }
}

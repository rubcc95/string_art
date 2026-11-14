use gpui::BackgroundExecutor;
use std::{
    cell::Cell,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use string_art::Executor;

/// [`Parallel`] backend on top of the gpui background pool.
///
/// `for_each` cannot run on the main thread: it distributes the work across the
/// pool and the caller waits, with its pool thread occupied. On
/// wasm, one more thread is also left free for the thread serving the page.
#[derive(Clone)]
pub struct GpuiExec {
    exec: Arc<BackgroundExecutor>,
    workers: usize,
}

impl GpuiExec {
    pub fn new(exec: BackgroundExecutor) -> Self {
        // `workers` is set here: the trait requires it to be stable for the whole
        // life of the backend, and on wasm every query crosses into JS.
        //
        // On wasm, `BackgroundExecutor::num_cpus` delegates to `num_cpus::get()`,
        // which always returns 1 because it cannot read the system. The real value
        // is in `navigator.hardwareConcurrency`, which exposes
        // `wasm_thread::available_parallelism()`.
        //
        // On wasm, one more thread is left free for the thread serving the page.
        #[cfg(target_family = "wasm")]
        let workers = wasm_thread::available_parallelism()
            .map(|cpus| cpus.get().saturating_sub(1).max(1))
            .unwrap_or(1);
        #[cfg(not(target_family = "wasm"))]
        let workers = exec.num_cpus().max(1);

        Self {
            exec: Arc::new(exec),
            workers,
        }
    }
}

impl Executor for GpuiExec {
    /// Computation threads. One per CPU on native; on wasm one more is left free
    /// for the thread serving the page.
    ///
    /// The caller of `for_each` occupies a pool thread while it waits, so
    /// it has to be discounted when distributing. It is computed once in [`Self::new`]
    /// because the trait requires it to be stable.
    fn workers(&self) -> usize {
        self.workers
    }

    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        T: Send,
        F: Fn(usize, &mut T) + Send + Sync,
    {
        /// Completion counter for a `for_each`.
        ///
        /// `done` counts the tasks as they finish and `waiter` is the thread that
        /// waits: one `fetch_add` per task and one `load` per loop of the waiter, with no
        /// allocations or locks.
        ///
        /// The waiter sleeps with `park`; on wasm std's `Parker` waits with
        /// `memory.atomic.wait32`, so it really sleeps too.
        struct Finish {
            done: AtomicUsize,
            total: usize,
            waiter: std::thread::Thread,
        }

        impl Finish {
            fn new(total: usize) -> Self {
                Self {
                    done: AtomicUsize::new(0),
                    total,
                    waiter: std::thread::current(),
                }
            }

            /// Called by each task when it finishes, and by the guard for the ones that
            /// never got to be launched.
            ///
            /// `unpark` leaves the flag set if the waiter is not asleep yet,
            /// so the notification is not lost.
            fn signal(&self) {
                if self.done.fetch_add(1, Ordering::AcqRel) + 1 == self.total {
                    self.waiter.unpark();
                }
            }

            /// Called by the distributor to wait until all tasks finish.
            fn wait(&self) {
                while self.done.load(Ordering::Acquire) < self.total {
                    std::thread::park();
                }
            }
        }

        /// Adds to the counter when the task finishes, even if it panics or
        /// is cancelled. Without this, `for_each` would wait forever.
        ///
        /// It is created before `spawn` and travels inside the future, so it also
        /// adds if the task is cancelled before it gets to run.
        struct TaskGuard<'a>(&'a Finish);

        impl Drop for TaskGuard<'_> {
            fn drop(&mut self) {
                self.0.signal()
            }
        }

        /// Waits for the tasks when leaving `for_each`, also during an
        /// unwind.
        ///
        /// If the distribution is cut short (for example, a panicking `spawn`), the
        /// tasks that never got to be launched count as finished: otherwise, `wait` would wait
        /// for signals that are never going to exist.
        struct Barrier<'a> {
            finish: &'a Finish,
            remaining: Cell<usize>,
        }

        impl Drop for Barrier<'_> {
            fn drop(&mut self) {
                for _ in 0..self.remaining.get() {
                    self.finish.signal();
                }
                self.finish.wait();
            }
        }

        assert!(
            !self.exec.is_main_thread(),
            "GpuiExec::for_each blocks the caller and cannot run on the main \
             thread: launch it from a background task \
             (cx.background_executor().spawn(..))"
        );

        let workers = self.workers();
        assert_eq!(
            bufs.len(),
            workers,
            "for_each expects one buffer per worker: the index is the board batch"
        );

        // No parallelism possible: inline.
        if workers == 1 {
            for (index, buf) in bufs.iter_mut().enumerate() {
                f(index, buf);
            }
            return;
        }

        let finish = Finish::new(bufs.len());
        let finish_addr = &finish as *const Finish as usize;
        let barrier = Barrier {
            finish: &finish,
            remaining: Cell::new(bufs.len()),
        };

        let f_addr = &f as *const F as usize;

        for (index, buf) in bufs.iter_mut().enumerate() {
            let ptr = buf as *mut T as usize;

            unsafe {
                // SAFETY: `spawn` requires `'static` and the reference to finish dies
                // in this same function. The asynchronous executions are guaranteed
                // to always finish before `finish` dies because:
                // - The drop of `Barrier` does not complete until `done == total`.
                // - Every task always reports its end in any case: on
                //   finishing or panicking through the drop of `TaskGuard`. Since
                //   `TaskGuard` is created before the `spawn`, also if the future is
                //   discarded before it gets to poll. Any task that is never even attempted
                //   is not counted because its counter is not subtracted from `remaining`.
                // - Each `ptr` comes from a different `iter_mut`, so there are no
                //   two tasks with the same `&mut T`, and `&f` is only read.
                // - The `Acquire` of `wait` pairs with the `AcqRel` of
                //   `signal`: seeing `done == total` publishes the writes of
                //   each task in its buffer, avoiding data races.
                let guard = TaskGuard(&*(finish_addr as *const Finish));
                barrier.remaining.set(barrier.remaining.get() - 1);
                self.exec
                    .spawn(async move {
                        let _done = guard;
                        let f = &*(f_addr as *const F);
                        let buf = &mut *(ptr as *mut T);

                        f(index, buf);
                    })
                    .detach();
            }
        }

        // Releases the tasks that never got to be launched (none on the
        // normal path) and waits for the ones that did.
        drop(barrier);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::mpsc;

    use gpui::{BackgroundExecutor, ThreadedDispatcher};
    use string_art::Executor;

    use super::GpuiExec;

    /// Cost of a `for_each` without work: dispatch of `workers` tasks and
    /// barrier.
    #[test]
    fn dispatch_overhead() {
        let exec = BackgroundExecutor::new(Arc::new(ThreadedDispatcher::new()));
        let par = GpuiExec::new(exec.clone());
        let (tx, rx) = mpsc::channel();

        exec.spawn(async move {
            let mut bufs = vec![0usize; par.workers()];
            par.for_each(&mut bufs, |_, buf| *buf += 1); // warm-up

            const CALLS: usize = 1000;
            let start = std::time::Instant::now();
            for _ in 0..CALLS {
                par.for_each(&mut bufs, |_, buf| *buf += 1);
            }
            let per_call = start.elapsed().as_secs_f64() * 1e6 / CALLS as f64;
            let _ = tx.send(per_call);
        })
        .detach();

        let per_call = rx.recv().expect("the bench task crashed");
        println!("for_each no-op: {per_call:.1} µs/task");
    }
}

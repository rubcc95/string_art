use std::future::Future;
use std::sync::{Arc, mpsc};

use gpui::{BackgroundExecutor, Task};
use string_art_sync::{CondSend, CondSync, Parallel};

/// Backend de [`Parallel`] sobre el pool de fondo de gpui.
///
/// Ver la documentación del crate para las dos reglas de uso (no llamar desde
/// el hilo principal y dejar un hilo libre).
#[derive(Clone)]
pub struct GpuiExec {
    exec: Arc<BackgroundExecutor>,
}

impl GpuiExec {
    pub fn new(exec: BackgroundExecutor) -> Self {
        Self {
            exec: Arc::new(exec),
        }
    }

    /// El executor de gpui que hay debajo.
    pub fn executor(&self) -> &BackgroundExecutor {
        &self.exec
    }
}

/// Suelta la barrera aunque la tarea entre en pánico o se cancele.
///
/// Sin esto, un pánico en una tarea dejaría a `for_each` esperando para
/// siempre.
struct Done(mpsc::Sender<()>);

impl Drop for Done {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

impl Parallel for GpuiExec {
    /// Un hilo menos que el pool: el que llama se bloquea esperando y también
    /// hace falta sitio para el resto de trabajo de la interfaz.
    fn workers(&self) -> usize {
        self.exec.num_cpus().saturating_sub(1).max(1)
    }

    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        T: CondSend,
        F: Fn(usize, &mut T) + CondSend + CondSync,
    {
        assert!(
            !self.exec.is_main_thread(),
            "GpuiExec::for_each bloquea al que llama y no puede correr en el hilo \
             principal: lánzalo desde una tarea de fondo \
             (cx.background_executor().spawn(..))"
        );

        let workers = self.workers().min(bufs.len()).max(1);

        // Sin paralelismo posible (un hilo, o un solo elemento): en línea.
        if workers == 1 || bufs.len() <= 1 {
            for (index, buf) in bufs.iter_mut().enumerate() {
                f(index, buf);
            }
            return;
        }

        // `spawn` exige `'static`, pero las tareas se llevan trozos prestados de
        // `bufs` y una referencia a `f`. Se pasan como direcciones crudas.
        //
        // SAFETY: los préstamos siguen vivos hasta que `for_each` vuelve, y no
        // vuelve hasta que todas las tareas han soltado su `Done` (barrera del
        // canal). Los trozos de `chunks_mut` no se solapan entre sí.
        let f_addr = &f as *const F as usize;

        let (tx, rx) = mpsc::channel::<()>();
        let chunk = bufs.len().div_ceil(workers);
        let mut dispatched = 0usize;

        for (chunk_index, slice) in bufs.chunks_mut(chunk).enumerate() {
            let tx = tx.clone();
            let start = chunk_index * chunk;
            let ptr = slice.as_mut_ptr() as usize;
            let len = slice.len();

            self.exec
                .spawn(async move {
                    let _done = Done(tx);

                    // SAFETY: ver el comentario de `f_addr`.
                    let f = unsafe { &*(f_addr as *const F) };
                    let slice = unsafe { core::slice::from_raw_parts_mut(ptr as *mut T, len) };

                    for (offset, item) in slice.iter_mut().enumerate() {
                        f(start + offset, item);
                    }
                })
                .detach();

            dispatched += 1;
        }

        drop(tx);

        // Barrera: no se sale hasta que cada tarea ha terminado.
        for _ in 0..dispatched {
            let _ = rx.recv();
        }
    }

    /// Versión sin bloqueo: reparte una tarea por hilo y las espera con
    /// `await`. No hace falta `unsafe` porque aquí `F` y `R` ya son `'static`.
    fn map<R, F>(&self, f: F) -> impl Future<Output = Vec<R>>
    where
        F: Fn(usize) -> R + CondSend + CondSync + 'static,
        R: CondSend + 'static,
    {
        let f = Arc::new(f);

        let tasks: Vec<Task<R>> = (0..self.workers())
            .map(|index| {
                let f = f.clone();
                self.exec.spawn(async move { f(index) })
            })
            .collect();

        async move {
            let mut out = Vec::with_capacity(tasks.len());
            for task in tasks {
                out.push(task.await);
            }
            out
        }
    }
}

use gpui::BackgroundExecutor;
use std::{
    cell::Cell,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use string_art::Executor;

/// Backend de [`Parallel`] sobre el pool de fondo de gpui.
///
/// `for_each` no puede correr en el hilo principal: reparte el trabajo en el
/// pool y el que llama se queda esperando, con su hilo del pool ocupado. En
/// wasm se deja además un hilo libre para el que atiende la página.
#[derive(Clone)]
pub struct GpuiExec {
    exec: Arc<BackgroundExecutor>,
    workers: usize,
}

impl GpuiExec {
    pub fn new(exec: BackgroundExecutor) -> Self {
        // `workers` se fija aquí: el trait exige que sea estable durante toda
        // la vida del backend, y en wasm cada consulta cruza a JS.
        //
        // En wasm `BackgroundExecutor::num_cpus` delega en `num_cpus::get()`,
        // que devuelve siempre 1 porque no puede leer el sistema. El dato real
        // está en `navigator.hardwareConcurrency`, que expone
        // `wasm_thread::available_parallelism()`.
        //
        // En wasm se deja un hilo más libre para el que atiende la página.
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
    /// Hilos de cómputo. Uno por CPU en nativo; en wasm se deja uno más libre
    /// para el que atiende la página.
    ///
    /// El que llama a `for_each` ocupa un hilo del pool mientras espera, así
    /// que hay que descontarlo al repartir. Se calcula una vez en [`Self::new`]
    /// porque el trait exige que sea estable.
    fn workers(&self) -> usize {
        self.workers
    }

    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        T: Send,
        F: Fn(usize, &mut T) + Send + Sync,
    {
        /// Contador de terminación de un `for_each`.
        ///
        /// `done` cuenta las tareas que van terminando y `waiter` es el hilo que
        /// espera: un `fetch_add` por tarea y un `load` por vuelta del que espera, sin
        /// asignaciones ni cerrojos.
        ///
        /// El que espera se duerme con `park`; en wasm el `Parker` de std espera con
        /// `memory.atomic.wait32`, así que también duerme de verdad.
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

            /// La llama cada tarea al terminar, y el guard por las que no llegaron a
            /// lanzarse.
            ///
            /// El `unpark` deja la marca puesta si el que espera aún no está dormido,
            /// así que no se pierde el aviso.
            fn signal(&self) {
                if self.done.fetch_add(1, Ordering::AcqRel) + 1 == self.total {
                    self.waiter.unpark();
                }
            }

            /// La llama el que reparte para esperar a que terminen todas las tareas.
            fn wait(&self) {
                // `Acquire`: lo que las tareas escribieron en `bufs` se ve aquí.
                while self.done.load(Ordering::Acquire) < self.total {
                    std::thread::park();
                }
            }
        }

        /// Suma al contador cuando la tarea termina, aunque entre en pánico o se
        /// cancele. Sin esto, `for_each` esperaría para siempre.
        ///
        /// Se crea en el llamante y viaja dentro del futuro, así que también suma si
        /// la tarea se cancela antes de llegar a ejecutarse.
        ///
        /// `spawn` exige `'static`, así que se guarda la dirección de `Finish`; sigue
        /// viva hasta que `for_each` vuelve (ver `WaitOnDrop`).
        struct Done<'a>(&'a Finish);

        impl Drop for Done<'_> {
            fn drop(&mut self) {
                // SAFETY: ver el comentario de `finish_addr` en `for_each`.
                self.0.signal()
            }
        }

        /// Espera a las tareas al salir de `for_each`, también durante un
        /// desenrollado.
        ///
        /// Si el reparto se corta (por ejemplo, un `spawn` en pánico), cuenta como
        /// terminadas las tareas que no llegaron a lanzarse: si no, `wait` esperaría
        /// señales que no van a existir. Tampoco se puede salir con tareas vivas que
        /// apunten a `bufs`.
        struct WaitOnDrop<'a> {
            finish: &'a Finish,
            remaining: Cell<usize>,
        }

        impl Drop for WaitOnDrop<'_> {
            fn drop(&mut self) {
                for _ in 0..self.remaining.get() {
                    self.finish.signal();
                }
                self.finish.wait();
            }
        }

        assert!(
            !self.exec.is_main_thread(),
            "GpuiExec::for_each bloquea al que llama y no puede correr en el hilo \
             principal: lánzalo desde una tarea de fondo \
             (cx.background_executor().spawn(..))"
        );

        let workers = self.workers();
        assert_eq!(
            bufs.len(),
            workers,
            "for_each espera un buffer por worker: el índice es el lote del tablero"
        );

        // Sin paralelismo posible: en línea.
        if workers == 1 {
            for (index, buf) in bufs.iter_mut().enumerate() {
                f(index, buf);
            }
            return;
        }

        // `Finish` va en la pila; `WaitOnDrop` se crea antes de lanzar nada,
        // así que ni el retorno ni un pánico pueden salir de aquí con tareas
        // vivas.
        let finish = Finish::new(bufs.len());
        let finish_addr = &finish as *const Finish as usize;
        let barrier = WaitOnDrop {
            finish: &finish,
            remaining: Cell::new(bufs.len()),
        };

        // `spawn` exige `'static`, pero las tareas se llevan un elemento
        // prestado de `bufs` y una referencia a `f`. Se pasan como direcciones
        // crudas.
        //
        // SAFETY: los préstamos de `bufs` y de `f` siguen vivos hasta que
        // `for_each` vuelve, y `WaitOnDrop` no deja volver (ni desenrollar)
        // hasta que todas las tareas lanzadas han hecho `signal` (o lo ha hecho
        // el propio guard por las que no llegaron a lanzarse). Cada tarea
        // recibe un elemento distinto de `bufs`, así que no se solapan.
        let f_addr = &f as *const F as usize;

        for (index, buf) in bufs.iter_mut().enumerate() {
            let ptr = buf as *mut T as usize;

            // El guard viaja dentro del futuro: si la tarea se cancela antes de
            // ejecutarse, su `Drop` lo suelta igualmente.
            //let done = Done(&finish);
            barrier.remaining.set(barrier.remaining.get() - 1);

            self.exec
                .spawn(async move {
                    unsafe {
                        let _done = Done(&*(finish_addr as *const Finish));
                        // SAFETY: ver el comentario de `f_addr`.
                        let f = &*(f_addr as *const F);
                        let buf = &mut *(ptr as *mut T);

                        f(index, buf);
                    }
                })
                .detach();
        }

        // Suelta las tareas que no llegaron a lanzarse (ninguna en el camino
        // normal) y espera a las que sí.
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

    /// Coste de un `for_each` sin trabajo: despacho de `workers` tareas y
    /// barrera.
    #[test]
    fn dispatch_overhead() {
        let exec = BackgroundExecutor::new(Arc::new(ThreadedDispatcher::new()));
        let par = GpuiExec::new(exec.clone());
        let (tx, rx) = mpsc::channel();

        exec.spawn(async move {
            let mut bufs = vec![0usize; par.workers()];
            par.for_each(&mut bufs, |_, buf| *buf += 1); // calentamiento

            const CALLS: usize = 1000;
            let start = std::time::Instant::now();
            for _ in 0..CALLS {
                par.for_each(&mut bufs, |_, buf| *buf += 1);
            }
            let per_call = start.elapsed().as_secs_f64() * 1e6 / CALLS as f64;
            let _ = tx.send(per_call);
        })
        .detach();

        let per_call = rx.recv().expect("la tarea del bench se cayó");
        println!("for_each no-op: {per_call:.1} µs/tarea");
    }
}

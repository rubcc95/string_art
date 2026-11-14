use string_art_sync::Executor;

/// Pool de rayon propio, con un número de hilos fijo.
///
/// Tener el pool aquí (y no el global de rayon) permite elegir el número
/// de hilos en runtime y medirlo sin tocar variables de entorno.
#[derive(Clone, Copy, Debug, Default)]
pub struct SerialExec;

impl Executor for SerialExec {
    fn workers(&self) -> usize {
        1
    }

    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        F: Fn(usize, &mut T),
    {
        f(0, &mut bufs[0])
    }
}

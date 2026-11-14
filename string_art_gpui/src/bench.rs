use string_art::{
    Computation, Decay, Ellipse, Executor, Frac, Monocolor,
    geometry::Rect,
    nails::{Anchor, Circular, circular::Direction},
};

const SIZE: u32 = 1024;
const NAILS: usize = 512;
const MIN_DISTANCE: usize = 20;
const STEPS: u32 = 3000;

fn test_image(size: u32) -> image::RgbImage {
    let center = size as f32 / 2.0;
    image::RgbImage::from_fn(size, size, |x, y| {
        let dx = x as f32 - center;
        let dy = y as f32 - center;
        let v = (dx.hypot(dy) / center * 255.0).min(255.0) as u8;
        image::Rgb([v, v, v])
    })
}

/// Igual que [`run`], pero devuelve el tiempo de la fase de cálculo (los
/// [`STEPS`] pasos), sin contar la preparación de imagen, mapa y tablero.
pub fn run_timed<X: Executor>(par: X) -> f64 {
    let image = test_image(SIZE);
    let monocolor: Monocolor<Anchor<Direction>> = Monocolor::from_image(&image, STEPS);
    let board = Ellipse::new(
        Rect::new(SIZE as f32, SIZE as f32),
        Circular(0.1),
        NAILS,
        MIN_DISTANCE,
    )
    .expect("tablero inválido");
    let rect = Rect::new(SIZE as usize, SIZE as usize);

    let computation = Computation::new(
        par,
        monocolor,
        board,
        rect,
        Decay::Flat(Frac::from_num(0.2)),
    )
    .expect("el mapa de color no cuadra con el área");

    let start = web_time::Instant::now();
    computation
        .map(|step| step.expect("paso fallido"))
        .for_each(drop);
    start.elapsed().as_secs_f64() * 1000.0
}

pub fn run<X: Executor>(par: X) {
    let _ = run_timed(par);
}

#[cfg(all(test, feature = "multithreaded"))]
mod tests {
    use std::sync::Arc;
    use std::sync::mpsc;

    use gpui::{BackgroundExecutor, ThreadedDispatcher};

    use super::run_timed;
    use crate::exec::GpuiExec;

    const REPS: usize = 7;

    #[test]
    fn gpui_exec() {
        // Mismo pool (available_parallelism) y misma cola de prioridad que el
        // dispatcher de Linux, pero sin ventana ni bucle de eventos.
        let exec = BackgroundExecutor::new(Arc::new(ThreadedDispatcher::new()));
        let (tx, rx) = mpsc::channel();

        let bg = exec.clone();
        exec.spawn(async move {
            let mut times = Vec::with_capacity(REPS);
            for _ in 0..REPS {
                times.push(run_timed(GpuiExec::new(bg.clone())));
            }
            let _ = tx.send(times);
        })
        .detach();

        let mut times = rx.recv().expect("la tarea del bench se cayó");
        times.remove(0); // calentamiento
        times.sort_by(f64::total_cmp);
        println!("muestras: {times:?}");
        println!(
            "min {:.1} ms | mediana {:.1} ms",
            times[0],
            times[times.len() / 2]
        );
    }
}

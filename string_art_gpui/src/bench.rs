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

/// Same as [`run`], but returns the time of the computation phase (the
/// [`STEPS`] steps), not counting image, map, and board preparation.
pub fn run_timed<X: Executor>(par: X) -> f64 {
    let image = test_image(SIZE);
    let monocolor: Monocolor<Anchor<Direction>> = Monocolor::from_image(&image, STEPS);
    let board = Ellipse::new(
        Rect::new(SIZE as f32, SIZE as f32),
        Circular(0.1),
        NAILS,
        MIN_DISTANCE,
    )
    .expect("invalid board");
    let rect = Rect::new(SIZE as usize, SIZE as usize);

    let computation = Computation::new(
        par,
        monocolor,
        board,
        rect,
        Decay::Flat(Frac::from_num(0.2)),
    )
    .expect("the color map does not match the area");

    let start = web_time::Instant::now();
    computation
        .map(|step| step.expect("failed step"))
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
        // Same pool (available_parallelism) and same priority queue as the
        // Linux dispatcher, but without a window or event loop.
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

        let mut times = rx.recv().expect("the bench task crashed");
        times.remove(0); // warm-up
        times.sort_by(f64::total_cmp);
        println!("samples: {times:?}");
        println!(
            "min {:.1} ms | median {:.1} ms",
            times[0],
            times[times.len() / 2]
        );
    }
}

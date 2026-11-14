// //! Comprueba que el backend paralelo da exactamente el mismo resultado que
// //! el serial, y cuánto tarda cada uno.
// //!
// //! Es la red de seguridad del reparto de trabajo: si cambia el troceado del
// //! tablero, el resultado no debe moverse.
// //!
// //! Necesita la feature `threads`: sin ella solo existe `Serial` y no hay nada
// //! que comparar.

// fn main() {
//     #[cfg(feature = "multithreaded")]
//     comparison::run_comparison();

//     #[cfg(not(feature = "multithreaded"))]
//     println!(
//         "Sin la feature `threads` solo existe `Serial`: no hay backend paralelo \
//          con el que comparar. Compílalo con `cargo run --example parallel_check \
//          --features threads`."
//     );
// }

// #[cfg(feature = "multithreaded")]
// mod comparison {
//     use geometry::Rect;
//     use std::time::Instant;
//     use string_art::nails::{self, circular::Direction};
//     use string_art::sync::Parallel;
//     use string_art::*;

//     const SIZE: u32 = 512;
//     const NAILS: usize = 128;
//     const MIN_DISTANCE: usize = 4;

//     fn test_image(size: u32) -> image::RgbImage {
//         let center = size as f32 / 2.0;
//         image::RgbImage::from_fn(size, size, |x, y| {
//             let dx = x as f32 - center;
//             let dy = y as f32 - center;
//             let v = (dx.hypot(dy) / center * 255.0).min(255.0) as u8;
//             image::Rgb([v, v, v])
//         })
//     }

//     /// Ejecuta `steps` pasos y devuelve la traza completa.
//     fn run<X: Parallel>(par: X, steps: u32) -> Vec<(usize, u8, f32, f32)> {
//         let image = test_image(SIZE);
//         let monocolor: Monocolor<nails::Anchor<Direction>> = Monocolor::from_image(&image, steps);
//         let board = Ellipse::new(
//             Rect::new(SIZE as f32, SIZE as f32),
//             nails::Circular(0.1),
//             NAILS,
//             MIN_DISTANCE,
//         )
//         .expect("tablero inválido");
//         let rect = Rect::new(SIZE as usize, SIZE as usize);

//         let computation = Computation::new(
//             par,
//             monocolor,
//             board,
//             rect,
//             Decay::Flat(Frac::from_num(0.2)),
//         )
//         .expect("el mapa de color no cuadra con el área");

//         computation
//             .map(|step| {
//                 let step = step.expect("paso fallido");
//                 (
//                     step.anchor.idx,
//                     u8::try_from(usize::from(step.anchor.link)).unwrap_or(0),
//                     step.segment.start.x,
//                     step.segment.start.y,
//                 )
//             })
//             .collect()
//     }

//     pub fn run_comparison() {
//         let steps = 500;

//         let start = Instant::now();
//         let serial = run(executor::Serial, steps);
//         let serial_us = start.elapsed().as_micros();

//         let par = sync::RayonExec::auto();
//         let workers = par.workers();
//         let start = Instant::now();
//         let parallel = run(par, steps);
//         let parallel_us = start.elapsed().as_micros();

//         println!(
//             "Serial: {:>10.2} ms  ({} pasos, 1 hilo)",
//             serial_us as f64 / 1000.0,
//             serial.len()
//         );
//         println!(
//             "Rayon : {:>10.2} ms  ({} pasos, {} hilos)",
//             parallel_us as f64 / 1000.0,
//             parallel.len(),
//             workers
//         );
//         if parallel_us > 0 {
//             println!("x{:.2}", serial_us as f64 / parallel_us as f64);
//         }

//         if serial == parallel {
//             println!("OK: mismo resultado paso a paso.");
//         } else {
//             let first = serial
//                 .iter()
//                 .zip(&parallel)
//                 .position(|(a, b)| a != b)
//                 .unwrap_or(0);
//             eprintln!(
//                 "DIFERENTE en el paso {first}: {:?} != {:?}",
//                 serial.get(first),
//                 parallel.get(first)
//             );
//             std::process::exit(1);
//         }
//     }
// }
fn main() {}

//! Uso mínimo del core: imagen sintética -> tablero -> cómputo.
//!
//! El backend se elige en el punto de construcción; el core no sabe cuál es.
//! Con la feature `threads` se usa el pool de rayon; sin ella, `Serial`, y
//! entonces el `Anchor` no tiene por qué ser `Send`/`Sync`.

use geometry::Rect;
use string_art::nails::{self, circular::Direction};
use string_art::sync::Parallel;
use string_art::*;

/// Imagen de prueba: degradado radial. No necesita assets.
fn test_image(size: u32) -> image::RgbImage {
    let center = size as f32 / 2.0;
    image::RgbImage::from_fn(size, size, |x, y| {
        let dx = x as f32 - center;
        let dy = y as f32 - center;
        let v = (dx.hypot(dy) / center * 255.0).min(255.0) as u8;
        image::Rgb([v, v, v])
    })
}

fn main() {
    let image = test_image(256);

    let monocolor: Monocolor<nails::Anchor<Direction>> = Monocolor::from_image(&image, 50);

    let board = Ellipse::new(
        Rect::new(image.height() as f32, image.width() as f32),
        nails::UniformCircular(0.1),
        24,
        2,
    )
    .expect("tablero inválido");

    let rect = Rect::new(image.height() as usize, image.width() as usize);

    // Sin la feature `threads` solo existe `Serial`: es el mismo código, un
    // backend u otro según lo que se haya compilado.
    #[cfg(feature = "threads")]
    let par = sync::RayonExec::auto();
    #[cfg(not(feature = "threads"))]
    let par = sync::Serial;

    let mut computation = Computation::new(
        par,
        monocolor,
        board,
        rect,
        decay::Decay::Flat(Frac::from_num(0.2)),
    )
    .expect("el mapa de color no cuadra con el área");

    println!("hilos: {}", computation.par().workers());

    for step in computation.by_ref().take(5) {
        let step = step.expect("paso fallido");
        println!(
            "{:>3} -> {:<3} ({:.1}, {:.1})",
            step.anchor.idx, step.anchor.link, step.segment.start.x, step.segment.start.y
        );
    }
}

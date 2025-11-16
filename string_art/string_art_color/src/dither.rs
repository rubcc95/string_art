use crate::{
    geometry::Point,
    image::Image,
    verboser::{Message, Verboser},
    Float, color::Rgb,
};

pub struct Weight<T> {
    pub pos: Point<isize>,
    pub weight: T,
}

impl<T: Float> Weight<T> {
    fn get_mut<'a>(
        &self,
        pos: Point<usize>,
        image_dithered: &'a mut Image<T>,
    ) -> Option<&'a mut Rgb<T>> {
        (pos.as_::<isize>() + self.pos)
            .cast::<usize>()
            .and_then(|point| {
                let res = image_dithered.get_mut(point);
                res
            })
    }

    fn apply(&self, pos: Point<usize>, color: Rgb<T>, image_dithered: &mut Image<T>) {
        if let Some(pixel) = self.get_mut(pos, image_dithered) {
            pixel.0 += color.0 * self.weight;
            pixel.1 += color.1 * self.weight;
            pixel.2 += color.2 * self.weight;
        }
    }
}

pub trait Palette<'a, T> {
    type Color<'u>: Color<T>
    where
        'a: 'u,
        Self: 'u;

    fn iter<'u>(&'u mut self) -> impl Iterator<Item = Self::Color<'u>>
    where
        'a: 'u,
        Self: 'u;

    fn find_closest_color(&mut self, color: &Rgb<T>, pixel_index: usize) -> Result<Rgb<T>, Error>
    where
        T: Float,
    {
        let mut iter = self.iter();
        if let Some(mut best) = iter.next() {
            let mut best_dt = best.color().distance_squared(color);
            while let Some(other) = iter.next() {
                let distance = other.color().distance_squared(color);
                if distance < best_dt {
                    best_dt = distance;
                    best = other;
                }
            }
            best.set_pixel(pixel_index);
            Ok(best.color())
        } else {
            Err(Error)
        }
    }
}

pub trait Color<T> {
    fn color(&self) -> Rgb<T>;

    fn set_pixel(&mut self, pixel_index: usize);
}

pub struct Dither<W> {
    weights: W,
}

impl<'a, W> Dither<W> {
    pub const fn new(weigths: W) -> Self {
        Self { weights: weigths }
    }
}
impl<T: Float> Dither<[Weight<T>; 4]> {
    pub const fn floyd_steinberg() -> Self {
        Self::new(T::FLOYD_STEINBERG)
    }
}

impl<W> Dither<W> {
    pub fn dither<'a, P: ?Sized, T: Float>(
        &mut self,
        counter: &mut P,
        image_dithered: &mut Image<T>,
        verboser: &mut impl Verboser,
    ) -> Result<(), Error>
    where
        P: Palette<'a, T>,
        W: AsRef<[Weight<T>]>,
    {
        let y = image_dithered.height;
        let x = image_dithered.width;

        for y in 0..y {
            verboser.verbose(Message::Dithering(y, image_dithered.height));
            for x in 0..x {
                let pixel_idx = unsafe { image_dithered.index_of_unchecked(Point { x, y }) };
                let old_color = unsafe { image_dithered.get_unchecked(pixel_idx) };

                let color = counter.find_closest_color(old_color, pixel_idx)?;

                let color_diff = Rgb(
                    old_color.0 - color.0,
                    old_color.1 - color.1,
                    old_color.2 - color.2,
                );

                //*old_color = color;
                for dither_weight in self.weights.as_ref() {
                    dither_weight.apply(Point { x, y }, color_diff, image_dithered);
                }
            }
        }
        verboser.verbose(Message::Dithering(
            image_dithered.height,
            image_dithered.height,
        ));
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Palette is empty")]
pub struct Error;
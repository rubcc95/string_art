use std::{fmt::Debug, vec};

use bresenham::Bresenham;
use image::{GenericImage, GenericImageView, ImageResult, Pixel, Rgb, RgbImage};
use num_traits::AsPrimitive;
use palette::{color_difference::EuclideanDistance, FromColor, Srgb};

use crate::{
    geometry::{Point, Segment},
    hooks::{self, Handle, Links},
    Float, Grid,
};

type Lab<T> = palette::Lab<palette::white_point::D65, T>;

pub trait ColorMapBuilder<L> {
    fn build(self) -> (Rgb<u8>, usize, L);
}

impl<L: Default> ColorMapBuilder<L> for Rgb<u8> {
    fn build(self) -> (Rgb<u8>, usize, L) {
        (self, 0, L::default())
    }
}

impl<L: Default> ColorMapBuilder<L> for (Rgb<u8>, usize) {
    fn build(self) -> (Rgb<u8>, usize, L) {
        (self.0, self.1, L::default())
    }
}

impl<L> ColorMapBuilder<L> for (Rgb<u8>, usize, L) {
    fn build(self) -> (Rgb<u8>, usize, L) {
        self
    }
}
pub struct ColorMap<T, L> {
    color: Rgb<u8>,
    data: Vec<(T, T)>,
    curr_idx: usize,
    curr_link: L,
    initial_idx: usize,
    initial_link: L,
    weight: T,
    count: usize,
}

impl<T: Float, L: Copy> ColorMap<T, L>
where
    u8: AsPrimitive<T>,
{
    fn new(
        image: &impl GenericImageView<Pixel: Pixel<Subpixel: AsPrimitive<T>>>,
        builder: impl ColorMapBuilder<L>,
    ) -> Self {
        let (color, idx, link) = builder.build();
        let lab = Lab::from_color(Srgb::new(
            color[0].as_() / T::TWO_FIVE_FIVE,
            color[1].as_() / T::TWO_FIVE_FIVE,
            color[2].as_() / T::TWO_FIVE_FIVE,
        ));
        let mut weight = T::ZERO;
        Self {
            color,
            data: image
                .pixels()
                .map(|pixel| {
                    let pixel_rgb = image::Pixel::to_rgb(&pixel.2);
                    let pixel_lab = Lab::from_color(Srgb::new(
                        pixel_rgb[0].as_() / T::TWO_FIVE_FIVE,
                        pixel_rgb[1].as_() / T::TWO_FIVE_FIVE,
                        pixel_rgb[2].as_() / T::TWO_FIVE_FIVE,
                    ));
                    let pixel_weigth = T::SQRT140050 - pixel_lab.distance(lab);
                    weight += pixel_weigth;
                    (T::ONE, pixel_weigth)
                })
                .collect(),
            initial_idx: idx,
            curr_idx: idx,
            initial_link: link,
            curr_link: link,
            weight,
            count: 0,
        }
    }
}

pub struct Step<T, L> {
    color_map: *mut ColorMap<T, L>,
    start_point: Point<T>,
    end_point: Point<T>,
    end_link: L,
    end_idx: usize,
}

impl<T: Float, L> Step<T, L> {
    pub fn update(
        &mut self,
        darkness: T,
        points: &[usize],
        handle: &impl Handle<T, Links: Links<Item = L>>,
    ) {
        let curr_map = unsafe { &mut *self.color_map };
        curr_map.curr_link = handle.get_next_link(self.end_link);
        curr_map.curr_idx = self.end_idx;
        for &point in points {
            curr_map.data[point].0 *= darkness;
        }
    }
}

pub struct StringArt<T, H: hooks::Handle<T>> {
    pub color_maps: Vec<ColorMap<T, <H::Links as IntoIterator>::Item>>,
    pub hooks: Vec<H::Hook>,
    grid: Grid,
    hook_handler: H,
    min_hook_distance: usize,
    steps: Vec<Step<T, <H::Links as IntoIterator>::Item>>,
    darkness: T,
    pub line_table: Vec<bool>,
}

impl<T: Float, H: hooks::Handle<T>> StringArt<T, H>
where
    u32: AsPrimitive<T>,
    usize: AsPrimitive<T>,
    u8: AsPrimitive<T>,
{
    fn select_next<'a>(
        structs: *mut [ColorMap<T, <H::Links as IntoIterator>::Item>],
    ) -> &'a mut ColorMap<T, <H::Links as IntoIterator>::Item> {
        let mut selected_idx = core::ptr::null_mut();
        let mut min_ratio = T::INFINITY;

        for s in unsafe { (*structs).iter_mut() } {
            let ratio = s.count.as_() / s.weight;
            if ratio < min_ratio {
                min_ratio = ratio;
                selected_idx = s;
            }
        }

        let res = unsafe { &mut *selected_idx };
        res.count += 1;
        res
    }
    pub fn show_weigths(&self)
    where
        T: std::fmt::Display,
    {
        for color_map in &self.color_maps {
            print!("Color: {:?}, ", color_map.color);
            println!("Weight: {}", color_map.weight);
        }
        //println!("Prop: {}", self.color_maps[0].weight / self.color_maps[1].weight);
    }
    pub fn index_of_link(
        &self,
        a_idx: usize,
        a_link: <H::Links as IntoIterator>::Item,
        b_idx: usize,
        b_link: <H::Links as IntoIterator>::Item,
    ) -> usize {
        let (mut big_idx, big_link, mut small_idx, small_link) = if a_idx > b_idx {
            (a_idx, a_link, b_idx, b_link)
        } else {
            (b_idx, b_link, a_idx, a_link)
        };
        let cap = self.hooks.len() - self.min_hook_distance - 1;
        let a = if big_idx > cap {
            let diff = big_idx - cap;
            big_idx -= diff;
            small_idx -= diff;
            diff * (cap - self.min_hook_distance) * <H::Links as Links>::SQ_LEN
        } else {
            0
        };

        let diff = big_idx - self.min_hook_distance;

        a + diff * (diff - 1) * <H::Links as Links>::SQ_LEN / 2
            + <H::Links as Links>::LEN * diff * big_link.into()
            + <H::Links as Links>::LEN * small_idx
            + small_link.into()
    }

    pub fn comprobate(&self) {
        let mut expected = 0;
        let max_step = self.hooks.len() - self.min_hook_distance;
        for big_main in 0..self.hooks.len() {
            for big_sec in H::LINKS.into_iter() {
                for small_main in 0..big_main {
                    let diff = big_main - small_main;
                    if diff > self.min_hook_distance && diff < max_step {
                        for small_sec in H::LINKS.into_iter() {
                            assert_eq!(
                                expected,
                                self.index_of_link(big_main, big_sec, small_main, small_sec)
                            );
                            expected += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(expected, self.line_table.len());
    }
    pub fn ellipse(
        image: &impl GenericImageView<Pixel: Pixel<Subpixel: AsPrimitive<T>>>,
        hook_builder: impl hooks::Builder<T, Handle = H, Hook = H::Hook>,
        hook_count: usize,
        palette: impl Iterator<Item = impl ColorMapBuilder<<H::Links as IntoIterator>::Item>>,
        min_hook_distance: usize,
        darkness: T,
        //weigths: &[T]
    ) -> Self {
        let width = T::HALF * image.width().as_();
        let height = T::HALF * image.height().as_();

        let res = Self {
            line_table: vec![false; Self::get_count(hook_count, min_hook_distance)],
            darkness,
            color_maps: palette
                .map(|builder| ColorMap::new(image, builder))
                .collect(),
            hooks: (0..hook_count)
                .into_iter()
                .map(|i| {
                    let theta: T = T::TWO * T::PI * (i.as_()) / (hook_count.as_());
                    hook_builder.build_hook(
                        Point {
                            x: width * (T::ONE + theta.cos()),
                            y: height * (T::ONE + theta.sin()),
                        },
                        theta,
                    )
                })
                .collect(),
            grid: Grid::new(image.height() as usize, image.width() as usize),
            hook_handler: hook_builder.build_handle(),
            min_hook_distance,
            steps: Vec::new(),
        };
        res
    }

    // fn index_of(&self, point: Point<isize>) -> Option<usize> {
    //     if point.x < 0
    //         || point.y < 0
    //         || point.x >= self.width as isize
    //         || point.y >= self.height as isize
    //     {
    //         None
    //     } else {
    //         Some(point.y as usize * self.width + point.x as usize)
    //     }
    // }

    fn get_count(hook_count: usize, min_hook_distance: usize) -> usize {
        <H::Links as Links>::SQ_LEN
            * (hook_count * (hook_count - 1) / 2 - min_hook_distance * hook_count)
    }

    fn get_best_line(
        &mut self,
        best_line_buffer: &mut Vec<usize>,
        line_buffer: &mut Vec<usize>,
    ) -> Option<(usize, Step<T, <H::Links as IntoIterator>::Item>)> {
        let max_hook_distance = self.hooks.len() - self.min_hook_distance;
        let mut best_weight = -T::INFINITY;
        let mut best_line = None;

        let color_map = Self::select_next(self.color_maps.as_mut_slice());
        let from_hook = unsafe { self.hooks.get_unchecked(color_map.curr_idx) };

        for to_idx in 0..self.hooks.len() {
            let hook_diff = color_map.curr_idx.abs_diff(to_idx);
            if hook_diff > self.min_hook_distance && hook_diff < max_hook_distance {
                let to_hook = unsafe { self.hooks.get_unchecked(to_idx) };
                for to_link in H::LINKS {
                    let line_idx = self.index_of_link(
                        color_map.curr_idx,
                        color_map.curr_link,
                        to_idx,
                        to_link,
                    );
                    if self.line_table[line_idx] {
                        continue;
                    }
                    let (line, weight) = self.calculate_line_weight(
                        from_hook,
                        color_map.curr_link,
                        to_hook,
                        to_link,
                        color_map,
                        line_buffer,
                    );
                    if weight > best_weight {
                        best_weight = weight;
                        core::mem::swap(best_line_buffer, line_buffer);
                        best_line = Some((
                            line_idx,
                            Step {
                                color_map: color_map,
                                start_point: line.start,
                                end_point: line.end,
                                end_link: to_link,
                                end_idx: to_idx,
                            },
                        ));
                    }
                }
            }
        }
        best_line
    }

    fn get_pixels_in_line(&self, seg: Segment<T>) -> impl Iterator<Item = usize> + '_ {
        seg.cast::<isize>()
            .unwrap()
            .points_between()
            .filter_map(|point| point.cast().and_then(|point| self.grid.index_of(point)))
    }

    fn calculate_line_weight(
        &self,
        from_hook: &H::Hook,
        from_link: <H::Links as IntoIterator>::Item,
        to_hook: &H::Hook,
        to_link: <H::Links as IntoIterator>::Item,
        color_map: &ColorMap<T, <H::Links as IntoIterator>::Item>,
        line_buffer: &mut Vec<usize>,
    ) -> (Segment<T>, T) {
        let line = self
            .hook_handler
            .get_segment((from_hook, from_link), (to_hook, to_link));

        let mut weight = T::ZERO;
        let mut count = T::ZERO;
        line_buffer.clear();
        line_buffer.extend(self.get_pixels_in_line(line).map(|idx| {
            let (active, delta) = unsafe { *color_map.data.get_unchecked(idx) };
            weight = weight + active * delta;
            count = count + T::ONE;

            idx
        }));

        (
            line,
            if count > T::ZERO {
                weight / count as T
            } else {
                -T::INFINITY
            },
        )
    }

    fn update_color_maps(
        &mut self,
        idx: usize,
        color_map: &mut ColorMap<T, <H::Links as IntoIterator>::Item>,
    ) {
        unsafe {
            color_map.data.get_unchecked_mut(idx).0 *= self.darkness;
        }
    }

    fn compute_once(&mut self, best_line_buffer: &mut Vec<usize>, line_buffer: &mut Vec<usize>) {
        if let Some(mut step) = self.get_best_line(best_line_buffer, line_buffer) {
            step.1
                .update(self.darkness, best_line_buffer, &self.hook_handler);
            // let curr_map = step.1.color_map_mut();
            // curr_map.curr_link = self.hook_handler.get_next_link(step.1.end_link);
            // curr_map.curr_idx = step.1.end_idx;
            // for &mut idx in best_line_buffer {
            //     unsafe { curr_map.data.get_unchecked_mut(idx).0 *= self.darkness };
            // }

            //println!("Selected {}", step.0);
            self.line_table[step.0] = true;
            self.steps.push(step.1);
        }
    }

    pub fn compute_steps_with_buffers(
        &mut self,
        step_count: usize,
        best_line_buffer: &mut Vec<usize>,
        line_buffer: &mut Vec<usize>,
    ) {
        while step_count > self.steps.len() {
            self.compute_once(best_line_buffer, line_buffer);
        }
    }

    pub fn compute_steps(&mut self, step_count: usize) {
        let mut best_line_buffer = Vec::new();
        let mut line_buffer = Vec::new();

        self.compute_steps_with_buffers(step_count, &mut best_line_buffer, &mut line_buffer);
    }

    pub fn save_image(&self, path: impl AsRef<std::path::Path>, scale: T) -> ImageResult<()> {
        let scaled_width = (self.grid.width.as_() * scale)
            .to_u32()
            .expect("this image is so big to be saved!");
        let scaled_height = (self.grid.height.as_() * scale)
            .to_u32()
            .expect("this image is so big to be saved!");
        let mut image = RgbImage::new(scaled_width, scaled_height);
        image.fill(255);
        // for pixel in image.pixels_mut() {
        //     *pixel = Rgb([255, 0, 255]);
        // }
        for step in self.steps.iter().rev() {
            let color = unsafe { (*step.color_map).color };
            for (x, y) in Bresenham::new(
                (
                    (step.start_point.x * scale)
                        .to_isize()
                        .expect("this image is so big to be saved!"),
                    (step.start_point.y * scale)
                        .to_isize()
                        .expect("this image is so big to be saved!"),
                ),
                (
                    (step.end_point.x * scale)
                        .to_isize()
                        .expect("this image is so big to be saved!"),
                    (step.end_point.y * scale)
                        .to_isize()
                        .expect("this image is so big to be saved!"),
                ),
            ) {
                if x >= 0 && y >= 0 && x < scaled_width as isize && y < scaled_height as isize {
                    unsafe { image.unsafe_put_pixel(x as u32, y as u32, color) };
                }
            }
        }
        image.save(path)
    }
}

// Exponer condicionalmente la función para pruebas
#[cfg(test)]
pub mod test_utils {
    pub use super::StringArt;
}

use std::ops::{Deref, Range};

use image::{ImageResult, RgbImage};
use num_traits::AsPrimitive;
use palette::{color_difference::EuclideanDistance, FromColor, Srgb};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use svg::Document;

use crate::{
    colors::{Lab, LabImage},
    ditherer::{Ditherer, WeightedColor},
    geometry::{Point, Segment},
    nails::{self, Links},
    Float, Grid,
};

pub trait CurrentNail: Copy {
    type Link;
    type Handle: Copy;

    fn get_current_nails(self, handle: Self::Handle) -> impl Iterator<Item = (usize, Self::Link)>;

    fn register_end(handle: Self::Handle, end: (usize, Self::Link)) -> Self;
}

pub struct ColorMapSettings<'a, T, L> {
    name: &'a str,
    color: Lab<T>,
    nail: usize,
    link: L,
}

impl<'a, T, L> ColorMapSettings<'a, T, L> {
    pub fn new(name: &'a str, color: Lab<T>, nail: usize, link: L) -> Self {
        Self {
            name,
            color,
            nail,
            link,
        }
    }
}
// pub trait ColorMapBuilder<T, L> {
//     fn build(&self) -> (String, Lab<T>, usize, L);
// }

// impl<T: Float, L: Default> ColorMapBuilder<T, L> for str {
//     fn build(&self) -> (String, Lab<T>, usize, L) {
//         let lab_color = match self.to_lowercase().as_str() {
//             "red" => T::RED,
//             "green" => T::GREEN,
//             "blue" => T::BLUE,
//             "yellow" => T::YELLOW,
//             "black" => T::BLACK,
//             "white" => T::WHITE,
//             "gray" | "grey" => T::GRAY,
//             "orange" => T::ORANGE,
//             "purple" => T::PURPLE,
//             "brown" => T::BROWN,
//             "pink" => T::PINK,
//             "cyan" => T::CYAN,
//             "magenta" => T::MAGENTA,
//             "lime" => T::LIME,
//             "teal" => T::TEAL,
//             "navy" => T::NAVY,
//             "indigo" => T::INDIGO,
//             "violet" => T::VIOLET,
//             "gold" => T::GOLD,
//             "silver" => T::SILVER,
//             "beige" => T::BEIGE,
//             "ivory" => T::IVORY,
//             "peach" => T::PEACH,
//             "chocolate" => T::CHOCOLATE,
//             _ => panic!(
//                 "Unknown color name: '{}'. Please provide a valid color name.",
//                 self
//             ),
//         };
//         (String::from(self), lab_color, 0, L::default())
//     }
// }

// impl<T: Copy, L: Default> ColorMapBuilder<T, L> for (&str, Lab<T>) {
//     fn build(&self) -> (String, Lab<T>, usize, L) {
//         (String::from(self.0), self.1, 0, L::default())
//     }
// }

// impl<T: Copy, L: Default> ColorMapBuilder<T, L> for (&str, Lab<T>, usize) {
//     fn build(&self) -> (String, Lab<T>, usize, L) {
//         (String::from(self.0), self.1, self.2, L::default())
//     }
// }

// impl<T: Copy, L: Copy> ColorMapBuilder<T, L> for (&str, Lab<T>, usize, L) {
//     fn build(&self) -> (String, Lab<T>, usize, L) {
//         (String::from(self.0), self.1, self.2, self.3)
//     }
// }

pub trait Darkness<T>: Copy {
    fn compute(self, weight: T) -> T;
}

#[derive(Clone, Copy)]
pub struct FlatDarkness<T>(pub T);

impl<T: Float> Darkness<T> for FlatDarkness<T> {
    fn compute(self, weight: T) -> T {
        (weight - self.0).max(T::ZERO)
    }
}

#[derive(Clone, Copy)]
pub struct PercentageDarkness<T>(pub T);

impl<S: Float> Darkness<S> for PercentageDarkness<S> {
    fn compute(self, weight: S) -> S {
        self.0 * weight
    }
}

struct ColorMap<'a, S, L> {
    settings: ColorMapSettings<'a, S, L>,
    data: Vec<(S, S)>,
    curr_idx: usize,
    curr_link: L,
    weight: usize,
    count: usize,
}

impl<'a, S, L> Deref for ColorMap<'a, S, L> {
    type Target = ColorMapSettings<'a, S, L>;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl<'a, T: Copy, L> WeightedColor<T> for ColorMap<'a, T, L> {
    fn color(&self) -> Lab<T> {
        self.color
    }

    fn add_weight(&mut self) {
        self.weight += 1;
    }
}
impl<'a, S: Float, L: Copy> ColorMap<'a, S, L>
where
    usize: AsPrimitive<S>,
{
    fn new(image: &LabImage<S>, settings: ColorMapSettings<'a, S, L>) -> Self {
        let mut total = S::ZERO;
        let this = Self {
            data: image
                .pixels()
                .iter()
                .map(|pixel_color| {
                    let val = S::SQRT140050 - pixel_color.distance(settings.color);
                    total += val;
                    (S::ONE, val)
                })
                .collect(),
            curr_idx: settings.nail,
            curr_link: settings.link,
            settings,
            weight: 0,
            count: 0,
        };
        total /= this.data.len().as_();
        this
    }

    fn calculate_weight(&self, segment: Segment<S>, grid: Grid) -> S {
        let mut weight = S::ZERO;
        let mut count = S::ZERO;
        for idx in grid.get_pixel_indexes_in_segment(segment) {
            let (active, delta) = unsafe { *self.data.get_unchecked(idx) };
            weight += active * delta;
            count += S::ONE;
        }
        if count > S::ZERO {
            weight / count as S
        } else {
            -S::INFINITY
        }
    }
}


#[derive(Copy, Clone)]
struct NextLine<'a, S, L> {
    line: *mut BakedSegment<S>,
    color_map: *mut ColorMap<'a, S, L>,
    end_link: L,
    end_idx: usize,
}

unsafe impl<'a, S: Send, L: Send> Send for NextLine<'a, S, L> {}
unsafe impl<'a, S: Sync, L: Sync> Sync for NextLine<'a, S, L> {}

#[derive(Clone, Copy)]
struct NailDistancer {
    min_nail_distance: usize,
    max_nail_distance: usize,
}

impl NailDistancer {
    fn is_valid(&self, a_idx: usize, b_idx: usize) -> bool {
        let nail_diff = a_idx.abs_diff(b_idx);
        nail_diff > self.min_nail_distance && nail_diff < self.max_nail_distance
    }

    fn nail_combs_iter<N: nails::Handle>(
        &self,
        range: Range<usize>,
    ) -> impl Iterator<Item = ((usize, N::Link), (usize, N::Link))> + '_ {
        range
            .map(move |big_idx| {
                N::LINKS
                    .into_iter()
                    .map(move |big_link| {
                        (0..big_idx)
                            .filter_map(move |small_idx| {
                                if self.is_valid(big_idx, small_idx) {
                                    Some(N::LINKS.into_iter().map(move |small_link| {
                                        ((big_idx, big_link), (small_idx, small_link))
                                    }))
                                } else {
                                    None
                                }
                            })
                            .flatten()
                    })
                    .flatten()
            })
            .flatten()
    }

    fn index_of<L: nails::Links>(
        &self,
        a_idx: usize,
        a_link: L::Item,
        b_idx: usize,
        b_link: L::Item,
    ) -> usize {
        let (mut big_idx, big_link, mut small_idx, small_link) = if a_idx > b_idx {
            (a_idx, a_link, b_idx, b_link)
        } else {
            (b_idx, b_link, a_idx, a_link)
        };
        let cap = self.max_nail_distance - 1;
        let a = if big_idx > cap {
            let diff = big_idx - cap;
            big_idx -= diff;
            small_idx -= diff;
            diff * (cap - self.min_nail_distance) * L::SQ_LEN
        } else {
            0
        };

        let diff = big_idx - self.min_nail_distance;

        a + diff * (diff - 1) * L::SQ_LEN / 2
            + L::LEN * diff * big_link.into()
            + L::LEN * small_idx
            + small_link.into()
    }
}

struct NextLineWeighted<'a, S, L> {
    next: NextLine<'a, S, L>,
    weight: S,
}

impl<'a, S: Float, L> Default for NextLineWeighted<'a, S, L> {
    #[allow(invalid_value)]
    fn default() -> Self {
        Self {
            //SAFETY: a -infinity weight will never be selected as best weight
            next: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            weight: -S::INFINITY,
        }
    }
}

#[derive(Clone)]
pub struct Table<S, N: nails::Handle> {
    nails: Vec<N::Nail>,
    handle: N,
    image: LabImage<S>,
}

impl<S: Float, N: nails::Handle> Table<S, N>
where
    usize: AsPrimitive<S>,
{
    pub fn ellipse(
        image: impl Into<LabImage<S>>,
        nail_builder: impl nails::Builder<Scalar = S, Handle = N, Nail = N::Nail>,
        nail_count: usize,
    ) -> Self {
        let image = image.into();
        Self {
            nails: (0..nail_count)
                .into_iter()
                .map(|i| {
                    let theta: S = S::TWO * S::PI * (i.as_()) / (nail_count.as_());
                    nail_builder.build_nail(
                        Point {
                            x: image.width.as_() * (S::ONE + theta.cos()),
                            y: image.height.as_() * (S::ONE + theta.sin()),
                        } * S::HALF,
                        theta,
                    )
                })
                .collect(),
            handle: nail_builder.build_handle(),
            image,
        }
    }
}

struct BatchBuffer<'a, S, L> {
    range: Range<usize>,
    result: NextLineWeighted<'a, S, L>,
}

#[derive(Clone, Copy)]
struct BakedSegment<S> {
    segment: Segment<S>,
    used: bool,
}

pub struct StringArt<'a, S, N: nails::Handle, D> {
    color_maps: Vec<ColorMap<'a, S, N::Link>>,
    grid: Grid,
    handle: N,
    distancer: NailDistancer,
    steps: Vec<NextLine<'a, S, N::Link>>,
    darkness: D,
    nails: Vec<N::Nail>,
    lines: Vec<BakedSegment<S>>,
    buffers: Vec<BatchBuffer<'a, S, N::Link>>,
}

impl<'a, S: Float, N: nails::Handle<Scalar = S>, D: Darkness<S>> StringArt<'a, S, N, D>
where
    usize: AsPrimitive<S>,
{
    pub fn new(
        mut table: Table<S, N>,
        palette: impl IntoIterator<Item = ColorMapSettings<'a, S, N::Link>>,
        min_nail_distance: usize,
        darkness: D,
    ) -> Self where {
        let nail_count = table.nails.len();
        let buffer_count = num_cpus::get();
        let chunk_size = (nail_count + buffer_count - 1) / buffer_count;
        let distancer = NailDistancer {
            min_nail_distance,
            max_nail_distance: nail_count - min_nail_distance,
        };

        let mut color_maps: Vec<ColorMap<S, N::Link>> = palette
            .into_iter()
            .map(|builder| ColorMap::new(&table.image, builder))
            .collect();
        Ditherer::floyd_steinberg(color_maps.as_mut_slice()).dither(&mut table.image);
        Self {
            lines: distancer
                .nail_combs_iter::<N>(0..nail_count)
                .map(|((a_idx, a_link), (b_idx, b_link))| BakedSegment {
                    segment: table.handle.get_segment(
                        (unsafe { table.nails.get_unchecked(a_idx) }, a_link),
                        (unsafe { table.nails.get_unchecked(b_idx) }, b_link),
                    ),
                    used: false,
                })
                .collect(),
            nails: table.nails,
            darkness,
            color_maps,
            grid: *table.image.deref(),
            handle: table.handle,
            distancer,
            steps: Vec::new(),
            buffers: (0..nail_count)
                .step_by(chunk_size)
                .map(|start| BatchBuffer {
                    range: start..std::cmp::min(start + chunk_size, nail_count),
                    #[allow(invalid_value)]
                    //Reason: will be initialized as default on get_best_line() before use
                    result: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
                })
                .collect(),
        }
    }

    fn select_next(structs: *mut [ColorMap<S, N::Link>]) -> *mut ColorMap<S, N::Link> {
        let mut selected_idx = core::ptr::null_mut();
        let mut min_ratio = S::INFINITY;

        for s in unsafe { (*structs).iter_mut() } {
            let ratio = s.count.as_() / s.weight.as_();
            if ratio < min_ratio {
                min_ratio = ratio;
                selected_idx = s;
            }
        }

        let res = unsafe { &mut *selected_idx };
        res.count += 1;
        res
    }

    pub fn save_image_svg(
        &self,
        path: impl AsRef<std::path::Path>,
        line_tickness: f32,
    ) -> Result<(), std::io::Error>
    {
        let mut doc = Document::new().set(
            "viewBox",
            (0.0, 0.0, self.grid.width.as_(), self.grid.height.as_()),
        );
        for &nail in self.nails.iter() {
            doc = doc.add(self.handle.draw_svg(nail))
        }

        for step in self.steps.iter().rev() {
            let segment = (unsafe { *step.line }).segment;
            let color = Srgb::from_color(unsafe { (*step.color_map).color });
            doc = doc.add(
                svg::node::element::Line::new()
                    .set("x1", segment.start.x)
                    .set("y1", segment.start.y)
                    .set("x2", segment.end.x)
                    .set("y2", segment.end.y)
                    .set(
                        "stroke",
                        format!(
                            "rgb({}, {}, {})",
                            (color.red * S::TWO_FIVE_FIVE).to_u8().unwrap(),
                            (color.green * S::TWO_FIVE_FIVE).to_u8().unwrap(),
                            (color.blue * S::TWO_FIVE_FIVE).to_u8().unwrap(),
                        ),
                    )
                    .set("stroke-width", line_tickness),
            );
        }
        svg::save(path, &doc)
    }

    pub fn save_image(&self, path: impl AsRef<std::path::Path>, scale: S) -> ImageResult<()>
    where
        S: AsPrimitive<f32>,
    {
        let grid = Grid::<u32> {
            height: (self.grid.height.as_() * scale)
                .to_u32()
                .expect("this image is so big to be saved!"),
            width: (self.grid.width.as_() * scale)
                .to_u32()
                .expect("this image is so big to be saved!"),
        };
        let mut buffer: Vec<u8> = vec![255; grid.height as usize * grid.width as usize * 3];
        for step in self.steps.iter().rev() {
            let mut segment = (unsafe { *step.line }).segment;
            let color = Srgb::from_color(unsafe { (*step.color_map).color });
            let color = (
                (color.red * S::TWO_FIVE_FIVE).to_u8().unwrap(),
                (color.green * S::TWO_FIVE_FIVE).to_u8().unwrap(),
                (color.blue * S::TWO_FIVE_FIVE).to_u8().unwrap(),
            );
            segment *= scale;
            for idx in grid.get_pixel_indexes_in_segment(segment) {
                unsafe {
                    let ptr = buffer.as_mut_ptr().add(3 * idx as usize);
                    *ptr = color.0;
                    *ptr.add(1) = color.1;
                    *ptr.add(2) = color.2;
                }
            }
        }
        let image = RgbImage::from_vec(grid.width, grid.height, buffer).unwrap();
        image.save(path)
    }
}

impl<
        'a,
        S: Float + Sync + Send,
        N: Sync + Send + nails::Handle<Scalar = S, Links: Links<Link: Sync + Send>>,
        D: Darkness<S>,
    > StringArt<'a, S, N, D>
where
    usize: AsPrimitive<S>,
{
    pub fn compute(&mut self, step_count: usize) {
        while step_count > self.steps.len() {
            if let Some(next) = self.get_best_line() {
                let color_map = unsafe { &mut *next.color_map };
                let line = unsafe { &mut *next.line };
                line.used = true;

                color_map.curr_link = self.handle.get_next_link(next.end_link);
                color_map.curr_idx = next.end_idx;
                for point in self.grid.get_pixel_indexes_in_segment(line.segment) {
                    let weight = unsafe { color_map.data.get_unchecked_mut(point) };
                    weight.1 = self.darkness.compute(weight.1);
                }
                self.steps.push(next);
            } else {
                return;
            }
        }
    }

    fn get_best_line(&mut self) -> Option<NextLine<'a, S, N::Link>> {
        #[derive(Copy, Clone)]
        struct SyncColorMapRef<'a, T, L>(*mut ColorMap<'a, T, L>);

        impl<'a, S, L> From<SyncColorMapRef<'a, S, L>> for *mut ColorMap<'a, S, L> {
            fn from(value: SyncColorMapRef<'a, S, L>) -> Self {
                value.0
            }
        }

        impl<'a, S, L> Deref for SyncColorMapRef<'a, S, L> {
            type Target = ColorMap<'a, S, L>;

            fn deref(&self) -> &Self::Target {
                unsafe { &*self.0 }
            }
        }

        unsafe impl<'a, S: Sync, L: Sync> Sync for SyncColorMapRef<'a, S, L> {}
        unsafe impl<'a, S: Send, L: Send> Send for SyncColorMapRef<'a, S, L> {}

        struct SyncLineTable<S>(*mut Vec<BakedSegment<S>>);

        impl<S> SyncLineTable<S> {
            pub unsafe fn get_unchecked(&self, idx: usize) -> *mut BakedSegment<S> {
                (*self.0).get_unchecked_mut(idx)
            }
        }

        unsafe impl<S: Sync> Sync for SyncLineTable<S> {}
        unsafe impl<S: Send> Send for SyncLineTable<S> {}

        let mut best_weight = -S::INFINITY;
        let mut best_line = None;
        let color_map = SyncColorMapRef(Self::select_next(self.color_maps.as_mut_slice()));
        let line_table = SyncLineTable(&mut self.lines);
        self.buffers.par_iter_mut().for_each(|buffer| {
            buffer.result = Default::default();
            for to_idx in buffer.range.clone() {
                if self.distancer.is_valid(color_map.curr_idx, to_idx) {
                    for to_link in N::LINKS {
                        let line_idx = self.distancer.index_of::<N::Links>(
                            color_map.curr_idx,
                            color_map.curr_link,
                            to_idx,
                            to_link,
                        );
                        let line: *mut _ = unsafe { line_table.get_unchecked(line_idx) };
                        let BakedSegment { segment, used } = unsafe { *line };
                        if used {
                            continue;
                        }
                        let weight = color_map.calculate_weight(segment, self.grid);
                        if weight > buffer.result.weight {
                            buffer.result = NextLineWeighted {
                                weight,
                                next: NextLine {
                                    line,
                                    color_map: color_map.into(),
                                    end_link: to_link,
                                    end_idx: to_idx,
                                },
                            };
                        }
                    }
                }
            }
        });
        for BatchBuffer { range: _, result } in self.buffers.iter() {
            if result.weight > best_weight {
                best_weight = result.weight;
                best_line = Some(result.next);
            }
        }

        best_line
    }
}

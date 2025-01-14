use std::ops::{Deref, Range};

use image::RgbImage;
use num_traits::AsPrimitive;
use palette::{color_difference::EuclideanDistance, FromColor, Srgb};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    algorithm::Error, color::Lab, image::{Image}, ditherer::{Ditherer, DitherCounter}, geometry::{Point, Segment}, nails::{self}, Float, Grid, nail_table::Table
};

pub trait CurrentNail: Copy {
    type Link;
    type Handle: Copy;

    fn get_current_nails(self, handle: Self::Handle) -> impl Iterator<Item = (usize, Self::Link)>;

    fn register_end(handle: Self::Handle, end: (usize, Self::Link)) -> Self;
}

pub struct ColorMapSettings<L> {
    name: String,
    color: (u8, u8, u8),
    nail: usize,
    link: L,
}

impl<L> ColorMapSettings<L> {
    pub fn new(name: String, color: (u8, u8, u8), nail: usize, link: L) -> Self {
        Self {
            name,
            color,
            nail,
            link,
        }
    }
}

pub trait Darkness<T> {
    fn compute(&self, weight: T) -> T;
}

#[derive(Clone, Copy)]
pub struct FlatDarkness<T>(pub T);

impl<T: Float> Darkness<T> for FlatDarkness<T> {
    fn compute(&self, weight: T) -> T {
        (weight - self.0).max(T::ZERO)
    }
}

#[derive(Clone, Copy)]
pub struct PercentageDarkness<T>(pub T);

impl<S: Float> Darkness<S> for PercentageDarkness<S> {
    fn compute(&self, weight: S) -> S {
        self.0 * weight
    }
}

struct LabColorMap<S, L> {
    color_map: ColorMap<S, L>,
    lab: Lab<S>,
    group_count: usize,
}
impl<S: Float, L: Copy> LabColorMap<S, L>
where
    u8: AsPrimitive<S>,
{
    fn new(image: &Image<S>, settings: ColorMapSettings<L>) -> Self {
        let lab = Lab::from_color(Srgb::new(
            settings.color.0.as_() / S::TWO_FIVE_FIVE,
            settings.color.1.as_() / S::TWO_FIVE_FIVE,
            settings.color.2.as_() / S::TWO_FIVE_FIVE,
        ));
        Self {
            lab,
            color_map: ColorMap::new(image, settings, lab),
            group_count: 0,
        }
    }
}
struct ColorMap<S, L> {
    settings: ColorMapSettings<L>,
    data: Vec<S>,
    curr_nail: usize,
    curr_link: L,
    weight: usize,
    count: usize,
}

impl<S, L> Deref for ColorMap<S, L> {
    type Target = ColorMapSettings<L>;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl<T: Copy, L> DitherCounter<T> for LabColorMap<T, L> {
    fn color(&self) -> Lab<T> {
        self.lab
    }

    fn add_pixel(&mut self) {
        self.color_map.weight += 1;
    }
}
impl<S: Float, L: Copy> ColorMap<S, L> {
    fn new(image: &Image<S>, settings: ColorMapSettings<L>, lab: Lab<S>) -> Self {
        Self {
            data: image
                .pixels()
                .iter()
                .map(|pixel_color| S::SQRT140050 - pixel_color.distance(lab))
                .collect(),
            curr_nail: settings.nail,
            curr_link: settings.link,
            settings,
            weight: 0,
            count: 0,
        }
    }

    fn calculate_weight(&self, segment: Segment<S>, grid: Grid) -> S {
        let mut weight = S::ZERO;
        let mut count = S::ZERO;
        for idx in grid.get_pixel_indexes_in_segment(segment) {
            let delta = unsafe { *self.data.get_unchecked(idx) };
            weight += delta;
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
struct NextLine<S, L> {
    line: *mut BakedSegment<S>,
    color_idx: usize,

    to_nail: usize,
    to_link: L,
    from_nail: usize,
    from_link: L,
}

unsafe impl<S: Send, L: Send> Send for NextLine<S, L> {}
unsafe impl<S: Sync, L: Sync> Sync for NextLine<S, L> {}

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

struct NextLineWeighted<S, L> {
    next: NextLine<S, L>,
    weight: S,
}

impl<S: Float, L> Default for NextLineWeighted<S, L> {
    #[allow(invalid_value)]
    fn default() -> Self {
        Self {
            //SAFETY: a -infinity weight will never be selected as best weight
            next: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            weight: -S::INFINITY,
        }
    }
}

struct BatchBuffer<S, L> {
    range: Range<usize>,
    result: NextLineWeighted<S, L>,
}

#[derive(Clone, Copy)]
struct BakedSegment<S> {
    segment: Segment<S>,
    used: bool,
}

struct ColorGroups {
    groups: Vec<ColorGroup>,
}

impl ColorGroups {
    fn new<S: Float, L>(
        threads: S,
        pixels_count: usize,
        color_maps: &mut [LabColorMap<S, L>],
        indexes: &[&[usize]],
    ) -> Self
    where
        usize: AsPrimitive<S>,
    {
        for &indexes in indexes{
            for &index in indexes{
                unsafe { color_maps.get_unchecked_mut(index) }.group_count += 1;
            }
        }
        Self {
            groups: indexes.into_iter()
                .map(|indexes| ColorGroup::new(threads, pixels_count, color_maps, indexes.iter().copied()))
                .collect(),
        }
    }

    fn select_next(&mut self) -> Option<usize> {
        while let Some(last) = self.groups.last_mut() {
            if let Some(res) = last.select_next() {
                return Some(res);
            } else {
                self.groups.pop();
            }
        }

        None
    }
}

struct ColorGroup {
    colors: Vec<(usize, usize)>,
}

impl ColorGroup {
    fn new<S: Float, L>(
        threads: S,
        pixels_count: usize,
        color_maps: &[LabColorMap<S, L>],
        indexes: impl Iterator<Item = usize>,
    ) -> Self
    where
        usize: AsPrimitive<S>,
    {
        Self {
            colors: indexes
                .map(|idx| {
                    let color = unsafe { color_maps.get_unchecked(idx) };
                    let prop: S =
                        color.color_map.weight.as_() / (pixels_count * color.group_count).as_();
                    (0, (threads * prop).to_usize().unwrap())
                })
                .collect(),
        }
    }

    fn select_next(&mut self) -> Option<usize> {
        let mut choice = None;
        let mut best_ratio = 1.0;
        for (color_idx, (count, cap)) in self.colors.iter_mut().enumerate() {
            let ratio = *count as f32 / *cap as f32;
            if ratio < best_ratio {
                best_ratio = ratio;
                choice = Some((color_idx, count));
            }
        }

        choice.map(|(idx, count)| {
            *count += 1;
            idx
        })
    }
}

pub struct Algorithm<S, N: nails::Handle, D> {
    color_maps: Vec<ColorMap<S, N::Link>>,
    grid: Grid,
    handle: N,
    distancer: NailDistancer,
    steps: Vec<NextLine<S, N::Link>>,
    darkness: D,
    nails: Vec<N::Nail>,
    lines: Vec<BakedSegment<S>>,
    buffers: Vec<BatchBuffer<S, N::Link>>,
}

impl<S: Float, N: nails::Handle<Scalar = S>, D: Darkness<S>> Algorithm<S, N, D> {
    #[must_use]
    pub fn new(
        mut table: Table<S, N>,
        palette: impl IntoIterator<Item = ColorMapSettings<N::Link>>,
        min_nail_distance: usize,
        darkness: D,
        groups: &[impl AsRef<[usize]>],
        threads: usize,
    ) -> Result<Self, Error>
    where
        u8: AsPrimitive<S>,
    {
        let nail_count = table.nails.len();
        if nail_count < 2 * min_nail_distance {
            return Err(Error::InvalidDistance(nail_count / 2));
        }
        let buffer_count = num_cpus::get();
        let chunk_size = (nail_count + buffer_count - 1) / buffer_count;
        let distancer = NailDistancer {
            min_nail_distance,
            max_nail_distance: nail_count - min_nail_distance,
        };

        let mut color_maps = palette
            .into_iter()
            .map(|settings| {
                if settings.nail >= nail_count {
                    Err(Error::InvalidInitialNail)
                } else {
                    Ok(LabColorMap::new(&table.image, settings))
                }
            })
            .collect::<Result<Vec<LabColorMap<S, N::Link>>, Error>>()?;
        Ditherer::floyd_steinberg(color_maps.as_mut_slice()).dither(&mut table.image)?;
        Ok(Self {
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
            color_maps: color_maps.into_iter().map(|map| map.color_map).collect(),
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
        })
    }

    #[must_use]
    fn select_next(
        colors: &mut [ColorMap<S, N::Link>],
    ) -> Result<(usize, &ColorMap<S, N::Link>), Error>
    where
        usize: AsPrimitive<S>,
    {
        let mut iter = colors.iter_mut().enumerate();

        if let Some((mut best_idx, mut best_color)) = iter.next() {
            let mut best_ratio = best_color.count.as_() / best_color.weight.as_();
            while let Some((idx, color)) = iter.next() {
                let ratio = color.count.as_() / color.weight.as_();
                if ratio < best_ratio {
                    best_ratio = ratio;
                    best_idx = idx;
                    best_color = color
                }
            }
            best_color.count += 1;
            Ok((best_idx, best_color))
        } else {
            Err(Error::EmptyPalette)
        }
        // let mut selected_idx = (core::ptr::null_mut(), ;
        // let mut min_ratio = S::INFINITY;

        // for (idx, s) in colors.iter_mut().enumerate() {
        //     let ratio = s.count.as_() / s.weight.as_();
        //     if ratio < min_ratio {
        //         min_ratio = ratio;
        //         selected_idx = s;
        //     }
        // }

        // let res = unsafe { &mut *selected_idx };
        // res.count += 1;
    }

    pub fn build_svg(&self, line_tickness: f32) -> svg::Document {
        let mut doc =
            svg::Document::new().set("viewBox", (0.0, 0.0, self.grid.width, self.grid.height));
        for &nail in self.nails.iter() {
            doc = doc.add(self.handle.draw_svg(nail))
        }

        for step in self.steps.iter().rev() {
            let segment = unsafe { (*step.line).segment };
            let color = unsafe { self.color_maps.get_unchecked(step.color_idx) }.color;
            doc = doc.add(
                svg::node::element::Line::new()
                    .set("x1", segment.start.x)
                    .set("y1", segment.start.y)
                    .set("x2", segment.end.x)
                    .set("y2", segment.end.y)
                    .set(
                        "stroke",
                        format!("rgb({}, {}, {})", color.0, color.1, color.2),
                    )
                    .set("stroke-width", line_tickness),
            );
        }
        doc
    }

    pub fn build_rgb(&self, resolution: Grid<usize>) -> RgbImage
    where
        usize: AsPrimitive<S>,
    {
        let h_res = Grid::<S> {
            height: resolution.height.as_(),
            width: resolution.width.as_(),
        };
        let s_grid = Grid::<S> {
            height: self.grid.height.as_(),
            width: self.grid.width.as_(),
        };
        let scale = (h_res.height / s_grid.height).min(h_res.width / s_grid.width);
        let grid = Grid::<u32> {
            height: (s_grid.height * scale)
                .to_u32()
                .expect("this image is so big to be saved!"),
            width: (s_grid.width * scale)
                .to_u32()
                .expect("this image is so big to be saved!"),
        };
        let mut buffer: Vec<u8> = vec![255; grid.height as usize * grid.width as usize * 3];
        for step in self.steps.iter().rev() {
            let mut segment = (unsafe { *step.line }).segment;
            let color = unsafe { self.color_maps.get_unchecked(step.color_idx) }.color;
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
        unsafe { RgbImage::from_vec(grid.width, grid.height, buffer).unwrap_unchecked() }
    }

    pub fn build_instructions(&self) -> String
    where
        N: nails::Handle<Link: ToString>,
    {
        let mut instructions = String::new();
        let iter = self.steps.iter().rev();

        let mut init_nails = vec![None; self.color_maps.len()];
        let mut init_iter = iter.clone();
        let mut done = 0;
        while done < self.color_maps.len() {
            match init_iter.next() {
                Some(step) => {
                    let nail = unsafe { init_nails.get_unchecked_mut(step.color_idx) };
                    if nail.is_none() {
                        *nail = Some((
                            unsafe { self.color_maps.get_unchecked(step.color_idx).name.as_str() },
                            step.to_nail,
                            step.to_link,
                        ));
                        done += 1;
                    }
                }
                None => break,
            }
        }

        instructions.extend(init_nails.into_iter().filter_map(|step| {
            step.map(|init_nail| {
                format!(
                    "{} {} {} \n",
                    init_nail.0,
                    init_nail.1,
                    init_nail.2.to_string()
                )
            })
        }));

        instructions.extend(iter.map(|step| {
            format!(
                "{} {} {} \n",
                unsafe { self.color_maps.get_unchecked(step.color_idx).name.as_str() },
                step.from_nail,
                step.from_link.to_string()
            )
        }));
        instructions
    }

    #[must_use]
    pub fn compute(&mut self, step_count: usize) -> Result<(), Error>
    where
        usize: AsPrimitive<S>,
    {
        while step_count > self.steps.len() {
            if let Some(next) = self.get_best_line()? {
                let color_map = unsafe { self.color_maps.get_unchecked_mut(next.color_idx) };
                let line = unsafe { &mut *next.line };
                line.used = true;

                color_map.curr_link = self.handle.get_next_link(next.to_link);
                color_map.curr_nail = next.to_nail;
                for point in self.grid.get_pixel_indexes_in_segment(line.segment) {
                    let weight = unsafe { color_map.data.get_unchecked_mut(point) };
                    *weight = self.darkness.compute(*weight);
                }
                self.steps.push(next);
            } else {
                return Ok(());
            }
        }

        Ok(())
    }

    fn get_best_line(&mut self) -> Result<Option<NextLine<S, N::Link>>, Error>
    where
        usize: AsPrimitive<S>,
    {
        #[derive(Copy, Clone)]
        struct SyncColorMapRef<T, L>(*mut ColorMap<T, L>);

        impl<S, L> From<SyncColorMapRef<S, L>> for *mut ColorMap<S, L> {
            fn from(value: SyncColorMapRef<S, L>) -> Self {
                value.0
            }
        }

        impl<S, L> Deref for SyncColorMapRef<S, L> {
            type Target = ColorMap<S, L>;

            fn deref(&self) -> &Self::Target {
                unsafe { &*self.0 }
            }
        }

        unsafe impl<S: Sync, L: Sync> Sync for SyncColorMapRef<S, L> {}
        unsafe impl<S: Send, L: Send> Send for SyncColorMapRef<S, L> {}

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
        let (color_idx, color_map) = Self::select_next(self.color_maps.as_mut_slice())?;
        let line_table = SyncLineTable(&mut self.lines);
        self.buffers.par_iter_mut().for_each(|buffer| {
            buffer.result = Default::default();
            for to_nail in buffer.range.clone() {
                if self.distancer.is_valid(color_map.curr_nail, to_nail) {
                    for to_link in N::LINKS {
                        let line_idx = self.distancer.index_of::<N::Links>(
                            color_map.curr_nail,
                            color_map.curr_link,
                            to_nail,
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
                                    color_idx,
                                    to_nail,
                                    to_link,
                                    from_nail: color_map.curr_nail,
                                    from_link: color_map.curr_link,
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

        Ok(best_line)
    }
}

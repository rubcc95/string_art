use crate::{
    color::{self, config::{Config, Handle as _}},
    darkness::Darkness,
    image::Image,
    nail_table::{self, BakedNailTable, BakedSegment},
    nails,
    slice::{Slice, SliceOwner},
    verboser::{Message, Verboser},
    Float, Grid, NailTable,
};
use num_traits::{AsPrimitive, ToPrimitive};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::ops::Range;

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

impl<S, L> BatchBuffer<S, L> {
    pub fn new(nail_count: usize) -> Vec<Self> {
        let buffer_count = num_cpus::get();
        let chunk_size = (nail_count + buffer_count - 1) / buffer_count;
        (0..buffer_count)
            .map(|index| {
                let start = index * chunk_size;
                BatchBuffer {
                            range: start..nail_count.min(start + chunk_size),
                            #[allow(invalid_value)]
                            //Reason: will be initialized as default on get_best_line() before use
                            result: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
                        }
            })
            .collect()
    }
}

pub fn compute<'a, N: 'a + nails::Handle, P: Config<'a, N::Link, N::Scalar>>(
    table: NailTable<N>,
    min_nail_distance: usize,
    image: &Image<N::Scalar>,
    palette: P,
    darkness: impl Darkness<N::Scalar>,
    contrast: N::Scalar,
    blur_radius: usize,
    verboser: &mut impl Verboser,
) -> Result<
    Computation<
        <<P::Handle as color::config::Handle<'a, N::Link, N::Scalar>>::Owner as SliceOwner<'a>>::Map<
            'a,
            color::Named,
        >,
        N,
    >,
    Error<N::Error, P::Error>,
>
where
    usize: AsPrimitive<N::Scalar>,
{
    let a = palette
        .into_color_handle(image, table.nails().len(), blur_radius, contrast)
        .map_err(Error::ColorConfig)?;
    let baked = BakedNailTable::new(table, min_nail_distance).map_err(Error::NailTable)?;
    let mut algorithm = Algorithm::new(a, baked, *image.grid());
    algorithm.compute(verboser, darkness);
    Ok(Computation {
        colors: algorithm.color_maps.into_colors().map(color::Named::from),
        table: algorithm.table,
        steps: algorithm.steps,
        grid: algorithm.grid,
    })
}

pub struct Computation<C, N: nails::Handle> {
    colors: C,
    table: BakedNailTable<N>,
    steps: Vec<NextLine<N::Scalar, N::Link>>,
    grid: Grid,
}

impl<'a, C: SliceOwner<'a, Item = color::Named>, N: nails::Handle> Computation<C, N> {
    pub fn build_svg(&self, line_tickness: f32) -> svg::Document {
        let mut doc =
            svg::Document::new().set("viewBox", (0.0, 0.0, self.grid.width, self.grid.height));
        for &nail in self.table.nails().iter() {
            doc = doc.add(self.table.handle().draw_svg(nail))
        }

        for step in self.steps.iter().rev() {
            let segment = unsafe { (*step.line).segment() };
            let color = unsafe { self.colors.as_slice().get_unchecked(step.color_idx) }.value;
            doc = doc.add(
                svg::node::element::Line::new()
                    .set("x1", format!("{:.4}", segment.start.x))
                    .set("y1", format!("{:.4}", segment.start.y))
                    .set("x2", format!("{:.4}", segment.end.x))
                    .set("y2", format!("{:.4}", segment.end.y))
                    .set(
                        "stroke",
                        format!("rgb({:.4}, {:.4}, {:.4})", color.0, color.1, color.2),
                    )
                    .set("stroke-width", format!("{:.4}", line_tickness))
                    .set("opacity", 1),
            );
        }
        doc
    }

    pub fn build_rgb(&self, resolution: Grid<usize>) -> image::RgbImage
    where
        usize: AsPrimitive<N::Scalar>,
    {
        let h_res = Grid::<N::Scalar> {
            height: resolution.height.as_(),
            width: resolution.width.as_(),
        };
        let s_grid = Grid::<N::Scalar> {
            height: self.grid.height.as_(),
            width: self.grid.width.as_(),
        };
        let scale = Float::min(h_res.height, h_res.width) / s_grid.height;
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
            let mut segment = *(unsafe { *step.line }).segment();
            let color = unsafe { self.colors.as_slice().get_unchecked(step.color_idx) }.value;
            segment *= scale;
            for idx in grid.get_pixel_indexes_in_segment(&segment) {
                unsafe {
                    let ptr = buffer.as_mut_ptr().add(3 * idx as usize);
                    *ptr = color.0;
                    *ptr.add(1) = color.1;
                    *ptr.add(2) = color.2;
                }
            }
        }
        unsafe { image::RgbImage::from_vec(grid.width, grid.height, buffer).unwrap_unchecked() }
    }

    pub fn build_instructions(&self) -> String
    where
        N: nails::Handle<Link: ToString>,
    {
        let mut instructions = String::new();
        let iter = self.steps.iter().rev();

        let mut init_nails = self.colors.as_slice().map(|_| None);
        let mut init_iter = iter.clone();
        let mut done = 0;
        while done < init_nails.len() {
            match init_iter.next() {
                Some(step) => {
                    let nail =
                        unsafe { init_nails.as_mut_slice().get_unchecked_mut(step.color_idx) };
                    if nail.is_none() {
                        *nail = Some((
                            unsafe {
                                self.colors
                                    .as_slice()
                                    .get_unchecked(step.color_idx)
                                    .name
                                    .as_str()
                            },
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
                unsafe {
                    self.colors
                        .as_slice()
                        .get_unchecked(step.color_idx)
                        .name
                        .as_str()
                },
                step.from_nail,
                step.from_link.to_string()
            )
        }));
        instructions
    }
}

struct Algorithm<C, N: nails::Handle> {
    color_maps: C,
    table: BakedNailTable<N>,
    steps: Vec<NextLine<N::Scalar, N::Link>>,
    buffers: Vec<BatchBuffer<N::Scalar, N::Link>>,
    grid: Grid,
}

impl<'a, C: color::config::Handle<'a, N::Link, N::Scalar>, N: 'a + nails::Handle> Algorithm<C, N> {
    fn compute<D: Darkness<N::Scalar>>(&mut self, verboser: &mut impl Verboser, darkness: D)
    where
        usize: AsPrimitive<N::Scalar>,
    {
        loop {
            verboser.verbose(Message::Computing(self.steps.len()));
            if let Some(next) = self.get_best_line() {
                let color_map = unsafe {
                    self.color_maps
                        .colors_mut()
                        .get_unchecked_mut(next.color_idx)
                };
                let line = unsafe { &mut *next.line };

                for point in self.grid.get_pixel_indexes_in_segment(line.segment()) {
                    let weight = unsafe { color_map.weights().get_unchecked_mut(point) };
                    *weight = darkness.compute(*weight);
                }

                color_map.link = self.table.handle().get_next_link(next.to_link);
                color_map.nail = next.to_nail;
                line.mark_used();
                self.steps.push(next);
            } else {
                return;
            }
        }
    }

    fn get_best_line(&mut self) -> Option<NextLine<N::Scalar, N::Link>>
    where
        usize: AsPrimitive<N::Scalar>,
    {
        struct SyncLineTable<S>(*mut [BakedSegment<S>]);

        impl<S> SyncLineTable<S> {
            pub unsafe fn get_unchecked(&self, idx: usize) -> *mut BakedSegment<S> {
                (*self.0).get_unchecked_mut(idx)
            }
        }

        unsafe impl<S: Sync> Sync for SyncLineTable<S> {}
        unsafe impl<S: Send> Send for SyncLineTable<S> {}

        self.color_maps.select_next().and_then(|color_idx| {
            let color_map = unsafe { self.color_maps.colors().get_unchecked(color_idx) };
            let mut best_weight = -N::Scalar::INFINITY;
            let mut best_line = None;

            let line_table = SyncLineTable(self.table.segments_mut());
            self.buffers.par_iter_mut().for_each(|buffer| {
                buffer.result = Default::default();
                for to_nail in buffer.range.clone() {
                    if self.table.distancer().is_valid(color_map.nail, to_nail) {
                        for to_link in N::LINKS {
                            let line_idx = unsafe {
                                self.table.distancer().index_of_unchecked::<N::Links>(
                                    color_map.nail,
                                    color_map.link,
                                    to_nail,
                                    to_link,
                                )
                            };
                            let line: *mut _ = unsafe { line_table.get_unchecked(line_idx) };
                            let segment = unsafe { &*line };
                            if segment.is_used() {
                                continue;
                            }

                            let weight = color_map.calculate_weight(&segment, &self.grid);
                            if weight > buffer.result.weight {
                                buffer.result = NextLineWeighted {
                                    weight,
                                    next: NextLine {
                                        line,
                                        color_idx,
                                        to_nail,
                                        to_link,
                                        from_nail: color_map.nail,
                                        from_link: color_map.link,
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
        })
    }
}

impl<C, N: nails::Handle> Algorithm<C, N> {
    fn new(color_maps: C, table: BakedNailTable<N>, grid: Grid) -> Self {
        Self {
            buffers: BatchBuffer::new(table.nails().len()),
            color_maps,
            table,
            steps: Vec::new(),
            grid,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error<N, C> {
    NailTable(nail_table::Error<N>),
    ColorConfig(C),
}
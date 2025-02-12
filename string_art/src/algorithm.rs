use crate::{
    color::{self, config::Config},
    darkness::Darkness,
    geometry::Rect,
    image::Image,
    nail_table::{BakedSegment, NailTable},
    nails,
    slice::{Slice, SliceOwner},
    verboser::{Message, Verboser},
    Float,
};
use image::imageops::ColorMap;
use num_traits::AsPrimitive;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::ops::Range;

pub fn compute<'a, N, B, P>(
    table: B,
    image: &Image<N::Scalar>,
    palette: P,
    darkness: impl Darkness<N::Scalar>,
    contrast: N::Scalar,
    blur_radius: usize,
    verboser: &mut impl Verboser,
) -> Result<
    Computation<
    N,
        B,
        <<P::Handle<B::Id, N::Link> as color::config::Handle<'a, B::Id, N::Link, N::Scalar>>::Owner as SliceOwner<'a>>::Map<
            'a,
            color::Named,
        >,
    >,
    Error<N::Error, P::Error>,
>
where
    usize: AsPrimitive<N::Scalar>,
    N: 'a + nails::Handle, 
    B: NailTable<Handle = N>, 
    P: Config<'a, N::Scalar>
{
    let a = palette
        .into_color_handle(image, blur_radius, contrast)
        .map_err(Error::ColorConfig)?;
    Ok(Algorithm::new(a, table, *image.rect()).compute(verboser, darkness))
}

pub struct Computation<N: nails::Handle, B: NailTable, C> {
    colors: C,
    table: B,
    steps: Vec<NextLine<B::Id, N::Scalar, N::Link>>,
    rect: Rect,
}

impl<'a, N: nails::Handle, B: NailTable<Handle = N>, C: SliceOwner<'a, Item = color::Named>>
    Computation<N, B, C>
{
    pub fn build_svg(&self, line_tickness: f32) -> svg::Document {
        let mut doc =
            svg::Document::new().set("viewBox", (0.0, 0.0, self.rect.width, self.rect.height));
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

    pub fn build_rgb(&self, resolution: Rect<usize>) -> image::RgbImage
    where
        usize: AsPrimitive<N::Scalar>,
    {
        let h_res = Rect::<N::Scalar> {
            height: resolution.height.as_(),
            width: resolution.width.as_(),
        };
        let s_rect: Rect<N::Scalar> = self.rect.as_();
        let scale = Float::min(h_res.height, h_res.width) / s_rect.height;
        let rect: Rect<u32> = s_rect.cast().unwrap();
        let mut buffer: Vec<u8> = vec![255; rect.height as usize * rect.width as usize * 3];
        for step in self.steps.iter().rev() {
            let mut segment = *(unsafe { *step.line }).segment();
            let color = unsafe { self.colors.as_slice().get_unchecked(step.color_idx) }.value;
            segment *= scale;
            for idx in rect.get_pixel_indexes_in_segment(&segment) {
                unsafe {
                    let ptr = buffer.as_mut_ptr().add(3 * idx as usize);
                    *ptr = color.0;
                    *ptr.add(1) = color.1;
                    *ptr.add(2) = color.2;
                }
            }
        }
        unsafe { image::RgbImage::from_vec(rect.width, rect.height, buffer).unwrap_unchecked() }
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
                    init_nail.1.to_string(),
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
                step.from_nail.to_string(),
                step.from_link.to_string()
            )
        }));
        instructions
    }
}

struct Algorithm<N: nails::Handle, B: NailTable, C> {
    color_maps: C,
    table: B,
    steps: Vec<NextLine<B::Id, N::Scalar, N::Link>>,
    buffers: Vec<BatchBuffer<B::Id, N::Scalar, N::Link>>,
    rect: Rect,
}

impl<N: nails::Handle, B: NailTable, C> Algorithm<N, B, C> {
    fn new(color_maps: C, table: B, rect: Rect) -> Self {
        Self {
            buffers: BatchBuffer::new(),
            color_maps,
            table,
            steps: Vec::new(),
            rect,
        }
    }
}

impl<'a, N, B, C> Algorithm<N, B, C>
where
    B: NailTable<Handle = N, Id: 'a>,
    C: color::config::Handle<'a, B::Id, N::Link, N::Scalar>,
    N: 'a + nails::Handle,
    usize: AsPrimitive<N::Scalar>,
{
    #[inline]
    fn compute<D: Darkness<N::Scalar>>(
        mut self,
        verboser: &mut impl Verboser,
        darkness: D,
    ) -> Computation<N, B, <C::Owner as SliceOwner<'a>>::Map<'a, color::Named>> {
        for idx in 0..self.color_maps.colors_mut().len() {
            let next = self.get_best_line_for_idx(idx);
            let color_map = unsafe {
                self.color_maps
                    .colors_mut()
                    .get_unchecked_mut(next.color_idx)
            };
            color_map.link = next.to_link;
            color_map.nail = next.to_nail;
        }
        loop {
            verboser.verbose(Message::Computing(self.steps.len()));
            if let Some(next) = self.get_best_line() {
                let color_map = unsafe {
                    self.color_maps
                        .colors_mut()
                        .get_unchecked_mut(next.color_idx)
                };
                let line = unsafe { &mut *next.line };

                for point in self.rect.get_pixel_indexes_in_segment(line.segment()) {
                    let weight = unsafe { color_map.weights().get_unchecked_mut(point) };
                    *weight = darkness.compute(*weight);
                }

                color_map.link = self.table.handle().get_next_link(next.to_link);
                color_map.nail = next.to_nail;
                line.mark_used();
                self.steps.push(next);
            } else {
                return Computation {
                    colors: self.color_maps.into_colors().map(color::Named::from),
                    table: self.table,
                    steps: self.steps,
                    rect: self.rect,
                };
            }
        }
    }

    fn get_best_line_for_idx(&mut self, color_idx: usize) -> NextLine<B::Id, N::Scalar, N::Link> {
        struct SyncLineTable<B>(*mut B);

        impl<B: NailTable> SyncLineTable<B> {
            pub unsafe fn get_segment(
                &self,
                nail_idx: B::Id,
                link: <B::Handle as nails::Handle>::Link,
                offset: usize,
                other_link: <B::Handle as nails::Handle>::Link,
            ) -> (
                B::Id,
                &mut BakedSegment<<B::Handle as nails::Handle>::Scalar>,
            ) {
                (*self.0).segment_idx(nail_idx, link, offset, other_link)
            }
        }

        unsafe impl<S: Sync> Sync for SyncLineTable<S> {}
        unsafe impl<S: Send> Send for SyncLineTable<S> {}

        let color_map = unsafe { self.color_maps.colors().get_unchecked(color_idx) };

        let line_table = SyncLineTable(&mut self.table);
        let nail_count = unsafe { self.table.comb_count(color_map.nail) };
        let chunk_size = (nail_count + self.buffers.len() - 1) / self.buffers.len();
        for (index, buffer) in self.buffers.iter_mut().enumerate() {
            let start = index * chunk_size;
            buffer.range = start..nail_count.min(start + chunk_size);
        }
        self.buffers.par_iter_mut().for_each(|buffer| {
            buffer.result = Default::default();
            for offset in buffer.range.clone() {
                for to_link in N::LINKS {
                    let (to_nail, segment) = unsafe {
                        line_table.get_segment(color_map.nail, color_map.link, offset, to_link)
                    };

                    if segment.is_used() {
                        continue;
                    }

                    let weight = color_map.calculate_weight(&segment, &self.rect);
                    if weight > buffer.result.weight {
                        buffer.result = NextLineWeighted {
                            weight,
                            next: NextLine {
                                line: segment,
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
        });
        let result = unsafe { self.buffers.get_unchecked(0) }.result();
        let mut best_weight = result.weight;
        let mut best_line = result.next;

        for BatchBuffer { range: _, result } in unsafe { self.buffers.get_unchecked(1..) } {
            if result.weight > best_weight {
                best_weight = result.weight;
                best_line = result.next;
            }
        }

        best_line
    }

    fn get_best_line(&mut self) -> Option<NextLine<B::Id, N::Scalar, N::Link>> {
        self.color_maps
            .select_next()
            .map(|color_idx| self.get_best_line_for_idx(color_idx))
    }
}

#[derive(Copy, Clone)]
struct NextLine<I, S, L> {
    line: *mut BakedSegment<S>,
    color_idx: usize,
    to_nail: I,
    to_link: L,
    from_nail: I,
    from_link: L,
}

unsafe impl<I: Send, S: Send, L: Send> Send for NextLine<I, S, L> {}
unsafe impl<I: Sync, S: Sync, L: Sync> Sync for NextLine<I, S, L> {}

struct NextLineWeighted<I, S, L> {
    next: NextLine<I, S, L>,
    weight: S,
}

impl<I: Copy, S: Float, L: Copy> Default for NextLineWeighted<I, S, L> {
    #[allow(invalid_value)]
    fn default() -> Self {
        Self {
            next: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            weight: -S::INFINITY,
        }
    }
}

struct BatchBuffer<I, S, L> {
    range: Range<usize>,
    result: NextLineWeighted<I, S, L>,
}

impl<I: Copy, S: Copy, L: Copy> BatchBuffer<I, S, L> {
    pub fn new() -> Vec<Self> {
        // SAFETY: Both structs will be initialized on get_best_line() before any read.
        // Since S and L are copy, BatchBuffer has not a relevant drop and it is safe to Drop it uninit.
        let mut vec = Vec::with_capacity(num_cpus::get().max(1));
        unsafe { vec.set_len(vec.capacity()) };
        vec
    }

    #[inline]
    pub fn result(&self) -> &NextLineWeighted<I, S, L> {
        &self.result
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error<N, C> {
    NailTable(N),
    ColorConfig(C),
}

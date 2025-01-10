use std::ops::{Deref, Range};

use image::{ImageResult, RgbImage};
use num_traits::AsPrimitive;
use palette::{color_difference::EuclideanDistance, FromColor, Srgb};
use rand::{
    distributions::Standard,
    prelude::{Distribution, SmallRng},
    Rng, SeedableRng,
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    colors::{Lab, LabImage},
    ditherer::{Ditherer, WeightedColor},
    geometry::{Point, Segment},
    hooks::{self, Links},
    Float, Grid,
};

pub trait HookLogicBuilder {
    type Logic;
    type Buffered;

    fn build_logic(&mut self, hook_count: usize) -> Self::Logic;

    fn build_buffer(&mut self) -> Self::Buffered;
}

pub trait HookLogic<H: hooks::Handle>: Copy {
    type Current: Copy;
    type Buffered: Send + Sync;

    fn get_start_hooks(
        self,
        current: Self::Current,
        buffered: &mut Self::Buffered,
    ) -> impl Iterator<Item = (usize, H::Link)>;

    fn register_end(self, handle: H, end_hook: usize, end_link: H::Link) -> Self::Current;
}

pub struct ExhaustiveHookLogic;

impl HookLogicBuilder for ExhaustiveHookLogic {
    type Logic = ExhaustiveHookLogicHandle;
    type Buffered = ();

    fn build_logic(&mut self, hook_count: usize) -> Self::Logic {
        ExhaustiveHookLogicHandle(hook_count)
    }

    fn build_buffer(&mut self) -> Self::Buffered {
        ()
    }
}

pub struct RandomizedHookLogic {
    seed_builder: SmallRng,
    iterations: usize,
}

impl RandomizedHookLogic {
    pub fn from_seed(seed: [u8; 32], iterations: usize) -> Self {
        Self {
            seed_builder: SmallRng::from_seed(seed),
            iterations,
        }
    }

    pub fn new(iterations: usize) -> Self {
        Self {
            seed_builder: SmallRng::from_entropy(),
            iterations,
        }
    }
}

impl HookLogicBuilder for RandomizedHookLogic {
    type Logic = RandomizedHookLogicHandle;

    type Buffered = SmallRng;

    fn build_logic(&mut self, hook_count: usize) -> Self::Logic {
        RandomizedHookLogicHandle {
            hook_count,
            iterations: self.iterations,
        }
    }

    fn build_buffer(&mut self) -> Self::Buffered {
        SmallRng::from_rng(&mut self.seed_builder).unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct RandomizedHookLogicHandle {
    hook_count: usize,
    iterations: usize,
}

impl<H: hooks::Handle> HookLogic<H> for RandomizedHookLogicHandle
where
    Standard: Distribution<H::Link>,
{
    type Current = ();
    type Buffered = SmallRng;

    fn get_start_hooks(
        self,
        _: Self::Current,
        rng: &mut Self::Buffered,
    ) -> impl Iterator<Item = (usize, H::Link)> {
        (0..self.iterations)
            .into_iter()
            .map(move |_| (rng.gen_range(0..self.hook_count), rng.gen()))
    }

    fn register_end(self, _: H, _: usize, _: H::Link) -> Self::Current {}
}

#[derive(Clone, Copy)]
pub struct ExhaustiveHookLogicHandle(usize);

impl<H: hooks::Handle> HookLogic<H> for ExhaustiveHookLogicHandle {
    type Current = ();
    type Buffered = ();

    fn get_start_hooks(
        self,
        _: Self::Current,
        _: &mut Self::Buffered,
    ) -> impl Iterator<Item = (usize, H::Link)> {
        (0..self.0)
            .into_iter()
            .map(|idx| H::LINKS.into_iter().map(move |link| (idx, link)))
            .flatten()
    }

    fn register_end(self, _: H, _: usize, _: H::Link) -> Self::Current {}
}

#[derive(Clone, Copy)]
pub struct ContinuosHookLogic;

impl HookLogicBuilder for ContinuosHookLogic {
    type Logic = Self;
    type Buffered = ();

    fn build_logic(&mut self, _: usize) -> Self::Logic {
        *self
    }

    fn build_buffer(&mut self) -> Self::Buffered {}
}

impl<H: hooks::Handle> HookLogic<H> for ContinuosHookLogic {
    type Current = (usize, H::Link);
    type Buffered = ();
    fn get_start_hooks(
        self,
        current: Self::Current,
        _: &mut Self::Buffered,
    ) -> impl Iterator<Item = (usize, H::Link)> {
        core::iter::once(current)
    }

    fn register_end(self, handle: H, end_hook: usize, end_link: H::Link) -> Self::Current {
        (end_hook, handle.get_next_link(end_link))
    }
}

pub trait CurrentHook: Copy {
    type Link;
    type Handle: Copy;

    fn get_current_hooks(self, handle: Self::Handle) -> impl Iterator<Item = (usize, Self::Link)>;

    fn register_end(handle: Self::Handle, end: (usize, Self::Link)) -> Self;
}

pub trait ColorMapBuilder<T, L> {
    fn build(self) -> (Lab<T>, usize, L);
}

impl<T, L: Default> ColorMapBuilder<T, L> for Lab<T> {
    fn build(self) -> (Lab<T>, usize, L) {
        (self, 0, L::default())
    }
}

impl<T, L: Default> ColorMapBuilder<T, L> for (Lab<T>, usize) {
    fn build(self) -> (Lab<T>, usize, L) {
        (self.0, self.1, L::default())
    }
}

impl<T, L> ColorMapBuilder<T, L> for (Lab<T>, usize, L) {
    fn build(self) -> (Lab<T>, usize, L) {
        self
    }
}

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

impl<T: Float> Darkness<T> for PercentageDarkness<T> {
    fn compute(self, weight: T) -> T {
        self.0 * weight
    }
}

struct ColorMap<T, C> {
    color: Lab<T>,
    data: Vec<(T, T)>,
    current_hook: C,
    intial_hook: C,
    weight: usize,
    count: usize,
}

impl<S: Float, C: Copy> ColorMap<S, C> {
    fn new<H: hooks::Handle, L: HookLogic<H, Current = C>>(
        image: &LabImage<S>,
        builder: impl ColorMapBuilder<S, H::Link>,
        handle: H,
        logic: L,
    ) -> Self {
        let (color, initial_idx, initial_link) = builder.build();
        let hook = logic.register_end(handle, initial_idx, initial_link);
        Self {
            color,
            data: image
                .pixels()
                .iter()
                .map(|pixel_color| (S::ONE, S::SQRT140050 - pixel_color.distance(color)))
                .collect(),
            intial_hook: hook,
            current_hook: hook,
            weight: 0,
            count: 0,
        }
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

impl<T: Copy, L> WeightedColor<T> for ColorMap<T, L> {
    fn color(&self) -> Lab<T> {
        self.color
    }

    fn add_weight(&mut self) {
        self.weight += 1;
    }
}

#[derive(Copy, Clone)]
struct NextLine<S, L, C> {
    line: *mut BakedSegment<S>,
    color_map: *mut ColorMap<S, C>,
    end_link: L,
    end_idx: usize,
}

unsafe impl<S, L, C> Send for NextLine<S, L, C> {}
unsafe impl<S, L, C> Sync for NextLine<S, L, C> {}

#[derive(Clone, Copy)]
struct HookDistancer {
    min_hook_distance: usize,
    max_hook_distance: usize,
}

impl HookDistancer {
    fn is_valid(&self, a_idx: usize, b_idx: usize) -> bool {
        let hook_diff = a_idx.abs_diff(b_idx);
        hook_diff > self.min_hook_distance && hook_diff < self.max_hook_distance
    }

    fn hook_combs_iter<T, H: hooks::Handle>(
        &self,
        range: Range<usize>,
    ) -> impl Iterator<Item = ((usize, H::Link), (usize, H::Link))> {
        range
            .map(|big_idx| {
                H::LINKS
                    .into_iter()
                    .map(move |big_link| {
                        (0..big_idx)
                            .map(move |small_idx| {
                                H::LINKS.into_iter().map(move |small_link| {
                                    ((big_idx, big_link), (small_idx, small_link))
                                })
                            })
                            .flatten()
                    })
                    .flatten()
            })
            .flatten()
    }

    fn index_of<L: hooks::Links>(
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
        let cap = self.max_hook_distance - 1;
        let a = if big_idx > cap {
            let diff = big_idx - cap;
            big_idx -= diff;
            small_idx -= diff;
            diff * (cap - self.min_hook_distance) * L::SQ_LEN
        } else {
            0
        };

        let diff = big_idx - self.min_hook_distance;

        a + diff * (diff - 1) * L::SQ_LEN / 2
            + L::LEN * diff * big_link.into()
            + L::LEN * small_idx
            + small_link.into()
    }
}

struct NextLineWeighted<S, L, C> {
    next: NextLine<S, L, C>,
    weight: S,
}

impl<T: Float, L, C> Default for NextLineWeighted<T, L, C> {
    #[allow(invalid_value)]
    fn default() -> Self {
        Self {
            //SAFETY: a -infinity weight will never be selected as best weight
            next: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            weight: -T::INFINITY,
        }
    }
}

#[derive(Clone)]
pub struct Table<S, H: hooks::Handle> {
    hooks: Vec<H::Hook>,
    handle: H,
    image: LabImage<S>,
}

impl<S: Float, H: hooks::Handle> Table<S, H>
where
    usize: AsPrimitive<S>,
{
    pub fn ellipse(
        image: LabImage<S>,
        hook_builder: impl hooks::Builder<Scalar = S, Handle = H, Hook = H::Hook>,
        hook_count: usize,
    ) -> Self {
        Self {
            hooks: (0..hook_count)
                .into_iter()
                .map(|i| {
                    let theta: S = S::TWO * S::PI * (i.as_()) / (hook_count.as_());
                    hook_builder.build_hook(
                        Point {
                            x: image.width.as_() * (S::ONE + theta.cos()),
                            y: image.height.as_() * (S::ONE + theta.sin()),
                        } * S::HALF,
                        theta,
                    )
                })
                .collect(),
            handle: hook_builder.build_handle(),
            image,
        }
    }
}

struct BatchBuffer<S, H: hooks::Handle, L: HookLogic<H>> {
    range: Range<usize>,
    result: NextLineWeighted<S, H::Link, L::Current>,
    logic_buffer: L::Buffered,
}

#[derive(Clone, Copy)]
struct BakedSegment<S> {
    segment: Segment<S>,
    used: bool,
}

pub struct StringArt<S, H: hooks::Handle, D, L: HookLogic<H>> {
    color_maps: Vec<ColorMap<S, L::Current>>,
    grid: Grid,
    handle: H,
    distancer: HookDistancer,
    steps: Vec<NextLine<S, H::Link, L::Current>>,
    darkness: D,
    logic: L,
    lines: Vec<BakedSegment<S>>,
    buffers: Vec<BatchBuffer<S, H, L>>,
}

impl<S: Float, H: hooks::Handle<Scalar = S>, D: Darkness<S>, L: HookLogic<H>>
    StringArt<S, H, D, L>
{
    pub fn new(
        mut table: Table<S, H>,
        palette: impl IntoIterator<Item = impl ColorMapBuilder<S, H::Link>>,
        min_hook_distance: usize,
        darkness: D,
        mut logic_builder: impl HookLogicBuilder<Logic = L, Buffered = L::Buffered>,
    ) -> Self {
        let hook_count = table.hooks.len();
        let buffer_count = num_cpus::get();
        let chunk_size = (hook_count + buffer_count - 1) / buffer_count;
        let distancer = HookDistancer {
            min_hook_distance,
            max_hook_distance: hook_count - min_hook_distance,
        };
        let logic = logic_builder.build_logic(hook_count);
        let mut color_maps: Vec<ColorMap<S, L::Current>> = palette
            .into_iter()
            .map(|builder| ColorMap::new(&table.image, builder, table.handle, logic))
            .collect();
        Ditherer::floyd_steinberg(color_maps.as_mut_slice()).dither(&mut table.image);
        Self {
            lines: distancer
                .hook_combs_iter::<S, H>(0..hook_count)
                .map(|((a_idx, a_link), (b_idx, b_link))| BakedSegment {
                    segment: table.handle.get_segment(
                        (unsafe { table.hooks.get_unchecked(a_idx) }, a_link),
                        (unsafe { table.hooks.get_unchecked(b_idx) }, b_link),
                    ),
                    used: false,
                })
                .collect(),
            darkness,
            color_maps,

            grid: *table.image.deref(),
            handle: table.handle,
            distancer,
            steps: Vec::new(),
            buffers: (0..hook_count)
                .step_by(chunk_size)
                .map(|start| BatchBuffer {
                range: start..std::cmp::min(start + chunk_size, hook_count),
                #[allow(invalid_value)]
                //Reason: will be initialized as default on get_best_line() before use
                result: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
                logic_buffer: logic_builder.build_buffer(),
            })
                .collect(),
            logic,
        }
    }
}

impl<S: Float, H: hooks::Handle, D: Darkness<S>, L: HookLogic<H>> StringArt<S, H, D, L>
where
    usize: AsPrimitive<S>,
{
    fn select_next<'a>(structs: *mut [ColorMap<S, L::Current>]) -> *mut ColorMap<S, L::Current> {
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
            let color = unsafe { (*step.color_map).color };
            let color = Srgb::from_color(color);
            segment *= scale;
            for idx in grid.get_pixel_indexes_in_segment(segment) {
                unsafe {
                    let mut ptr = buffer.as_mut_ptr().add(3 * idx as usize);
                    *ptr = (color.red * S::TWO_FIVE_FIVE).to_u8().unwrap();
                    ptr = ptr.add(1);
                    *ptr = (color.green * S::TWO_FIVE_FIVE).to_u8().unwrap();
                    ptr = ptr.add(1);
                    *ptr = (color.blue * S::TWO_FIVE_FIVE).to_u8().unwrap();
                }
            }
        }
        let image = RgbImage::from_vec(grid.width, grid.height, buffer).unwrap();
        image.save(path)
    }
}

impl<
        S: Float + Sync + Send,
        H: Sync + Send + hooks::Handle<Links: Links<Link: Sync + Send>>,
        D: Darkness<S>,
        L: HookLogic<H, Current: Sync + Send> + Sync + Send,
    > StringArt<S, H, D, L>
where
    usize: AsPrimitive<S>,
{
    pub fn compute(&mut self, step_count: usize) {
        while step_count > self.steps.len() {
            self.compute_once();
        }
    }

    fn compute_once(&mut self) {
        if let Some(next) = self.get_best_line() {
            let color_map = unsafe { &mut *next.color_map };
            let line = unsafe { &mut *next.line };
            line.used = true;
            color_map.current_hook =
                self.logic
                    .register_end(self.handle, next.end_idx, next.end_link);
            for point in self.grid.get_pixel_indexes_in_segment(line.segment) {
                let weight = unsafe { color_map.data.get_unchecked_mut(point) };
                weight.0 = self.darkness.compute(weight.0);
            }
            self.steps.push(next);
        }
    }

    fn get_best_line(&mut self) -> Option<NextLine<S, H::Link, L::Current>> {
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
        let color_map = SyncColorMapRef(Self::select_next(self.color_maps.as_mut_slice()));
        let line_table = SyncLineTable(&mut self.lines);
        self.buffers.par_iter_mut().for_each(|buffer| {
            buffer.result = Default::default();
            for to_idx in buffer.range.clone() {
                for (curr_idx, curr_link) in self
                    .logic
                    .get_start_hooks(color_map.current_hook, &mut buffer.logic_buffer)
                {
                    if self.distancer.is_valid(curr_idx, to_idx) {
                        for to_link in H::LINKS {
                            let line_idx = self
                                .distancer
                                .index_of::<H::Links>(curr_idx, curr_link, to_idx, to_link);
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
            }
        });

        for BatchBuffer {
            range: _,
            result,
            logic_buffer: _,
        } in self.buffers.iter()
        {
            if result.weight > best_weight {
                best_weight = result.weight;
                best_line = Some(result.next);
            }
        }

        best_line
    }
}

//! How the computation visits the pixels of a segment.
//!
//! The walk is the hottest loop in the program, so the choice is a type
//! parameter: `W: PixelWalk` is monomorphized and never dispatched dynamically.
//! Two implementations ship with the crate:
//!
//! * [`Clip`] walks each segment on demand, with one bounds test per segment.
//! * [`Cached`] precomputes the pixel list of every segment once.
//!
//! The trait deliberately exposes `accumulate` and `apply_decay` instead of a
//! generic `for_each(.., visit: FnMut)`. In wasm a generic `for_each` in the
//! trait costs ~20%: the closure is not inlined through the extra layer. With
//! these two concrete methods the walker is within ~2% of a free function.

#[cfg(feature = "cached")]
use crate::board::Board;
use crate::decay::Decay;
#[cfg(any(feature = "clip", feature = "cached"))]
use crate::geometry::Rect;
use crate::geometry::Segment;
#[cfg(any(feature = "clip", feature = "cached"))]
use crate::num::scalar_from_frac;
use crate::num::{Frac, Scalar};
#[cfg(feature = "cached")]
use thiserror::Error;

/// Walks the pixels of a segment for the computation.
pub trait PixelWalk: Send + Sync {
    /// Mean remaining energy along `segment`, or zero when it crosses no pixel.
    ///
    /// `key` is the board's segment index (`SegmentRef::index`). Walkers that do
    /// not cache may ignore it.
    fn accumulate(&self, key: usize, segment: &Segment<f32>, weights: &[Frac]) -> Scalar;

    /// Applies `decay` to every pixel of `segment`.
    fn apply_decay(&self, key: usize, segment: &Segment<f32>, weights: &mut [Frac], decay: Decay);
}

/// Walks segments on demand with the integer Bresenham line.
///
/// The bounding box of each segment is tested once; fully-inside segments skip
/// the per-pixel bounds test.
#[cfg(feature = "clip")]
#[derive(Clone, Copy, Debug)]
pub struct Clip {
    rect: Rect<usize>,
}

#[cfg(feature = "clip")]
impl Clip {
    pub fn new(rect: Rect<usize>) -> Self {
        Self { rect }
    }

    pub fn rect(&self) -> Rect<usize> {
        self.rect
    }
}

#[cfg(feature = "clip")]
impl PixelWalk for Clip {
    #[inline]
    fn accumulate(&self, _key: usize, segment: &Segment<f32>, weights: &[Frac]) -> Scalar {
        let mut total = Scalar::ZERO;
        let mut count = 0u32;
        self.rect.for_each_pixel_index_in_segment(segment, |index| {
            total += scalar_from_frac(weights[index]);
            count += 1;
        });

        if count > 0 {
            total / Scalar::from_num(count)
        } else {
            Scalar::ZERO
        }
    }

    #[inline]
    fn apply_decay(&self, _key: usize, segment: &Segment<f32>, weights: &mut [Frac], decay: Decay) {
        self.rect.for_each_pixel_index_in_segment(segment, |index| {
            let weight = &mut weights[index];
            *weight = decay.compute(*weight);
        });
    }
}

/// Precomputes the pixel list of every board segment.
#[cfg(feature = "cached")]
#[derive(Clone, Debug)]
pub struct Cached {
    rect: Rect<usize>,
    offsets: Vec<u32>,
    lengths: Vec<u32>,
    indices: Vec<u32>,
}

/// The cache does not fit in the caller's memory budget.
#[cfg(feature = "cached")]
#[derive(Debug, Error)]
#[error("the line cache needs {needed} bytes but the budget is {limit}")]
pub struct CacheTooLarge {
    pub needed: usize,
    pub limit: usize,
}

#[cfg(feature = "cached")]
impl Cached {
    /// Builds the cache from every segment of `board`.
    ///
    /// `max_bytes` is a hard budget checked before allocating anything. The
    /// index array is reserved at once (its size is the sum of the segment
    /// lengths), so it never grows by doubling. That keeps it under the 2 GiB
    /// per-allocation limit of wasm32.
    pub fn from_board<B: Board>(
        board: &B,
        rect: Rect<usize>,
        max_bytes: usize,
    ) -> Result<Self, CacheTooLarge> {
        let mut offsets = Vec::new();
        let mut lengths = Vec::new();
        let mut total = 0usize;

        let mut key = 0;
        while let Some(segment) = board.get_segment(key) {
            let length = line_len(segment.get_segment());
            offsets.push(u32::try_from(total).unwrap_or(u32::MAX));
            lengths.push(length);
            total = total.saturating_add(length as usize);
            key += 1;
        }

        let needed = total
            .saturating_mul(size_of::<u32>())
            .saturating_add((offsets.len() + lengths.len()).saturating_mul(size_of::<u32>()));

        let fits = total <= u32::MAX as usize
            && total <= isize::MAX as usize / size_of::<u32>()
            && needed <= max_bytes;

        if !fits {
            return Err(CacheTooLarge {
                needed,
                limit: max_bytes,
            });
        }

        let mut indices = Vec::with_capacity(total);
        for key in 0..offsets.len() {
            let start = indices.len();
            let segment = board
                .get_segment(key)
                .expect("a board's segments are contiguous")
                .get_segment();
            rect.for_each_pixel_index_in_segment(segment, |pixel| {
                indices.push(pixel as u32);
            });
            offsets[key] = start as u32;
            lengths[key] = (indices.len() - start) as u32;
        }

        Ok(Self {
            rect,
            offsets,
            lengths,
            indices,
        })
    }

    /// Estimated size of the cache for `board`, without building it.
    pub fn estimate<B: Board>(board: &B) -> usize {
        let mut total = 0usize;
        let mut segments = 0usize;
        let mut key = 0;
        while let Some(segment) = board.get_segment(key) {
            total = total.saturating_add(line_len(segment.get_segment()) as usize);
            segments += 1;
            key += 1;
        }

        total
            .saturating_mul(size_of::<u32>())
            .saturating_add(segments.saturating_mul(2 * size_of::<u32>()))
    }

    /// Bytes actually allocated.
    pub fn bytes(&self) -> usize {
        self.indices.len() * size_of::<u32>()
            + (self.offsets.len() + self.lengths.len()) * size_of::<u32>()
    }

    pub fn rect(&self) -> Rect<usize> {
        self.rect
    }
}

#[cfg(feature = "cached")]
impl PixelWalk for Cached {
    #[inline]
    fn accumulate(&self, key: usize, _segment: &Segment<f32>, weights: &[Frac]) -> Scalar {
        let start = self.offsets[key] as usize;
        let len = self.lengths[key] as usize;

        let mut total = Scalar::ZERO;
        for &pixel in &self.indices[start..start + len] {
            total += scalar_from_frac(weights[pixel as usize]);
        }

        if len > 0 {
            total / Scalar::from_num(len as u32)
        } else {
            Scalar::ZERO
        }
    }

    #[inline]
    fn apply_decay(&self, key: usize, _segment: &Segment<f32>, weights: &mut [Frac], decay: Decay) {
        let start = self.offsets[key] as usize;
        let len = self.lengths[key] as usize;

        for &pixel in &self.indices[start..start + len] {
            let weight = &mut weights[pixel as usize];
            *weight = decay.compute(*weight);
        }
    }
}

/// Number of pixels the Bresenham line would yield, before dropping the ones
/// outside the image. It is an upper bound, used to reserve capacity.
#[cfg(feature = "cached")]
fn line_len(segment: &Segment<f32>) -> u32 {
    let x0 = segment.start.x.floor() as i64;
    let y0 = segment.start.y.floor() as i64;
    let x1 = segment.end.x.floor() as i64;
    let y1 = segment.end.y.floor() as i64;

    (x1 - x0).abs().max((y1 - y0).abs()) as u32
}

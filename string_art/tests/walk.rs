//! `Clip` and `Cached` must visit the same pixels and produce the same steps.
#![cfg(all(feature = "clip", feature = "cached"))]

use string_art::board::{Board, IntoBoard};
use string_art::geometry::Rect;
use string_art::nails::{Anchor, Circular, circular::Direction};
use string_art::walk::{Cached, Clip, PixelWalk};
use string_art::{Computation, Decay, Ellipse, Executor, Frac, Monocolor, Work};

/// Minimal single-threaded backend, so the test does not need `string_art_serial`.
#[derive(Clone, Copy)]
struct Serial;

impl<T, W: Work<T>> Executor<T, W> for Serial {
    fn workers(&self) -> usize {
        1
    }

    fn for_each(&self, buffers: &mut [T], work: W) {
        for (index, buffer) in buffers.iter_mut().enumerate() {
            work.run(index, buffer);
        }
    }
}

const SIZE: usize = 64;
const NAILS: usize = 32;
const MIN_DISTANCE: usize = 2;
const STEPS: u32 = 40;

fn ellipse() -> Ellipse<Circular> {
    Ellipse::new(
        Rect::new(SIZE as f32, SIZE as f32),
        Circular(0.1),
        NAILS,
        MIN_DISTANCE,
    )
    .expect("invalid board")
}

fn board() -> <Ellipse<Circular> as IntoBoard>::Board {
    ellipse().into_board(1)
}

fn weights() -> Vec<Frac> {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    (0..SIZE * SIZE)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            Frac::from_bits(((state >> 40) as u32 & 0xffff) * 0x0001_0001)
        })
        .collect()
}

fn segments(
    board: &<Ellipse<Circular> as IntoBoard>::Board,
) -> Vec<(usize, string_art::geometry::Segment<f32>)> {
    let mut out = Vec::new();
    let mut index = 0;
    while let Some(segment) = board.get_segment(index) {
        out.push((index, *segment.get_segment()));
        index += 1;
    }
    out
}

#[test]
fn clip_and_cached_accumulate_the_same() {
    let board = board();
    let rect = Rect::new(SIZE, SIZE);
    let segments = segments(&board);
    let weights = weights();

    let clip = Clip::new(rect);
    let cached = Cached::from_board(&board, rect, usize::MAX).expect("the cache should fit");

    for (key, segment) in &segments {
        assert_eq!(
            clip.accumulate(*key, segment, &weights),
            cached.accumulate(*key, segment, &weights),
            "segment {key} differs"
        );
    }
}

#[test]
fn clip_and_cached_decay_the_same() {
    let board = board();
    let rect = Rect::new(SIZE, SIZE);
    let segments = segments(&board);
    let decay = Decay::Flat(Frac::from_num(0.17));

    let clip = Clip::new(rect);
    let cached = Cached::from_board(&board, rect, usize::MAX).expect("the cache should fit");

    let mut from_clip = weights();
    let mut from_cached = weights();

    for (key, segment) in &segments {
        clip.apply_decay(*key, segment, &mut from_clip, decay);
        cached.apply_decay(*key, segment, &mut from_cached, decay);
    }

    assert_eq!(from_clip, from_cached);
}

#[test]
fn cache_budget_is_enforced_before_allocating() {
    let board = board();
    let rect = Rect::new(SIZE, SIZE);
    let error = Cached::from_board(&board, rect, 16).expect_err("16 bytes cannot fit");
    assert!(error.needed > 16);

    let estimate = Cached::estimate(&board);
    let cache = Cached::from_board(&board, rect, usize::MAX).expect("the cache should fit");
    assert!(estimate >= cache.bytes());
}

#[test]
fn cached_computation_matches_clip() {
    let rect = Rect::new(SIZE, SIZE);
    let decay = Decay::Flat(Frac::from_num(0.2));

    let pipeline =
        || -> Monocolor<Anchor<Direction>> { Monocolor::new(Anchor::default(), weights(), STEPS) };
    let trace = |steps: Vec<string_art::computation::Step<Monocolor<Anchor<Direction>>>>| {
        steps
            .into_iter()
            .map(|step| {
                (
                    step.anchor.idx,
                    usize::from(step.anchor.link),
                    step.segment.start.x,
                    step.segment.start.y,
                )
            })
            .collect::<Vec<_>>()
    };

    let clip = Computation::new(Serial, pipeline(), ellipse(), rect, decay).expect("invalid map");
    let clip_steps: Vec<_> = clip.map(|step| step.expect("step failed")).collect();

    let cached =
        Computation::new_cached(Serial, pipeline(), ellipse(), rect, decay, 64 * 1024 * 1024)
            .expect("the cache should fit");
    let cached_steps: Vec<_> = cached.map(|step| step.expect("step failed")).collect();

    assert_eq!(clip_steps.len(), cached_steps.len());
    assert_eq!(trace(clip_steps), trace(cached_steps));
}

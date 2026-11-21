//! Microbenchmark for the pixel walkers.
//!
//! It compares the two shipped strategies against the ideal free function with
//! the same loop shape:
//!
//! * `ideal`  — `Rect::for_each_pixel_index_in_segment` with the arithmetic
//!   written inline. It is the reference.
//! * `clip`   — [`Clip`], which walks each segment on demand.
//! * `cached` — [`Cached`], which reads a precomputed pixel list.
//!
//! The three must produce the same checksum; the benchmark aborts otherwise.
//!
//! Run with:
//!
//! ```sh
//! cargo bench -p string_art --bench walk
//!
//! # wasm, without the `image` dependency
//! cargo bench -p string_art --bench walk --target wasm32-wasip1 \
//!     --no-default-features --features clip,cached
//! ```
//!
//! The bench needs both built-in walkers.
//!
//! Environment:
//!
//! * `WALK_SIZES` — square resolutions (default `512,1024`).
//! * `WALK_CACHE` — resolutions that also bench the cache (default `512`).
//! * `WALK_CACHE_MIB` — cache budget per resolution (default `2048`).
//! * `WALK_NAILS` — nail count (default `512`).
//! * `WALK_MIN` — minimum nail distance (default `36`).
//! * `WALK_REPS` — timed repetitions per variant (default `5`).
//! * `WALK_STEPS` — steps used for the projection (default `3000`).

#[cfg(not(all(feature = "clip", feature = "cached")))]
compile_error!("the walk bench needs the `clip` and `cached` features");

use std::hint::black_box;
use std::time::Instant;

use string_art::board::{Board, IntoBoard};
use string_art::geometry::{Rect, Segment};
use string_art::nails::Circular;
use string_art::walk::{Cached, Clip, PixelWalk};
use string_art::{Ellipse, Frac, Scalar};

type Seg = Segment<f32>;
type BenchBoard = <Ellipse<Circular> as IntoBoard>::Board;

fn env_list(name: &str, default: &str) -> Vec<u32> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .filter_map(|item| item.trim().parse().ok())
        .collect()
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}

fn build_board(size: usize, nails: usize, min_distance: usize) -> BenchBoard {
    Ellipse::new(
        Rect::new(size as f32, size as f32),
        Circular(0.1),
        nails,
        min_distance,
    )
    .expect("invalid board")
    .into_board(1)
}

/// Enumerates every segment through the public `Board` API.
fn segments_of(board: &BenchBoard) -> Vec<Seg> {
    let mut segments = Vec::new();
    let mut index = 0;
    while let Some(segment) = board.get_segment(index) {
        segments.push(*segment.get_segment());
        index += 1;
    }
    segments
}

/// Deterministic weights, so every variant reads the same values.
fn weights(len: usize) -> Vec<Frac> {
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        // `U0F32` is in `[0, 1)`: build the value from bits.
        out.push(Frac::from_bits(
            ((state >> 40) as u32 & 0xffff) * 0x0001_0001,
        ));
    }
    out
}

/// Reference: the same arithmetic as `Clip::accumulate`, written inline.
fn sum_ideal(segments: &[Seg], rect: &Rect<usize>, weights: &[Frac]) -> u64 {
    let mut check = 0u64;
    for segment in segments {
        let mut total = Scalar::ZERO;
        let mut count = 0u32;
        rect.for_each_pixel_index_in_segment(segment, |index| {
            total += Scalar::from_bits(u64::from(weights[index].to_bits()));
            count += 1;
        });
        if count > 0 {
            check = check.wrapping_add((total / Scalar::from_num(count)).to_bits());
        }
    }
    check
}

/// Drives a real walker over every segment, as the computation does.
fn sum_walk<W: PixelWalk>(walk: &W, segments: &[Seg], weights: &[Frac]) -> u64 {
    let mut check = 0u64;
    for (key, segment) in segments.iter().enumerate() {
        check = check.wrapping_add(walk.accumulate(key, segment, weights).to_bits());
    }
    check
}

fn count_pixels(segments: &[Seg], rect: &Rect<usize>) -> u64 {
    let mut count = 0u64;
    for segment in segments {
        rect.for_each_pixel_index_in_segment(segment, |_| count += 1);
    }
    count
}

struct Trial<'a> {
    name: &'static str,
    body: Box<dyn FnMut() -> u64 + 'a>,
    /// One-off cost paid before the first pass, in seconds.
    startup: f64,
}

/// Runs every trial once per repetition, in round-robin order, so that frequency
/// drift hits all variants equally. Returns `(median, min)` in seconds.
fn run_trials(trials: &mut [Trial<'_>], reps: usize) -> Vec<(f64, f64)> {
    for _ in 0..2 {
        for trial in trials.iter_mut() {
            black_box((trial.body)());
        }
    }

    let mut samples = vec![Vec::with_capacity(reps); trials.len()];
    for _ in 0..reps {
        for (index, trial) in trials.iter_mut().enumerate() {
            let start = Instant::now();
            let value = (trial.body)();
            let elapsed = start.elapsed().as_secs_f64();
            black_box(value);
            samples[index].push(elapsed);
        }
    }

    samples
        .into_iter()
        .map(|mut samples| {
            samples.sort_by(f64::total_cmp);
            (samples[samples.len() / 2], samples[0])
        })
        .collect()
}

fn fmt_ms(seconds: f64) -> String {
    format!("{:>10.2}", seconds * 1000.0)
}

fn main() {
    let sizes = env_list("WALK_SIZES", "512,1024");
    let cache_sizes = env_list("WALK_CACHE", "512");
    let cache_budget = env_usize("WALK_CACHE_MIB", 2048) * 1024 * 1024;
    let nails = env_usize("WALK_NAILS", 512);
    let min_distance = env_usize("WALK_MIN", 36);
    let reps = env_usize("WALK_REPS", 5).max(1);
    let steps = env_usize("WALK_STEPS", 3000);

    println!(
        "walk microbenchmark | nails={nails} min_distance={min_distance} reps={reps} steps={steps}\n\
         (times in ms; min is the headline because the governor is `schedutil`)\n"
    );

    for size in sizes {
        let board = build_board(size as usize, nails, min_distance);
        let segments = segments_of(&board);
        let rect = Rect::new(size as usize, size as usize);
        let weights = weights(rect.area());
        let pixels = count_pixels(&segments, &rect);

        let expected = sum_ideal(&segments, &rect, &weights);

        println!(
            "== {size}x{size} | {} segments | {} M valid pixels",
            segments.len(),
            pixels as f64 / 1e6
        );

        let clip = Clip::new(rect);
        assert_eq!(
            expected,
            sum_walk(&clip, &segments, &weights),
            "Clip does not match the reference at {size}x{size}"
        );

        let cached = if cache_sizes.contains(&size) {
            let estimate = Cached::estimate(&board);
            let start = Instant::now();
            match Cached::from_board(&board, rect, cache_budget) {
                Ok(cache) => {
                    let build = start.elapsed().as_secs_f64();
                    assert_eq!(
                        expected,
                        sum_walk(&cache, &segments, &weights),
                        "Cached does not match the reference at {size}x{size}"
                    );
                    println!(
                        "  cache: {:.1} MiB built in {:.1} ms",
                        cache.bytes() as f64 / (1024.0 * 1024.0),
                        build * 1000.0
                    );
                    Some((cache, build))
                }
                Err(error) => {
                    println!(
                        "  cache: skipped, needs {:.1} MiB (estimate {:.1} MiB), budget {:.1} MiB",
                        error.needed as f64 / (1024.0 * 1024.0),
                        estimate as f64 / (1024.0 * 1024.0),
                        cache_budget as f64 / (1024.0 * 1024.0),
                    );
                    None
                }
            }
        } else {
            println!("  (cache skipped; add {size} to WALK_CACHE to include it)");
            None
        };

        let mut trials = vec![
            Trial {
                name: "ideal",
                body: Box::new(|| sum_ideal(&segments, &rect, &weights)),
                startup: 0.0,
            },
            Trial {
                name: "clip",
                body: Box::new(|| sum_walk(&clip, &segments, &weights)),
                startup: 0.0,
            },
        ];
        if let Some((cache, build)) = &cached {
            trials.push(Trial {
                name: "cached",
                body: Box::new(|| sum_walk(cache, &segments, &weights)),
                startup: *build,
            });
        }

        let results = run_trials(&mut trials, reps);

        // Approximate how many full passes over the segment set a real run does:
        // every step evaluates `(nails - 2*min - 1) * links` candidates.
        let candidates_per_step = (nails.saturating_sub(2 * min_distance + 1)) * 2;
        let passes = (candidates_per_step * steps) as f64 / segments.len() as f64;

        println!(
            "  {:<14} {:>10} {:>10} {:>8} {:>10}",
            "variant", "median", "min", "ns/px", "proj. s"
        );
        for (trial, (median, min)) in trials.iter().zip(&results) {
            println!(
                "  {:<14} {} {} {:>8.2} {:>9.2}",
                trial.name,
                fmt_ms(*median),
                fmt_ms(*min),
                min * 1e9 / pixels as f64,
                min * passes + trial.startup
            );
        }

        let baseline = results[0].1;
        let line = trials
            .iter()
            .zip(&results)
            .skip(1)
            .map(|(trial, (_, min))| format!("{} {:.2}x", trial.name, baseline / min))
            .collect::<Vec<_>>()
            .join(" | ");
        println!("  speedup vs ideal: {line}");
        println!("  projection: {passes:.1} passes over the segment set for {steps} steps");
        println!();
    }
}

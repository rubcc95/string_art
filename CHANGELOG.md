# Changelog

## [Unreleased]

### Added
- `string_art`: pluggable pixel traversal through the `walk` module. `Clip`
  walks each segment on demand (the default) and `Cached` precomputes the pixel
  list of every board segment with a hard memory budget.
- `string_art`: `Computation::new_cached` and `Computation::new_with` select the
  walker at construction; `Computation::with_board` accepts an already-built
  board. `Computation` gained the walker as a type parameter (`W: PixelWalk`,
  defaulting to `Clip`), so the choice stays statically dispatched.
- `string_art`: `Rect::for_each_pixel_index_in_segment`, the hot-path rasterizer
  with one bounds test per segment instead of one per pixel.
- `string_art`: the `clip` and `cached` cargo features, both enabled by default.
  Disabling one removes its walker; disabling both leaves only `PixelWalk` so a
  consumer can plug its own implementation without paying for the built-ins.
- `string_art_gpui`: English, Spanish, French, Portuguese, Italian, German,
  Catalan, Galician and Basque UI. The language follows the system by default
  and can be forced with the new `config.json` (`STRING_ART_LANG` overrides it
  on native builds).
- `string_art_gpui`: locale-aware number formatting and parsing through ICU4X,
  including decimal separators, grouping and plural forms.
- `string_art_gpui`: `GpuiExec` backend that distributes the sweep over the
  multithreaded pool of `gpui_web` (the dispatcher's `wasm_thread` threads), plus a
  non-blocking `map` on `Parallel` for backends that cannot wait.
- `Parallel::map`: non-blocking way to distribute work. By default it delegates
  to `for_each`, so backends that can block pay nothing.

### Changed
- `string_art`: the computation no longer recomputes Bresenham pairs through
  `Option`/`NumCast` adapters. The pixel walk is a strategy: `Clip` calls the
  geometry rasterizer directly and `Cached` reads a precomputed index list.
- `string_art`: `Rect::get_pixel_indexes_in_segment` and `Segment::bresenham`
  were removed in favour of `Rect::for_each_pixel_index_in_segment`.
- `string_art_gpui` now requires Rust 1.88 because of ICU4X. The publishable
  core crates keep the workspace MSRV of 1.85.
- `string_art` is no longer generic over the numeric type: it uses 64-bit
  fixed point (`U0F32` for each pixel's energy, `U32F32` for the accumulator).
- `decay` goes from a generic trait to a `Decay` enum.
- `ColorMap` and `Pipeline` lose the weight type parameters.
- The GUI loses the precision selector (P8/P16/P32/P64), which never ended up
  connected to the computation.
- `Computation` receives the parallelism backend in `new` (`string_art_sync::Parallel`)
  instead of deciding the threads on its own; the core no longer names `rayon`.
- Tie-breaking between segments with the same weight uses the segment index, so
  the result does not depend on the number of threads.
- `CondSend`/`CondSync` are real `Send`/`Sync` also in wasm: the empty traits
  allowed crossing threads with types that are not.

### Removed
- `string_art_geometry`: the `bresenham` crate. The rasterizer now lives in
  `Rect::for_each_pixel_index_in_segment`.
- The build scaffolding for Android/Termux: the `clippy-driver` wrapper,
  the `.harness` `TMPDIR` and the rest of the `lint.sh` patches.
- The `gpui` feature of `string_art_gpui`: gpui is no longer optional.
- `string_art_math`: nobody uses it anymore.
- The entire Dioxus GUI: `string_art_gui`, `string_art_gui_common`,
  `string_art_gui_web_worker`, `string_art_gui_web_worker_registrar` and `xtask`.
- The `wasm-dev`, `server-dev` and `android-dev` build profiles.

## [0.1.0-alpha.1] - 2024-09-30

### Added
- Initial release.
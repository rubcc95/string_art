# Changelog

## [Unreleased]

### Added
- `string_art_gpui`: `GpuiExec` backend that distributes the sweep over the
  multithreaded pool of `gpui_web` (the dispatcher's `wasm_thread` threads), plus a
  non-blocking `map` on `Parallel` for backends that cannot wait.
- `Parallel::map`: non-blocking way to distribute work. By default it delegates
  to `for_each`, so backends that can block pay nothing.

### Changed
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
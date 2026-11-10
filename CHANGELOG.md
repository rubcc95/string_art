# Changelog

## [Unreleased]

### Added
- `string_art_gpui`: backend `GpuiExec` que reparte el barrido sobre el pool
  multihilo de `gpui_web` (los hilos `wasm_thread` del dispatcher), más `map`
  sin bloqueo en `Parallel` para los backends que no pueden esperar.
- `Parallel::map`: forma no bloqueante de repartir trabajo. Por defecto delega
  en `for_each`, así que los backends que sí pueden bloquear no pagan nada.

### Changed
- `string_art` deja de ser genérico sobre el tipo numérico: usa punto fijo de
  64 bits (`U0F32` para la energía de cada píxel, `U32F32` para el acumulador).
- `decay` pasa de trait genérico a un enum `Decay`.
- `ColorMap` y `Pipeline` pierden los parámetros de tipo del peso.
- La GUI pierde el selector de precisión (P8/P16/P32/P64), que nunca llegó a
  estar conectado al cálculo.
- `Computation` recibe el backend de paralelismo en `new` (`string_art_sync::Parallel`)
  en vez de decidir los hilos por su cuenta; el core ya no nombra `rayon`.
- El desempate entre segmentos con el mismo peso usa el índice de segmento, así
  que el resultado no depende del número de hilos.
- `CondSend`/`CondSync` son `Send`/`Sync` reales también en wasm: los traits
  vacíos dejaban cruzar hilos con tipos que no lo son.

### Removed
- El andamiaje de compilación para Android/Termux: el wrapper de `clippy-driver`,
  el `TMPDIR` de `.harness` y el resto de parches de `lint.sh`.
- La feature `gpui` de `string_art_gpui`: gpui ya no es opcional.
- `string_art_math`: ya no lo usa nadie.
- La GUI de Dioxus entera: `string_art_gui`, `string_art_gui_common`,
  `string_art_gui_web_worker`, `string_art_gui_web_worker_registrar` y `xtask`.
- Los perfiles de compilación `wasm-dev`, `server-dev` y `android-dev`.

## [0.1.0-alpha.1] - 2024-09-30

### Added
- Initial release.
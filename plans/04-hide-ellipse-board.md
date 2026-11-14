# Plan 04 — Ocultar `EllipseBoard` y el `SegmentRef` concreto

## Objetivo

Que `string_art::ellipse::EllipseBoard` deje de ser alcanzable por path, sin perder
`string_art::ellipse::Error`, que sí debe poder manejar un dependiente.

`EllipseBoard` debe seguir siendo `pub` porque es el tipo asociado
`<Ellipse<N> as IntoBoard>::Board`; lo que se elimina es su path público. El
`SegmentRef<'a, N>` concreto de `ellipse.rs` no necesita ser público en absoluto:
el tipo concreto va escondido detrás de `impl Iterator`.

## Dependencias

- Requiere el Plan 01 (árbol en verde).
- Es independiente de los planes 02 y 03.

## Contexto para el agente

- Repositorio: `/home/david/projects/string_art`
- Crate: `string_art`
- `lib.rs` tiene `pub mod ellipse;` y `pub use ellipse::Ellipse;`.
- `ellipse.rs` contiene:
  - `pub struct Ellipse<N>`
  - `pub struct EllipseBoard<B>`
  - `pub struct SegmentRef<'a, N>` (el struct concreto, no el trait de `board`)
  - `pub enum Error<N>`
- `ellipse.rs` empieza con `use board::*;`. **No llames `board` al submódulo nuevo**:
  chocaría con ese import. Usa `board_impl`.
- `EllipseBoard` necesita seguir siendo `pub` dentro de su módulo para satisfacer
  `IntoBoard::Board`; el módulo que lo contiene será privado.

## Cambios

### 1. Crear `string_art/src/ellipse/board_impl.rs`

Mover a este fichero:

- `EllipseBoard<B>`.
- `impl<B> Display for EllipseBoard<B>`.
- `impl<B> Board for EllipseBoard<B>`.
- `impl<N> IntoBoard for Ellipse<N>` (recomendado moverlo también).

El fichero puede empezar con imports equivalentes a:

```rust
use super::Ellipse;
use super::SegmentRef;
use crate::board::{Board, IntoBoard, Segment, SegmentRef as SegmentRefTrait};
use crate::nails;
use derive_more::{Deref, DerefMut};
use std::fmt::{Display, Formatter};
```

Ajustar los imports a lo que realmente use el código movido. Mantener
`pub struct EllipseBoard<B: nails::Builder>` como `pub`.

El `impl IntoBoard for Ellipse` puede acceder a los campos privados de `Ellipse`
porque `ellipse::board_impl` es descendiente de `ellipse`.

### 2. Modificar `string_art/src/ellipse.rs`

- Añadir `mod board_impl;` al principio.
- Borrar de `ellipse.rs` los elementos movidos a `board_impl.rs`.
- Cambiar:

```rust
pub struct SegmentRef<'a, N: nails::Builder> {
```

por:

```rust
struct SegmentRef<'a, N: nails::Builder> {
```

El struct sigue siendo visible para `board_impl` porque los hijos pueden acceder a
items privados del módulo padre.
- Mantener `pub enum Error<N>` en `ellipse.rs`.
- Mantener `pub struct Ellipse<N>` en `ellipse.rs`.
- Si el `impl IntoBoard for Ellipse` se queda en `ellipse.rs`, añadir un constructor
  `pub(super) fn new(...)` a `EllipseBoard` en `board_impl.rs` y usarlo desde
  `ellipse.rs`. La opción recomendada es mover el `impl` completo.

### 3. Ajustar los tests de `ellipse.rs`

Los tests acceden a `board.ellipse.index_of(...)` y `board.ellipse.segments.len()`.
Como `EllipseBoard.ellipse` es privado de `board_impl` (hermano de `test`), hay que
usar `Deref`:

- `board.ellipse.index_of(...)` → `board.index_of(...)`
- `board.ellipse.segments.len()` → `board.segments.len()`

`EllipseBoard` implementa `Deref<Target = Ellipse<B>>`, así que `board.handle`,
`board.segments` y `board.index_of` funcionan igual. Los tests son descendientes de
`ellipse`, por lo que pueden llamar a `Ellipse::index_of` aunque sea privado.

Añadir el import del `EllipseBoard` movido:

```rust
use super::board_impl::EllipseBoard;
```

dentro del módulo `test`, si `use super::*;` ya no lo trae.

### 4. Mantener la API que debe seguir existiendo

- `string_art::ellipse::Ellipse` → sí.
- `string_art::ellipse::Error` → sí, sin cambios.
- `string_art::Ellipse` (reexport de la raíz) → sí.
- `string_art::ellipse::EllipseBoard` → no debe existir.

## Verificación

```bash
cargo test -p string_art
```

Comprobar desde un crate externo o test temporal:

```rust
// Debe compilar:
type Error = string_art::ellipse::Error<core::convert::Infallible>;
type Board = <string_art::Ellipse<string_art::nails::Point> as string_art::board::IntoBoard>::Board;
let _: Option<Error> = None;
let _: Option<Board> = None;

// Debe fallar con E0603: es lo esperado.
// type Privado = string_art::ellipse::EllipseBoard<string_art::nails::Point>;
```

También:

```bash
cargo rustc -p string_art --lib -- -W unnameable_types
```

Debe avisar de `EllipseBoard` como tipo no nombrable por path. Es el resultado
esperado.

## Criterio de aceptación

- `string_art::ellipse::EllipseBoard` no compila desde fuera.
- `string_art::ellipse::Error` compila y se puede usar en `match`:
  `Err(EllipseError::Distancer(max))`, `Err(EllipseError::Nail(e))`.
- `<Ellipse<N> as IntoBoard>::Board` sigue compilando.
- `cargo test -p string_art` pasa.
- No se ha cambiado ningún cálculo ni ninguna firma pública salvo la desaparición
  del path de `EllipseBoard`.

## Fuera de alcance

- No tocar los módulos de `nails` (Plan 03).
- No borrar `TryFrom` ni errores de `point` (Plan 02).
- No cambiar `Board`, `IntoBoard`, `SegmentRef` (trait) ni `Segment`.

# Plan 05 — Campos internos y auditoría final de la API

## Objetivo

Terminar de reducir la superficie pública que no necesita el dependiente y hacer una
auditoría final de los tipos que quedan accesibles por path y por proyección.

Este plan es opcional en su parte de campos, pero la auditoría final debe ejecutarse
siempre después de los planes 01–04.

## Dependencias

- Requiere que los planes 01, 02, 03 y 04 estén aplicados.
- Es el último plan.

## Principio de diseño

- Un tipo que solo se necesita a través de un tipo asociado se deja **sin path
  público** y se usa por proyección.
- Un tipo que el dependiente necesita para implementar un trait o para operar con
  los tipos existentes se queda público.
- No reexportar un tipo de proyección solo por comodidad.
- Los campos de una estructura son **privados** por defecto. Si otro módulo necesita
  leerlos, se añade un accesor de solo lectura (`&T`, `&[T]`, etc.); no se abre el
  campo con `pub` ni con `pub(crate)`.
- `pub(crate)` se reserva para funciones o items que realmente se usan desde otros
  módulos del crate y no pueden ser privados de su propio módulo.

## Cambios opcionales

### 1. `string_art/src/ellipse.rs`

Los campos deben quedar **privados**:

```rust
handle: N::Handle,
nails: Vec<N::Nail>,
```

`handle` se usa desde `EllipseBoard` (`ellipse::board_impl`) y desde los tests de
`ellipse.rs`, pero ambos son módulos descendientes de `ellipse`, así que pueden
acceder a los campos privados del módulo padre. `nails` no se usa fuera de
`Ellipse::new`.

Comprobar que ejemplos y `string_art_gpui` no los usan desde fuera. Si en el futuro
un consumidor necesita leerlos, añadir un accesor de solo lectura, por ejemplo:

```rust
impl<N: nails::Builder> Ellipse<N> {
    pub fn nails(&self) -> &[N::Nail] {
        &self.nails
    }
}
```

No añadir accesores que nadie usa.

### 2. `string_art/src/monocolor.rs`

Los dos campos deben quedar **privados**:

```rust
map: ColorMap<A, Black>,
threads: u32,
```

- `threads` solo se usa dentro de `monocolor.rs` (iterador, `len`, `size_hint`).
- `map` se usa dentro de `monocolor.rs` y en el test de integración
  `string_art/tests/local_anchor.rs`. Para ese test, añadir un accesor de solo
  lectura:

```rust
impl<A> Monocolor<A> {
    pub fn weights(&self) -> &[Frac] {
        &self.map.weights
    }
}
```

y cambiar en `tests/local_anchor.rs`:

```rust
assert_eq!(monocolor.map.weights.len(), 4);
```

por:

```rust
assert_eq!(monocolor.weights().len(), 4);
```

Si hiciera falta más información del mapa, exponer accesores concretos de solo
lectura. No hacer público `map` ni ponerlo `pub(crate)`. Para leer el mapa completo
ya existe `Pipeline::color_maps()`, que es API pública.

### 3. `string_art/src/num.rs`

Aquí `pub(crate)` sí es la visibilidad mínima: las dos funciones se usan desde
módulos hermanos (`monocolor.rs` y `computation.rs`), y una función privada de `num`
no sería visible fuera de `num`. No se exponen fuera del crate porque `mod num;` es
privado.

Cambiar:

```rust
pub fn frac_from_u8(...)
pub fn scalar_from_frac(...)
```

por:

```rust
pub(crate) fn frac_from_u8(...)
pub(crate) fn scalar_from_frac(...)
```

## Auditoría final

### 1. Compilación y tests

```bash
cargo check -p string_art --all-targets
cargo test -p string_art
cargo check -p string_art_gpui
```

### 2. Tipos que deben seguir teniendo path público

Traits necesarios para implementar:

- `string_art::board::{Board, IntoBoard, SegmentRef}`
- `string_art::board::Segment`
- `string_art::nails::{Builder, Links, Handle}`
- `string_art::computation::Pipeline`
- `string_art::executor::Executor`
- `string_art::Color`

Tipos para construir u operar:

- `string_art::ellipse::{Ellipse, Error}`
- `string_art::computation::{Computation, ColorMap, Step, RectAreaError, Error}`
- `string_art::nails::{Anchor, Circular, Direction, Point}`
- `string_art::{Monocolor, Decay, Frac, Scalar}`
- `string_art::executor::Serial`
- `string_art::ColorMap` no es obligatorio: el path válido es
  `string_art::computation::ColorMap`.

### 3. Tipos que deben quedar solo por proyección

- `<Ellipse<N> as IntoBoard>::Board` (el antiguo `EllipseBoard`).
- `<Monocolor<A> as Pipeline>::Color` (el antiguo `Black`).
- `<Monocolor<A> as Pipeline>::Index` (el antiguo `MonocolorIndex`).
- `<Point as Builder>::Link`.
- `<Circular as Builder>::Error`.
- `<Point as Builder>::Error` = `core::convert::Infallible`.

No debe existir path público para:

- `string_art::ellipse::EllipseBoard`
- `string_art::nails::point::Link`
- `string_art::nails::circular::Error`
- `string_art::nails::anchor::Debugger`

### 4. Comprobación automática

```bash
cargo rustc -p string_art --lib -- -W unnameable_types
```

Los avisos esperados son los tipos de proyección que no tienen path (por ejemplo
`EllipseBoard`, `Black`, `MonocolorIndex`, `point::Link`, `circular::Error`). No es
un fallo; es el resultado buscado.

Opcional: añadir en cada uno de esos tipos un comentario breve explicando que su
`pub` es obligatorio por un tipo asociado y que no se reexporta a propósito. No es
necesario añadir `#![warn(unnameable_types)]` al crate; si se hace, acompañar cada
tipo intencional con `#[allow(unnameable_types)]`.

### 5. Proyecciones que deben compilar

Desde un crate externo o test temporal:

```rust
use string_art::board::IntoBoard;
use string_art::computation::Pipeline;
use string_art::nails::{Builder, Circular, Point};
use string_art::{Ellipse, Monocolor, nails::Anchor};

type Board = <Ellipse<Point> as IntoBoard>::Board;
type Color = <Monocolor<Anchor<()>> as Pipeline>::Color;
type Index = <Monocolor<Anchor<()>> as Pipeline>::Index;
type Link = <Point as Builder>::Link;
type CircError = <Circular as Builder>::Error;
```

Todo debe compilar.

## Criterio de aceptación

- `cargo check -p string_art --all-targets`, `cargo test -p string_art` y
  `cargo check -p string_art_gpui` pasan.
- La matriz de tipos públicos coincide con la de este plan.
- Los tipos de proyección solo se pueden nombrar mediante
  `<Tipo as Trait>::Asociado`.
- No se ha modificado ningún cálculo ni ninguna firma de trait.

## Fuera de alcance

- No volver a exponer `EllipseBoard`, `point::Link`, `circular::Error` ni `Debugger`.
- No cambiar la API abierta de `Board`, `IntoBoard`, `SegmentRef`, `Segment`,
  `Builder`, `Links`, `Handle`, `Pipeline` ni `Executor`.

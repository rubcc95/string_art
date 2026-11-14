# Plan 02 — Borrar `TryFrom` no usados y errores muertos de `point`

## Objetivo

Eliminar superficie pública que no usa nadie en el workspace:

- `impl TryFrom<usize> for Direction` y su `FromIndexError`.
- `impl TryFrom<usize> for Link` (de `Point`) y su `FromIndexError`.
- El `Error` de `Point`, que nunca se construye porque `Point::create_segment`
  siempre devuelve `Ok`.

## Dependencias

- Requiere que el Plan 01 esté aplicado (árbol en verde).
- Es independiente del Plan 03 (privatización de módulos de `nails`), aunque toca
  los mismos ficheros; si se ejecutan en paralelo, usar worktrees distintos.

## Contexto para el agente

- Repositorio: `/home/david/projects/string_art`
- Crate: `string_art`
- `nails::circular::Direction` implementa `From<Direction> for usize`, que **sí se usa**
  (por ejemplo en `examples/parallel_check.rs`). Ese `impl` no se toca.
- `nails::point::Link` implementa `Into<usize> for Link`; no forma parte de este plan,
  se puede dejar como está.
- Los `TryFrom<usize>` no tienen ninguna referencia en el workspace.

## Cambios

### 1. `string_art/src/nails/circular.rs`

Borrar este bloque completo:

```rust
impl TryFrom<usize> for Direction {
    type Error = FromIndexError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self(circle::Direction::ClockWise)),
            1 => Ok(Self(circle::Direction::CounterClockWise)),
            val => Err(FromIndexError(val)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Index {0} is out of range")]
pub struct FromIndexError(usize);
```

Mantener:

```rust
impl From<Direction> for usize {
    fn from(direction: Direction) -> Self {
        direction.0 as usize
    }
}
```

Mantener también `pub struct Error;` de `Circular`, que sí se usa cuando falla
la tangente.

### 2. `string_art/src/nails/point.rs`

Borrar este bloque:

```rust
impl TryFrom<usize> for Link {
    type Error = FromIndexError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self),
            val => Err(FromIndexError(val)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Index {0} is out of range")]
pub struct FromIndexError(usize);
```

Cambiar el tipo de error del `Builder`:

```rust
type Error = Error;
```

por:

```rust
type Error = core::convert::Infallible;
```

Y borrar el struct:

```rust
#[derive(Debug, thiserror::Error)]
#[error("Point error")]
pub struct Error;
```

`create_segment` ya devuelve `Ok(geometry::Segment::new(*start.0, *end.0))`, así que
sigue compilando con `Infallible`.

### 3. Comprobar que no quedan referencias

```bash
grep -rn "FromIndexError\|TryFrom<usize>" string_art/src
```

No debe quedar ninguna.

## Verificación

```bash
cargo check -p string_art --all-targets
cargo test -p string_art
```

Comprobar además que:

- `nails::circular::Error` sigue existiendo.
- `Point::create_segment` sigue devolviendo `Ok`.
- `<Point as nails::Builder>::Error` es `core::convert::Infallible`.

## Criterio de aceptación

- No existe `FromIndexError` en `string_art/src`.
- No existe `TryFrom<usize>` para `Direction` ni para `point::Link`.
- `cargo check -p string_art --all-targets` y `cargo test -p string_art` pasan.
- No se ha tocado `impl From<Direction> for usize` ni `impl Into<usize> for Link`.

## Fuera de alcance

- No privatizar `nails::circular` ni `nails::point` (eso es el Plan 03).
- No tocar `nails::anchor`, `EllipseBoard` ni `SegmentRef`.

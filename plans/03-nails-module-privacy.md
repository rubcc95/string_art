# Plan 03 — Privatizar los submódulos de `nails`

## Objetivo

Dejar de exponer por path los tipos de `nails` que solo son necesarios como tipos
asociados de `Builder`:

- `nails::point::Link` (solo `<Point as Builder>::Link`).
- `nails::circular::Error` (solo `<Circular as Builder>::Error`).

Además, `nails::anchor::Debugger` es un helper que solo usan los tests unitarios de
`ellipse.rs`. No debe existir en el módulo `anchor`: se mueve directamente al
`mod test` de `ellipse.rs`.

Se mantienen con path público los tipos que un dependiente necesita nombrar
directamente: `Anchor`, `Circular`, `Direction`, `Point`, `Builder`, `Links`, `Handle`.

## Dependencias

- Requiere el Plan 01 (árbol en verde).
- Puede ejecutarse después del Plan 02 o en paralelo en otro worktree. No requiere
  que el Plan 02 esté hecho para compilar, pero es más limpio hacerlo después.
- Es independiente del Plan 04.

## Principio de diseño

Exportar solo lo mínimo para que un dependiente pueda implementar los traits y operar
con los tipos existentes. Los tipos que solo se necesitan a través de un tipo asociado
se dejan **sin path público** y se nombran por proyección:

- `<Point as Builder>::Link`
- `<Circular as Builder>::Error`

No reexportar `Link` ni `Error`.

## Contexto para el agente

- Repositorio: `/home/david/projects/string_art`
- Crate: `string_art`
- `string_art/src/nails/mod.rs` tiene actualmente (esto es el estado de partida,
  **no** el objetivo):

```rust
pub mod anchor;   // <-- en este plan pasa a `mod anchor;`
pub mod circular;
pub mod point;

pub use anchor::Anchor;
pub use circular::Circular;
pub use point::Point;
```

`Anchor` se sigue exportando con `pub use anchor::Anchor;`; el módulo `anchor` no
necesita ser público.

- `ellipse.rs` usa `anchor::Debugger` únicamente dentro de `#[cfg(test)] mod test`.
  Ese helper debe pasar a definirse dentro de ese mismo módulo de tests.
- Los consumidores externos usan `nails::circular::Direction`; hay que actualizarlos
  a `nails::Direction`.

## Cambios

### 1. `string_art/src/nails/mod.rs`

Cambiar las declaraciones y reexports por:

```rust
mod anchor;
mod circular;
mod point;

pub use anchor::Anchor;
pub use circular::{Circular, Direction};
pub use point::Point;
```

`anchor` queda completamente privado. No hace falta `pub(crate)` porque ya no queda
ningún uso de `anchor::Debugger` fuera de `ellipse.rs`, y el helper se mueve al
módulo de tests de `ellipse.rs` en el paso 3.

No reexportar `circular::Error` ni `point::Link`.

### 2. `string_art/src/nails/anchor.rs`

Borrar por completo `Debugger` y su `impl Debug`:

```rust
pub struct Debugger<'a, H, L> {
    pub handle: &'a H,
    pub anchor: &'a Anchor<L>,
}

impl<B: super::Handle> core::fmt::Debug for Debugger<'_, B, B::Link> {
    // ...
}
```

El fichero debe quedarse solo con `Anchor` y su `impl Display`.

### 3. `string_art/src/ellipse.rs` — mover `Debugger` a `mod test`

Dentro de `#[cfg(test)] mod test`, añadir el helper local:

```rust
#[cfg(test)]
mod test {
    use crate::board::SegmentRef;
    use crate::nails::Handle;

    use super::*;

    /// Helper de debug solo para estos tests. Antes vivía en
    /// `nails::anchor::Debugger`, pero no forma parte de la API pública.
    struct Debugger<'a, H, L> {
        handle: &'a H,
        anchor: &'a Anchor<L>,
    }

    impl<H: Handle> core::fmt::Debug for Debugger<'_, H, H::Link> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Anchor")
                .field("idx", &self.anchor.idx)
                .field("link", &self.handle.index_of(&self.anchor.link))
                .finish()
        }
    }

    // ... el resto de test_ellipse y tests, sin cambios salvo los nombres de abajo
}
```

Sustituir las cuatro apariciones de:

```rust
anchor::Debugger {
    handle: &board.handle,
    anchor: &a,
}
```

por:

```rust
Debugger {
    handle: &board.handle,
    anchor: &a,
}
```

y lo mismo para las que usan `&b`.

No cambiar el resto de la lógica de los tests.

### 4. Actualizar consumidores de `Direction`

En:

- `string_art/examples/basic_example.rs`
- `string_art/examples/parallel_check.rs`
- `string_art_gpui/src/bench.rs`

cambiar:

```rust
use string_art::nails::{self, circular::Direction};
```

por:

```rust
use string_art::nails::{self, Direction};
```

y en `bench.rs`, si el import es:

```rust
nails::{Anchor, Circular, circular::Direction}
```

cambiarlo por:

```rust
nails::{Anchor, Circular, Direction}
```

Buscar restos:

```bash
grep -rn "nails::circular::Direction\|circular::Direction" string_art string_art_gpui
```

### 5. Comprobar que no queda ninguna referencia a `anchor::Debugger`

```bash
grep -rn "anchor::Debugger\|Debugger" string_art/src
```

Solo debe aparecer la definición local dentro de `#[cfg(test)] mod test` de
`ellipse.rs` y sus usos en ese mismo módulo.

## Verificación

```bash
cargo check -p string_art --all-targets
cargo test -p string_art
cargo check -p string_art_gpui
```

Comprobar desde un crate externo (o con un test temporal) que:

- `string_art::nails::Anchor`, `Circular`, `Direction`, `Point` compilan.
- `string_art::nails::point::Link` **no** compila (E0603).
- `string_art::nails::circular::Error` **no** compila (E0603).
- `string_art::nails::anchor` **no** compila (E0603).
- `<Point as string_art::nails::Builder>::Link` compila.
- `<Circular as string_art::nails::Builder>::Error` compila.

## Criterio de aceptación

- `nails::Direction` es el path público para `Direction`.
- No hay path público para `point::Link`, `circular::Error` ni `nails::anchor`.
- `Debugger` ya no existe en `nails::anchor`; solo existe dentro de
  `#[cfg(test)] mod test` de `ellipse.rs`.
- `cargo check -p string_art --all-targets`, `cargo test -p string_art` y
  `cargo check -p string_art_gpui` pasan.
- No se ha cambiado el comportamiento de ningún trait ni de ningún cálculo.

## Fuera de alcance

- No tocar `EllipseBoard` ni el `SegmentRef` concreto de `ellipse.rs` (Plan 04).
- No borrar `TryFrom` ni `point::Error` (Plan 02).
- No cambiar las firmas de `Builder`, `Links` ni `Handle`.

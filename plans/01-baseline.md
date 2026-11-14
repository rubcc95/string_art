# Plan 01 — Dejar el árbol en verde

## Objetivo

Arreglar los dos imports rotos que dejan `string_art` sin compilar en tests/examples
después de convertir `decay` y `monocolor` en módulos privados. No se cambia ninguna
visibilidad ni comportamiento en este plan.

## Contexto para el agente

- Repositorio: `/home/david/projects/string_art`
- Crate: `string_art` (workspace Rust, edition 2024).
- `string_art/src/lib.rs` declara:
  - `mod decay;`
  - `mod monocolor;`
  - `mod num;`
  y reexporta `Decay`, `Monocolor`, `Frac`, `Scalar` en la raíz.
- Por eso los consumidores externos ya no pueden usar `string_art::decay::...` ni
  `string_art::monocolor::...`. Deben usar las reexportaciones de la raíz.

Errores actuales:

```text
error[E0603]: module `monocolor` is private
  --> string_art/tests/local_anchor.rs:14:17

error[E0433]: cannot find module or crate `decay` in this scope
  --> string_art/examples/basic_example.rs:49:9
```

## Cambios

### 1. `string_art/tests/local_anchor.rs`

Cambiar:

```rust
use string_art::monocolor::Monocolor;
```

por:

```rust
use string_art::Monocolor;
```

No tocar el resto del test. En particular, `Monocolor::new(...)` y
`monocolor.map.weights` deben seguir funcionando igual.

### 2. `string_art/examples/basic_example.rs`

Cambiar la construcción de `Decay`:

```rust
decay::Decay::Flat(Frac::from_num(0.2)),
```

por:

```rust
Decay::Flat(Frac::from_num(0.2)),
```

El fichero ya tiene `use string_art::*;`, que trae `Decay` desde la raíz.

### 3. `string_art/examples/parallel_check.rs`

Hacer el mismo cambio dentro de `mod comparison`:

```rust
decay::Decay::Flat(Frac::from_num(0.2)),
```

por:

```rust
Decay::Flat(Frac::from_num(0.2)),
```

### 4. Buscar restos

Ejecutar:

```bash
grep -rn "decay::Decay\|monocolor::Monocolor" string_art string_art_gpui
```

Si aparece algún otro uso, cambiarlo por la reexportación de la raíz
(`Decay` o `Monocolor`).

## Verificación

```bash
cargo check -p string_art --all-targets
cargo test -p string_art
```

Opcional, para comprobar que la app externa sigue compilando:

```bash
cargo check -p string_art_gpui
```

## Criterio de aceptación

- `cargo check -p string_art --all-targets` termina sin errores.
- `cargo test -p string_art` pasa.
- No se ha modificado ninguna visibilidad ni ningún comportamiento.

## Fuera de alcance

- No privatizar submódulos de `nails`.
- No tocar `EllipseBoard`, `SegmentRef`, `TryFrom`, `FromIndexError` ni `point::Error`.
- No reorganizar módulos ni renombrar tipos.

mod anchor;
pub mod circular;
mod point;

pub use anchor::Anchor;
pub use circular::Circular;
pub use point::Point;

use crate::geometry;
use crate::sync::*;

pub trait Builder {
    #[cfg(feature = "multithreaded")]
    type Nail: Sync;
    #[cfg(not(feature = "multithreaded"))]
    type Nail;
    type Links: Links<Link = Self::Link>;
    type Handle: Handle<Link = Self::Link, Nail = Self::Nail>;

    #[cfg(feature = "multithreaded")]
    type Link: Clone + Sync + Send + Default;
    #[cfg(not(feature = "multithreaded"))]
    type Link: Clone + Default;

    type Error: std::error::Error;

    const LINKS: Self::Links;

    fn create_nail(&self, position: geometry::Point<f32>, rotation: f32) -> Self::Nail;

    fn create_segment(
        &self,
        start: (&Self::Nail, Self::Link),
        end: (&Self::Nail, Self::Link),
    ) -> Result<geometry::Segment<f32>, Self::Error>;

    fn anchor_builder(self) -> Self::Handle;
}

/// Conjunto de enlaces que salen de un clavo.
///
/// La implementación habitual es un array —`[Direction; 2]`, `[Link; 1]`—, que
/// obtiene `LEN` de su propia longitud. Si alguien implementa esto a mano:
///
/// # Contrato
///
/// * `LEN` debe coincidir con el número de elementos que produce el iterador.
/// * `Handle::index_of` debe numerar esos enlaces de `0` a `LEN - 1`, sin
///   repetir y **en el mismo orden en que los recorre el iterador**.
///
/// Lo segundo no es cosmético: `EllipseBoard::index_of` usa ese número como
/// coordenada dentro de un bloque de `LEN * LEN` segmentos. Un desajuste
/// produce índices equivocados; el acceso al segmento va comprobado, así que
/// eso es un panic y no una lectura fuera de rango. `Ellipse::new` lo
/// verifica en debug.
pub trait Links: IntoIterator<Item = Self::Link> {
    type Link;

    const LEN: usize;
    const SQ_LEN: usize = Self::LEN * Self::LEN;
}

/// La longitud la lleva el tipo, así que `LEN` no puede discrepar del número de
/// elementos que produce el iterador. Es lo que sustituye al antiguo
/// `unsafe impl`.
impl<L, const N: usize> Links for [L; N] {
    type Link = L;

    const LEN: usize = N;
}

pub trait Handle: CondSync {
    type Nail: CondSync;
    type Link: Default;

    fn next_anchor(&self, anchor: Anchor<Self::Link>) -> Anchor<Self::Link>;

    fn index_of(&self, link: &Self::Link) -> usize;

    fn reversed(&self, anchor: Anchor<Self::Link>) -> Anchor<Self::Link>;
}

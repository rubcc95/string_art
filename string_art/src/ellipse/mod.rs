mod internal;

use crate::*;
use board::*;
use geometry::Rect;
use nails::*;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Ellipse<N: nails::Builder> {
    segments: Vec<Segment>,
    handle: N::Handle,
    min_nail_distance: usize,
    nail_count: usize,
    // No se lee después de construir, pero se conserva como parte del estado
    // del tablero. Si un consumidor lo necesita, se añadirá un accesor.
    #[allow(dead_code)]
    nails: Vec<N::Nail>,
}

impl<N: nails::Builder> Ellipse<N> {
    pub fn new(
        rect: Rect<f32>,
        nail_builder: N,
        nail_count: usize,
        min_nail_distance: usize,
    ) -> Result<Self, Error<N::Error>> {
        // La cota no es opcional: `into_board` calcula `nail_count - 2 * min - 1`
        // y `index_of` despeja `nail_count - (min + 1)`. Con `min` demasiado
        // grande eso desborda (en release, envolviendo el índice de segmento).
        let max_distance = nail_count.saturating_sub(1) / 2;
        if min_nail_distance > max_distance {
            return Err(Error::Distancer(max_distance));
        }

        let nails: Vec<_> = (0..nail_count)
            .map(|i| {
                let theta = 2.0 * PI * i as f32 / nail_count as f32;
                nail_builder.create_nail(
                    geometry::Point {
                        x: 0.5 * rect.width * (1.0 + theta.cos()),
                        y: 0.5 * rect.height * (1.0 + theta.sin()),
                    },
                    theta,
                )
            })
            .collect();

        let nails_view = nails.as_slice();
        let builder = &nail_builder;

        let this = Self {
            segments: Self::get_anchors(nail_count, min_nail_distance)
                .map(|(big, small)| {
                    Ok(builder
                        .create_segment(
                            (&nails_view[small.idx], small.link),
                            (&nails_view[big.idx], big.link),
                        )?
                        .into())
                })
                .collect::<Result<Vec<_>, N::Error>>()
                .map_err(Error::Nail)?,
            handle: nail_builder.anchor_builder(),
            min_nail_distance,
            nail_count,
            nails,
        };
        #[cfg(debug_assertions)]
        Self::debug_assert_links(&this.handle);
        Ok(this)
    }

    /// Verifica una sola vez, y solo en debug, el contrato de `Links`: que el
    /// iterador produzca `LEN` enlaces y que `Handle::index_of` los numere de
    /// `0` a `LEN - 1` sin repetir. Es el chequeo que sustituye a la antigua
    /// promesa del `unsafe trait`.
    #[cfg(debug_assertions)]
    fn debug_assert_links(handle: &N::Handle) {
        let links: Vec<_> = N::LINKS.into_iter().collect();
        assert_eq!(
            links.len(),
            <N::Links as nails::Links>::LEN,
            "El iterador de `Links` produce un número de enlaces distinto de `LEN`"
        );
        let mut seen = vec![false; links.len()];
        for link in &links {
            let idx = handle.index_of(link);
            assert!(
                idx < seen.len() && !seen[idx],
                "`Handle::index_of` debe numerar los enlaces de 0 a LEN - 1 sin repetir; devolvió {idx} con LEN = {}",
                seen.len()
            );
            seen[idx] = true;
        }
    }

    fn get_anchors(
        nail_count: usize,
        min_nail_distance: usize,
    ) -> impl Iterator<Item = (nails::Anchor<N::Link>, nails::Anchor<N::Link>)> {
        (0..nail_count)
            .into_iter()
            .map(move |big| {
                let dt_to_end = nail_count - big - 1; //1
                let range = min_nail_distance.saturating_sub(dt_to_end) //0..499
                    ..big.saturating_sub(min_nail_distance);
                range.into_iter().flat_map(move |small| {
                    N::LINKS.into_iter().flat_map(move |b_link| {
                        N::LINKS.into_iter().map(move |s_link| {
                            (
                                Anchor {
                                    idx: big,
                                    link: b_link.clone(),
                                },
                                Anchor {
                                    idx: small,
                                    link: s_link,
                                },
                            )
                        })
                    })
                })
            })
            .flatten()
    }

    fn index_of(&self, a: nails::Anchor<N::Link>, b: nails::Anchor<N::Link>) -> Option<usize> {
        let (mut big, mut small) = match a.idx.cmp(&b.idx) {
            std::cmp::Ordering::Less => (self.handle.reversed(b), self.handle.reversed(a)),
            std::cmp::Ordering::Equal => return None,
            std::cmp::Ordering::Greater => (a, b),
        };

        let cap = self.nail_count - (self.min_nail_distance + 1); //479
        let first = match big.idx.checked_sub(cap) {
            // 1
            Some(diff) => {
                big.idx = cap;
                small.idx = small.idx.checked_sub(diff)?;
                diff * (cap - self.min_nail_distance)
            }
            None => 0,
        };

        let diff = big.idx.checked_sub(self.min_nail_distance)?; // 1
        let index = first + diff * (diff - 1) / 2 + small.idx;
        Some(
            <N::Links as nails::Links>::SQ_LEN * index
                + <N::Links as nails::Links>::LEN * self.handle.index_of(&big.link)
                + self.handle.index_of(&small.link),
        )
    }
}

struct SegmentRef<'a, N: nails::Builder> {
    segment: &'a Segment,
    next_anchor: nails::Anchor<N::Link>,
    index: usize,
}

impl<'a, N: nails::Builder> board::SegmentRef for SegmentRef<'a, N> {
    type Anchor = nails::Anchor<N::Link>;

    fn segment(&self) -> &Segment {
        self.segment
    }

    fn index(&self) -> usize {
        self.index
    }

    fn anchor(&self) -> Self::Anchor {
        self.next_anchor.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<N> {
    #[error(transparent)]
    Nail(N),
    #[error("The minimum distance between nails must be smaller than {0}.")]
    Distancer(usize),
}

#[cfg(test)]
mod test {
    use crate::board::SegmentRef;
    use crate::nails::Handle;

    use super::*;
    use internal::EllipseBoard;

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

    pub fn test_ellipse<N: nails::Builder>(
        nails: N,
        nail_count: usize,
        nail_distance: usize,
        batches: usize,
    ) {
        let ellipse =
            Ellipse::new(Rect::new(1000.0, 1000.0), nails, nail_count, nail_distance).unwrap();
        let board = ellipse.into_board(batches);
        let mut i = 0;
        for (a, b) in Ellipse::<N>::get_anchors(nail_count, nail_distance) {
            match board.index_of(a.clone(), b.clone()) {
                Some(index) => {
                    assert_eq!(
                        i,
                        index,
                        "Expected index {}, but got {} for anchors: {:?} and {:?}",
                        i,
                        index,
                        Debugger {
                            handle: &board.handle,
                            anchor: &a
                        },
                        Debugger {
                            handle: &board.handle,
                            anchor: &b
                        }
                    );
                    i += 1;
                }
                None => panic!(
                    "Failed to get index for anchors: {:?} and {:?}",
                    Debugger {
                        handle: &board.handle,
                        anchor: &a
                    },
                    Debugger {
                        handle: &board.handle,
                        anchor: &b
                    }
                ),
            }
        }
        let mut counter = vec![0; board.segments.len()];

        for anchor_idx in 0..nail_count {
            for link in N::LINKS.into_iter() {
                for batch_idx in 0..batches {
                    for item in EllipseBoard::batch(
                        &board,
                        &Anchor {
                            idx: anchor_idx,
                            link: link.clone(),
                        },
                        batch_idx,
                    ) {
                        let i = SegmentRef::index(&item);
                        let anchor = SegmentRef::anchor(&item);
                        assert!(
                            board.segments.len() > i,
                            "Overflow range from: ({}), to: ({}) -> {}. (Max value expected: {})",
                            anchor_idx,
                            anchor.idx,
                            i,
                            board.segments.len()
                        );
                        counter[i] += 1;
                    }
                }
            }
        }
        assert!(counter.iter().all(|&c| c == 2));
    }

    #[test]
    pub fn test() {
        test_ellipse(Circular(0.5), 10, 1, 4);
        test_ellipse(Point, 123, 1, 10);
    }

    /// Antes estas cotas desbordaban `into_board` en lugar de devolver el error
    /// que la API ya declaraba: `Distancer` era una variante muerta.
    #[test]
    fn rejects_min_distance_too_large() {
        let result = Ellipse::new(Rect::new(10.0, 10.0), Point, 10, 5);
        assert!(matches!(result, Err(Error::Distancer(4))));
    }
}

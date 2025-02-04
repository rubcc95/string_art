use crate::nails;
use thiserror::Error;

#[derive(Clone, Copy)]
pub struct NailDistancer {
    min: usize,
    max: usize,
}

impl NailDistancer {
    pub fn new(count: usize, distance: usize) -> Result<Self, Error> {
        if count < 2 * distance {
            Err(Error((1 + count) / 2))
        } else {
            Ok(Self {
                min: distance,
                //SAFETY: checked bounds in if
                max: unsafe { count.unchecked_sub(distance) },
            })
        }
    }

    // pub fn min(&self) -> usize{
    //     self.min
    // }

    pub fn distance(&self) -> usize {
        unsafe { self.max.unchecked_sub(self.min) }
    }   

    //NOTE: This does not check if a_idx and b_idx are inside bounds.
    // a_idx and b_idx must be also inside bounds in order to be valid.
    pub fn is_valid(&self, a_idx: usize, b_idx: usize) -> bool {
        let diff = a_idx.abs_diff(b_idx);
        diff > self.min && diff < self.max
    }

    // pub fn index_of<L: nails::Links>(
    //     &self,
    //     a_idx: usize,
    //     a_link: L::Item,
    //     b_idx: usize,
    //     b_link: L::Item,
    // ) -> Option<usize> {
    //     if self.is_valid(a_idx, b_idx) {
    //         Some(unsafe { self.index_of_unchecked::<L>(a_idx, a_link, b_idx, b_link) })
    //     } else {
    //         None
    //     }
    // }

    //SAFETY: caller must ensure that the indices are valid via NailDistancer::is_valid
    pub unsafe fn index_of_unchecked<L: nails::Links>(
        &self,
        a_idx: usize,
        a_link: L::Item,
        b_idx: usize,
        b_link: L::Item,
    ) -> usize {
        let (mut big_idx, big_link, mut small_idx, small_link) = if a_idx > b_idx {
            (a_idx, a_link, b_idx, b_link)
        } else {
            (b_idx, b_link, a_idx, a_link)
        };
        let cap = self.max.unchecked_sub(1);
        let first = if big_idx > cap {
            let diff = big_idx.unchecked_sub(cap);
            big_idx = cap;
            small_idx = small_idx.unchecked_sub(diff);
            diff.unchecked_mul(cap.unchecked_sub(self.min)).unchecked_mul(L::SQ_LEN)
        } else {
            0
        };

        let diff = big_idx.unchecked_sub(self.min);

        first
            .unchecked_add((diff * diff.unchecked_sub(1) / 2).unchecked_mul(L::SQ_LEN))
            .unchecked_add(L::LEN.unchecked_mul(diff).unchecked_mul(big_link.into()))
            .unchecked_add(L::LEN.unchecked_mul(small_idx))
            .unchecked_add(small_link.into())
    }
}

#[derive(Debug, Error)]
#[error("The minimum distance between nails must be smaller than {0}.")]
pub struct Error(usize);

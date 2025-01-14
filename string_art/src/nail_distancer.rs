use crate::nails;
use thiserror::Error;

#[derive(Clone, Copy)]
pub struct NailDistancer {
    min: usize,
    max: usize,
}

impl NailDistancer {
    pub fn new(count: usize, distance: usize) -> Result<Self, NailDistanceError> {
        if count < 2 * distance {
            Err(NailDistanceError((1 + count) / 2))
        } else {
            Ok(Self {
                min: distance,
                //SAFETY: checked bounds in if
                max: unsafe {
                    count.unchecked_sub(distance.unchecked_add(1))
                },
            })
        }
    }

    pub fn is_valid(&self, a_idx: usize, b_idx: usize) -> bool {
        let diff = a_idx.abs_diff(b_idx);
        diff > self.min && diff < self.max
    }

    pub fn index_of<L: nails::Links>(
        &self,
        a_idx: usize,
        a_link: L::Item,
        b_idx: usize,
        b_link: L::Item,
    ) -> Option<usize> {
        if self.is_valid(a_idx, b_idx) {
            Some(unsafe { self.index_of_unchecked::<L>(a_idx, a_link, b_idx, b_link) })
        } else {
            None
        }
    }

    //SAFETY: caller must ensure that the indices are valid
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
        let first_part = if big_idx > self.max {
            let diff = big_idx.unchecked_sub(self.max);
            big_idx = self.max;
            small_idx = small_idx.unchecked_sub(diff);
            diff * self.max.unchecked_sub(self.min) * L::SQ_LEN
        } else {
            0
        };

        let diff = big_idx - self.min;

        first_part + diff * diff.unchecked_sub(1) * L::SQ_LEN / 2
            + L::LEN * diff * big_link.into()
            + L::LEN * small_idx
            + small_link.into()
    }
}

#[derive(Debug, Error)]
#[error("The minimum distance between nails must be smaller than {0}.")]
pub struct NailDistanceError(usize);

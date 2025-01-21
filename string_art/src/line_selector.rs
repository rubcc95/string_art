use std::ops::Deref;

use crate::{
    verboser::Verboser, AsLab, Image
};

pub unsafe trait Builder<S> {
    fn build_line_selector(
        &self,
        image: &Image<S>,
        palette: &[impl AsLab<S>],
        verboser: &mut impl Verboser,
    ) -> Result<LineSelector, Error>;
}

#[derive(Clone, Copy)]
pub struct LineItemSelector {
    color_idx: usize,
    count: usize,
    cap: usize,
}

impl LineItemSelector {
    pub fn new(color_idx: usize, count: usize, cap: usize) -> Self {
        LineItemSelector {
            color_idx,
            count,
            cap,
        }
    }

    pub fn color_idx(&self) -> usize {
        self.color_idx
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

pub struct LineGroupSelector(Vec<LineItemSelector>);

impl FromIterator<LineItemSelector> for LineGroupSelector {
    fn from_iter<T: IntoIterator<Item = LineItemSelector>>(iter: T) -> Self {
        LineGroupSelector(iter.into_iter().collect())
    }
}

impl LineGroupSelector {
    fn select_next(&mut self) -> Option<usize> {
        let mut choice = None;
        let mut best_ratio = 1.0;
        for item in self.0.iter_mut() {
            let ratio = item.count as f32 / item.cap as f32;
            if ratio < best_ratio {
                best_ratio = ratio;
                choice = Some(item);
            }
        }

        choice.map(|item| {
            item.count += 1;
            item.color_idx
        })
    }    
}

impl Deref for LineGroupSelector{
    type Target = [LineItemSelector];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct LineSelector{
    lines: Vec<LineGroupSelector>,
    curr: usize,
}

impl LineSelector {
    pub (crate) fn select_next(&mut self) -> Option<usize> {        
        while let Some(last) = self.lines.get_mut(self.curr) {
            if let Some(res) = last.select_next() {
                return Some(res);
            } else {
                self.curr = if self.curr == 0{
                    self.lines.len()
                } else{
                    unsafe { self.curr.unchecked_sub(1) }
                }
            }
        }

        None
    }    
}

impl Deref for LineSelector {
    type Target = [LineGroupSelector];

    fn deref(&self) -> &Self::Target {
        &self.lines
    }
}

impl FromIterator<LineGroupSelector> for LineSelector {
    fn from_iter<T: IntoIterator<Item = LineGroupSelector>>(iter: T) -> Self {
        let lines: Vec<_> = iter.into_iter().collect();
        LineSelector{
            curr: lines.len().checked_sub(1).unwrap_or(lines.len()),
            lines, 
        
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid group index")]
pub struct Error;
use crate::{
    ditherer::{DitherCounter, DitherError, Ditherer}, AsLab, Float, Image, Lab
};
use num_traits::AsPrimitive;
use thiserror::Error;

pub trait LineCounterBuilder<S> {    
    fn line_counters(
        &self,
        image: &mut Image<S>,
        palette: &[impl AsLab<S>],
    ) -> Result<LineCounter, LineCounterError>;
}

#[derive(Clone, Copy)]
pub struct LineCounterItem {
    color_idx: usize,
    count: usize,
    cap: usize,
}

pub type LineCounterGroup = Vec<LineCounterItem>;

pub type LineCounter = Vec<LineCounterGroup>;


#[derive(Clone, Copy)]
pub struct LineCountItem {
    color_idx: usize,
    cap: usize,
}

pub type LineCountGroup = Vec<LineCountItem>;

pub type LineCount = Vec<LineCountGroup>;

impl<S> LineCounterBuilder<S> for LineCount{
    fn line_counters(
        &self,
        _: &mut Image<S>,
        palette: &[impl AsLab<S>],
    ) -> Result<LineCounter, LineCounterError> {
        self.iter().map(|group|{
            group.iter().map(|item|{
                if item.color_idx >= palette.len(){
                    Err(LineCounterError::InvalidGroupIndex)
                } else{
                    Ok(LineCounterItem{
                        color_idx: item.color_idx,
                        count: 0,
                        cap: item.cap,
                    })
                }
            }).collect()
        }).collect()
    }
}

#[derive(Clone)]
pub struct AutoLineCount<S> {
    pub groups: Vec<AutoLineCountGroup<S>>,
    pub threads: usize,
}

impl<S: Float> LineCounterBuilder<S> for AutoLineCount<S>
where
    usize: AsPrimitive<S>,
{
    fn line_counters(
        &self,
        image: &mut Image<S>,
        palette: &[impl AsLab<S>],
    ) -> Result<LineCounter, LineCounterError> {
        let mut dither_counters: Vec<AutoLineCountDitherCounter<S>> = palette
            .iter()
            .map(|color| AutoLineCountDitherCounter {
                lab: color.as_lab(),
                weight: S::ZERO,
                pixel_count: S::ZERO,
            })
            .collect();

        Ditherer::floyd_steinberg(dither_counters.as_mut_slice())
            .dither(image)
            .map_err(|err| LineCounterError::Dither(err))?;

        for group in &self.groups {
            let weight = group.weight;
            for &index in &group.colors {
                match dither_counters.get_mut(index) {
                    Some(counter) => counter.weight += weight,
                    None => return Err(LineCounterError::InvalidGroupIndex),
                }
            }
        }

        let pixel_count = image.pixels().len().as_();
        let threads = self.threads.as_();

        Ok(self
            .groups
            .iter()
            .map(|group| {
                group
                    .colors
                    .iter()
                    .map(|&idx| {
                        let counter = unsafe { dither_counters.get_unchecked(idx) };
                        let prop =
                            (group.weight * counter.pixel_count) / (pixel_count * counter.weight);
                        LineCounterItem {
                            color_idx: idx,
                            count: 0,
                            cap: (threads * prop).to_usize().unwrap(),
                        }
                    })
                    .collect()
            })
            .collect())
    }
}

#[derive(Clone)]
pub struct AutoLineCountGroup<S> {
    pub colors: Vec<usize>,
    pub weight: S,
}

struct AutoLineCountDitherCounter<S> {
    lab: Lab<S>,
    weight: S,
    pixel_count: S,
}

impl<S: Float> DitherCounter<S> for AutoLineCountDitherCounter<S> {
    fn color(&self) -> Lab<S> {
        self.lab
    }

    fn add_pixel(&mut self) {
        self.pixel_count += S::ONE;
    }
}

#[derive(Debug, Error)]
pub enum LineCounterError {
    #[error("Invalid group index")]
    InvalidGroupIndex,
    #[error(transparent)]
    Dither(DitherError),
}

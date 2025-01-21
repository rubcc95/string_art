use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    line_selector::{self, LineItemSelector, LineSelector}, verboser::Verboser, AsLab, Image
};
    
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct LineItemConfig {
    pub color_idx: usize,
    pub cap: usize,
}

impl LineItemConfig {
    pub fn new(color_idx: usize, cap: usize) -> Self {
        LineItemConfig { color_idx, cap }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct LineGroupConfig<C = Vec<LineItemConfig>>(C);

impl<C> LineGroupConfig<C> {
    pub fn new(items: C) -> Self {
        LineGroupConfig(items)
    }
}   
impl<C> Deref for LineGroupConfig<C>{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for LineGroupConfig<C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct LineConfig<G = Vec<LineGroupConfig>, C = Vec<LineItemConfig>> {
    groups: G,
    _panthom: std::marker::PhantomData<C>,
}

impl<G, C> LineConfig<G, C>{
    pub fn new(groups: G) -> Self{
        Self{
            groups,
            _panthom: std::marker::PhantomData,
        }
    }
}

impl<G, C> Deref for LineConfig<G, C>{
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

impl<G, C> DerefMut for LineConfig<G, C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.groups
    }
}

impl From<&LineSelector> for LineConfig{
    fn from(value: &LineSelector) -> Self {
        LineConfig::new(value.iter().map(|group| {
            LineGroupConfig::new(group.iter().map(|item| {
                LineItemConfig::new(item.color_idx(), item.cap())
            }).collect())
        }).collect())   
    }
}

unsafe impl<S, G, C> line_selector::Builder<S> for LineConfig<G, C>
where
    G: AsRef<[LineGroupConfig<C>]>,
    C: AsRef<[LineItemConfig]>,
{
    fn build_line_selector(
        &self,
        _: &Image<S>,
        palette: &[impl AsLab<S>],
        _: &mut impl Verboser,
    ) -> Result<LineSelector, line_selector::Error> {
        self.groups
            .as_ref()
            .iter()
            .map(|group| {
                group
                    .0
                    .as_ref()
                    .iter()
                    .map(|item| {
                        if item.color_idx >= palette.len() {
                            Err(line_selector::Error)
                        } else {
                            Ok(LineItemSelector::new(item.color_idx, 0, item.cap))
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

impl From<LineItemConfig> for LineItemSelector {
    fn from(value: LineItemConfig) -> Self {
        LineItemSelector::new(value.color_idx, 0, value.cap)
    }
}

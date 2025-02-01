use egui::WidgetText;
use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};
use string_art::{
    auto_line_config::AutoLineGroupConfig,
    line_config::{Group, Item},
    color_handle::{self, Handle},
    verboser::Verboser,
    AutoLineConfig, ColorWeight, Float, Image, Config,
};

use super::NamedColor;

#[derive(Clone, Serialize, Deserialize)]
pub struct ArgLineCount {
    pub manual: string_art::Config,
    pub auto: AutoLineConfig<f32>,
    pub state: ArgLineCountState,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArgLineCountState {
    Manual,
    Auto,
}

impl From<&ArgLineCount> for WidgetText {
    fn from(value: &ArgLineCount) -> Self {
        match value.state {
            ArgLineCountState::Manual => "Manual".into(),
            ArgLineCountState::Auto => "Auto".into(),
        }
    }
}

impl ArgLineCount {
    pub fn new(
        manual: string_art::Config,
        auto: AutoLineConfig<f32>,
        state: ArgLineCountState,
    ) -> Self {
        Self {
            manual,
            auto,
            state,
        }
    }

    pub fn form(&mut self, ui: &mut egui::Ui, palette: &[NamedColor]) {
        egui::ComboBox::from_id_salt(self as *mut _)
            .selected_text(&*self)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.state, ArgLineCountState::Auto, "Auto");
                ui.selectable_value(&mut self.state, ArgLineCountState::Manual, "Manual");
            });
        match self.state {
            ArgLineCountState::Manual => {
                Self::manual_form(&mut self.manual, ui, palette);
            }
            ArgLineCountState::Auto => {
                Self::auto_form(&mut self.auto, ui, palette);
            }
        }
    }

    fn manual_form(groups: &mut Config, ui: &mut egui::Ui, palette: &[NamedColor]) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label("Color order:");
            if palette.len() > 0 && ui.button("+").clicked() {
                groups.push(Group::new(
                    (0..palette.len())
                        .into_iter()
                        .map(|idx| Item::new(idx, 1000))
                        .collect(),
                ));
            }
        });
        let mut removed_group = None;
        for (group_idx, group) in groups.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                //ui.label();
                if ui.button("🗑").clicked() {
                    removed_group = Some(group_idx);
                }
                if ui.button("+").clicked() {
                    group.push(Item::new(0, 1000));
                }
                egui::CollapsingHeader::new(format!("Group {}.", group_idx + 1))
                    .default_open(false)
                    .show(ui, |ui| {
                        let mut removed_element = None;
                        for (element_idx, item_config) in group.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add_space(30.0);
                                egui::ComboBox::from_id_salt(item_config as *mut _)
                                    .selected_text(unsafe {
                                        &palette.get_unchecked(item_config.color_idx).name
                                    })
                                    .show_ui(ui, |ui| {
                                        for (idx, color) in palette.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut item_config.color_idx,
                                                idx,
                                                &color.name,
                                            );
                                        }
                                    });
                                ui.add(
                                    egui::Slider::new(&mut item_config.cap, 1..=10000)
                                        .clamping(egui::SliderClamping::Never),
                                );
                                if ui.button("🗑").clicked() {
                                    removed_element = Some(element_idx);
                                }
                            });
                        }
                        if let Some(idx) = removed_element {
                            group.remove(idx);
                        }
                    });
            });
        }
        if let Some(idx) = removed_group {
            groups.remove(idx);
        }
    }

    fn auto_form(groups: &mut AutoLineConfig<f32>, ui: &mut egui::Ui, palette: &[NamedColor]) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label("Threads:")
                .on_hover_text("Number of threads used when generating the image.");
            ui.add(
                egui::Slider::new(&mut groups.threads, 1..=20000)
                    .clamping(egui::SliderClamping::Never),
            );
        });
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label("Color order:");
            if palette.len() > 0 && ui.button("+").clicked() {
                groups.push(AutoLineGroupConfig::new(
                    (0..palette.len()).into_iter().collect(),
                    0.5,
                ));
            }
        });
        let mut removed_group = None;
        for (group_idx, group) in groups.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                if ui.button("🗑").clicked() {
                    removed_group = Some(group_idx);
                }
                ui.add(egui::Slider::new(&mut group.weight, 0.0..=1.0).show_value(false));
                if ui.button("+").clicked() {
                    group.push(0);
                }
                egui::CollapsingHeader::new(format!("Group {}.", group_idx + 1))
                    .default_open(false)
                    .show(ui, |ui| {
                        let mut removed_element = None;
                        for (element_idx, color_idx) in group.colors.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt(color_idx as *mut _)
                                    .selected_text(unsafe {
                                        &palette.get_unchecked(*color_idx).name
                                    })
                                    .show_ui(ui, |ui| {
                                        for (idx, color) in palette.iter().enumerate() {
                                            ui.selectable_value(color_idx, idx, &color.name);
                                        }
                                    });
                                if ui.button("🗑").clicked() {
                                    removed_element = Some(element_idx);
                                }
                            });
                        }
                        if let Some(idx) = removed_element {
                            group.colors.remove(idx);
                        }
                    });
            });
        }
        if let Some(idx) = removed_group {
            groups.remove(idx);
        }
    }
}

unsafe impl<S: Float> color_handle::Builder<S> for ArgLineCount
where
    f32: AsPrimitive<S>,
    usize: AsPrimitive<S>,
{
    fn build_line_selector<L>(
        &self,
        image: &Image<S>,
        palette: &[ColorWeight<L, S>],
        verboser: &mut impl Verboser,
    ) -> Result<Handle, color_handle::Error> {
        match self.state {
            ArgLineCountState::Manual => self.manual.build_line_selector(image, palette, verboser),
            ArgLineCountState::Auto => self.auto.build_line_selector(image, palette, verboser),
        }
    }
}

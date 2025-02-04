use egui::WidgetText;
use serde::{Deserialize, Serialize};
use string_art::color::{self, config::multi};

#[derive(Clone, Serialize, Deserialize)]
pub struct LineConfig {
    pub manual: multi::Manual,
    pub auto: multi::Auto,
    pub single: usize,
    pub state: LineConfigState,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineConfigState {
    Manual,
    Auto,
}

impl From<&LineConfig> for WidgetText {
    fn from(value: &LineConfig) -> Self {
        match value.state {
            LineConfigState::Manual => "Manual".into(),
            LineConfigState::Auto => "Auto".into(),
        }
    }
}

impl LineConfig {
    pub fn new(
        manual: multi::Manual,
        auto: multi::Auto,
        single: usize,
        state: LineConfigState,
    ) -> Self {
        Self {
            manual,
            auto,
            single,
            state,
        }
    }

    pub fn form(&mut self, ui: &mut egui::Ui, palette: &[color::Named]) {
        if palette.len() > 1 {
            egui::ComboBox::from_id_salt(self as *mut _)
                .selected_text(&*self)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.state, LineConfigState::Auto, "Auto");
                    ui.selectable_value(&mut self.state, LineConfigState::Manual, "Manual");
                });
            match self.state {
                LineConfigState::Manual => {
                    Self::manual_form(&mut self.manual, ui, palette);
                }
                LineConfigState::Auto => {
                    Self::auto_form(&mut self.auto, ui, palette);
                }
            }
        } else {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label("Threads:")
                    .on_hover_text("Number of threads used when generating the image.");
                ui.add(
                    egui::Slider::new(&mut self.single, 1..=20000)
                        .clamping(egui::SliderClamping::Never),
                );
            });
        }
    }

    fn manual_form(groups: &mut multi::Manual, ui: &mut egui::Ui, palette: &[color::Named]) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label("Color order:");
            if palette.len() > 0 && ui.button("+").clicked() {
                groups.push(multi::manual::Group::new(
                    (0..palette.len())
                        .into_iter()
                        .map(|idx| multi::manual::Item::new(idx, 1000))
                        .collect(),
                ));
            }
        });
        let mut removed_group = None;
        let groups_len = groups.len();
        for (group_idx, group) in groups.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                //ui.label();
                if groups_len > 1 && ui.button("🗑").clicked() {
                    removed_group = Some(group_idx);
                }
                if ui.button("+").clicked() {
                    group.push(multi::manual::Item::new(0, 1000));
                }
                egui::CollapsingHeader::new(format!("Group {}.", group_idx + 1))
                    .default_open(false)
                    .show(ui, |ui| {
                        let mut removed_element = None;
                        let element_len = group.len();
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
                                if element_len > 1 && ui.button("🗑").clicked() {
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

    fn auto_form(groups: &mut multi::Auto, ui: &mut egui::Ui, palette: &[color::Named]) {
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
                groups.push(multi::auto::Group::new(
                    (0..palette.len()).into_iter().collect(),
                    0.5,
                ));
            }
        });
        let mut removed_group = None;
        let groups_len = groups.len();
        for (group_idx, group) in groups.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                if groups_len > 1 && ui.button("🗑").clicked() {
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
                        let element_len = group.colors.len();
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
                                if element_len > 1 && ui.button("🗑").clicked() {
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

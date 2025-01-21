use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precision {
    Single,
    Double,
}

impl Precision {
    pub fn form(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
                        ui.label("Precision:").on_hover_text("Floating-point precision.\n\nThis \
            was added since the initial project. However, using double precision will increase memory costs to achieve a similar result. \
            As a general rule, single precision is enough.");
                        egui::ComboBox::from_label("Precision")
                            .selected_text(format!("{:?}", *self))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(self, Precision::Single, "Single");
                                ui.selectable_value(self, Precision::Double, "Double");
                            });
                    });
    }
}

use egui::{RichText, WidgetText};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DarknessMode {
    pub flat: f32,
    pub percentage: f32,
    pub mode: DarknessType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DarknessType{
    Flat, Percentage,
}

impl DarknessMode {
    pub fn form(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Darkness Mode:").on_hover_text("An algorithm that calculates the weight decay of each pixel.\n\n\
The greater the decay of the algorithm, the more spaced out the lines of the same color will be. An algorithm with very little \
decay will generate images with a high concentration of a single color in certain regions.\n\nThe algorithm takes a floating-point \
value as input and returns another floating-point value based on the input. The returned value is expected to be greater than or \
equal to zero and less than the input value. While an algorithm that does not adhere to this premise will not produce undefined \
behavior, it will likely result in a completely nonsensical image.\n\nThe input value will be in the range [0, 3] (range for \
Euclidean distance for a color in normalized RGB format), as long as the previous condition is met.");
            egui::ComboBox::from_id_salt("Darkness Mode")
                .selected_text(self.mode)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.mode,
                        DarknessType::Flat,
                        "Flat",
                    ).on_hover_text("Subtracts the specified value from the input. Return zero if the result is negative.");
                    ui.selectable_value(
                        &mut self.mode,
                        DarknessType::Percentage,
                        "Percentage",
                    ).on_hover_text("Multiplies the input by the specified value.");
                });
            match self.mode {
                DarknessType::Flat => {
                    ui.add(egui::Slider::new(&mut self.flat, 0.0..=3.0));
                }
                DarknessType::Percentage => {
                    ui.add(egui::Slider::new(&mut self.percentage, 0.0..=1.0));
                }
            }
        });
    }
}

impl From<DarknessType> for WidgetText {
    fn from(value: DarknessType) -> Self {
        WidgetText::RichText(RichText::new(match value {
            DarknessType::Flat => "Flat",
            DarknessType::Percentage => "Percentage",
        }))
    }
}

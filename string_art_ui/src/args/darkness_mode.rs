use egui::{RichText, WidgetText};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DarknessMode {
    Flat(f32),
    Percentage(f32),
}

impl DarknessMode {
    pub fn form(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Darkness Mode:").on_hover_text("An algorithm that calculates the weight decay of each pixel.\n\n\
The greater the decay of the algorithm, the more spaced out the lines of the same color will be. An algorithm with very little \
decay will generate images with a high concentration of a single color in certain regions.\n\nThe algorithm takes a floating-point \
value as input and returns another floating-point value based on the input. The returned value is expected to be greater than or \
equal to zero and less than the input value. While an algorithm that does not adhere to this premise will not produce undefined \
behavior, it will likely result in a completely nonsensical image.\n\nThe input value will be in the range [0, √140050] (the maximum \
Euclidean distance for a color in Lab format), as long as the previous condition is met.");
            egui::ComboBox::from_id_salt("Darkness Mode")
                .selected_text(*self)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        self,
                        DarknessMode::Flat(10.0),
                        "Flat",
                    ).on_hover_text("Subtracts the specified value from the input. Return zero if the result is negative.");
                    ui.selectable_value(
                        self,
                        DarknessMode::Percentage(0.93),
                        "Percentage",
                    ).on_hover_text("Multiplies the input by the specified value.");
                });
            match self {
                DarknessMode::Flat(flat) => {
                    ui.add(egui::Slider::new(flat, 0.1..=f32::from_bits(0x43bb1dc4)));
                }
                DarknessMode::Percentage(per) => {
                    ui.add(egui::Slider::new(per, 0.0..=1.0));
                }
            }
        });
    }
}

impl From<DarknessMode> for WidgetText {
    fn from(value: DarknessMode) -> Self {
        WidgetText::RichText(RichText::new(match value {
            DarknessMode::Flat(_) => "Flat",
            DarknessMode::Percentage(_) => "Percentage",
        }))
    }
}

use egui::{RichText, WidgetText};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum NailShape {
    Circular(f32),
    Point,
}

impl NailShape {
    pub fn form(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Nail shape:").on_hover_text("Shape of the nail used to draw the image.");
            egui::ComboBox::from_id_salt("Nail Shape")
                .selected_text(*self)
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, NailShape::Point, "Point")
                        .on_hover_text("A classic circular nail.");
                    ui.selectable_value(
                        self,
                        NailShape::Circular(1.0),
                        "Circular",
                    ).on_hover_text("It is equivalent to a circular nail with a radius of zero.\n\nAlthough \
physically impossible, it significantly accelerates calculations by avoiding the need to compute tangents and the entry \
and exit points of the nail.");
                });
            if let NailShape::Circular(radius) = self {
                ui.label("Radius:");
                ui.add(egui::Slider::new(radius, 0.1..=10.0));
            }
        });
    }
}

impl From<NailShape> for WidgetText {
    fn from(value: NailShape) -> Self {
        WidgetText::RichText(RichText::new(match value {
            NailShape::Circular(_) => "Circular",
            NailShape::Point => "Point",
        }))
    }
}

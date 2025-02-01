use std::num::NonZero;

use egui::{RichText, WidgetText};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TableShape{
    pub rectangle: NonZero<usize>,
    pub ellipse: NonZero<usize>,
    pub shape: TableShapeMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableShapeMode{
    Ellipse,
    Rectangle,
} 


impl TableShape {
pub fn form(&mut self, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Output shape:").on_hover_text("Shape of the output string art.");
        egui::ComboBox::from_id_salt("Table Shape")
            .selected_text(self.shape)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.shape,
                    TableShapeMode::Ellipse,
                    "Ellipse",
                ).on_hover_text("Ellipse shape. Circular if image is a square.");
                ui.selectable_value(
                    &mut self.shape,
                    TableShapeMode::Rectangle,
                    "Rectangle",
                ).on_hover_text("Rectangle shape. Same proportion than image.");
            });
        match self.shape {
            TableShapeMode::Ellipse => {
                ui.label("Nail Count:").on_hover_text("Number of nails placed around the image.");
                ui.add(
                    egui::Slider::new(
                        &mut self.ellipse,
                        unsafe { NonZero::new_unchecked(1) }..=unsafe {
                            NonZero::new_unchecked(1000)
                        },
                    )
                    .clamping(egui::SliderClamping::Never),
                )
            }
            TableShapeMode::Rectangle => {
                ui.label("Nail Count:").on_hover_text("Number of nails placed around the image.");
                ui.add(
                    egui::Slider::new(
                        &mut self.rectangle,
                        unsafe { NonZero::new_unchecked(1) }..=unsafe {
                            NonZero::new_unchecked(1000)
                        },
                    )
                    .clamping(egui::SliderClamping::Never).step_by(4.0),
                )                    
            }
        }
    });
}
}

impl From<TableShapeMode> for WidgetText {
fn from(value: TableShapeMode) -> Self {
    WidgetText::RichText(RichText::new(match value {
        TableShapeMode::Ellipse => "Ellipse",
        TableShapeMode::Rectangle => "Rectangle",
    }))
}
}

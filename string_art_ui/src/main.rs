use eframe::egui;
use rfd::FileDialog; // Para escritorio

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default(); // Ventana para aplicaciones nativas
    eframe::run_native(
        "Mi Aplicación UI",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

struct MyApp {
    selected_option: String,
    input_number: String,
    file_path: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            selected_option: "Opción 1".to_owned(),
            input_number: String::new(),
            file_path: String::from("No seleccionado"),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Seleccione una opción:");
            egui::ComboBox::from_label("Opciones")
                .selected_text(&self.selected_option)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.selected_option,
                        "Opción 1".to_string(),
                        "Opción 1",
                    );
                    ui.selectable_value(
                        &mut self.selected_option,
                        "Opción 2".to_string(),
                        "Opción 2",
                    );
                });

            ui.separator();

            ui.label("Ingrese un número:");
            ui.text_edit_singleline(&mut self.input_number);

            ui.separator();

            if ui.button("Seleccionar archivo").clicked() {
                // Usamos rfd en el entorno nativo
                if let Some(path) = FileDialog::new().pick_file() {
                    self.file_path = path.display().to_string();
                }
            }

            ui.label(format!("Archivo: {}", self.file_path));

            ui.separator();
            if ui.button("Enviar").clicked() {
                println!(
                    "Enviado: Opción '{}', Número '{}', Archivo '{}'",
                    self.selected_option, self.input_number, self.file_path
                );
            }
        });
    }
}

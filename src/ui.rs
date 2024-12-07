
#[derive(Default)]
pub struct SecretSanta {}

impl eframe::App for SecretSanta {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
            });
        });
    }
}
mod secret_santa;
mod ui;
mod test;
mod participant;

use crate::ui::SecretSanta;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Secret Santa",
        options,
        Box::new(|cc| {
            Ok(Box::<SecretSanta>::default())
        }),
    )
}
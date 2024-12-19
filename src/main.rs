mod secret_santa;
mod ui;
mod test;
mod participant;
mod listview;

use eframe::egui;
use crate::test::run_test;
use crate::ui::SecretSanta;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "Secret Santa",
        options,
        Box::new(|cc| Ok(Box::new(SecretSanta::default())))
    )
}
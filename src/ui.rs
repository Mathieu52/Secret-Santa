use std::collections::HashMap;
use eframe::egui;
use eframe::egui::{Align, Color32, Frame, Id, Layout, Rounding, Sense};
use eframe::egui::Key::{Backspace};
use eframe::egui::panel::Side;
use egui::{Context, SidePanel, CentralPanel};
use itertools::Itertools;
use levenshtein::levenshtein;
use rnglib::{Language, RNG};
use crate::listview::item_trait::ItemTrait;
use crate::listview::listview::ListView;
use crate::participant::Participant;
use crate::test::generate_participants;

impl ItemTrait for Participant {
    type Data<'a> = ();

    fn id(&self, _data: Self::Data<'_>) -> egui::Id {
        egui::Id::new(self.name.clone())
    }

    fn style_clicked(&self, frame: &mut Frame) {
        //frame.fill = Color32::LIGHT_GRAY;
        frame.fill = Color32::LIGHT_GREEN;
        frame.rounding = Rounding::same(5.0)
    }

    fn style_hovered(&self, frame: &mut Frame) {
        //frame.fill = Color32::LIGHT_BLUE;
        frame.fill = Color32::LIGHT_YELLOW;
        frame.rounding = Rounding::same(5.0)
    }

    fn show(&self, selected: bool, hovered: bool, ctx: &egui::Context, ui: &mut egui::Ui, _data: Self::Data<'_>) {
        ui.horizontal(|ui| {
            ui.style_mut().interaction.selectable_labels = false;
            ui.label(self.name.clone());

            // Add a filler space to occupy remaining width
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.allocate_space(ui.available_size());
            });
        });
    }

    fn show_on_search(&self, text: &str, _data: Self::Data<'_>) -> bool {
        true
        //self.name.contains(text)
    }

    fn score_on_search(&self, text: &str, _data: Self::Data<'_>) -> usize {
        levenshtein(text, &*self.name.clone())
    }
}


// Modify the `SecretSanta` struct to wrap `participants` in `Rc<RefCell<...>>`.
pub struct SecretSanta {
    searched_participant: String,
    participants: Vec<Participant>, // Shared and mutable
    exclusions: HashMap<Participant, Participant>,
}

impl Default for SecretSanta {
    fn default() -> Self {
        Self {
            searched_participant: String::default(),
            participants: generate_participants(50).iter().cloned().collect_vec(),
            exclusions: HashMap::default(),
        }
    }
}

impl eframe::App for SecretSanta {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let Self { searched_participant, participants, exclusions } = self;

        // Clone Rc<RefCell<...>> to pass shared ownership to ListView
        //let participants_clone = participants.iter().cloned().map(|item| item);

        let participants_copy: Vec<_> = participants.iter().map(|s| s.clone()).collect();

        SidePanel::new(Side::Left, "participants_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                let (selected, header, list_rect) = ListView::new(&participants_copy, ())
                    .title("Search".into())
                    .hold_text("something".into())
                    .striped()
                    .show(ctx, ui).inner;

                let response = ui.interact(
                    list_rect, // Interact with the entire panel area
                 Id::new("grid_interaction"),
                    Sense::click(),
                );

                // Attach the context menu to the overall grid response
                response.context_menu(|ui| {
                    if ui.button("Remove Participants").clicked() {
                        participants.retain(|x| !selected.contains(x));
                        ui.close_menu();
                    }

                    if ui.button("Add Participant").clicked() {
                        let rng = RNG::try_from(&Language::Elven).unwrap();

                        let first_name = rng.generate_name();
                        let last_name = rng.generate_name();

                        participants.push(Participant {name: format!("{first_name} {last_name}")});
                        println!("Another action triggered!");
                        ui.close_menu();
                    }
                });

                // Remove selected items when Backspace is pressed
                ui.input(|i| {

                    if i.key_pressed(Backspace) {
                        // Borrow `participants` mutably and retain non-selected items
                        participants.retain(|x| !selected.contains(x));
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
        });
    }
}

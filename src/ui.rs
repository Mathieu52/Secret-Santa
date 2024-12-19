use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use eframe::egui;
use eframe::egui::{Align, Color32, Direction, Frame, Label, Layout, RichText, Rounding, Sense};
use eframe::egui::CursorIcon::Text;
use eframe::egui::panel::Side;
use egui::{Context, SidePanel, CentralPanel, TextEdit, ComboBox, Widget, Ui, Response, ScrollArea, SelectableLabel};
use itertools::Itertools;
use levenshtein::levenshtein;
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

pub struct SecretSanta {
    searched_participant: String,
    participants: Vec<Participant>,
    exclusions: HashMap<Participant, Participant>
}

impl Default for SecretSanta {
    fn default() -> Self {
        Self {
            searched_participant: String::default(),
            participants: generate_participants(50).iter().cloned().collect_vec(),
            exclusions: HashMap::default()
        }
    }
}

impl eframe::App for SecretSanta {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {

        let Self {searched_participant, participants, exclusions} = self;

        SidePanel::new(Side::Left, "participants_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ListView::new(participants.iter(), ()).title("Search".into())
                        .hold_text("something".into())
                        .striped()
                        .show(ctx, ui);
                });

                ui.horizontal(|ui| {
                    ui.button("+");
                    ui.button("-");
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
            });
        });
    }
}
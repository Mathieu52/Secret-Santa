use eframe::egui::{Align, Color32, Id, Label, Layout, Margin, PointerButton, PointerState, Pos2, Rect, RichText, Rounding, scroll_area, ScrollArea, Sense, Stroke, TextEdit};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use eframe::egui;
use eframe::glow::COLOR;
use itertools::{Itertools, sorted};
use crate::listview::item_trait::ItemTrait;
pub struct ListView<'a, W: ItemTrait + Eq + PartialEq + Hash + 'a, L: Iterator<Item = &'a W>> {
    pub(crate) title: Cow<'a, str>,
    pub(crate) hold_text: Option<Cow<'a, str>>,
    pub(crate) items: L,
    pub(crate) data: W::Data<'a>,
    pub(crate) inner_margin: Margin,
    pub(crate) outer_margin: Margin,
    pub(crate) rounding: Rounding,
    pub(crate) striped: bool,
}

impl<'a, W: ItemTrait + Eq + PartialEq + Hash + 'a, L: Iterator<Item = &'a W>> ListView<'a, W, L> {
    pub fn new(items: L, data: W::Data<'a>) -> Self {
        Self {
            title: Cow::Borrowed("Search"),
            hold_text: None,
            items,
            data,
            inner_margin: Margin::same(3.0),
            outer_margin: Margin::default(),
            rounding: Rounding::default(),
            striped: false,
        }
    }
}

impl<'a, W: ItemTrait + Eq + PartialEq + Hash + 'a, L: Iterator<Item = &'a W>> ListView<'a, W, L> {
    pub fn title(mut self, title: Cow<'a, str>) -> Self {
        self.title = title;
        self
    }

    pub fn hold_text(mut self, text: Cow<'a, str>) -> Self {
        self.hold_text = Some(text);
        self
    }

    pub fn inner_margin(mut self, margin: Margin) -> Self {
        self.inner_margin = margin;
        self
    }

    pub fn outer_margin(mut self, margin: Margin) -> Self {
        self.outer_margin = margin;
        self
    }

    pub fn rounding(mut self, rounding: Rounding) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn striped(mut self) -> Self {
        self.striped = true;
        self
    }

    pub fn show(
        self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) -> egui::InnerResponse<HashSet<&'a W>> {
        let mut selected_items: HashSet<&'a W> = HashSet::new();

        let mut resp = ui.vertical(|outer_ui| {
            let ListView {
                title,
                hold_text,
                items,
                data,
                inner_margin,
                outer_margin,
                rounding,
                striped,
            } = self;

            let resp = outer_ui.scope(|ui| {
                let area_select_id = ui.auto_id_with("area_select");

                let mut area_select: AreaSelect = ui.data_mut(|d| d.get_temp(area_select_id)).unwrap_or_default();
                let selected_area = area_select.area();

                let root_id = ui.auto_id_with("ListView");
                let selected_id = ui.auto_id_with("selected");
                let search_id = root_id.with("search");
                let hovered_id = root_id.with("hovered");

                let mut search: String = ui.data_mut(|d| d.get_temp(search_id)).unwrap_or_default();

                let mut selected: HashSet<Id> = ui.data_mut(|d| d.get_temp(selected_id)).unwrap_or_default();
                let old_selected = selected.clone();

                let old_hovered: HashSet<Id> = ui.data_mut(|d| d.get_temp(hovered_id)).unwrap_or_default();
                let mut hovered = HashSet::new();

                ui.horizontal_top(|ui| {
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.visuals_mut().button_frame = true;
                        ui.add(Label::new(RichText::new(title).strong()));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if !search.is_empty() {
                            ui.visuals_mut().button_frame = false;
                            if ui.button("✖").on_hover_text("Clear search text").clicked() {
                                search.clear();
                            }
                        }

                        let mut search_text = TextEdit::singleline(&mut search);
                        if let Some(text) = hold_text {
                            search_text = search_text.hint_text(text);
                        }
                        ui.add(search_text);
                    });
                });

                ui.separator();

                ScrollArea::vertical()
                    .id_salt(root_id.with("list"))
                    .hscroll(true)
                    .drag_to_scroll(false)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("list view container")
                            .num_columns(1)
                            .striped(striped)
                            .show(ui, |ui| {
                                let sorted_items = items.sorted_by(| item_a, item_b | Ord::cmp(&item_a.score_on_search(&search, data), &item_b.score_on_search(&search, data)));

                                ui.input(|i| area_select.update(&i.pointer));

                                if area_select.is_pressed() {
                                    selected.clear();
                                }

                                if area_select.is_released() {
                                    selected.extend(old_hovered.iter());
                                    hovered.clear();
                                }

                                for item in sorted_items {
                                    let id = item.id(data);
                                    let checked = selected.contains(&id);
                                    let hover = old_hovered.contains(&id);

                                    if search.is_empty() || item.show_on_search(&search, data) {
                                        let mut child_frame = egui::Frame::default()
                                            .inner_margin(inner_margin)
                                            .outer_margin(outer_margin)
                                            .rounding(rounding);
                                        if checked {
                                            item.style_clicked(&mut child_frame);
                                        } else if hover {
                                            item.style_hovered(&mut child_frame);
                                        } else {
                                            item.style_normal(&mut child_frame);
                                        }

                                        let mut interact_area = child_frame
                                            .show(ui, |ui| {
                                                item.show(checked, hover, ctx, ui, data);
                                                ui.interact(
                                                    ui.max_rect(),
                                                    item.id(data),
                                                    Sense::click(),
                                                )
                                            })
                                            .inner;

                                        if let Some(tips) = item.hovered_text() {
                                            interact_area = interact_area.on_hover_text(tips);
                                        }

                                        if interact_area.hovered() {
                                            hovered.insert(id);
                                        }

                                        if selected_area.map_or(false, |area| interact_area.rect.intersects(area)) {
                                            hovered.insert(id);
                                        } else if interact_area.clicked() && !checked {
                                            selected.clear();
                                            selected.insert(id);
                                        }

                                        if checked {
                                            item.selected_item(data);
                                            selected_items.insert(item);
                                        }

                                        ui.end_row();
                                    }
                                }
                            });
                    });

                ui.data_mut(|d| {
                    d.insert_temp(search_id, search);
                    d.insert_temp(selected_id, selected.clone());
                    d.insert_temp(hovered_id, hovered.clone());

                    d.insert_temp(area_select_id, area_select);
                });

                old_selected != selected || hovered != old_hovered
            });

            resp.inner
        });

        if resp.inner {
            resp.response.mark_changed();
        }

        egui::InnerResponse::new(selected_items, resp.response)
    }
}

#[derive(Default, Copy, Clone)]
struct AreaSelect {
    start: Option<Pos2>,
    end: Option<Pos2>,

    pressed: bool,
    down: bool,
    released: bool,
}

impl AreaSelect {
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }
    pub fn is_down(&self) -> bool {
        self.down
    }

    pub fn is_released(&self) -> bool {
        self.released
    }

    pub fn area(&self) -> Option<Rect> {
        if self.down {
            self.start.zip(self.end).map(|(start, end)| {
                let min = Pos2::new(start.x.min(end.x), start.y.min(end.y));
                let max = Pos2::new(start.x.max(end.x), start.y.max(end.y));

                Rect { min, max }
            })
        } else {
            None
        }
    }

    pub fn update(&mut self, pointer_state: &PointerState) {
        self.pressed = pointer_state.button_pressed(PointerButton::Primary);
        self.down = pointer_state.button_down(PointerButton::Primary);
        self.released =pointer_state.button_released(PointerButton::Primary);

        if self.pressed {
            self.start = pointer_state.latest_pos();
        }

        if self.down {
            self.end = pointer_state.latest_pos();
        }
    }
}
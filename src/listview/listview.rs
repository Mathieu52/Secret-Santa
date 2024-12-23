use eframe::egui::{Align, Color32, Id, Label, Layout, Margin, PointerButton, Pos2, Rect, RichText, Rounding, scroll_area, ScrollArea, Sense, Stroke, TextEdit};
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
                let start_click_id = ui.auto_id_with("start_click");
                let selecting_id = ui.auto_id_with("selecting");
                let end_click_id = ui.auto_id_with("end_click");
                let in_selection_id = ui.auto_id_with("in_selection");

                let mut start_click: Option<Pos2> = ui.data_mut(|d| d.get_temp(start_click_id)).unwrap_or_default();
                let mut selecting = ui.data_mut(|d| d.get_temp(selecting_id)).unwrap_or(false);
                let mut end_click: Option<Pos2> = ui.data_mut(|d| d.get_temp(end_click_id)).unwrap_or_default();

                let mut in_selection_arc: Arc<Mutex<HashSet<Id>>> = ui.data_mut(|d| d.get_temp(in_selection_id)).unwrap_or_default();
                let old_in_selection = in_selection_arc.lock().unwrap();
                let mut in_selection = HashSet::new();

                let selected_area = if selecting {
                    start_click.zip(end_click).map(|(min, max)| Rect { min: Pos2::new(min.x.min(max.x), min.y.min(max.y)), max: Pos2::new(min.x.max(max.x), min.y.max(max.y)) })
                } else {
                    None
                };

                let root_id = ui.auto_id_with("ListView");
                let selected_id = ui.auto_id_with("selected");
                let search_id = root_id.with("search");
                let hovered_id = root_id.with("hovered");

                let mut search: String = ui.data_mut(|d| d.get_temp(search_id)).unwrap_or_default();

                let mut selected_arc: Arc<Mutex<HashSet<Id>>> = ui.data_mut(|d| d.get_temp(selected_id)).unwrap_or_default();
                let mut selected = selected_arc.lock().unwrap();
                let old_selected = selected.clone();
                let mut hovered: Option<Id> =
                    ui.data_mut(|d| d.get_temp(hovered_id)).unwrap_or_default();

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

                                ui.input(|i| {
                                    if i.pointer.button_pressed(PointerButton::Primary) {
                                        start_click = i.pointer.latest_pos();
                                        selecting = true;
                                        selected.clear();
                                    }

                                    if i.pointer.button_down(PointerButton::Primary) {
                                        end_click = i.pointer.latest_pos();
                                    }

                                    if i.pointer.button_released(PointerButton::Primary) {
                                        selecting = false;
                                        selected.extend(old_in_selection.iter());
                                        in_selection.clear();
                                    }
                                });

                                for item in sorted_items {
                                    let id = item.id(data);
                                    let checked = selected.contains(&id) || old_in_selection.contains(&id);
                                    let hover = hovered == Some(id);

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

                                        if interact_area.hovered() && hovered != Some(id) {
                                            hovered = Some(id);
                                        }

                                        if selected_area.map_or(false, |area| interact_area.rect.intersects(area)) {
                                            in_selection.insert(id);
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
                    d.insert_temp(selected_id, Arc::new(Mutex::new(selected.clone())));
                    d.insert_temp(hovered_id, hovered);

                    d.insert_temp(start_click_id, start_click);
                    d.insert_temp(selecting_id, selecting);
                    d.insert_temp(end_click_id, end_click);
                    d.insert_temp(in_selection_id, Arc::new(Mutex::new(in_selection.clone())));
                });

                old_selected != *selected || in_selection != *old_in_selection
            });

            resp.inner
        });

        if resp.inner {
            resp.response.mark_changed();
        }

        egui::InnerResponse::new(selected_items, resp.response)
    }
}

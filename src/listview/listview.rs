use eframe::egui::{Align, Id, Key, Label, Layout, Margin, PointerButton, PointerState, Pos2, Rect, RichText, Rounding, ScrollArea, Sense, TextEdit};
use std::borrow::Cow;
use std::collections::{HashSet};
use std::hash::Hash;
use eframe::egui;
use itertools::{Itertools};
use crate::listview::item_trait::ItemTrait;
use crate::listview::listview::SelectMode::{NORMAL, RANGE};

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
                // Initialize IDs and temporary state variables
                let root_id = ui.auto_id_with("ListView");
                let search_id = root_id.with("search");
                let selected_id = root_id.with("selected");
                let hovered_id = root_id.with("hovered");
                let range_select_id = root_id.with("range_close");
                let mode_id = ui.auto_id_with("mode");
                let toggle_id = ui.auto_id_with("toggle");

                let mut search: String = ui.data_mut(|d| d.get_temp(search_id)).unwrap_or_default();
                let mut selected: HashSet<Id> = ui.data_mut(|d| d.get_temp(selected_id)).unwrap_or_default();
                let old_selected = selected.clone();

                let old_hovered: HashSet<Id> = ui.data_mut(|d| d.get_temp(hovered_id)).unwrap_or_default();
                let mut hovered = HashSet::new();

                let mut old_range_select: RangeSelect = ui.data_mut(|d| d.get_temp(range_select_id)).unwrap_or_default();
                let mut range_select: RangeSelect = old_range_select.clone();

                let mut mode = ui.data_mut(|d| d.get_temp(mode_id)).unwrap_or_default();
                let mut toggle = ui.data_mut(|d| d.get_temp(toggle_id)).unwrap_or_default();

                // Header with title and search bar
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

                // Scrollable list of items
                ScrollArea::vertical()
                    .id_salt(root_id.with("list"))
                    .hscroll(true)
                    .drag_to_scroll(false)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("list_view_container")
                            .num_columns(1)
                            .striped(striped)
                            .show(ui, |ui| {
                                let sorted_items = items.filter(|&item| search.is_empty() || item.show_on_search(&search, data)).sorted_by(|a, b| {
                                    Ord::cmp(&a.score_on_search(&search, data), &b.score_on_search(&search, data))
                                });

                                let (pressed, down, released, key_up, key_down) = ui.input(|i| {
                                    let pressed = i.pointer.button_pressed(PointerButton::Primary);
                                    let down = i.pointer.button_down(PointerButton::Primary);
                                    let released = i.pointer.button_released(PointerButton::Primary);

                                    (pressed, down, released, i.key_pressed(Key::ArrowUp), i.key_pressed(Key::ArrowDown))
                                });

                                let active_toggle = ui.input(|i| i.modifiers.command);
                                let active_mode = if ui.input(|i| i.modifiers.shift) {
                                    RANGE
                                } else {
                                    NORMAL
                                };

                                if mode != active_mode {
                                    if active_mode == RANGE && selected.len() == 1 {
                                        if let Some(first_id) = selected.iter().next() {
                                            range_select.start_id = Some(*first_id);
                                        }
                                    }
                                }

                                if pressed {
                                    toggle = active_toggle;
                                    mode = active_mode;

                                    if mode == NORMAL && !toggle {
                                        selected.clear();
                                    }
                                }

                                let close_range = if mode == RANGE {
                                    active_mode != mode || active_toggle != toggle
                                } else { released };

                                let mut inside_range_select = false;

                                let mut previous_id = None;
                                for item in sorted_items {
                                    let id = item.id(data);
                                    let mut checked = selected.contains(&id);
                                    let mut hover = old_hovered.contains(&id);
                                    let remove = checked && hover && toggle;

                                    let mut child_frame = egui::Frame::default()
                                        .inner_margin(inner_margin)
                                        .outer_margin(outer_margin)
                                        .rounding(rounding);

                                    if remove {
                                        item.style_removal(&mut child_frame);
                                    } else if checked {
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

                                    if interact_area.hovered() {
                                        hovered.insert(id);
                                    }

                                    if range_select.is_closed() {
                                        if key_down && old_range_select.end_id.and_then(|end_id| previous_id.and_then(|previous_id| Some(end_id == previous_id))).unwrap_or(false) {
                                            range_select.end_id = Some(id);
                                        }

                                        if key_up && old_range_select.end_id.and_then(|end_id| Some(end_id == id)).unwrap_or(false) {
                                            range_select.end_id = previous_id;
                                        }
                                    }

                                    if ui.rect_contains_pointer(interact_area.interact_rect) && down {
                                        if old_range_select.start_id.is_none() {
                                            range_select.start_id = Some(id);
                                        }

                                        range_select.end_id = Some(id);
                                    }

                                    let mut in_selection = false;
                                    if old_range_select.is_closed() {
                                        if inside_range_select && old_range_select.is_single() {
                                            inside_range_select = false;
                                        }

                                        if inside_range_select {
                                            in_selection = true;
                                        }

                                        if old_range_select.is_boundary(&id) {
                                            in_selection = true;
                                            inside_range_select = !inside_range_select;
                                        }
                                    }

                                    if interact_area.clicked() && mode == NORMAL {
                                        if toggle {
                                            checked = !checked;
                                        } else {
                                            checked = true;
                                        }
                                    } else if in_selection {
                                        if close_range {
                                            if toggle {
                                                checked = !checked;
                                            } else {
                                                checked = true;
                                            }
                                        } else {
                                            hovered.insert(id);
                                        }
                                    }

                                    if checked {
                                        selected.insert(id);
                                    } else {
                                        selected.remove(&id);
                                    }

                                    if checked {
                                        selected_items.insert(item);
                                        item.selected_item(data);
                                    }

                                    ui.end_row();
                                    previous_id = Some(id);
                                }

                                if close_range {
                                    range_select.start_id = None;
                                    range_select.end_id = None;
                                    mode = NORMAL;
                                }
                            });
                    });

                // Save temporary state
                ui.data_mut(|d| {
                    d.insert_temp(search_id, search);
                    d.insert_temp(selected_id, selected.clone());
                    d.insert_temp(hovered_id, hovered.clone());
                    d.insert_temp(range_select_id, range_select.clone());
                    d.insert_temp(mode_id, mode);
                    d.insert_temp(toggle_id, toggle);
                });

                old_selected != selected || hovered != old_hovered || old_range_select != range_select
            });

            resp.inner
        });

        if resp.inner {
            resp.response.mark_changed();
        }

        egui::InnerResponse::new(selected_items, resp.response)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum SelectMode {
    NORMAL,
    RANGE,
}

impl Default for SelectMode {
    fn default() -> Self {
        NORMAL
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

#[derive(Default, Copy, Clone, Eq, PartialEq)]
struct RangeSelect {
    pub start_id: Option<Id>,
    pub end_id: Option<Id>,
}

impl RangeSelect {
    pub fn is_single(&self) -> bool {
        return self.is_closed() && self.start_id == self.end_id;
    }

    pub fn is_boundary(&self, id: &Id) -> bool {
        return self.start_id == Some(*id) || self.end_id == Some(*id);
    }

    pub fn is_opened(&self) -> bool {
        self.start_id.is_some() ^ self.end_id.is_some()
    }

    pub fn is_closed(&self) -> bool {
        self.start_id.is_some() && self.end_id.is_some()
    }
}
use eframe::egui;

use crate::app::message::{Message, MessageSender, PresetMessage};
use crate::app::state::UiState;
use crate::ui::widget::window::{Modal, focus_if_pending, render_centered, render_modal};

const EMPTY_HEIGHT_PIXELS: f32 = 20.0;
const LOAD_SIZE: [f32; 2] = [420.0, 300.0];
const SAVE_SIZE: [f32; 2] = [420.0, 350.0];

pub fn render_load(context: &egui::Context, ui_state: &UiState, sender: &MessageSender) {
    let dialog = Modal {
        identifier: "load_preset",
        size: LOAD_SIZE,
        title: "Load Preset",
    };

    let closed = render_modal(context, &dialog, |modal| {
        modal.vertical(|column| render_load_list(column, ui_state, sender));
    });

    if closed {
        sender.send(Message::Preset(PresetMessage::LoadDialogClosed));
    }
}

fn render_load_list(ui: &mut egui::Ui, ui_state: &UiState, sender: &MessageSender) {
    let available_height = ui.available_height();

    egui::Frame::dark_canvas(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(8.0)
        .stroke(egui::Stroke::NONE)
        .show(ui, |content| {
            content.set_min_height(available_height - 16.0);

            if ui_state.preset_load_entries.is_empty() {
                render_centered(content, EMPTY_HEIGHT_PIXELS, |centered| {
                    centered.label("No presets available.");
                });

                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(content, |scroll| {
                    for entry in &ui_state.preset_load_entries {
                        let response = scroll.selectable_label(false, &entry.name);

                        if response.clicked() {
                            sender.send(Message::Preset(PresetMessage::Loaded(
                                entry.identifier.clone(),
                            )));
                        }

                        response.context_menu(|menu| {
                            if menu.button("Delete").clicked() {
                                sender.send(Message::Preset(PresetMessage::Deleted(
                                    entry.identifier.clone(),
                                )));

                                menu.close();
                            }
                        });
                    }
                });
        });
}

pub fn render_save(context: &egui::Context, ui_state: &mut UiState, sender: &MessageSender) {
    let dialog = Modal {
        identifier: "save_preset",
        size: SAVE_SIZE,
        title: "Save Preset",
    };

    let closed = render_modal(context, &dialog, |modal| {
        modal.vertical(|column| render_save_fields(column, ui_state, sender));
    });

    if closed {
        sender.send(Message::Preset(PresetMessage::SaveDialogClosed));
    }
}

fn render_save_fields(ui: &mut egui::Ui, ui_state: &mut UiState, sender: &MessageSender) {
    ui.label("Preset name:");

    let response =
        ui.add(egui::TextEdit::singleline(&mut ui_state.preset_name).desired_width(f32::INFINITY));

    focus_if_pending(&response, &mut ui_state.preset_name_focus);

    ui.add_space(12.0);

    let mut include_selection = ui_state.preset_include_selection;

    if ui
        .checkbox(&mut include_selection, "Include selection")
        .clicked()
    {
        sender.send(Message::Preset(PresetMessage::IncludeSelectionChanged(
            include_selection,
        )));
    }

    let mut include_search = ui_state.preset_include_search;

    if ui
        .checkbox(&mut include_search, "Include search/filter")
        .clicked()
    {
        sender.send(Message::Preset(PresetMessage::IncludeSearchChanged(
            include_search,
        )));
    }

    let mut generic = ui_state.preset_generic;

    if ui
        .checkbox(&mut generic, "Generic (available in any project)")
        .clicked()
    {
        sender.send(Message::Preset(PresetMessage::GenericChanged(generic)));
    }

    ui.add_space(8.0);

    let has_name = !ui_state.preset_name.trim().is_empty();
    let can_save = has_name && (include_selection || include_search);

    ui.horizontal(|row| {
        let enter = row.input(|input| input.key_pressed(egui::Key::Enter));

        let clicked = row
            .add_enabled(can_save, egui::Button::new("Save"))
            .clicked();

        let submitted = clicked || (enter && can_save);

        if submitted {
            let name = ui_state.preset_name.trim().to_owned();

            sender.send(Message::Preset(PresetMessage::Saved(name)));
        }

        if row.button("Cancel").clicked() {
            sender.send(Message::Preset(PresetMessage::SaveDialogClosed));
        }
    });
}

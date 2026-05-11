use std::sync::mpsc::Sender;
use std::thread;

use eframe::egui;
use rfd::FileDialog;

use crate::app::message::{App, Message, Options_, Preset_, Session};
use crate::app::state::{Model, UiState};
use crate::constants::APP_NAME;

pub fn render(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Message>,
) {
    egui::Panel::top("top_panel")
        .min_size(60.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let can_open = true;

                    if ui.add_enabled(can_open, egui::Button::new("Open")).clicked() {
                        ui.close();
                        open_file_dialog(sender);
                    }

                    let has_tree = !model.tree.nodes.is_empty();

                    if ui.add_enabled(has_tree, egui::Button::new("Open in Explorer")).clicked() {
                        sender.send(Message::App(App::OpenInExplorer)).ok();
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("New Session").clicked() {
                        sender.send(Message::Session(Session::Created("Session".to_string()))).ok();
                        ui.close();
                    }

                    let has_restorable = model.sessions.has_restorable_session();

                    if ui.add_enabled(has_restorable, egui::Button::new("Restore Last Session")).clicked() {
                        sender.send(Message::App(App::RestoreLastSession)).ok();
                        ui.close();
                    }

                    ui.separator();

                    if ui.add_enabled(has_tree, egui::Button::new("Save Preset")).clicked() {
                        sender.send(Message::Preset(Preset_::SaveDialogOpened)).ok();
                        ui.close();
                    }

                    let has_presets = !model.presets.presets.is_empty();

                    if ui.add_enabled(has_presets, egui::Button::new("Load Preset")).clicked() {
                        sender.send(Message::Preset(Preset_::LoadDialogOpened)).ok();
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        ui.close();
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Options").clicked() {
                        sender.send(Message::Options(Options_::Opened)).ok();
                        ui.close();
                    }
                });

                ui.menu_button("About", |ui| {
                    if ui.button(format!("About {}", APP_NAME)).clicked() {
                        sender.send(Message::App(App::AboutOpened)).ok();
                        ui.close();
                    }
                });
            });

            ui.separator();
            ui.add_space(10.0);

            render_session_tabs(ui, model, ui_state, sender);
        });
}

fn render_session_tabs(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Message>,
) {
    let mut sessions: Vec<_> = model.sessions.sessions.iter().collect();
    sessions.sort_by_key(|(_, s)| s.created_at);

    let available_width = ui.available_width();

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(available_width, 28.0),
        egui::Sense::click(),
    );

    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center))
    );

    child_ui.spacing_mut().item_spacing.x = 2.0;

    for (id, session) in sessions {
        if Some(id.clone()) == ui_state.session_editing {
            render_edit_tab(&mut child_ui, id, ui_state, sender);
        } else {
            render_tab_label(&mut child_ui, id, session, model, sender);
        }

        child_ui.separator();
    }

    if response.double_clicked() {
        sender.send(Message::Session(Session::Created("Session".to_string()))).ok();
    }

    response.context_menu(|ui| {
        if ui.button("New Session").clicked() {
            sender.send(Message::Session(Session::Created("Session".to_string()))).ok();
            ui.close();
        }
    });
}

fn render_edit_tab(
    ui: &mut egui::Ui,
    id: &str,
    ui_state: &UiState,
    sender: &Sender<Message>,
) {
    let mut name = ui_state.name_edit.clone();

    let response = ui.add(
        egui::TextEdit::singleline(&mut name)
            .desired_width(75.0)
    );

    response.request_focus();

    if response.changed() {
        sender.send(Message::Session(Session::NameEdited(name.clone()))).ok();
    }

    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let clicked_away = ui.input(|i| i.pointer.any_released()) && !response.hovered();

    if enter || clicked_away {
        if !name.trim().is_empty() {
            sender.send(Message::Session(Session::Renamed {
                id: id.to_string(),
                name,
            })).ok();
        } else {
            sender.send(Message::Session(Session::EditCancelled)).ok();
        }
    }
}

fn render_tab_label(
    ui: &mut egui::Ui,
    id: &str,
    session: &crate::app::state::SessionData,
    model: &Model,
    sender: &Sender<Message>,
) {
    let selected = model.sessions.active_id.as_deref() == Some(id);

    let text = if selected {
        egui::RichText::new(&session.name).strong()
    } else {
        egui::RichText::new(&session.name)
    };

    let response = ui.selectable_label(selected, text);

    if response.clicked() && !selected {
        sender.send(Message::Session(Session::Selected(id.to_string()))).ok();
    }

    response.context_menu(|ui| {
        if ui.button("Rename Session").clicked() {
            sender.send(Message::Session(Session::EditStarted(id.to_string()))).ok();
            ui.close();
        }

        if ui.button("Delete Session").clicked() {
            sender.send(Message::Session(Session::Deleted(id.to_string()))).ok();
            ui.close();
        }
    });
}

fn open_file_dialog(sender: &Sender<Message>) {
    let sender = sender.clone();

    thread::spawn(move || {
        if let Some(path) = FileDialog::new()
            .set_title("Select File or Directory")
            .pick_folder()
        {
            sender.send(Message::App(App::PathSelected(path))).ok();
        }
    });
}

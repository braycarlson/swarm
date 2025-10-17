use std::sync::mpsc::Sender;
use std::thread;

use eframe::egui;
use rfd::FileDialog;

use crate::app::message::{App, Msg, Options_, Session};
use crate::app::state::{Model, UiState};
use crate::constants::APP_NAME;

pub fn render(
    ctx: &egui::Context,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    egui::TopBottomPanel::top("top_panel")
        .min_height(60.0)
        .resizable(false)
        .show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let can_open = true;

                    if ui.add_enabled(can_open, egui::Button::new("Open")).clicked() {
                        ui.close();
                        open_file_dialog(sender);
                    }

                    ui.separator();

                    if ui.button("New Session").clicked() {
                        sender.send(Msg::Session(Session::Created(format!("Session")))).ok();
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
                        sender.send(Msg::Options(Options_::Opened)).ok();
                        ui.close();
                    }
                });

                ui.menu_button("About", |ui| {
                    if ui.button(&format!("About {}", APP_NAME)).clicked() {
                        sender.send(Msg::App(App::AboutOpened)).ok();
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
    sender: &Sender<Msg>,
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
        if Some(id.clone()) == ui_state.editing_session {
            render_edit_tab(&mut child_ui, id, ui_state, sender);
        } else {
            render_tab_label(&mut child_ui, id, session, model, sender);
        }

        child_ui.separator();
    }

    if response.double_clicked() {
        sender.send(Msg::Session(Session::Created(format!("Session")))).ok();
    }

    response.context_menu(|ui| {
        if ui.button("New Session").clicked() {
            sender.send(Msg::Session(Session::Created(format!("Session")))).ok();
            ui.close();
        }
    });
}

fn render_edit_tab(
    ui: &mut egui::Ui,
    id: &str,
    ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    let mut name = ui_state.edit_name.clone();

    let response = ui.add(
        egui::TextEdit::singleline(&mut name)
            .desired_width(75.0)
    );

    response.request_focus();

    if response.changed() {
        sender.send(Msg::Session(Session::NameEdited(name.clone()))).ok();
    }

    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let clicked_away = ui.input(|i| i.pointer.any_released()) && !response.hovered();

    if enter || clicked_away {
        if !name.trim().is_empty() {
            sender.send(Msg::Session(Session::Renamed {
                id: id.to_string(),
                name,
            })).ok();
        } else {
            sender.send(Msg::Session(Session::EditCancelled)).ok();
        }
    }
}

fn render_tab_label(
    ui: &mut egui::Ui,
    id: &str,
    session: &crate::app::state::SessionData,
    model: &Model,
    sender: &Sender<Msg>,
) {
    let selected = model.sessions.active_id.as_deref() == Some(id);

    let text = if selected {
        egui::RichText::new(&session.name).strong()
    } else {
        egui::RichText::new(&session.name)
    };

    let response = ui.selectable_label(selected, text);

    if response.clicked() && !selected {
        sender.send(Msg::Session(Session::Selected(id.to_string()))).ok();
    }

    response.context_menu(|ui| {
        if ui.button("Rename Session").clicked() {
            sender.send(Msg::Session(Session::EditStarted(id.to_string()))).ok();
            ui.close();
        }

        if ui.button("Delete Session").clicked() {
            sender.send(Msg::Session(Session::Deleted(id.to_string()))).ok();
            ui.close();
        }
    });
}

fn open_file_dialog(sender: &Sender<Msg>) {
    let sender = sender.clone();

    thread::spawn(move || {
        if let Some(path) = FileDialog::new()
            .set_title("Select File or Directory")
            .pick_folder()
        {
            sender.send(Msg::App(App::PathSelected(path))).ok();
        }
    });
}

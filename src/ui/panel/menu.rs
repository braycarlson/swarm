use eframe::egui;

use crate::app::handler::common::SESSION_NAME_DEFAULT;
use crate::app::message::{App, Message, MessageSender, OptionsMessage, PresetMessage, Session};
use crate::app::state::{Model, SessionData, UiState};
use crate::constants::APP_NAME;
use crate::ui::widget::window::focus_if_pending;

const SESSION_NAME_WIDTH_PIXELS: f32 = 75.0;
const TAB_STRIP_HEIGHT_PIXELS: f32 = 28.0;

fn new_session_message() -> Message {
    Message::Session(Session::Created(SESSION_NAME_DEFAULT.to_owned()))
}

pub fn render(ui: &mut egui::Ui, model: &Model, ui_state: &mut UiState, sender: &MessageSender) {
    egui::Panel::top("top_panel")
        .min_size(60.0)
        .resizable(false)
        .show_inside(ui, |panel| {
            egui::MenuBar::new().ui(panel, |bar| {
                render_file_menu(bar, model, ui_state, sender);
                render_edit_menu(bar, sender);
                render_about_menu(bar, sender);
            });

            panel.separator();
            panel.add_space(10.0);

            render_session_tabs(panel, model, ui_state, sender);
        });
}

fn render_about_menu(ui: &mut egui::Ui, sender: &MessageSender) {
    ui.menu_button("About", |menu| {
        if menu.button(format!("About {APP_NAME}")).clicked() {
            sender.send(Message::App(App::AboutOpened));
            menu.close();
        }
    });
}

fn render_edit_menu(ui: &mut egui::Ui, sender: &MessageSender) {
    ui.menu_button("Edit", |menu| {
        if menu.button("Options").clicked() {
            sender.send(Message::Options(OptionsMessage::Opened));
            menu.close();
        }
    });
}

fn render_edit_tab(
    ui: &mut egui::Ui,
    identifier: &str,
    ui_state: &mut UiState,
    sender: &MessageSender,
) {
    let response = ui.add(
        egui::TextEdit::singleline(&mut ui_state.session_editing_name)
            .desired_width(SESSION_NAME_WIDTH_PIXELS),
    );

    focus_if_pending(&response, &mut ui_state.session_edit_focus);

    let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
    let clicked_away = ui.input(|input| input.pointer.any_released()) && !response.hovered();
    let committed = enter || clicked_away;

    if !committed {
        return;
    }

    let name = ui_state.session_editing_name.trim();

    if name.is_empty() {
        sender.send(Message::Session(Session::EditCancelled));

        return;
    }

    sender.send(Message::Session(Session::Renamed {
        identifier: identifier.to_owned(),
        name: name.to_owned(),
    }));
}

fn render_file_menu(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &MessageSender) {
    ui.menu_button("File", |menu| {
        if menu
            .add_enabled(!ui_state.file_dialog_open, egui::Button::new("Open"))
            .clicked()
        {
            sender.send(Message::App(App::FileDialogOpened));
            menu.close();
        }

        let has_tree = !model.tree.nodes.is_empty();

        if menu
            .add_enabled(has_tree, egui::Button::new("Open in Explorer"))
            .clicked()
        {
            sender.send(Message::App(App::OpenInExplorer));
            menu.close();
        }

        menu.separator();

        if menu.button("New Session").clicked() {
            sender.send(new_session_message());
            menu.close();
        }

        let has_restorable = model.sessions.has_restorable_session();

        if menu
            .add_enabled(has_restorable, egui::Button::new("Restore Last Session"))
            .clicked()
        {
            sender.send(Message::App(App::RestoreLastSession));
            menu.close();
        }

        menu.separator();

        if menu
            .add_enabled(has_tree, egui::Button::new("Save Preset"))
            .clicked()
        {
            sender.send(Message::Preset(PresetMessage::SaveDialogOpened));
            menu.close();
        }

        if menu
            .add_enabled(!model.presets.is_empty(), egui::Button::new("Load Preset"))
            .clicked()
        {
            sender.send(Message::Preset(PresetMessage::LoadDialogOpened));
            menu.close();
        }

        menu.separator();

        if menu.button("Exit").clicked() {
            menu.close();
            menu.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

fn render_session_tabs(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &mut UiState,
    sender: &MessageSender,
) {
    let (rectangle, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), TAB_STRIP_HEIGHT_PIXELS),
        egui::Sense::click(),
    );

    let mut strip = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rectangle)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    strip.spacing_mut().item_spacing.x = 2.0;

    for session in model.sessions.ordered() {
        let editing = ui_state.session_editing.as_deref() == Some(session.identifier.as_str());

        if editing {
            render_edit_tab(&mut strip, &session.identifier, ui_state, sender);
        } else {
            render_tab_label(&mut strip, session, model, sender);
        }

        strip.separator();
    }

    if response.double_clicked() {
        sender.send(new_session_message());
    }

    response.context_menu(|menu| {
        if menu.button("New Session").clicked() {
            sender.send(new_session_message());
            menu.close();
        }
    });
}

fn render_tab_label(
    ui: &mut egui::Ui,
    session: &SessionData,
    model: &Model,
    sender: &MessageSender,
) {
    let selected = model.sessions.active_identifier() == Some(session.identifier.as_str());

    let text = if selected {
        egui::RichText::new(&session.name).strong()
    } else {
        egui::RichText::new(&session.name)
    };

    let response = ui.selectable_label(selected, text);

    if response.clicked() {
        if !selected {
            sender.send(Message::Session(Session::Selected(
                session.identifier.clone(),
            )));
        }
    }

    response.context_menu(|menu| {
        if menu.button("Rename Session").clicked() {
            sender.send(Message::Session(Session::EditStarted(
                session.identifier.clone(),
            )));
            menu.close();
        }

        if menu.button("Delete Session").clicked() {
            sender.send(Message::Session(Session::Deleted(
                session.identifier.clone(),
            )));
            menu.close();
        }
    });
}

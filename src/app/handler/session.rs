use crate::app::message::{Cmd, CmdBuilder, Session};
use crate::app::state::{Model, UiState};

use super::sync_to_active_session;

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Session) -> Cmd {
    match msg {
        Session::Created(name) => handle_session_created(model, name),
        Session::Selected(id) => handle_session_selected(model, ui, id),
        Session::Deleted(id) => handle_session_deleted(model, id),
        Session::NameEdited(name) => handle_session_name_edited(ui, name),
        Session::Renamed { id, name } => handle_session_renamed(model, ui, id, name),
        Session::EditStarted(id) => handle_session_edit_started(model, ui, id),
        Session::EditCancelled => handle_session_edit_cancelled(ui),
    }
}

fn handle_session_created(model: &mut Model, name: String) -> Cmd {
    sync_to_active_session(model);

    model.sessions.create_session(name);
    model.tree = Default::default();
    model.search = Default::default();

    let builder = CmdBuilder::new();
    builder.build()
}

fn handle_session_selected(model: &mut Model, _ui: &mut UiState, id: String) -> Cmd {
    if model.sessions.active_id.as_ref() == Some(&id) {
        return Cmd::None;
    }

    sync_to_active_session(model);

    if let Some(session) = model.sessions.select_session(id.clone()) {
        model.tree = session.tree_state.clone();
        model.search = session.search_state.clone();

        let builder = CmdBuilder::new();
        builder.build()
    } else {
        Cmd::None
    }
}

fn handle_session_deleted(model: &mut Model, id: String) -> Cmd {
    if let Some(new_active_id) = model.sessions.delete_session(&id) {
        if let Some(session) = model.sessions.sessions.get(&new_active_id) {
            model.tree = session.tree_state.clone();
            model.search = session.search_state.clone();

            let builder = CmdBuilder::new()
                .add(Cmd::DeleteSessionData(id));

            return builder.build();
        }
    } else {
        model.tree = Default::default();
        model.search = Default::default();
    }

    Cmd::DeleteSessionData(id)
}

fn handle_session_renamed(model: &mut Model, ui: &mut UiState, id: String, name: String) -> Cmd {
    model.sessions.rename_session(&id, name);
    ui.cancel_session_edit();

    Cmd::None
}

fn handle_session_name_edited(ui: &mut UiState, name: String) -> Cmd {
    ui.edit_name = name;
    Cmd::None
}

fn handle_session_edit_started(model: &mut Model, ui: &mut UiState, id: String) -> Cmd {
    if let Some(session) = model.sessions.sessions.get(&id) {
        ui.start_session_edit(id, session.name.clone());
    }

    Cmd::None
}

fn handle_session_edit_cancelled(ui: &mut UiState) -> Cmd {
    ui.cancel_session_edit();
    Cmd::None
}

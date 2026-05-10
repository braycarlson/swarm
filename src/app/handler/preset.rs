use std::collections::HashMap;
use std::path::PathBuf;

use crate::app::message::{Cmd, Preset_};
use crate::app::state::{Model, UiState};
use crate::model::node::FileNode;
use crate::model::preset::{Preset, PresetModel};

use super::sync_to_active_session;

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Preset_) -> Cmd {
    match msg {
        Preset_::SaveDialogOpened => handle_save_dialog_opened(ui),
        Preset_::SaveDialogClosed => handle_save_dialog_closed(ui),
        Preset_::LoadDialogOpened => handle_load_dialog_opened(ui),
        Preset_::LoadDialogClosed => handle_load_dialog_closed(ui),
        Preset_::NameChanged(name) => handle_name_changed(ui, name),
        Preset_::Saved(name) => handle_saved(model, ui, name),
        Preset_::Loaded(id) => handle_loaded(model, ui, id),
        Preset_::Deleted(id) => handle_deleted(model, ui, id),
    }
}

fn handle_save_dialog_opened(ui: &mut UiState) -> Cmd {
    ui.show_save_preset = true;
    ui.preset_name.clear();
    Cmd::None
}

fn handle_save_dialog_closed(ui: &mut UiState) -> Cmd {
    ui.show_save_preset = false;
    ui.preset_name.clear();
    Cmd::None
}

fn handle_load_dialog_opened(ui: &mut UiState) -> Cmd {
    ui.show_load_preset = true;
    Cmd::None
}

fn handle_load_dialog_closed(ui: &mut UiState) -> Cmd {
    ui.show_load_preset = false;
    Cmd::None
}

fn handle_name_changed(ui: &mut UiState, name: String) -> Cmd {
    ui.preset_name = name;
    Cmd::None
}

fn handle_saved(model: &mut Model, ui: &mut UiState, name: String) -> Cmd {
    let states = model.tree.collect_checkbox_states();
    let paths: Vec<PathBuf> = states.into_iter()
        .filter(|(_, checked)| *checked)
        .map(|(path, _)| path)
        .collect();

    if paths.is_empty() {
        ui.toast.error("No files selected to save as preset");
        ui.show_save_preset = false;
        ui.preset_name.clear();
        return Cmd::None;
    }

    let root = model.tree.nodes.first()
        .map(|n| n.path.clone())
        .unwrap_or_default();

    let preset = Preset::new(name, root, paths);
    model.presets.add(preset);

    let _ = model.presets.save_to_disk();

    ui.show_save_preset = false;
    ui.preset_name.clear();
    ui.toast.success("Preset saved");

    Cmd::None
}

fn handle_loaded(model: &mut Model, ui: &mut UiState, id: String) -> Cmd {
    let paths = match model.presets.get(&id) {
        Some(preset) => preset.paths.clone(),
        None => {
            ui.toast.error("Preset not found");
            return Cmd::None;
        }
    };

    uncheck_all(&mut model.tree.nodes);

    let states: HashMap<PathBuf, bool> = paths.into_iter()
        .map(|p| (p, true))
        .collect();

    model.tree.restore_checkbox_states(&states);
    model.tree.update_file_count();

    sync_to_active_session(model);

    ui.show_load_preset = false;
    ui.toast.success("Preset loaded");

    Cmd::None
}

fn handle_deleted(model: &mut Model, ui: &mut UiState, id: String) -> Cmd {
    let _ = PresetModel::delete_from_disk(&id);
    model.presets.remove(&id);

    if model.presets.presets.is_empty() {
        ui.show_load_preset = false;
    }

    Cmd::None
}

fn uncheck_all(nodes: &mut [FileNode]) {
    for node in nodes {
        node.checked = false;
        uncheck_all(&mut node.children);
    }
}

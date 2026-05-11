use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::app::message::{Command, Preset_};
use crate::app::state::{Model, UiState};
use crate::model::node::FileNode;
use crate::model::preset::{Preset, PresetModel};

use super::sync_to_active_session;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Preset_) -> Command {
    match message {
        Preset_::SaveDialogOpened => handle_save_dialog_opened(model, ui),
        Preset_::SaveDialogClosed => handle_save_dialog_closed(ui),
        Preset_::LoadDialogOpened => handle_load_dialog_opened(ui),
        Preset_::LoadDialogClosed => handle_load_dialog_closed(ui),
        Preset_::NameChanged(name) => handle_name_changed(ui, name),
        Preset_::IncludeSelectionChanged(v) => { ui.preset_include_selection = v; Command::None }
        Preset_::IncludeSearchChanged(v) => { ui.preset_include_search = v; Command::None }
        Preset_::GenericChanged(v) => { ui.preset_generic = v; Command::None }
        Preset_::Saved(name) => handle_saved(model, ui, name),
        Preset_::Loaded(id) => handle_loaded(model, ui, id),
        Preset_::Deleted(id) => handle_deleted(model, ui, id),
    }
}

fn handle_save_dialog_opened(model: &Model, ui: &mut UiState) -> Command {
    ui.preset_save_show = true;
    ui.preset_name.clear();
    ui.preset_include_selection = true;
    ui.preset_include_search = model.search.has_query();
    ui.preset_generic = false;
    Command::None
}

fn handle_save_dialog_closed(ui: &mut UiState) -> Command {
    ui.preset_save_show = false;
    ui.preset_name.clear();
    Command::None
}

fn handle_load_dialog_opened(ui: &mut UiState) -> Command {
    ui.preset_load_show = true;
    Command::None
}

fn handle_load_dialog_closed(ui: &mut UiState) -> Command {
    ui.preset_load_show = false;
    Command::None
}

fn handle_name_changed(ui: &mut UiState, name: String) -> Command {
    ui.preset_name = name;
    Command::None
}

fn handle_saved(model: &mut Model, ui: &mut UiState, name: String) -> Command {
    let include_selection = ui.preset_include_selection;
    let include_search = ui.preset_include_search;
    let generic = ui.preset_generic;

    if !include_selection && !include_search {
        ui.toast.error("Select at least one option to save");
        return Command::None;
    }

    let paths = if include_selection {
        let states = model.tree.collect_checkbox_states();
        let checked: Vec<PathBuf> = states.into_iter()
            .filter(|(_, checked)| *checked)
            .map(|(path, _)| path)
            .collect();

        if checked.is_empty() {
            ui.toast.error("No files selected to save");
            return Command::None;
        }

        Some(checked)
    } else {
        None
    };

    let query = if include_search && model.search.has_query() {
        Some(model.search.query.clone())
    } else {
        None
    };

    let root = if generic {
        None
    } else {
        model.tree.nodes.first().map(|n| n.path.clone())
    };

    let preset = Preset::new(name, root, paths, query);
    model.presets.add(preset);

    let _ = model.presets.save_to_disk();

    ui.preset_save_show = false;
    ui.preset_name.clear();
    ui.toast.success("Preset saved");

    Command::None
}

fn handle_loaded(model: &mut Model, ui: &mut UiState, id: String) -> Command {
    let preset = match model.presets.get(&id) {
        Some(p) => p.clone(),
        None => {
            ui.toast.error("Preset not found");
            return Command::None;
        }
    };

    if let Some(ref paths) = preset.paths {
        uncheck_all(&mut model.tree.nodes);

        let states: FxHashMap<PathBuf, bool> = paths.iter()
            .map(|p| (p.clone(), true))
            .collect();

        model.tree.restore_checkbox_states(&states);
        model.tree.update_file_count();
    }

    if let Some(ref query) = preset.query {
        model.search.set_query(query.clone());
    }

    sync_to_active_session(model);

    ui.preset_load_show = false;
    ui.toast.success("Preset loaded");

    Command::None
}

fn handle_deleted(model: &mut Model, ui: &mut UiState, id: String) -> Command {
    let _ = PresetModel::delete_from_disk(&id);
    model.presets.remove(&id);

    if model.presets.presets.is_empty() {
        ui.preset_load_show = false;
    }

    Command::None
}

fn uncheck_all(nodes: &mut [FileNode]) {
    for node in nodes {
        node.checked = false;
        uncheck_all(&mut node.children);
    }
}

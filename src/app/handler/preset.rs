use std::path::PathBuf;

use rustc_hash::FxHashSet;

use crate::app::message::{Command, PresetMessage};
use crate::app::state::ui::PresetEntry;
use crate::app::state::{Model, UiState};
use crate::model::preset::{Preset, PresetModel, PresetSelection};

use super::common::refresh_matches;

pub fn handle(model: &mut Model, ui: &mut UiState, message: PresetMessage) -> Command {
    match message {
        PresetMessage::Deleted(identifier) => handle_deleted(model, ui, &identifier),
        PresetMessage::GenericChanged(value) => ui.preset_generic = value,
        PresetMessage::IncludeSearchChanged(value) => ui.preset_include_search = value,
        PresetMessage::IncludeSelectionChanged(value) => ui.preset_include_selection = value,
        PresetMessage::LoadDialogClosed => {
            ui.preset_load_show = false;
            ui.preset_load_entries.clear();
        }
        PresetMessage::LoadDialogOpened => handle_load_dialog_opened(model, ui),
        PresetMessage::Loaded(identifier) => return handle_loaded(model, ui, &identifier),
        PresetMessage::SaveDialogClosed => {
            ui.preset_save_show = false;
            ui.preset_name.clear();
        }
        PresetMessage::SaveDialogOpened => handle_save_dialog_opened(model, ui),
        PresetMessage::Saved(name) => handle_saved(model, ui, name),
    }

    Command::None
}

fn handle_deleted(model: &mut Model, ui: &mut UiState, identifier: &str) {
    if let Err(error) = PresetModel::delete_from_disk(identifier) {
        ui.toast
            .error(format!("Failed to delete the preset: {error}"));

        return;
    }

    let removed = model.presets.remove(identifier);

    debug_assert!(removed.is_some());

    ui.preset_load_entries
        .retain(|entry| entry.identifier != identifier);

    if ui.preset_load_entries.is_empty() {
        ui.preset_load_show = false;
    }
}

fn handle_load_dialog_opened(model: &Model, ui: &mut UiState) {
    let root = model.tree.nodes.first().map(|node| node.path.as_path());

    ui.preset_load_entries = model
        .presets
        .list_for_root(root)
        .into_iter()
        .map(|preset| PresetEntry {
            identifier: preset.identifier.clone(),
            name: preset.name.clone(),
        })
        .collect();

    ui.preset_load_show = true;
}

fn handle_loaded(model: &mut Model, ui: &mut UiState, identifier: &str) -> Command {
    let Some(preset) = model.presets.get(identifier).cloned() else {
        ui.toast.error("Preset not found");

        return Command::None;
    };

    let mut command = Command::None;

    if let Some(paths) = preset.paths.as_ref() {
        let checked: FxHashSet<PathBuf> = paths.iter().cloned().collect();

        model.tree.restore_checked_paths(&checked);
        model.tree.update_files_count();
    }

    if let Some(query) = preset.query {
        model.search.set_query(query);
        ui.sync_search_text(&model.search);
        command = refresh_matches(model, ui);
    }

    model.mark_session_dirty();
    ui.preset_load_show = false;
    ui.preset_load_entries.clear();
    ui.toast.success("Preset loaded");

    command
}

fn handle_save_dialog_opened(model: &Model, ui: &mut UiState) {
    ui.preset_generic = false;
    ui.preset_include_search = model.search.has_query();
    ui.preset_include_selection = true;
    ui.preset_name.clear();
    ui.preset_name_focus = true;
    ui.preset_save_show = true;
}

fn handle_saved(model: &mut Model, ui: &mut UiState, name: String) {
    if name.trim().is_empty() {
        ui.toast.error("Name the preset before saving it");

        return;
    }

    if !ui.preset_include_selection {
        if !ui.preset_include_search {
            ui.toast.error("Select at least one option to save");

            return;
        }
    }

    let paths = if ui.preset_include_selection {
        let mut checked: Vec<PathBuf> = model.tree.collect_checked_paths().into_iter().collect();

        if checked.is_empty() {
            ui.toast.error("No files selected to save");

            return;
        }

        checked.sort();

        Some(checked)
    } else {
        None
    };

    let query = ui
        .preset_include_search
        .then(|| model.search.query().to_owned())
        .filter(|query| !query.is_empty());

    let root = if ui.preset_generic {
        None
    } else {
        model.tree.nodes.first().map(|node| node.path.clone())
    };

    model
        .presets
        .add(Preset::new(name, PresetSelection { paths, query, root }));

    match model.presets.save_to_disk() {
        Ok(()) => ui.toast.success("Preset saved"),
        Err(error) => ui
            .toast
            .error(format!("Failed to save the preset: {error}")),
    }

    ui.preset_save_show = false;
    ui.preset_name.clear();
}

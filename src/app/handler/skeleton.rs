use std::sync::Arc;

use crate::app::message::{Cmd, Skeleton};
use crate::app::state::{Model, UiState};
use crate::app::state::ui::GenerateMode;

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Skeleton) -> Cmd {
    match msg {
        Skeleton::ModeChanged(mode) => handle_mode_changed(ui, mode),
        Skeleton::Requested => handle_skeleton_requested(model, ui),
        Skeleton::Started => handle_skeleton_started(ui),
        Skeleton::Generated(output) => handle_skeleton_generated(model, ui, output),
        Skeleton::Failed(error) => handle_skeleton_failed(ui, error),
    }
}

fn handle_mode_changed(ui: &mut UiState, mode: GenerateMode) -> Cmd {
    ui.generate_mode = mode;
    Cmd::None
}

fn handle_skeleton_requested(model: &mut Model, ui: &mut UiState) -> Cmd {
    if ui.skeleton_gen_in_progress {
        return Cmd::None;
    }

    model.refresh_git_status();

    let paths = model.tree.gather_checked_paths_with_git(&model.search, Some(&model.git));

    if paths.is_empty() {
        return Cmd::None;
    }

    ui.skeleton_gen_in_progress = true;

    Cmd::GenerateSkeleton {
        paths,
        options: Arc::clone(&model.options),
    }
}

fn handle_skeleton_started(ui: &mut UiState) -> Cmd {
    ui.skeleton_gen_in_progress = true;
    Cmd::None
}

fn handle_skeleton_generated(model: &mut Model, ui: &mut UiState, output: String) -> Cmd {
    model.tree.output = output;
    ui.skeleton_gen_in_progress = false;

    ui.toast.success("Skeleton copied to clipboard");

    Cmd::None
}

fn handle_skeleton_failed(ui: &mut UiState, error: String) -> Cmd {
    ui.skeleton_gen_in_progress = false;
    eprintln!("Skeleton generation failed: {}", error);

    ui.toast.error(format!("Skeleton failed: {}", error));

    Cmd::None
}

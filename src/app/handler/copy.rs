use alloc::sync::Arc;

use crate::app::message::{Command, CopyMessage};
use crate::app::state::{Model, UiState};
use crate::services::worker::{GatherJob, Report};

use super::common::prepare_selection;

pub fn handle(model: &mut Model, ui: &mut UiState, message: CopyMessage) -> Command {
    match message {
        CopyMessage::Completed(report) => handle_completed(ui, report),
        CopyMessage::Failed(error) => handle_failed(ui, &error),
        CopyMessage::Requested => handle_requested(model, ui),
    }
}

fn handle_completed(ui: &mut UiState, report: Report) -> Command {
    debug_assert!(ui.copy_in_progress);

    ui.copy_in_progress = false;
    ui.clipboard_pending = Some(report.output);
    ui.toast.success(report.summary);

    Command::None
}

fn handle_failed(ui: &mut UiState, error: &str) -> Command {
    debug_assert!(ui.copy_in_progress);

    ui.copy_in_progress = false;
    ui.toast.error(format!("Copy failed: {error}"));

    Command::None
}

fn handle_requested(model: &mut Model, ui: &mut UiState) -> Command {
    if ui.copy_in_progress {
        return Command::None;
    }

    prepare_selection(model, ui);

    let paths = model.checked_paths();

    if paths.is_empty() {
        return Command::None;
    }

    ui.copy_in_progress = true;

    Command::GatherFiles(Box::new(GatherJob {
        git: model.git_service.clone(),
        options: Arc::clone(&model.options),
        paths,
        query: model.search.parsed().into_owned(),
    }))
}

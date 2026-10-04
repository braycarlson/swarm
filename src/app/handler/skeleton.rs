use alloc::sync::Arc;

use crate::app::message::{Command, Skeleton};
use crate::app::state::{Model, UiState};
use crate::services::worker::SkeletonJob;

use super::common::prepare_selection;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Skeleton) -> Command {
    match message {
        Skeleton::Completed(report) => {
            debug_assert!(ui.skeleton_generate_in_progress);

            ui.skeleton_generate_in_progress = false;
            ui.clipboard_pending = Some(report.output);
            ui.toast.success(report.summary);

            Command::None
        }
        Skeleton::Failed(error) => {
            debug_assert!(ui.skeleton_generate_in_progress);

            ui.skeleton_generate_in_progress = false;
            ui.toast.error(format!("Skeleton failed: {error}"));

            Command::None
        }
        Skeleton::ModeChanged(mode) => {
            ui.generate_mode = mode;

            Command::None
        }
        Skeleton::Requested => handle_requested(model, ui),
    }
}

fn handle_requested(model: &mut Model, ui: &mut UiState) -> Command {
    if ui.skeleton_generate_in_progress {
        return Command::None;
    }

    prepare_selection(model, ui);

    let paths = model.checked_paths();

    if paths.is_empty() {
        return Command::None;
    }

    let format = model
        .search
        .parsed()
        .format_override
        .unwrap_or(model.options.output_format);

    ui.skeleton_generate_in_progress = true;

    Command::GenerateSkeleton(Box::new(SkeletonJob {
        format,
        options: Arc::clone(&model.options),
        paths,
    }))
}

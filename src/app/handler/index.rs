use std::sync::Arc;

use crate::app::message::{Cmd, Index};
use crate::app::state::{IndexStatus, Model};

pub fn handle(model: &mut Model, msg: Index) -> Cmd {
    match msg {
        Index::StartRequested => handle_index_start_requested(model),
        Index::StopRequested => handle_index_stop_requested(model),
        Index::PauseRequested => handle_index_pause_requested(model),
        Index::ResumeRequested => handle_index_resume_requested(model),
        Index::Progress(stats) => handle_index_progress(model, stats),
        Index::Completed { count, extensions } => handle_index_completed(model, count, extensions),
        Index::Failed(error) => handle_index_failed(model, error),
        Index::SearchQueryChanged(query) => handle_index_search_query_changed(model, query),
        Index::ExtensionSelected(ext) => handle_index_extension_selected(model, ext),
    }
}

fn handle_index_start_requested(model: &mut Model) -> Cmd {
    if matches!(model.index.status, IndexStatus::Running { .. }) {
        return Cmd::None;
    }

    let paths: Vec<_> = model.tree.nodes.iter().map(|n| n.path.clone()).collect();

    model.index.status = IndexStatus::Running { paused: false };

    Cmd::StartIndexing {
        paths,
        options: Arc::clone(&model.options),
    }
}

fn handle_index_stop_requested(model: &mut Model) -> Cmd {
    model.index.status = IndexStatus::Idle;
    Cmd::StopIndexing
}

fn handle_index_pause_requested(model: &mut Model) -> Cmd {
    if let IndexStatus::Running { paused } = model.index.status {
        if !paused {
            model.index.status = IndexStatus::Running { paused: true };
            return Cmd::PauseIndexing;
        }
    }

    Cmd::None
}

fn handle_index_resume_requested(model: &mut Model) -> Cmd {
    if let IndexStatus::Running { paused } = model.index.status {
        if paused {
            model.index.status = IndexStatus::Running { paused: false };
            return Cmd::ResumeIndexing;
        }
    }

    Cmd::None
}

fn handle_index_progress(model: &mut Model, stats: crate::services::filesystem::index::IndexStatistics) -> Cmd {
    model.index.statistics = Some(stats);
    Cmd::None
}

fn handle_index_completed(model: &mut Model, _count: usize, extensions: Vec<String>) -> Cmd {
    model.index.status = IndexStatus::Completed;
    model.index.extensions = extensions.into_iter().collect();

    if let Some(session) = model.sessions.active_session_mut() {
        session.has_been_indexed = true;
        session.mark_modified();
    }

    Cmd::None
}

fn handle_index_failed(model: &mut Model, error: String) -> Cmd {
    model.index.status = IndexStatus::Failed(error);
    Cmd::None
}

fn handle_index_search_query_changed(model: &mut Model, query: String) -> Cmd {
    model.index.search_query = query;
    Cmd::None
}

fn handle_index_extension_selected(model: &mut Model, ext: Option<String>) -> Cmd {
    model.index.active_extension = ext;
    Cmd::None
}

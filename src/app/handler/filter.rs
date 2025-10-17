use crate::app::message::{Cmd, Filter};
use crate::app::state::{Model, UiState};

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Filter) -> Cmd {
    match msg {
        Filter::IncludeAdded(filter) => handle_include_filter_added(model, ui, filter),
        Filter::IncludeRemoved(index) => handle_include_filter_removed(model, index),
        Filter::IncludesCleared => handle_include_filters_cleared(model),
        Filter::IncludeFilterChanged(text) => handle_include_filter_changed(ui, text),
        Filter::ExcludeAdded(filter) => handle_exclude_filter_added(model, ui, filter),
        Filter::ExcludeRemoved(index) => handle_exclude_filter_removed(model, index),
        Filter::ExcludesReset => handle_exclude_filters_reset(model),
        Filter::ExcludeFilterChanged(text) => handle_exclude_filter_changed(ui, text),
    }
}

fn handle_include_filter_added(model: &mut Model, ui: &mut UiState, filter: String) -> Cmd {
    let mut new_options = (*model.options).clone();

    if new_options.add_include_filter(filter) {
        ui.new_include_filter.clear();
        model.update_options(new_options);
    }

    Cmd::None
}

fn handle_include_filter_removed(model: &mut Model, index: usize) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.remove_include_filter(index);
    model.update_options(new_options);

    Cmd::None
}

fn handle_include_filters_cleared(model: &mut Model) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.include.clear();

    let _ = new_options.save();
    model.update_options(new_options);

    Cmd::None
}

fn handle_include_filter_changed(ui: &mut UiState, text: String) -> Cmd {
    ui.new_include_filter = text;
    Cmd::None
}

fn handle_exclude_filter_added(model: &mut Model, ui: &mut UiState, filter: String) -> Cmd {
    let mut new_options = (*model.options).clone();

    if new_options.add_exclude_filter(filter) {
        ui.new_exclude_filter.clear();
        model.update_options(new_options);
    }

    Cmd::None
}

fn handle_exclude_filter_removed(model: &mut Model, index: usize) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.remove_exclude_filter(index);
    model.update_options(new_options);

    Cmd::None
}

fn handle_exclude_filters_reset(model: &mut Model) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.reset_excludes_to_defaults();
    model.update_options(new_options);

    Cmd::None
}

fn handle_exclude_filter_changed(ui: &mut UiState, text: String) -> Cmd {
    ui.new_exclude_filter = text;
    Cmd::None
}

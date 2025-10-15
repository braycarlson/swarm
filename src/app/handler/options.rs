use std::sync::Arc;

use crate::app::message::{Cmd, Options_};
use crate::app::state::{Model, UiState};

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Options_) -> Cmd {
    match msg {
        Options_::Opened => handle_options_opened(model, ui),
        Options_::Closed => handle_options_closed(model, ui),
        Options_::TabChanged(tab) => handle_options_tab_changed(ui, tab),
        Options_::ThemeChanged(theme) => handle_option_theme_changed(model, ui, theme),
        Options_::UseIconChanged(value) => handle_option_use_icon_changed(model, value),
        Options_::AutoIndexChanged(value) => handle_option_auto_index_changed(model, value),
        Options_::DeleteSessionsChanged(value) => handle_option_delete_sessions_changed(model, value),
        Options_::SingleInstanceChanged(value) => handle_option_single_instance_changed(model, value),
        Options_::OutputFormatChanged(format) => handle_option_output_format_changed(model, format),
    }
}

fn handle_options_opened(model: &mut Model, ui: &mut UiState) -> Cmd {
    ui.show_options = true;
    model.save_original_options();
    Cmd::None
}

fn handle_options_closed(model: &mut Model, ui: &mut UiState) -> Cmd {
    ui.show_options = false;

    if model.options_changed() {
        model.tree.states = Some(model.tree.collect_checkbox_states());

        Cmd::RefreshTree {
            nodes: model.tree.nodes.clone(),
            options: Arc::clone(&model.options),
        }
    } else {
        Cmd::None
    }
}

fn handle_options_tab_changed(ui: &mut UiState, tab: crate::app::state::OptionsTab) -> Cmd {
    ui.options_tab = tab;
    Cmd::None
}

fn handle_option_theme_changed(model: &mut Model, ui: &mut UiState, theme: crate::ui::themes::Theme) -> Cmd {
    ui.theme = theme;
    let mut new_options = (*model.options).clone();
    new_options.theme = theme;
    let _ = new_options.save();
    model.update_options(new_options);

    Cmd::None
}

fn handle_option_use_icon_changed(model: &mut Model, value: bool) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.use_icon = value;
    let _ = new_options.save();
    model.update_options(new_options);

    Cmd::None
}

fn handle_option_auto_index_changed(model: &mut Model, value: bool) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.auto_index_on_startup = value;
    let _ = new_options.save();
    model.update_options(new_options);

    Cmd::None
}

fn handle_option_delete_sessions_changed(model: &mut Model, value: bool) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.delete_sessions_on_exit = value;
    let _ = new_options.save();
    model.update_options(new_options);

    Cmd::None
}

fn handle_option_single_instance_changed(model: &mut Model, value: bool) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.single_instance = value;
    let _ = new_options.save();
    model.update_options(new_options);

    Cmd::None
}

fn handle_option_output_format_changed(model: &mut Model, format: crate::model::output::OutputFormat) -> Cmd {
    let mut new_options = (*model.options).clone();
    new_options.output_format = format;
    let _ = new_options.save();
    model.update_options(new_options);

    Cmd::None
}

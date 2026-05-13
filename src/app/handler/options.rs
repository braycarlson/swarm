use std::sync::Arc;

use crate::app::message::{Command, Options_};
use crate::app::state::{Model, UiState};

pub fn handle(model: &mut Model, ui: &mut UiState, message: Options_) -> Command {
    match message {
        Options_::Opened => handle_options_opened(model, ui),
        Options_::Closed => handle_options_closed(model, ui),
        Options_::TabChanged(tab) => handle_options_tab_changed(ui, tab),
        Options_::ThemeChanged(theme) => handle_option_theme_changed(model, ui, theme),
        Options_::UiScaleApplied => handle_option_ui_scale_applied(model, ui),
        Options_::UiScaleChanged(scale) => handle_option_ui_scale_changed(ui, scale),
        Options_::UiScaleReset => handle_option_ui_scale_reset(model, ui),
        Options_::UseIconChanged(value) => handle_option_use_icon_changed(model, value),
        Options_::ShowHiddenChanged(value) => handle_option_show_hidden_changed(model, value),
        Options_::DeleteSessionsChanged(value) => handle_option_delete_sessions_changed(model, value),
        Options_::SingleInstanceChanged(value) => handle_option_single_instance_changed(model, value),
        Options_::OutputFormatChanged(format) => handle_option_output_format_changed(model, format),
    }
}

fn handle_options_opened(model: &mut Model, ui: &mut UiState) -> Command {
    ui.options_show = true;
    ui.ui_scale_draft = None;
    model.save_original_options();

    Command::None
}

fn handle_options_closed(model: &mut Model, ui: &mut UiState) -> Command {
    ui.options_show = false;

    if let Some(scale) = ui.ui_scale_draft.take() {
        let mut new_options = (*model.options).clone();
        new_options.ui_scale = Some(scale.clamp(0.5, 3.0));

        let _ = new_options.save();
        model.update_options(new_options);
    }

    if model.options_changed() {
        model.tree.states = Some(model.tree.collect_checkbox_states());

        Command::RefreshTree {
            nodes: model.tree.nodes.clone(),
            options: Arc::clone(&model.options),
        }
    } else {
        Command::None
    }
}

fn handle_options_tab_changed(ui: &mut UiState, tab: crate::app::state::OptionsTab) -> Command {
    ui.options_tab = tab;
    Command::None
}

fn handle_option_theme_changed(model: &mut Model, ui: &mut UiState, theme: crate::ui::themes::Theme) -> Command {
    ui.theme = theme;

    let mut new_options = (*model.options).clone();
    new_options.theme = theme;

    let _ = new_options.save();
    model.update_options(new_options);

    Command::None
}

fn handle_option_ui_scale_applied(model: &mut Model, ui: &mut UiState) -> Command {
    if let Some(scale) = ui.ui_scale_draft.take() {
        let mut new_options = (*model.options).clone();
        new_options.ui_scale = Some(scale.clamp(0.5, 3.0));

        let _ = new_options.save();
        model.update_options(new_options);
    }

    Command::None
}

fn handle_option_ui_scale_changed(ui: &mut UiState, scale: f32) -> Command {
    ui.ui_scale_draft = Some(scale.clamp(0.5, 3.0));
    Command::None
}

fn handle_option_use_icon_changed(model: &mut Model, value: bool) -> Command {
    let mut new_options = (*model.options).clone();
    new_options.use_icon = value;

    let _ = new_options.save();
    model.update_options(new_options);

    Command::None
}

fn handle_option_show_hidden_changed(model: &mut Model, value: bool) -> Command {
    let mut new_options = (*model.options).clone();
    new_options.show_hidden = value;

    let _ = new_options.save();
    model.update_options(new_options);

    Command::None
}

fn handle_option_delete_sessions_changed(model: &mut Model, value: bool) -> Command {
    let mut new_options = (*model.options).clone();
    new_options.delete_sessions_on_exit = value;

    let _ = new_options.save();
    model.update_options(new_options);

    Command::None
}

fn handle_option_single_instance_changed(model: &mut Model, value: bool) -> Command {
    let mut new_options = (*model.options).clone();
    new_options.single_instance = value;

    let _ = new_options.save();
    model.update_options(new_options);

    Command::None
}

fn handle_option_output_format_changed(model: &mut Model, format: crate::model::output::OutputFormat) -> Command {
    let mut new_options = (*model.options).clone();
    new_options.output_format = format;

    let _ = new_options.save();
    model.update_options(new_options);

    Command::None
}

fn handle_option_ui_scale_reset(model: &mut Model, ui: &mut UiState) -> Command {
    ui.ui_scale_draft = None;

    let mut new_options = (*model.options).clone();
    new_options.ui_scale = None;

    let _ = new_options.save();
    model.update_options(new_options);

    Command::None
}

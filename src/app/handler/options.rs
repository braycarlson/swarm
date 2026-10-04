use alloc::sync::Arc;

use crate::app::message::{Command, OptionsMessage};
use crate::app::state::{LoadStatus, Model, UiState};
use crate::constants::{UI_SCALE_MAX, UI_SCALE_MIN};
use crate::model::options::Options;
use crate::platform;
use crate::services::worker::TreeRefreshRequest;

pub fn commit_options(model: &mut Model, ui: &mut UiState, options: Options) {
    if let Err(error) = model.update_options(options) {
        ui.toast
            .error(format!("The options were not applied: {error}"));

        return;
    }

    if let Err(error) = model.options.save() {
        ui.toast
            .error(format!("Failed to save the options: {error}"));
    }
}

pub fn handle(model: &mut Model, ui: &mut UiState, message: OptionsMessage) -> Command {
    match message {
        OptionsMessage::Closed => return handle_closed(model, ui),
        OptionsMessage::ContextMenuRegisterRequested => handle_context_menu(ui, true),
        OptionsMessage::ContextMenuUnregisterRequested => handle_context_menu(ui, false),
        OptionsMessage::DeleteSessionsChanged(value) => {
            update_option(model, ui, |options| options.delete_sessions_on_exit = value);
        }
        OptionsMessage::Opened => {
            ui.options_show = true;
            ui.ui_scale_draft = None;
            model.save_original_options();
        }
        OptionsMessage::OutputFormatChanged(format) => {
            update_option(model, ui, |options| options.output_format = format);
        }
        OptionsMessage::ShowHiddenChanged(value) => {
            update_option(model, ui, |options| options.show_hidden = value);
        }
        OptionsMessage::SingleInstanceChanged(value) => {
            update_option(model, ui, |options| options.single_instance = value);
        }
        OptionsMessage::TabChanged(tab) => ui.options_tab = tab,
        OptionsMessage::ThemeChanged(theme) => {
            ui.theme = theme;
            ui.theme_dirty = true;
            update_option(model, ui, |options| options.theme = theme);
        }
        OptionsMessage::UiScaleApplied => {
            if let Some(scale) = ui.ui_scale_draft.take() {
                update_option(model, ui, |options| {
                    options.ui_scale = Some(clamp_scale(scale));
                });
            }
        }
        OptionsMessage::UiScaleChanged(scale) => ui.ui_scale_draft = Some(clamp_scale(scale)),
        OptionsMessage::UiScaleReset => {
            ui.ui_scale_draft = None;
            update_option(model, ui, |options| options.ui_scale = None);
        }
        OptionsMessage::UseIconChanged(value) => {
            update_option(model, ui, |options| options.use_icon = value);
        }
    }

    Command::None
}

fn clamp_scale(scale: f32) -> f32 {
    let clamped = if scale.is_finite() {
        scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX)
    } else {
        UI_SCALE_MIN
    };

    debug_assert!(clamped >= UI_SCALE_MIN);
    debug_assert!(clamped <= UI_SCALE_MAX);

    clamped
}

fn handle_closed(model: &mut Model, ui: &mut UiState) -> Command {
    ui.options_show = false;

    if let Some(scale) = ui.ui_scale_draft.take() {
        update_option(model, ui, |options| {
            options.ui_scale = Some(clamp_scale(scale));
        });
    }

    if !model.options_changed() {
        return Command::None;
    }

    let Some(session) = model.sessions.active_identifier().map(str::to_owned) else {
        return Command::None;
    };

    if model.tree.nodes.is_empty() {
        return Command::None;
    }

    if model.tree.is_loading() {
        return Command::None;
    }

    model.tree.states = Some(model.tree.collect_checked_paths());
    model.tree.load_status = LoadStatus::Loading("Applying the options...".to_owned());

    let nodes = core::mem::take(&mut model.tree.nodes);

    Command::batch(vec![
        Command::StopBackground,
        Command::RefreshTree(Box::new(TreeRefreshRequest {
            filter: Arc::clone(&model.filter),
            nodes,
            session,
        })),
    ])
}

fn handle_context_menu(ui: &mut UiState, register: bool) {
    let outcome = if register {
        platform::context_menu_register()
    } else {
        platform::context_menu_unregister()
    };

    let action = if register { "register" } else { "unregister" };

    match outcome {
        Ok(()) => ui.toast.success(format!("Context menu {action}ed")),
        Err(error) => ui
            .toast
            .error(format!("Failed to {action} the context menu: {error}")),
    }
}

fn update_option(model: &mut Model, ui: &mut UiState, mutate: impl FnOnce(&mut Options)) {
    let mut options = (*model.options).clone();

    mutate(&mut options);
    commit_options(model, ui, options);
}

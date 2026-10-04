use std::time::Instant;

use crate::app::state::{SearchModel, SessionData};
use crate::model::theme::Theme;
use crate::ui::panel::tree::TreeRow;
use crate::ui::widget::toast::ToastSystem;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FilterStatus {
    Complete,
    Filtering,
    #[default]
    Idle,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GenerateMode {
    Skeleton,
    #[default]
    Tree,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OptionsTab {
    Excludes,
    #[default]
    General,
    Includes,
}

pub struct PresetEntry {
    pub identifier: String,
    pub name: String,
}

pub struct UiState {
    pub about_show: bool,
    pub bulk_select_show: bool,
    pub bulk_select_text: String,
    pub clipboard_pending: Option<String>,
    pub copy_in_progress: bool,
    pub file_dialog_open: bool,
    pub filter_status: FilterStatus,
    pub generate_mode: GenerateMode,
    pub options_show: bool,
    pub options_tab: OptionsTab,
    pub pattern_exclude_new: String,
    pub pattern_include_new: String,
    pub preset_generic: bool,
    pub preset_include_search: bool,
    pub preset_include_selection: bool,
    pub preset_load_entries: Vec<PresetEntry>,
    pub preset_load_show: bool,
    pub preset_name: String,
    pub preset_name_focus: bool,
    pub preset_save_show: bool,
    pub search_debounce: Option<Instant>,
    pub search_text: String,
    pub session_edit_focus: bool,
    pub session_editing: Option<String>,
    pub session_editing_name: String,
    pub skeleton_generate_in_progress: bool,
    pub theme: Theme,
    pub theme_dirty: bool,
    pub toast: ToastSystem,
    pub tree_generate_in_progress: bool,
    pub tree_rows: Vec<TreeRow>,
    pub ui_scale_draft: Option<f32>,
}

impl UiState {
    pub fn cancel_session_edit(&mut self) {
        self.session_edit_focus = false;
        self.session_editing = None;
        self.session_editing_name.clear();
    }

    pub fn is_generating(&self) -> bool {
        self.tree_generate_in_progress || self.skeleton_generate_in_progress
    }

    pub fn new(theme: Theme) -> Self {
        Self {
            about_show: false,
            bulk_select_show: false,
            bulk_select_text: String::new(),
            clipboard_pending: None,
            copy_in_progress: false,
            file_dialog_open: false,
            filter_status: FilterStatus::Idle,
            generate_mode: GenerateMode::Tree,
            options_show: false,
            options_tab: OptionsTab::General,
            pattern_exclude_new: String::new(),
            pattern_include_new: String::new(),
            preset_generic: false,
            preset_include_search: false,
            preset_include_selection: true,
            preset_load_entries: Vec::new(),
            preset_load_show: false,
            preset_name: String::new(),
            preset_name_focus: false,
            preset_save_show: false,
            search_debounce: None,
            search_text: String::new(),
            session_edit_focus: false,
            session_editing: None,
            session_editing_name: String::new(),
            skeleton_generate_in_progress: false,
            theme,
            theme_dirty: true,
            toast: ToastSystem::default(),
            tree_generate_in_progress: false,
            tree_rows: Vec::new(),
            ui_scale_draft: None,
        }
    }

    pub fn start_search_debounce(&mut self) {
        self.search_debounce = Some(Instant::now());
    }

    pub fn start_session_edit(&mut self, session: &SessionData) {
        assert_ne!(session.identifier, "");

        self.session_edit_focus = true;
        self.session_editing = Some(session.identifier.clone());
        self.session_editing_name.clone_from(&session.name);
    }

    pub fn sync_search_text(&mut self, search: &SearchModel) {
        self.search_text.clear();
        self.search_text.push_str(search.query());
        self.search_debounce = None;
    }

    pub fn take_debounced_search(&mut self, debounce_ms: u64) -> bool {
        let Some(instant) = self.search_debounce else {
            return false;
        };

        if instant.elapsed().as_millis() < u128::from(debounce_ms) {
            return false;
        }

        self.search_debounce = None;

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_debounce_fires_once_after_the_delay() {
        let mut state = UiState::new(Theme::default());

        assert!(!state.take_debounced_search(0));

        state.start_search_debounce();

        assert!(state.take_debounced_search(0));
        assert!(!state.take_debounced_search(0));
    }

    #[test]
    fn cancelling_an_edit_clears_its_state() {
        let mut state = UiState::new(Theme::default());

        state.start_session_edit(&SessionData::new("name".to_owned()));

        assert!(state.session_editing.is_some());

        state.cancel_session_edit();

        assert!(state.session_editing.is_none());
        assert_eq!(state.session_editing_name, "");
        assert!(!state.session_edit_focus);
    }
}

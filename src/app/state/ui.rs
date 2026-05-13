use std::time::Instant;

use crate::ui::themes::Theme;
use crate::ui::widget::toast::ToastSystem;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OptionsTab {
    Excludes,
    #[default]
    General,
    Includes,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FilterStatus {
    #[default]
    Idle,
    Filtering,
    Complete,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GenerateMode {
    #[default]
    Tree,
    Skeleton,
}

#[derive(Clone, Default)]
pub struct OptionsState {
    pub active_tab: OptionsTab,
    pub exclude_new: String,
    pub exclude_selected: Option<usize>,
    pub include_new: String,
    pub include_selected: Option<usize>,
}

impl OptionsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_exclude_new(&mut self) {
        self.exclude_new.clear();
    }

    pub fn clear_include_new(&mut self) {
        self.include_new.clear();
    }
}

#[derive(Clone)]
pub struct UiState {
    pub about_show: bool,
    pub copy_in_progress: bool,
    pub search_debounce: Option<Instant>,
    pub file_dialog_pending: bool,
    pub filter_exclude_new: String,
    pub filter_include_new: String,
    pub filter_status: FilterStatus,
    pub generate_mode: GenerateMode,
    pub editing_session_name: String,
    pub options_show: bool,
    pub options_tab: OptionsTab,
    pub preset_generic: bool,
    pub preset_include_search: bool,
    pub preset_include_selection: bool,
    pub preset_load_show: bool,
    pub preset_name: String,
    pub preset_save_show: bool,
    pub search_pending: Option<String>,
    pub bulk_select_show: bool,
    pub bulk_select_text: String,
    pub session_editing: Option<String>,
    pub should_focus: bool,
    pub skeleton_generate_in_progress: bool,
    pub theme: Theme,
    pub toast: ToastSystem,
    pub tree_generate_in_progress: bool,
    pub ui_scale_draft: Option<f32>,
}

impl UiState {
    pub fn new(theme: Theme) -> Self {
        Self {
            about_show: false,
            copy_in_progress: false,
            search_debounce: None,
            file_dialog_pending: false,
            filter_exclude_new: String::new(),
            filter_include_new: String::new(),
            filter_status: FilterStatus::Idle,
            generate_mode: GenerateMode::default(),
            editing_session_name: String::new(),
            options_show: false,
            options_tab: OptionsTab::default(),
            preset_generic: false,
            preset_include_search: false,
            preset_include_selection: true,
            preset_load_show: false,
            preset_name: String::new(),
            preset_save_show: false,
            search_pending: None,
            bulk_select_show: false,
            bulk_select_text: String::new(),
            session_editing: None,
            should_focus: false,
            skeleton_generate_in_progress: false,
            theme,
            toast: ToastSystem::new(),
            tree_generate_in_progress: false,
            ui_scale_draft: None,
        }
    }

    pub fn start_session_edit(&mut self, identifier: String, name: String) {
        self.session_editing = Some(identifier);
        self.editing_session_name = name;
    }

    pub fn cancel_session_edit(&mut self) {
        self.session_editing = None;
        self.editing_session_name.clear();
    }

    pub fn set_search_pending(&mut self, query: String) {
        self.search_pending = Some(query);
        self.search_debounce = Some(Instant::now());
    }

    pub fn take_debounced_search(&mut self, debounce_ms: u64) -> Option<String> {
        if let Some(instant) = self.search_debounce {
            if instant.elapsed().as_millis() >= debounce_ms as u128 {
                self.search_debounce = None;
                return self.search_pending.take();
            }
        }

        None
    }
}

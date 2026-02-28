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
    pub new_exclude: String,
    pub new_include: String,
    pub selected_exclude: Option<usize>,
    pub selected_include: Option<usize>,
}

impl OptionsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_new_exclude(&mut self) {
        self.new_exclude.clear();
    }

    pub fn clear_new_include(&mut self) {
        self.new_include.clear();
    }
}

#[derive(Clone)]
pub struct UiState {
    pub copy_in_progress: bool,
    pub edit_name: String,
    pub editing_session: Option<String>,
    pub file_dialog_pending: bool,
    pub filter_status: FilterStatus,
    pub generate_mode: GenerateMode,
    pub new_exclude_filter: String,
    pub new_include_filter: String,
    pub options_tab: OptionsTab,
    pub search_debounce: Option<Instant>,
    pub search_pending: Option<String>,
    pub should_focus: bool,
    pub show_about: bool,
    pub show_options: bool,
    pub skeleton_gen_in_progress: bool,
    pub theme: Theme,
    pub toast: ToastSystem,
    pub tree_gen_in_progress: bool,
}

impl UiState {
    pub fn new(theme: Theme) -> Self {
        Self {
            copy_in_progress: false,
            edit_name: String::new(),
            editing_session: None,
            file_dialog_pending: false,
            filter_status: FilterStatus::Idle,
            generate_mode: GenerateMode::default(),
            new_exclude_filter: String::new(),
            new_include_filter: String::new(),
            options_tab: OptionsTab::default(),
            search_debounce: None,
            search_pending: None,
            should_focus: false,
            show_about: false,
            show_options: false,
            skeleton_gen_in_progress: false,
            theme,
            toast: ToastSystem::new(),
            tree_gen_in_progress: false,
        }
    }

    pub fn start_session_edit(&mut self, id: String, name: String) {
        self.editing_session = Some(id);
        self.edit_name = name;
    }

    pub fn cancel_session_edit(&mut self) {
        self.editing_session = None;
        self.edit_name.clear();
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

use crate::ui::themes::Theme;
use crate::ui::widget::toast::ToastSystem;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OptionsTab {
    Excludes,
    #[default]
    General,
    Includes,
}

#[derive(Default)]
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
    pub theme: Theme,
    pub should_focus: bool,
    pub show_options: bool,
    pub show_about: bool,
    pub editing_session: Option<String>,
    pub edit_name: String,
    pub options_tab: OptionsTab,
    pub new_include_filter: String,
    pub new_exclude_filter: String,
    pub file_dialog_pending: bool,
    pub copy_in_progress: bool,
    pub toast: ToastSystem,
    pub tree_gen_in_progress: bool,
}

impl UiState {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            should_focus: false,
            show_options: false,
            show_about: false,
            editing_session: None,
            edit_name: String::new(),
            options_tab: OptionsTab::default(),
            new_include_filter: String::new(),
            new_exclude_filter: String::new(),
            file_dialog_pending: false,
            copy_in_progress: false,
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
}

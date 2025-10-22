use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::worker::BackgroundLoader;
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
pub struct Model {
    pub background_loader: BackgroundLoader,
    pub sessions: SessionsModel,
    pub tree: TreeModel,
    pub search: SearchModel,
    pub options: Arc<Options>,
    pub original_options: Arc<Options>,
}

impl Model {
    pub fn new(initial_paths: Vec<String>) -> Self {
        let options = Arc::new(Options::load().unwrap_or_default());
        let sessions = SessionsModel::new();

        Self {
            background_loader: BackgroundLoader::new(),
            sessions,
            tree: TreeModel::new(initial_paths),
            search: SearchModel::default(),
            options: Arc::clone(&options),
            original_options: options,
        }
    }

    pub fn update_options(&mut self, new_options: Options) {
        self.options = Arc::new(new_options);
    }

    pub fn save_original_options(&mut self) {
        self.original_options = Arc::clone(&self.options);
    }

    pub fn options_changed(&self) -> bool {
        !self.options.is_equal(&self.original_options)
    }
}

#[derive(Clone)]
pub struct SessionsModel {
    pub active_id: Option<String>,
    pub sessions: HashMap<String, SessionData>,
}

impl SessionsModel {
    pub fn new() -> Self {
        Self {
            active_id: None,
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, name: String) -> String {
        let session = SessionData::new(name);
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.active_id = Some(id.clone());
        id
    }

    pub fn select_session(&mut self, id: String) -> Option<&SessionData> {
        if self.sessions.contains_key(&id) {
            self.active_id = Some(id.clone());
            self.sessions.get(&id)
        } else {
            None
        }
    }

    pub fn delete_session(&mut self, id: &str) -> Option<String> {
        let position = self.get_session_position(id);
        self.sessions.remove(id);

        if self.active_id.as_ref() == Some(&id.to_string()) {
            self.active_id = self.get_closest_left_session(position);
        }

        self.active_id.clone()
    }

    pub fn active_session(&self) -> Option<&SessionData> {
        self.active_id.as_ref().and_then(|id| self.sessions.get(id))
    }

    pub fn active_session_mut(&mut self) -> Option<&mut SessionData> {
        self.active_id.as_ref().and_then(|id| self.sessions.get_mut(id))
    }

    pub fn sync_from_tree_and_search(&mut self, tree: TreeModel, search: SearchModel) {
        if let Some(session) = self.active_session_mut() {
            session.tree_state = tree;
            session.search_state = search;
            session.mark_modified();
        }
    }

    pub fn session_list(&self) -> Vec<&SessionData> {
        let mut sessions: Vec<&SessionData> = self.sessions.values().collect();
        sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        sessions
    }

    pub fn rename_session(&mut self, id: &str, new_name: String) {
        if let Some(session) = self.sessions.get_mut(id) {
            session.name = new_name;
            session.mark_modified();
        }
    }

    fn get_session_position(&self, id: &str) -> usize {
        let mut ids: Vec<_> = self.sessions.keys().cloned().collect();

        ids.sort_by(|a, b| {
            let sa = self.sessions.get(a).unwrap();
            let sb = self.sessions.get(b).unwrap();
            sa.created_at.cmp(&sb.created_at)
        });

        ids.iter().position(|sid| sid == id).unwrap_or(0)
    }

    fn get_closest_left_session(&self, position: usize) -> Option<String> {
        if self.sessions.is_empty() {
            return None;
        }

        let mut ids: Vec<_> = self.sessions.keys().cloned().collect();

        ids.sort_by(|a, b| {
            let sa = self.sessions.get(a).unwrap();
            let sb = self.sessions.get(b).unwrap();
            sa.created_at.cmp(&sb.created_at)
        });

        if position > 0 && position - 1 < ids.len() {
            Some(ids[position - 1].clone())
        } else if !ids.is_empty() {
            Some(ids[0].clone())
        } else {
            None
        }
    }
}

impl Default for SessionsModel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SessionData {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub last_modified: u64,
    pub tree_state: TreeModel,
    pub search_state: SearchModel,
}

impl SessionData {
    pub fn new(name: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: now,
            last_modified: now,
            tree_state: TreeModel::default(),
            search_state: SearchModel::default(),
        }
    }

    pub fn mark_modified(&mut self) {
        self.last_modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct TreeModel {
    pub nodes: Vec<FileNode>,
    pub output: String,
    pub load_status: LoadStatus,
    #[serde(skip)]
    pub states: Option<std::collections::HashMap<PathBuf, bool>>,
    #[serde(skip)]
    pub file_count: usize,
}

impl TreeModel {
    pub fn new(paths: Vec<String>) -> Self {
        let nodes = paths.into_iter()
            .map(|p| FileNode::new(PathBuf::from(p)))
            .collect();

        Self {
            nodes,
            output: String::new(),
            load_status: LoadStatus::NotStarted,
            states: None,
            file_count: 0,
        }
    }

    pub fn collect_checkbox_states(&self) -> std::collections::HashMap<PathBuf, bool> {
        let mut states = std::collections::HashMap::new();
        for node in &self.nodes {
            node.collect_checkbox_states_recursive(&mut states);
        }
        states
    }

    pub fn count_files(&self) -> usize {
        self.nodes.iter().map(count_files_recursive).sum()
    }

    pub fn restore_checkbox_states(&mut self, states: &std::collections::HashMap<PathBuf, bool>) {
        for node in &mut self.nodes {
            node.restore_checkbox_states_recursive(states);
        }
    }

    pub fn gather_checked_paths(&self, search: &SearchModel) -> Vec<String> {
        let mut results = Vec::new();
        let query = if search.has_query() { &search.query } else { "" };

        for node in &self.nodes {
            node.gather_checked_paths_recursive(&mut results, query);
        }
        results
    }

    pub fn create_filtered_tree(&self, search: &SearchModel) -> Vec<FileNode> {
        let query = if search.has_query() { &search.query } else { "" };
        self.nodes.iter().filter_map(|n| n.filter_selected(query)).collect()
    }

    pub fn update_file_count(&mut self) {
        self.file_count = self.count_files();
    }
}

fn count_files_recursive(node: &FileNode) -> usize {
    if node.is_file() {
        1
    } else {
        node.children.iter().map(count_files_recursive).sum()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LoadStatus {
    NotStarted,
    Loading { message: String, progress: (usize, usize) },
    Loaded,
    Failed(String),
}

impl Default for LoadStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SearchModel {
    pub query: String,
    pub active: bool,
}

impl SearchModel {
    pub fn has_query(&self) -> bool {
        !self.query.is_empty() && self.active
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.active = false;
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.active = true;
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

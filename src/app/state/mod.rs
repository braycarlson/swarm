pub mod search;
pub mod session;
pub mod tree;
pub mod ui;

use std::sync::Arc;

use crate::model::options::Options;
use crate::services::worker::BackgroundLoader;

pub use search::SearchModel;
pub use session::{SessionData, SessionsModel};
pub use tree::{LoadStatus, TreeModel};
pub use ui::{OptionsState, OptionsTab, UiState};

#[derive(Clone)]
pub struct Model {
    pub background_loader: BackgroundLoader,
    pub options: Arc<Options>,
    pub original_options: Arc<Options>,
    pub search: SearchModel,
    pub sessions: SessionsModel,
    pub tree: TreeModel,
}

impl Model {
    pub fn new(initial_paths: Vec<String>) -> Self {
        let options = Arc::new(Options::load().unwrap_or_default());

        Self {
            background_loader: BackgroundLoader::new(),
            options: Arc::clone(&options),
            original_options: options,
            search: SearchModel::default(),
            sessions: SessionsModel::new(),
            tree: TreeModel::new(initial_paths),
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

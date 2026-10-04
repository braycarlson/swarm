use alloc::sync::Arc;
use std::path::PathBuf;

use crate::app::message::{BackgroundStart, Command, FilterStart};
use crate::model::error::SwarmResult;
use crate::model::node::FileNode;
use crate::model::options::{Options, PatternList};
use crate::model::preset::PresetModel;
use crate::model::query::ParsedQuery;
use crate::services::filesystem::filter::GlobPathFilter;
use crate::services::filesystem::git::GitService;
use crate::services::tree::query::{self, MatchContext, Matcher};
use crate::services::worker::filter;

use super::{SearchModel, SessionsModel, TreeModel};

pub struct Model {
    pub filter: Arc<GlobPathFilter>,
    pub git_service: GitService,
    pub options: Arc<Options>,
    pub options_original: Arc<Options>,
    pub presets: PresetModel,
    pub search: SearchModel,
    pub session_dirty: bool,
    pub sessions: SessionsModel,
    pub tree: TreeModel,
}

impl Model {
    fn filter_start(&self, parsed: ParsedQuery) -> Command {
        let Some(session) = self.sessions.active_identifier() else {
            return Command::None;
        };

        if self.tree.nodes.is_empty() {
            return Command::None;
        }

        let entries = filter::build_filter_entries(&self.tree.nodes, &parsed);

        Command::StartFilter(Box::new(FilterStart {
            entries,
            git: self.git_service.clone(),
            query: parsed,
            session: session.to_owned(),
        }))
    }

    fn matcher<'model>(&'model self, parsed: &'model ParsedQuery) -> Matcher<'model> {
        if self.search.has_query() {
            if let Some(paths) = self.search.matching_paths() {
                return Matcher::Set(paths);
            }
        }

        Matcher::Query(MatchContext::new(parsed, Some(&self.git_service)))
    }

    pub fn background_start(&mut self) -> Command {
        let Some(session) = self.sessions.active_identifier() else {
            return Command::None;
        };

        if self.tree.nodes.is_empty() {
            return Command::None;
        }

        let start = BackgroundStart {
            filter: Arc::clone(&self.filter),
            nodes: self.tree.nodes.clone(),
            session: session.to_owned(),
        };

        self.tree.background_loading = true;

        Command::LoadBackground(Box::new(start))
    }

    pub fn checked_paths(&self) -> Vec<PathBuf> {
        let parsed = self.search.parsed();
        let matcher = self.matcher(&parsed);

        query::gather_checked_paths(&self.tree.nodes, &matcher, &parsed)
    }

    pub fn flush_session(&mut self) {
        if !self.session_dirty {
            return;
        }

        self.sessions
            .sync_from_tree_and_search(&self.tree, &self.search);

        self.session_dirty = false;
    }

    pub fn mark_session_dirty(&mut self) {
        self.session_dirty = true;
    }

    pub fn new(mut options: Options, paths: Vec<PathBuf>) -> Self {
        let filter = GlobPathFilter::from_options(&options).unwrap_or_else(|error| {
            eprintln!("Falling back to the default patterns: {error}");

            options.reset_patterns(PatternList::Exclude);
            options.reset_patterns(PatternList::Include);

            GlobPathFilter::from_options(&options).expect("the default patterns compile")
        });

        let shared = Arc::new(options);

        Self {
            filter: Arc::new(filter),
            git_service: GitService::default(),
            options: Arc::clone(&shared),
            options_original: shared,
            presets: PresetModel::load_from_disk(),
            search: SearchModel::default(),
            session_dirty: false,
            sessions: SessionsModel::default(),
            tree: TreeModel::new(paths),
        }
    }

    pub fn options_changed(&self) -> bool {
        *self.options != *self.options_original
    }

    pub fn refresh_git_status(&mut self) {
        if let Some(node) = self.tree.nodes.first() {
            self.git_service.refresh(&node.path);
        }
    }

    pub fn refresh_search_matches(&mut self) -> Command {
        if !self.search.has_query() {
            self.search.set_matching_paths(None);

            return Command::None;
        }

        self.search.ensure_parsed();

        let parsed = self.search.parsed().into_owned();

        if parsed.is_empty() {
            self.search.set_matching_paths(None);

            return Command::None;
        }

        if parsed.is_expensive() {
            return self.filter_start(parsed);
        }

        let matches = query::matching_paths(&self.tree.nodes, &parsed, Some(&self.git_service), 0);

        self.search.set_matching_paths(Some(matches));

        debug_assert!(self.search.matching_paths().is_some());

        Command::None
    }

    pub fn save_original_options(&mut self) {
        self.options_original = Arc::clone(&self.options);
    }

    pub fn selected_tree(&self) -> Vec<FileNode> {
        let parsed = self.search.parsed();
        let matcher = self.matcher(&parsed);

        query::filter_selected(&self.tree.nodes, &matcher, &parsed)
    }

    pub fn update_options(&mut self, options: Options) -> SwarmResult<()> {
        let filter = GlobPathFilter::from_options(&options)?;

        self.filter = Arc::new(filter);
        self.options = Arc::new(options);

        Ok(())
    }
}

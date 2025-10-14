use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::app::state::OptionsTab;
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::filesystem::index::IndexStatistics;
use crate::ui::themes::Theme;

#[derive(Debug, Clone)]
pub enum Msg {
    Session(Session),
    Tree(Tree),
    Search(Search),
    Index(Index),
    Copy(Copy),
    TreeGen(TreeGen),
    Options(Options_),
    Filter(Filter),
    App(App),
}

#[derive(Debug, Clone)]
pub enum Session {
    Created(String),
    Selected(String),
    IndexDataLoaded { statistics: Option<IndexStatistics>, extensions: HashSet<String> },
    Deleted(String),
    NameEdited(String),
    Renamed { id: String, name: String },
    EditStarted(String),
    EditCancelled,
}

#[derive(Debug, Clone)]
pub enum Tree {
    RefreshRequested,
    NodeToggled { path: Vec<usize>, checked: bool, propagate: bool },
    NodeExpanded { path: Vec<usize> },
    Loaded(Vec<FileNode>),
    LoadProgress { current: String, processed: usize, total: usize },
    LoadFailed(String),
}

#[derive(Debug, Clone)]
pub enum Search {
    QueryChanged(String),
    Activated,
    Cleared,
}

#[derive(Debug, Clone)]
pub enum Index {
    StartRequested,
    StopRequested,
    PauseRequested,
    ResumeRequested,
    Progress(IndexStatistics),
    Completed { count: usize, extensions: Vec<String> },
    Failed(String),
    SearchQueryChanged(String),
    ExtensionSelected(Option<String>),
}

#[derive(Debug, Clone)]
pub enum Copy {
    Requested,
    Started,
    Completed(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum TreeGen {
    Requested,
    Started,
    Generated(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum Options_ {
    Opened,
    Closed,
    TabChanged(OptionsTab),
    ThemeChanged(Theme),
    UseIconChanged(bool),
    AutoIndexChanged(bool),
    DeleteSessionsChanged(bool),
    OutputFormatChanged(crate::model::output::OutputFormat),
}

#[derive(Debug, Clone)]
pub enum Filter {
    IncludeAdded(String),
    IncludeRemoved(usize),
    IncludesCleared,
    ExcludeAdded(String),
    ExcludeRemoved(usize),
    ExcludesReset,
}

#[derive(Debug, Clone)]
pub enum App {
    Initialized,
    FileDialogOpened,
    PathSelected(PathBuf),
    AboutOpened,
    AboutClosed,
    Tick,
}

#[derive(Debug)]
pub enum Cmd {
    LoadSession { path: PathBuf, options: Arc<Options> },
    RefreshTree { nodes: Vec<FileNode>, options: Arc<Options> },
    StartIndexing { paths: Vec<PathBuf>, options: Arc<Options> },
    StopIndexing,
    PauseIndexing,
    ResumeIndexing,
    SwitchIndexSession(String),
    LoadSessionIndexData(String),
    GatherFiles { paths: Vec<String>, options: Arc<Options> },
    GenerateTree { nodes: Vec<FileNode>, options: Arc<Options> },
    SaveSessions,
    DeleteSessionData(String),
    Batch(Vec<Cmd>),
    None,
}

pub struct CmdBuilder {
    commands: Vec<Cmd>,
}

impl CmdBuilder {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn add(mut self, cmd: Cmd) -> Self {
        if !matches!(cmd, Cmd::None) {
            self.commands.push(cmd);
        }
        self
    }

    pub fn add_if(self, condition: bool, cmd: Cmd) -> Self {
        if condition {
            self.add(cmd)
        } else {
            self
        }
    }

    pub fn build(self) -> Cmd {
        match self.commands.len() {
            0 => Cmd::None,
            1 => self.commands.into_iter().next().unwrap(),
            _ => Cmd::Batch(self.commands),
        }
    }
}

impl Default for CmdBuilder {
    fn default() -> Self {
        Self::new()
    }
}

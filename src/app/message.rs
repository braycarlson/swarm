use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::app::state::search::ParsedQuery;
use crate::app::state::ui::GenerateMode;
use crate::app::state::OptionsTab;
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::model::output::OutputFormat;
use crate::services::filesystem::git::GitService;
use crate::ui::themes::Theme;

#[derive(Debug)]
pub enum Msg {
    Session(Session),
    Tree(Tree),
    Search(Search),
    Copy(Copy),
    Render(Render),
    Skeleton(Skeleton),
    Options(Options_),
    Filter(Filter),
    App(App),
}

#[derive(Debug, Clone)]
pub enum Session {
    Created(String),
    Selected(String),
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
    PropagateStarted,
    PropagateCompleted(Vec<FileNode>),
    PropagateFailed(String),
    BackgroundLoadProgress { loaded: usize, total: usize },
    BackgroundLoadCompleted(Vec<FileNode>),
}

#[derive(Debug, Clone)]
pub enum Search {
    QueryChanged(String),
    Activated,
    Cleared,
    DebounceTick,
    FilterStarted,
    FilterProgress(usize, usize),
    FilterComplete(HashSet<PathBuf>),
    FilterCancelled,
}

#[derive(Debug, Clone)]
pub enum Copy {
    Requested,
    Started,
    Completed(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum Render {
    Requested,
    Started,
    Generated(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum Skeleton {
    ModeChanged(GenerateMode),
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
    UiScaleChanged(f32),
    UiScaleReset,
    UseIconChanged(bool),
    DeleteSessionsChanged(bool),
    SingleInstanceChanged(bool),
    OutputFormatChanged(OutputFormat),
}

#[derive(Debug, Clone)]
pub enum Filter {
    IncludeAdded(String),
    IncludeRemoved(usize),
    IncludesCleared,
    IncludeFilterChanged(String),
    ExcludeAdded(String),
    ExcludeRemoved(usize),
    ExcludesReset,
    ExcludeFilterChanged(String),
}

#[derive(Debug, Clone)]
pub enum App {
    Initialized,
    FileDialogOpened,
    PathSelected(PathBuf),
    PathsReceivedFromIpc(Vec<PathBuf>),
    AboutOpened,
    AboutClosed,
    Tick,
    OpenInExplorer
}

pub enum Cmd {
    LoadSession { path: PathBuf, options: Arc<Options> },
    RefreshTree { nodes: Vec<FileNode>, options: Arc<Options> },
    GatherFiles { paths: Vec<String>, options: Arc<Options>, git: GitService, query: ParsedQuery },
    RenderTree { nodes: Vec<FileNode>, options: Arc<Options> },
    GenerateSkeleton { paths: Vec<String>, options: Arc<Options> },
    SaveSessions,
    DeleteSessionData(String),
    PropagateCheckedWithLoad {
        nodes: Vec<FileNode>,
        path: Vec<u32>,
        checked: bool,
        options: Arc<Options>,
    },
    StartExpensiveFilter {
        nodes: Vec<FileNode>,
        query: ParsedQuery,
        git: GitService,
    },
    CancelFilter,
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

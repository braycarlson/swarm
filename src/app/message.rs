use std::path::PathBuf;
use std::sync::Arc;

use rustc_hash::FxHashSet;

use crate::app::state::search::ParsedQuery;
use crate::app::state::ui::GenerateMode;
use crate::app::state::{OptionsTab, SearchModel};
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::model::output::OutputFormat;
use crate::services::filesystem::git::GitService;
use crate::services::worker::filter::FilterEntry;
use crate::ui::themes::Theme;

#[derive(Debug)]
pub enum Message {
    Session(Session),
    Tree(Tree),
    Search(Search),
    Copy(Copy),
    Render(Render),
    Skeleton(Skeleton),
    Options(Options_),
    Filter(Filter),
    Preset(Preset_),
    App(App),
}

#[derive(Debug, Clone)]
pub enum Session {
    Created(String),
    Selected(String),
    Deleted(String),
    NameEdited(String),
    Renamed { identifier: String, name: String },
    EditStarted(String),
    EditCancelled,
}

#[derive(Debug, Clone)]
pub enum Tree {
    RefreshRequested,
    NodeToggled { path: Vec<usize>, checked: bool, propagate: bool },
    NodeExpanded { path: Vec<usize> },
    SelectAll,
    DeselectAll,
    Loaded(Vec<FileNode>),
    LoadProgress { current: String, processed: usize, total: usize },
    LoadFailed(String),
    PropagateStarted,
    PropagateCompleted(Vec<FileNode>),
    PropagateFailed(String),
    BackgroundLoadProgress { loaded: usize, total: usize },
    BackgroundLoadCompleted(Vec<FileNode>),
    BulkSelectToggled,
    BulkSelectTextChanged(String),
    BulkSelectApplied,
}

#[derive(Debug, Clone)]
pub enum Search {
    QueryChanged(String),
    Activated,
    Cleared,
    DebounceTick,
    FilterStarted,
    FilterProgress(usize, usize),
    FilterComplete(FxHashSet<PathBuf>),
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
    UiScaleApplied,
    UiScaleChanged(f32),
    UiScaleReset,
    UseIconChanged(bool),
    ShowHiddenChanged(bool),
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
pub enum Preset_ {
    SaveDialogOpened,
    SaveDialogClosed,
    LoadDialogOpened,
    LoadDialogClosed,
    NameChanged(String),
    IncludeSelectionChanged(bool),
    IncludeSearchChanged(bool),
    GenericChanged(bool),
    Saved(String),
    Loaded(String),
    Deleted(String),
}

#[derive(Debug, Clone)]
pub enum App {
    Initialized,
    RestoreLastSession,
    FileDialogOpened,
    PathSelected(PathBuf),
    PathsReceivedFromIpc(Vec<PathBuf>),
    AboutOpened,
    AboutClosed,
    Tick,
    OpenInExplorer,
}

pub enum Command {
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
        search: SearchModel,
        git: GitService,
    },
    StartExpensiveFilter {
        entries: Vec<FilterEntry>,
        query: ParsedQuery,
        git: GitService,
    },
    CancelFilter,
    Batch(Vec<Command>),
    None,
}

pub struct CommandBuilder {
    commands: Vec<Command>,
}

impl CommandBuilder {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn add(mut self, command: Command) -> Self {
        if !matches!(command, Command::None) {
            self.commands.push(command);
        }
        self
    }

    pub fn build(self) -> Command {
        match self.commands.len() {
            0 => Command::None,
            1 => self.commands.into_iter().next().unwrap(),
            _ => Command::Batch(self.commands),
        }
    }
}

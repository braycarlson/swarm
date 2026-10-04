use alloc::sync::Arc;
use std::path::PathBuf;
use std::sync::mpsc::{SyncSender, TrySendError};

use crate::app::state::ui::{GenerateMode, OptionsTab};
use crate::model::node::FileNode;
use crate::model::options::PatternList;
use crate::model::output::OutputFormat;
use crate::model::query::ParsedQuery;
use crate::model::theme::Theme;
use crate::services::filesystem::filter::GlobPathFilter;
use crate::services::filesystem::git::GitService;
use crate::services::worker::filter::FilterEntry;
use crate::services::worker::{
    BackgroundOutcome,
    FilterComplete,
    GatherJob,
    PropagateJob,
    PropagateOutcome,
    Report,
    SessionLoadRequest,
    SessionLoadResult,
    SkeletonJob,
    TreeRefreshRequest,
    TreeRefreshResult,
};

pub enum App {
    AboutClosed,
    AboutOpened,
    FileDialogClosed,
    FileDialogOpened,
    Initialized,
    OpenInExplorer,
    PathSelected(PathBuf),
    PathsReceivedFromIpc(Vec<PathBuf>),
    RestoreLastSession,
}

pub struct BackgroundStart {
    pub filter: Arc<GlobPathFilter>,
    pub nodes: Vec<FileNode>,
    pub session: String,
}

pub enum Command {
    Batch(Vec<Self>),
    CancelFilter,
    DeleteSessionData(String),
    GatherFiles(Box<GatherJob>),
    GenerateSkeleton(Box<SkeletonJob>),
    LoadBackground(Box<BackgroundStart>),
    LoadSession(SessionLoadRequest),
    None,
    OpenFileDialog,
    PropagateChecked(Box<PropagateJob>),
    RefreshTree(Box<TreeRefreshRequest>),
    RenderTree {
        nodes: Vec<FileNode>,
        use_icons: bool,
    },
    StartFilter(Box<FilterStart>),
    StopBackground,
}

impl Command {
    pub fn batch(commands: Vec<Self>) -> Self {
        let mut kept: Vec<Self> = commands
            .into_iter()
            .filter(|command| !matches!(command, Self::None))
            .collect();

        debug_assert!(!kept.iter().any(|command| matches!(command, Self::Batch(_))));

        match kept.len() {
            0 => Self::None,
            1 => kept.pop().expect("a single command is present"),
            _ => Self::Batch(kept),
        }
    }
}

pub enum CopyMessage {
    Completed(Report),
    Failed(String),
    Requested,
}

pub enum Filter {
    Added { list: PatternList, pattern: String },
    Removed { index: usize, list: PatternList },
    Reset(PatternList),
}

pub struct FilterStart {
    pub entries: Vec<FilterEntry>,
    pub git: GitService,
    pub query: ParsedQuery,
    pub session: String,
}

pub enum Message {
    App(App),
    Copy(CopyMessage),
    Filter(Filter),
    Notice(Notice),
    Options(OptionsMessage),
    Preset(PresetMessage),
    Render(Render),
    Search(Search),
    Session(Session),
    Skeleton(Skeleton),
    Tree(Tree),
}

#[derive(Clone)]
pub struct MessageSender {
    sender: SyncSender<Message>,
}

impl MessageSender {
    pub fn new(sender: SyncSender<Message>) -> Self {
        Self { sender }
    }

    pub fn send(&self, message: Message) {
        match self.sender.try_send(message) {
            Ok(()) => {}
            Err(TrySendError::Disconnected(_)) => {
                eprintln!("The message queue is closed; a message was dropped");
            }
            Err(TrySendError::Full(_)) => {
                eprintln!("The message queue is full; a message was dropped");
            }
        }
    }
}

pub enum Notice {
    Error(String),
    Success(String),
}

#[derive(Clone, Copy)]
pub enum OptionsMessage {
    Closed,
    ContextMenuRegisterRequested,
    ContextMenuUnregisterRequested,
    DeleteSessionsChanged(bool),
    Opened,
    OutputFormatChanged(OutputFormat),
    ShowHiddenChanged(bool),
    SingleInstanceChanged(bool),
    TabChanged(OptionsTab),
    ThemeChanged(Theme),
    UiScaleApplied,
    UiScaleChanged(f32),
    UiScaleReset,
    UseIconChanged(bool),
}

pub enum PresetMessage {
    Deleted(String),
    GenericChanged(bool),
    IncludeSearchChanged(bool),
    IncludeSelectionChanged(bool),
    LoadDialogClosed,
    LoadDialogOpened,
    Loaded(String),
    SaveDialogClosed,
    SaveDialogOpened,
    Saved(String),
}

pub enum Render {
    Failed(String),
    Generated(String),
    Requested,
}

pub enum Search {
    Cleared,
    DebounceTick,
    FilterCompleted(Box<FilterComplete>),
    QueryEdited,
}

pub enum Session {
    Created(String),
    Deleted(String),
    EditCancelled,
    EditStarted(String),
    Renamed { identifier: String, name: String },
    Selected(String),
}

pub enum Skeleton {
    Completed(Report),
    Failed(String),
    ModeChanged(GenerateMode),
    Requested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToggleScope {
    Node,
    Subtree,
}

pub enum Tree {
    BackgroundLoadCompleted(Box<BackgroundOutcome>),
    BulkSelectApplied,
    BulkSelectToggled,
    DeselectAll,
    NodeExpanded {
        path: Vec<u32>,
    },
    NodeToggled {
        checked: bool,
        path: Vec<u32>,
        scope: ToggleScope,
    },
    PropagateCompleted(Box<PropagateOutcome>),
    RefreshRequested,
    Refreshed(Box<TreeRefreshResult>),
    SelectAll,
    SessionLoaded(Box<SessionLoadResult>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_batch_drops_empty_commands_and_unwraps_a_single_one() {
        assert!(matches!(Command::batch(vec![]), Command::None));
        assert!(matches!(Command::batch(vec![Command::None, Command::None]), Command::None));

        assert!(matches!(
            Command::batch(vec![Command::None, Command::CancelFilter]),
            Command::CancelFilter,
        ));

        assert!(matches!(
            Command::batch(vec![Command::CancelFilter, Command::StopBackground]),
            Command::Batch(_),
        ));
    }
}

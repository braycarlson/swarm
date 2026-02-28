use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;

use copypasta::{ClipboardContext, ClipboardProvider};

use crate::app::message::{Cmd, Copy, Msg, Render, Search, Skeleton};
use crate::app::state::{SessionData, SessionsModel};
use crate::constants::APP_NAME;
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::filesystem::gather::GatherService;
use crate::services::filesystem::git::GitService;
use crate::services::skeleton::SkeletonGenerator;
use crate::app::state::search::ParsedQuery;
use crate::services::tree::generator::TreeGenerator;
use crate::services::tree::traversal::Traversable;
use crate::services::worker::filter::{FilterResult, FilterWorker};
use crate::services::worker::session::{SessionLoadResult, SessionLoader};
use crate::services::worker::tree::{TreeLoadResult, TreeLoader};

const MAX_MESSAGES_PER_POLL: u32 = 20;

pub struct Runtime {
    filter_worker: FilterWorker,
    gather_service: GatherService,
    gather_tx: Option<Sender<()>>,
    msg_sender: Sender<Msg>,
    session_loader: SessionLoader,
    skeleton_gen_tx: Option<Sender<()>>,
    skeleton_generator: SkeletonGenerator,
    tree_gen_tx: Option<Sender<()>>,
    tree_loader: TreeLoader,
}

impl Runtime {
    pub fn new(msg_sender: Sender<Msg>) -> Self {
        Self {
            filter_worker: FilterWorker::new(),
            gather_service: GatherService::new(),
            gather_tx: None,
            msg_sender: msg_sender.clone(),
            session_loader: SessionLoader::new(),
            skeleton_gen_tx: None,
            skeleton_generator: SkeletonGenerator::new(),
            tree_gen_tx: None,
            tree_loader: TreeLoader::new(),
        }
    }

    pub fn load_sessions(&self) -> SessionsModel {
        if let Some(dir) = self.get_sessions_directory()
            && let Ok(entries) = std::fs::read_dir(dir) {
                let mut sessions = SessionsModel::new();

                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("json")
                        && let Ok(content) = std::fs::read_to_string(&path)
                            && let Ok(session) = serde_json::from_str::<SessionData>(&content) {
                                sessions.sessions.insert(session.id.clone(), session);
                            }
                }

                if let Some((first_id, _)) = sessions.sessions.iter().next() {
                    sessions.active_id = Some(first_id.clone());
                }

                return sessions;
            }

        SessionsModel::new()
    }

    pub fn execute(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::LoadSession { path, options } => {
                self.session_loader.start_loading(path, (*options).clone());
            }

            Cmd::RefreshTree { nodes, options } => {
                let mut refreshed = nodes.clone();
                for node in &mut refreshed {
                    let _ = node.refresh(&options);
                }

                self.tree_loader.start_load(refreshed, (*options).clone());
            }

            Cmd::GatherFiles { paths, options, git, query } => {
                self.execute_gather(paths, options, git, query);
            }

            Cmd::RenderTree { nodes, options } => {
                self.execute_tree_render(nodes, options);
            }

            Cmd::GenerateSkeleton { paths, options } => {
                self.execute_skeleton_generate(paths, options);
            }

            Cmd::SaveSessions => {
            }

            Cmd::DeleteSessionData(id) => {
                self.delete_session_file(&id);
            }

            Cmd::PropagateCheckedWithLoad { nodes, path, checked, options } => {
                self.execute_propagate_with_load(nodes, path, checked, options);
            }

            Cmd::StartExpensiveFilter { nodes, query, git } => {
                self.filter_worker.start_filter(nodes, query, git);
            }

            Cmd::CancelFilter => {
                self.filter_worker.cancel();
            }

            Cmd::Batch(cmds) => {
                for cmd in cmds {
                    self.execute(cmd);
                }
            }

            Cmd::None => {}
        }
    }

    pub fn poll(&mut self) -> Vec<Msg> {
        let mut all_messages = Vec::new();

        let session_messages = self.poll_session_loader();
        all_messages.extend(session_messages);

        let tree_messages = self.poll_tree_loader();
        all_messages.extend(tree_messages);

        let filter_messages = self.poll_filter_worker();
        all_messages.extend(filter_messages);

        all_messages
    }

    pub fn save_sessions(&self, sessions: &SessionsModel) {
        if let Some(dir) = self.get_sessions_directory() {
            let _ = std::fs::create_dir_all(&dir);

            for (id, session) in &sessions.sessions {
                let path = dir.join(format!("{}.json", id));

                if let Ok(json) = serde_json::to_string_pretty(session) {
                    let _ = std::fs::write(path, json);
                }
            }
        }
    }

    fn get_sessions_directory(&self) -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| {
            dir.join(APP_NAME.to_lowercase()).join("sessions")
        })
    }

    fn poll_session_loader(&self) -> Vec<Msg> {
        let mut messages = Vec::new();
        let mut count: u32 = 0;

        while let Some(result) = self.session_loader.check_results() {
            count += 1;

            if count > MAX_MESSAGES_PER_POLL {
                break;
            }

            let msg = match result {
                SessionLoadResult::Loaded(nodes) => Msg::Tree(crate::app::message::Tree::Loaded(nodes)),
                SessionLoadResult::Loading(message) => Msg::Tree(crate::app::message::Tree::LoadProgress {
                    current: message,
                    processed: 0,
                    total: 0,
                }),
                SessionLoadResult::Error(error) => Msg::Tree(crate::app::message::Tree::LoadFailed(error)),
            };

            messages.push(msg);
        }

        messages
    }

    fn poll_tree_loader(&mut self) -> Vec<Msg> {
        let mut messages = Vec::new();

        while let Some(result) = self.tree_loader.check_results() {
            let msg = match result {
                TreeLoadResult::LoadedTree(nodes) => Msg::Tree(crate::app::message::Tree::Loaded(nodes)),
                TreeLoadResult::ProcessingPath(path) => Msg::Tree(crate::app::message::Tree::LoadProgress {
                    current: path,
                    processed: 0,
                    total: 0,
                }),
                TreeLoadResult::CountUpdate(processed, total) => Msg::Tree(crate::app::message::Tree::LoadProgress {
                    current: String::new(),
                    processed,
                    total,
                }),
                TreeLoadResult::Error(error) => Msg::Tree(crate::app::message::Tree::LoadFailed(error)),
            };

            messages.push(msg);
        }

        messages
    }

    fn poll_filter_worker(&mut self) -> Vec<Msg> {
        let mut messages = Vec::new();

        while let Some(result) = self.filter_worker.check_results() {
            let msg = match result {
                FilterResult::Started => Msg::Search(Search::FilterStarted),
                FilterResult::Progress(current, total) => Msg::Search(Search::FilterProgress(current, total)),
                FilterResult::Complete(matching) => Msg::Search(Search::FilterComplete(matching)),
                FilterResult::Cancelled => Msg::Search(Search::FilterCancelled),
            };

            messages.push(msg);
        }

        messages
    }

    fn execute_gather(&mut self, paths: Vec<String>, options: Arc<Options>, git: GitService, query: ParsedQuery) {
        let gather = self.gather_service.clone();
        let sender = self.msg_sender.clone();

        sender.send(Msg::Copy(Copy::Started)).ok();

        let (tx, rx) = mpsc::channel();
        self.gather_tx = Some(tx);

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(500));

            if rx.try_recv().is_ok() {
                return;
            }

            match gather.gather_with_context(&paths, &options, Some(&git), Some(&query)) {
                Ok((output, stats)) => {
                    if let Ok(mut clipboard) = ClipboardContext::new() {
                        let _ = clipboard.set_contents(output.clone());
                    }

                    let message = format!("{} lines / {} tokens copied", stats.line_count, stats.token_count);
                    let _ = sender.send(Msg::Copy(Copy::Completed(message)));
                }
                Err(e) => {
                    let _ = sender.send(Msg::Copy(Copy::Failed(e.to_string())));
                }
            }
        });
    }

    fn execute_tree_render(&mut self, nodes: Vec<FileNode>, options: Arc<Options>) {
        let sender = self.msg_sender.clone();

        sender.send(Msg::Render(Render::Started)).ok();

        let (tx, rx) = mpsc::channel();
        self.tree_gen_tx = Some(tx);

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(500));

            if rx.try_recv().is_ok() {
                return;
            }

            let generator = TreeGenerator::new(&options);
            let output = generator.generate_tree(&nodes);

            if let Ok(mut clipboard) = ClipboardContext::new() {
                let _ = clipboard.set_contents(output.clone());
            }

            let _ = sender.send(Msg::Render(Render::Generated(output)));
        });
    }

    fn execute_skeleton_generate(&mut self, paths: Vec<String>, options: Arc<Options>) {
        let generator = self.skeleton_generator.clone();
        let sender = self.msg_sender.clone();

        sender.send(Msg::Skeleton(Skeleton::Started)).ok();

        let (tx, rx) = mpsc::channel();
        self.skeleton_gen_tx = Some(tx);

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(500));

            if rx.try_recv().is_ok() {
                return;
            }

            match generator.generate(&paths, &options) {
                Ok((output, stats)) => {
                    if let Ok(mut clipboard) = ClipboardContext::new() {
                        let _ = clipboard.set_contents(output.clone());
                    }

                    let message = format!(
                        "{} files / {} lines / {} tokens skeleton copied",
                        stats.file_count, stats.line_count, stats.token_count,
                    );

                    let _ = sender.send(Msg::Skeleton(Skeleton::Generated(message)));
                }
                Err(e) => {
                    let _ = sender.send(Msg::Skeleton(Skeleton::Failed(e.to_string())));
                }
            }
        });
    }

    fn execute_propagate_with_load(
        &mut self,
        mut nodes: Vec<FileNode>,
        path: Vec<u32>,
        checked: bool,
        options: Arc<Options>,
    ) {
        let sender = self.msg_sender.clone();

        sender.send(Msg::Tree(crate::app::message::Tree::PropagateStarted)).ok();

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(100));

            let mut current = &mut nodes;
            let mut target_node: Option<&mut FileNode> = None;

            for (i, &index) in path.iter().enumerate() {
                if i == path.len() - 1 {
                    if let Some(node) = current.get_mut(index as usize) {
                        target_node = Some(node);
                    }

                    break;
                } else if let Some(node) = current.get_mut(index as usize) {
                    current = &mut node.children;
                } else {
                    break;
                }
            }

            if let Some(node) = target_node {
                node.propagate_checked_with_load(checked, &options);
            }

            let _ = sender.send(Msg::Tree(crate::app::message::Tree::PropagateCompleted(nodes)));
        });
    }

    fn delete_session_file(&self, id: &str) {
        if let Some(dir) = self.get_sessions_directory() {
            let path = dir.join(format!("{}.json", id));

            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

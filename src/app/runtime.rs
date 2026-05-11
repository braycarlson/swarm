use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;

use copypasta::{ClipboardContext, ClipboardProvider};

use crate::app::message::{Command, Copy, Message, Render, Search, Skeleton};
use crate::app::state::{SessionData, SessionsModel, SearchModel};
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

const MESSAGES_PER_POLL_MAX: u32 = 20;

pub struct Runtime {
    filter_worker: FilterWorker,
    gather_sender: Option<Sender<()>>,
    gather_service: GatherService,
    message_sender: Sender<Message>,
    session_loader: SessionLoader,
    skeleton_generate_sender: Option<Sender<()>>,
    skeleton_generator: SkeletonGenerator,
    tree_generate_sender: Option<Sender<()>>,
    tree_loader: TreeLoader,
}

impl Runtime {
    pub fn new(message_sender: Sender<Message>) -> Self {
        Self {
            filter_worker: FilterWorker::new(),
            gather_sender: None,
            gather_service: GatherService::new(),
            message_sender: message_sender.clone(),
            session_loader: SessionLoader::new(),
            skeleton_generate_sender: None,
            skeleton_generator: SkeletonGenerator::new(),
            tree_generate_sender: None,
            tree_loader: TreeLoader::new(),
        }
    }

    pub fn load_sessions(&self) -> SessionsModel {
        let mut sessions = SessionsModel::new();

        if let Some(dir) = self.get_sessions_directory() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("json")
                        && let Ok(content) = std::fs::read_to_string(&path)
                            && let Ok(session) = serde_json::from_str::<SessionData>(&content) {
                                sessions.sessions.insert(session.id.clone(), session);
                            }
                }

                if !sessions.sessions.is_empty() {
                    let saved_active_id = self.load_active_session_id(&dir);

                    if let Some(ref id) = saved_active_id {
                        if sessions.sessions.contains_key(id) {
                            sessions.active_id = Some(id.clone());
                        }
                    }

                    if sessions.active_id.is_none() {
                        sessions.active_id = sessions.sessions.values()
                            .max_by_key(|s| s.last_modified)
                            .map(|s| s.id.clone());
                    }
                }
            }
        }

        if let Some(last) = self.load_last_session() {
            sessions.last_closed_session = Some(last);
        }

        sessions
    }

    pub fn execute(&mut self, command: Command) {
        match command {
            Command::LoadSession { path, options } => {
                self.session_loader.start_loading(path, (*options).clone());
            }

            Command::RefreshTree { mut nodes, options } => {
                for node in &mut nodes {
                    let _ = node.refresh(&options);
                }

                self.tree_loader.start_load(nodes, (*options).clone());
            }

            Command::GatherFiles { paths, options, git, query } => {
                self.execute_gather(paths, options, git, query);
            }

            Command::RenderTree { nodes, options } => {
                self.execute_tree_render(nodes, options);
            }

            Command::GenerateSkeleton { paths, options } => {
                self.execute_skeleton_generate(paths, options);
            }

            Command::SaveSessions => {
            }

            Command::DeleteSessionData(id) => {
                self.delete_session_file(&id);
            }

            Command::PropagateCheckedWithLoad { nodes, path, checked, options, search, git } => {
                self.execute_propagate_with_load(nodes, path, checked, options, search, git);
            }

            Command::StartExpensiveFilter { entries, query, git } => {
                self.filter_worker.start_filter(entries, query, git);
            }

            Command::CancelFilter => {
                self.filter_worker.cancel();
            }

            Command::Batch(commands) => {
                for command in commands {
                    self.execute(command);
                }
            }

            Command::None => {}
        }
    }

    pub fn poll(&mut self) -> Vec<Message> {
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

            if let Some(ref active_id) = sessions.active_id {
                let active_path = dir.join("active");
                let _ = std::fs::write(active_path, active_id);
            }
        }
    }

    pub fn save_last_session(&self, session: &SessionData) {
        if let Some(dir) = self.get_app_directory() {
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join("last_session.json");

            if let Ok(json) = serde_json::to_string_pretty(session) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    fn load_last_session(&self) -> Option<SessionData> {
        let dir = self.get_app_directory()?;
        let path = dir.join("last_session.json");
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn get_app_directory(&self) -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| dir.join(APP_NAME.to_lowercase()))
    }

    fn get_sessions_directory(&self) -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| {
            dir.join(APP_NAME.to_lowercase()).join("sessions")
        })
    }

    fn load_active_session_id(&self, sessions_dir: &PathBuf) -> Option<String> {
        let path = sessions_dir.join("active");
        std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    }

    fn poll_session_loader(&self) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut count: u32 = 0;

        while let Some(result) = self.session_loader.check_results() {
            count += 1;

            if count > MESSAGES_PER_POLL_MAX {
                break;
            }

            let message = match result {
                SessionLoadResult::Loaded(nodes) => Message::Tree(crate::app::message::Tree::Loaded(nodes)),
                SessionLoadResult::Loading(text) => Message::Tree(crate::app::message::Tree::LoadProgress {
                    current: text,
                    processed: 0,
                    total: 0,
                }),
                SessionLoadResult::Error(error) => Message::Tree(crate::app::message::Tree::LoadFailed(error)),
            };

            messages.push(message);
        }

        messages
    }

    fn poll_tree_loader(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();

        while let Some(result) = self.tree_loader.check_results() {
            let message = match result {
                TreeLoadResult::LoadedTree(nodes) => Message::Tree(crate::app::message::Tree::Loaded(nodes)),
                TreeLoadResult::ProcessingPath(path) => Message::Tree(crate::app::message::Tree::LoadProgress {
                    current: path,
                    processed: 0,
                    total: 0,
                }),
                TreeLoadResult::CountUpdate(processed, total) => Message::Tree(crate::app::message::Tree::LoadProgress {
                    current: String::new(),
                    processed,
                    total,
                }),
                TreeLoadResult::Error(error) => Message::Tree(crate::app::message::Tree::LoadFailed(error)),
            };

            messages.push(message);
        }

        messages
    }

    fn poll_filter_worker(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();

        while let Some(result) = self.filter_worker.check_results() {
            let message = match result {
                FilterResult::Started => Message::Search(Search::FilterStarted),
                FilterResult::Progress(current, total) => Message::Search(Search::FilterProgress(current, total)),
                FilterResult::Complete(matching) => Message::Search(Search::FilterComplete(matching)),
                FilterResult::Cancelled => Message::Search(Search::FilterCancelled),
            };

            messages.push(message);
        }

        messages
    }

    fn execute_gather(&mut self, paths: Vec<String>, options: Arc<Options>, git: GitService, query: ParsedQuery) {
        let gather = self.gather_service.clone();
        let sender = self.message_sender.clone();

        sender.send(Message::Copy(Copy::Started)).ok();

        let (transmitter, receiver) = mpsc::channel();
        self.gather_sender = Some(transmitter);

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(500));

            if receiver.try_recv().is_ok() {
                return;
            }

            match gather.gather_with_context(&paths, &options, Some(&git), Some(&query)) {
                Ok((output, stats)) => {
                    if let Ok(mut clipboard) = ClipboardContext::new() {
                        let _ = clipboard.set_contents(output.clone());
                    }

                    let text = format!("{} lines / {} tokens copied", stats.count_line, stats.count_token);
                    let _ = sender.send(Message::Copy(Copy::Completed(text)));
                }
                Err(e) => {
                    let _ = sender.send(Message::Copy(Copy::Failed(e.to_string())));
                }
            }
        });
    }

    fn execute_tree_render(&mut self, nodes: Vec<FileNode>, options: Arc<Options>) {
        let sender = self.message_sender.clone();

        sender.send(Message::Render(Render::Started)).ok();

        let (transmitter, receiver) = mpsc::channel();
        self.tree_generate_sender = Some(transmitter);

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(500));

            if receiver.try_recv().is_ok() {
                return;
            }

            let generator = TreeGenerator::new(&options);
            let output = generator.generate_tree(&nodes);

            if let Ok(mut clipboard) = ClipboardContext::new() {
                let _ = clipboard.set_contents(output.clone());
            }

            let _ = sender.send(Message::Render(Render::Generated(output)));
        });
    }

    fn execute_skeleton_generate(&mut self, paths: Vec<String>, options: Arc<Options>) {
        let generator = self.skeleton_generator.clone();
        let sender = self.message_sender.clone();

        sender.send(Message::Skeleton(Skeleton::Started)).ok();

        let (transmitter, receiver) = mpsc::channel();
        self.skeleton_generate_sender = Some(transmitter);

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(500));

            if receiver.try_recv().is_ok() {
                return;
            }

            match generator.generate(&paths, &options) {
                Ok((output, stats)) => {
                    if let Ok(mut clipboard) = ClipboardContext::new() {
                        let _ = clipboard.set_contents(output.clone());
                    }

                    let text = format!(
                        "{} files / {} lines / {} tokens skeleton copied",
                        stats.count_file, stats.count_line, stats.count_token,
                    );

                    let _ = sender.send(Message::Skeleton(Skeleton::Generated(text)));
                }
                Err(e) => {
                    let _ = sender.send(Message::Skeleton(Skeleton::Failed(e.to_string())));
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
        search: SearchModel,
        git: GitService,
    ) {
        let sender = self.message_sender.clone();

        sender.send(Message::Tree(crate::app::message::Tree::PropagateStarted)).ok();

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
                if search.has_query() {
                    node.propagate_checked_filtered(checked, &options, &search, Some(&git));
                } else {
                    node.propagate_checked_with_load(checked, &options);
                }
            }

            let _ = sender.send(Message::Tree(crate::app::message::Tree::PropagateCompleted(nodes)));
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

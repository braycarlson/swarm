use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;

use copypasta::{ClipboardContext, ClipboardProvider};

use crate::app::message::{Cmd, Copy, Index, Msg, Session, TreeGen};
use crate::app::state::{SessionData, SessionsModel};
use crate::constants::APP_NAME;
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::filesystem::gather::GatherService;
use crate::services::filesystem::index::{IndexResult, IndexService};
use crate::services::tree::generator::TreeGenerator;
use crate::services::tree::operations::TreeOperations;
use crate::services::worker::session::{SessionLoadResult, SessionLoader};
use crate::services::worker::tree::{TreeLoadResult, TreeLoader};

const MAX_MESSAGES_PER_POLL: u32 = 20;

pub struct Runtime {
    gather_service: GatherService,
    gather_tx: Option<Sender<()>>,
    index_service: IndexService,
    msg_sender: Sender<Msg>,
    session_loader: SessionLoader,
    tree_gen_tx: Option<Sender<()>>,
    tree_loader: TreeLoader,
}

impl Runtime {
    pub fn new(msg_sender: Sender<Msg>) -> Self {
        Self {
            gather_service: GatherService::new(),
            gather_tx: None,
            index_service: IndexService::new(),
            msg_sender: msg_sender.clone(),
            session_loader: SessionLoader::new(),
            tree_gen_tx: None,
            tree_loader: TreeLoader::new(),
        }
    }

    pub fn load_sessions(&self) -> SessionsModel {
        if let Some(dir) = self.get_sessions_directory() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                let mut sessions = SessionsModel::new();

                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(session) = serde_json::from_str::<SessionData>(&content) {
                                sessions.sessions.insert(session.id.clone(), session);
                            }
                        }
                    }
                }

                if let Some((first_id, _)) = sessions.sessions.iter().next() {
                    sessions.active_id = Some(first_id.clone());
                }

                return sessions;
            }
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

            Cmd::StartIndexing { paths, options } => {
                self.index_service.start_indexing(paths, (*options).clone());
            }

            Cmd::StopIndexing => {
                self.index_service.stop_indexing();
            }

            Cmd::PauseIndexing => {
                self.index_service.pause_indexing();
            }

            Cmd::ResumeIndexing => {
                self.index_service.resume_indexing();
            }

            Cmd::SwitchIndexSession(id) => {
                self.index_service.switch_session(id);
            }

            Cmd::LoadSessionIndexData(id) => {
                let statistics = self.index_service.get_session_statistics(&id);
                let extensions = self.index_service.get_session_extensions(&id);

                let _ = self.msg_sender.send(Msg::Session(Session::IndexDataLoaded {
                    statistics,
                    extensions,
                }));
            }

            Cmd::GatherFiles { paths, options } => {
                self.execute_gather(paths, options);
            }

            Cmd::GenerateTree { nodes, options } => {
                self.execute_tree_gen(nodes, options);
            }

            Cmd::SaveSessions => {
            }

            Cmd::DeleteSessionData(id) => {
                self.delete_session_file(&id);
            }

            Cmd::PropagateCheckedWithLoad { nodes, path, checked, options } => {
                self.execute_propagate_with_load(nodes, path, checked, options);
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

        let index_messages = self.poll_index_service();
        all_messages.extend(index_messages);

        all_messages
    }

    fn poll_session_loader(&mut self) -> Vec<Msg> {
        let mut messages = Vec::new();
        let mut count: u32 = 0;

        loop {
            if count >= MAX_MESSAGES_PER_POLL {
                break;
            }

            let result = self.session_loader.check_results();

            if result.is_none() {
                break;
            }

            count = count + 1;

            let result = result.unwrap();

            let msg = match result {
                SessionLoadResult::Loaded(nodes) => Some(Msg::Tree(crate::app::message::Tree::Loaded(nodes))),
                SessionLoadResult::Error(error) => Some(Msg::Tree(crate::app::message::Tree::LoadFailed(error))),
                SessionLoadResult::Loading(_) => None,
            };

            if msg.is_some() {
                messages.push(msg.unwrap());
            }
        }

        messages
    }

    fn poll_tree_loader(&mut self) -> Vec<Msg> {
        let mut messages = Vec::new();
        let mut count: u32 = 0;

        loop {
            if count >= MAX_MESSAGES_PER_POLL {
                break;
            }

            let result = self.tree_loader.check_results();

            if result.is_none() {
                break;
            }

            count = count + 1;

            let result = result.unwrap();

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

    fn poll_index_service(&mut self) -> Vec<Msg> {
        let mut messages = Vec::new();
        let mut count: u32 = 0;

        loop {
            if count >= MAX_MESSAGES_PER_POLL {
                break;
            }

            let result = self.index_service.check_results();

            if result.is_none() {
                break;
            }

            count = count + 1;

            let result = result.unwrap();

            let msg = match result {
                IndexResult::Progress(stats) => Msg::Index(Index::Progress(stats)),
                IndexResult::Completed(_count) => {
                    let extensions: Vec<_> = self.index_service
                        .get_all_extensions()
                        .into_iter()
                        .collect();

                    Msg::Index(Index::Completed {
                        count: self.index_service.get_indexed_count(),
                        extensions,
                    })
                }
                IndexResult::Error(error) => Msg::Index(Index::Failed(error)),
            };

            messages.push(msg);
        }

        messages
    }

    fn execute_gather(&mut self, paths: Vec<String>, options: Arc<Options>) {
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

            match gather.gather(&paths, &options) {
                Ok((output, line_count)) => {
                    if let Ok(mut clipboard) = ClipboardContext::new() {
                        let _ = clipboard.set_contents(output.clone());
                    }

                    let message = format!("{} lines copied", line_count);
                    let _ = sender.send(Msg::Copy(Copy::Completed(message)));
                }
                Err(e) => {
                    let _ = sender.send(Msg::Copy(Copy::Failed(e.to_string())));
                }
            }
        });
    }

    fn execute_tree_gen(&mut self, nodes: Vec<FileNode>, options: Arc<Options>) {
        let sender = self.msg_sender.clone();

        sender.send(Msg::TreeGen(TreeGen::Started)).ok();

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

            let _ = sender.send(Msg::TreeGen(TreeGen::Generated(output)));
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

    fn get_sessions_directory(&self) -> Option<PathBuf> {
        dirs::data_local_dir()
            .map(|d| d.join(APP_NAME.to_lowercase()).join("sessions"))
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
}

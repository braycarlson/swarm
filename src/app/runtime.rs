use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;

use copypasta::{ClipboardContext, ClipboardProvider};

use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::filesystem::gather::GatherService;
use crate::services::filesystem::index::{IndexResult, IndexService};
use crate::services::tree::generator::TreeGenerator;
use crate::services::tree::operations::TreeOperations;
use crate::services::worker::session::{SessionLoadResult, SessionLoader};
use crate::services::worker::tree::{TreeLoadResult, TreeLoader};

use super::message::{Cmd, Copy, Index, Msg, Session, Tree, TreeGen};
use super::state::SessionsModel;

pub struct Runtime {
    index_service: IndexService,
    session_loader: SessionLoader,
    tree_loader: TreeLoader,
    gather_service: GatherService,
    msg_sender: Sender<Msg>,
    gather_tx: Option<Sender<()>>,
    tree_gen_tx: Option<Sender<()>>,
}

impl Runtime {
    pub fn new(msg_sender: Sender<Msg>) -> Self {
        Self {
            index_service: IndexService::new(),
            session_loader: SessionLoader::new(),
            tree_loader: TreeLoader::new(),
            gather_service: GatherService::new(),
            msg_sender,
            gather_tx: None,
            tree_gen_tx: None,
        }
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

            Cmd::Batch(cmds) => {
                for cmd in cmds {
                    self.execute(cmd);
                }
            }

            Cmd::None => {}
        }
    }

    pub fn poll(&mut self) -> Vec<Msg> {
        [
            self.poll_session_loader(),
            self.poll_tree_loader(),
            self.poll_index_service(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn poll_session_loader(&mut self) -> Vec<Msg> {
        std::iter::from_fn(|| self.session_loader.check_results())
            .filter_map(|result| match result {
                SessionLoadResult::Loaded(nodes) => Some(Msg::Tree(Tree::Loaded(nodes))),
                SessionLoadResult::Error(error) => Some(Msg::Tree(Tree::LoadFailed(error))),
                SessionLoadResult::Loading(_) => None,
            })
            .collect()
    }

    fn poll_tree_loader(&mut self) -> Vec<Msg> {
        std::iter::from_fn(|| self.tree_loader.check_results())
            .map(|result| match result {
                TreeLoadResult::LoadedTree(nodes) => Msg::Tree(Tree::Loaded(nodes)),
                TreeLoadResult::ProcessingPath(path) => Msg::Tree(Tree::LoadProgress {
                    current: path,
                    processed: 0,
                    total: 0,
                }),
                TreeLoadResult::CountUpdate(processed, total) => Msg::Tree(Tree::LoadProgress {
                    current: String::new(),
                    processed,
                    total,
                }),
                TreeLoadResult::Error(error) => Msg::Tree(Tree::LoadFailed(error)),
            })
            .collect()
    }

    fn poll_index_service(&mut self) -> Vec<Msg> {
        std::iter::from_fn(|| self.index_service.check_results())
            .map(|result| match result {
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
            })
            .collect()
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
                Ok(output) => {
                    if let Ok(mut clipboard) = ClipboardContext::new() {
                        let _ = clipboard.set_contents(output.clone());
                    }

                    let _ = sender.send(Msg::Copy(Copy::Completed(output)));
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

    fn delete_session_file(&self, id: &str) {
        if let Some(dir) = self.get_sessions_directory() {
            let path = dir.join(format!("{}.json", id));

            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    fn get_sessions_directory(&self) -> Option<std::path::PathBuf> {
        dirs::data_local_dir()
            .map(|d| d.join(crate::constants::APP_NAME.to_lowercase()).join("sessions"))
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

            if let Some(active_id) = &sessions.active_id {
                let active_path = dir.join("active_session.txt");
                let _ = std::fs::write(active_path, active_id);
            }
        }
    }

    pub fn load_sessions(&self) -> SessionsModel {
        let mut sessions = SessionsModel::new();

        if let Some(dir) = self.get_sessions_directory() {
            if !dir.exists() {
                return sessions;
            }

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(session) = serde_json::from_str(&content) {
                                let id = path.file_stem()
                                    .and_then(|s| s.to_str())
                                    .map(String::from)
                                    .unwrap_or_default();

                                sessions.sessions.insert(id, session);
                            }
                        }
                    }
                }
            }

            let active = dir.join("active_session.txt");

            if active.exists() {
                if let Ok(id) = std::fs::read_to_string(&active) {
                    if sessions.sessions.contains_key(&id) {
                        sessions.active_id = Some(id);
                    }
                }
            }

            if sessions.active_id.is_none() && !sessions.sessions.is_empty() {
                sessions.active_id = sessions.sessions.keys().next().cloned();
            }
        }

        sessions
    }
}

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::model::node::FileNode;
use crate::model::options::Options;

use super::command::SessionLoadCommand;
use super::result::SessionLoadResult;

pub struct SessionLoader {
    loading: Arc<AtomicBool>,
    receiver: Arc<Mutex<Receiver<SessionLoadResult>>>,
    sender: Sender<SessionLoadCommand>,
}

impl SessionLoader {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        let loading = Arc::new(AtomicBool::new(false));

        let loading_clone = Arc::clone(&loading);

        thread::spawn(move || {
            Self::worker_thread(command_receiver, result_sender, loading_clone);
        });

        Self {
            loading,
            receiver: Arc::new(Mutex::new(result_receiver)),
            sender: command_sender,
        }
    }

    pub fn check_results(&mut self) -> Option<SessionLoadResult> {
        let receiver_lock = self.receiver.lock();

        if receiver_lock.is_err() {
            return None;
        }

        let receiver = receiver_lock.unwrap();
        let result = receiver.try_recv();

        if result.is_err() {
            return None;
        }

        Some(result.unwrap())
    }

    pub fn start_loading(&self, path: PathBuf, options: Options) -> bool {
        if self.loading.load(Ordering::Relaxed) {
            return false;
        }

        let command = SessionLoadCommand::Load(path, options);
        let send_result = self.sender.send(command);

        if send_result.is_ok() {
            self.loading.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    fn load_path(path: PathBuf, _options: &Options) -> Result<Vec<FileNode>, String> {
        let actual_path = if path.is_file() {
            path.parent()
                .map(|parent| parent.to_path_buf())
                .unwrap_or(path)
        } else {
            path
        };

        let node = FileNode::new(actual_path);

        Ok(vec![node])
    }

    fn worker_thread(
        command_receiver: Receiver<SessionLoadCommand>,
        result_sender: Sender<SessionLoadResult>,
        loading: Arc<AtomicBool>,
    ) {
        loop {
            let command = command_receiver.recv();

            if command.is_err() {
                break;
            }

            let command = command.unwrap();

            match command {
                SessionLoadCommand::Load(path, options) => {
                    loading.store(true, Ordering::Relaxed);

                    let loading_msg = format!("Loading {}", path.display());
                    let _ = result_sender.send(SessionLoadResult::Loading(loading_msg));

                    match Self::load_path(path.clone(), &options) {
                        Ok(nodes) => {
                            loading.store(false, Ordering::Relaxed);
                            let _ = result_sender.send(SessionLoadResult::Loaded(nodes));
                        }
                        Err(error) => {
                            loading.store(false, Ordering::Relaxed);
                            let error_msg = format!("Failed to load path {}: {}", path.display(), error);
                            let _ = result_sender.send(SessionLoadResult::Error(error_msg));
                        }
                    }
                }
                SessionLoadCommand::Cancel => {
                    loading.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    }
}

impl Clone for SessionLoader {
    fn clone(&self) -> Self {
        Self {
            loading: Arc::clone(&self.loading),
            receiver: Arc::clone(&self.receiver),
            sender: self.sender.clone(),
        }
    }
}

impl Drop for SessionLoader {
    fn drop(&mut self) {
        let _ = self.sender.send(SessionLoadCommand::Cancel);
    }
}

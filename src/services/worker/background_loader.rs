use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::tree::operations::TreeOperations;

pub enum BackgroundLoadCommand {
    Start(Vec<FileNode>, Options),
    Stop,
}

pub enum BackgroundLoadResult {
    NodesUpdated(Vec<FileNode>),
    Progress(usize, usize),
}

pub struct BackgroundLoader {
    is_running: Arc<AtomicBool>,
    receiver: Arc<Mutex<Receiver<BackgroundLoadResult>>>,
    sender: Sender<BackgroundLoadCommand>,
}

impl BackgroundLoader {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        let is_running = Arc::new(AtomicBool::new(false));

        let is_running_clone = Arc::clone(&is_running);

        thread::spawn(move || {
            Self::worker_thread(command_receiver, result_sender, is_running_clone);
        });

        Self {
            is_running,
            receiver: Arc::new(Mutex::new(result_receiver)),
            sender: command_sender,
        }
    }

    pub fn check_results(&self) -> Option<BackgroundLoadResult> {
        let receiver_lock = self.receiver.lock();

        if receiver_lock.is_err() {
            return None;
        }

        let receiver = receiver_lock.unwrap();
        receiver.try_recv().ok()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn start_loading(&self, nodes: Vec<FileNode>, options: Options) -> bool {
        if self.is_running() {
            return false;
        }

        let command = BackgroundLoadCommand::Start(nodes, options);
        self.sender.send(command).is_ok()
    }

    pub fn stop(&self) {
        let _ = self.sender.send(BackgroundLoadCommand::Stop);
    }

    fn worker_thread(
        command_receiver: Receiver<BackgroundLoadCommand>,
        result_sender: Sender<BackgroundLoadResult>,
        is_running: Arc<AtomicBool>,
    ) {
        loop {
            let command = command_receiver.recv();

            if command.is_err() {
                break;
            }

            match command.unwrap() {
                BackgroundLoadCommand::Start(mut nodes, options) => {
                    is_running.store(true, Ordering::Relaxed);

                    let total = Self::count_directories(&nodes);
                    let mut loaded = 0;

                    Self::load_recursively(&mut nodes, &options, &result_sender, &mut loaded, total);

                    let _ = result_sender.send(BackgroundLoadResult::NodesUpdated(nodes));
                    is_running.store(false, Ordering::Relaxed);
                }
                BackgroundLoadCommand::Stop => {
                    is_running.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    }

    fn count_directories(nodes: &[FileNode]) -> usize {
        let mut count = 0;

        for node in nodes {
            if node.is_directory() {
                count += 1;
                count += Self::count_directories(&node.children);
            }
        }

        count
    }

    fn load_recursively(
        nodes: &mut [FileNode],
        options: &Options,
        result_sender: &Sender<BackgroundLoadResult>,
        loaded: &mut usize,
        total: usize,
    ) {
        for node in nodes {
            if node.is_directory() && !node.loaded {
                if node.load_children(options).is_ok() {
                    *loaded += 1;

                    if *loaded % 100 == 0 {
                        let _ = result_sender.send(BackgroundLoadResult::Progress(*loaded, total));
                    }
                }

                Self::load_recursively(&mut node.children, options, result_sender, loaded, total);
            }
        }
    }
}

impl Clone for BackgroundLoader {
    fn clone(&self) -> Self {
        Self {
            is_running: Arc::clone(&self.is_running),
            receiver: Arc::clone(&self.receiver),
            sender: self.sender.clone(),
        }
    }
}

impl Drop for BackgroundLoader {
    fn drop(&mut self) {
        self.stop();
    }
}

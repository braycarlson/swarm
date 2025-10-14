use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::tree::operations::TreeOperations;

use super::command::TreeLoadCommand;
use super::result::TreeLoadResult;
use super::status::TreeLoadStatus;

pub struct TreeLoader {
    receiver: Receiver<TreeLoadResult>,
    sender: Sender<TreeLoadCommand>,
    status: TreeLoadStatus,
}

impl TreeLoader {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();

        thread::spawn(move || {
            Self::worker_thread(command_receiver, result_sender);
        });

        Self {
            receiver: result_receiver,
            sender: command_sender,
            status: TreeLoadStatus::NotStarted,
        }
    }

    pub fn check_results(&mut self) -> Option<TreeLoadResult> {
        match self.receiver.try_recv() {
            Ok(result) => {
                if let TreeLoadResult::LoadedTree(_) = &result {
                    self.status = TreeLoadStatus::Complete;
                }
                Some(result)
            }
            Err(_) => None,
        }
    }

    pub fn reset_status(&mut self) {
        self.status = TreeLoadStatus::NotStarted;
    }

    pub fn start_load(&mut self, nodes: Vec<FileNode>, options: Options) -> bool {
        if self.status == TreeLoadStatus::InProgress {
            return false;
        }

        if self.sender.send(TreeLoadCommand::Load(nodes, options)).is_err() {
            return false;
        }

        self.status = TreeLoadStatus::InProgress;
        true
    }

    pub fn status(&self) -> TreeLoadStatus {
        self.status
    }

    fn worker_thread(command_receiver: Receiver<TreeLoadCommand>, result_sender: Sender<TreeLoadResult>) {
        while let Ok(command) = command_receiver.recv() {
            match command {
                TreeLoadCommand::Load(nodes, options) => {
                    Self::process_tree_load(nodes, options, result_sender.clone());
                }
                TreeLoadCommand::Stop => break,
            }
        }
    }

    fn process_tree_load(mut nodes: Vec<FileNode>, options: Options, result_sender: Sender<TreeLoadResult>) {
        let mut visible = Vec::new();
        let mut total_count = Self::calculate_initial_count(&nodes);

        let _ = result_sender.send(TreeLoadResult::CountUpdate(0, total_count));
        let mut processed_count = 0;

        for node in nodes.iter_mut() {
            if let Err(error) = Self::process_single_node(
                node,
                &options,
                &result_sender,
                &mut processed_count,
                &mut total_count,
                &mut visible,
            ) {
                let _ = result_sender.send(TreeLoadResult::Error(error));
            }
        }

        if visible.is_empty() && !nodes.is_empty() {
            visible.push(nodes[0].clone());
        }

        thread::sleep(Duration::from_millis(300));
        let _ = result_sender.send(TreeLoadResult::LoadedTree(visible));
    }

    fn calculate_initial_count(nodes: &[FileNode]) -> usize {
        nodes.iter().fold(0, |acc, _node| acc + 1)
    }

    fn process_single_node(
        node: &mut FileNode,
        options: &Options,
        result_sender: &Sender<TreeLoadResult>,
        processed_count: &mut usize,
        total_count: &mut usize,
        visible: &mut Vec<FileNode>,
    ) -> Result<(), String> {
        let path_string = node.path.to_string_lossy().to_string();
        let _ = result_sender.send(TreeLoadResult::ProcessingPath(path_string));

        *processed_count += 1;
        let _ = result_sender.send(TreeLoadResult::CountUpdate(*processed_count, *total_count));

        thread::sleep(Duration::from_millis(10));
        node.loaded = false;

        match node.refresh(options) {
            Ok(true) => {
                if node.load_all_children(options).is_ok() {
                    let child_count = Self::count_loaded_nodes(node);
                    *total_count += child_count;
                    let _ = result_sender.send(TreeLoadResult::CountUpdate(*processed_count, *total_count));

                    Self::process_loaded_nodes(node, result_sender, processed_count, *total_count);
                    visible.push(node.clone());
                    Ok(())
                } else {
                    Err(format!("Error loading node {}", node.path.display()))
                }
            }
            Err(error) => Err(format!("Error refreshing node {}: {}", node.path.display(), error)),
            Ok(false) => Ok(()),
        }
    }

    fn count_loaded_nodes(node: &FileNode) -> usize {
        node.children.iter().fold(0, |acc, child| {
            let child_count = if child.is_directory() && child.loaded {
                Self::count_loaded_nodes(child)
            } else {
                0
            };
            acc + 1 + child_count
        })
    }

    fn process_loaded_nodes(
        node: &FileNode,
        result_sender: &Sender<TreeLoadResult>,
        processed_count: &mut usize,
        total_count: usize,
    ) {
        for child in &node.children {
            let path_string = child.path.to_string_lossy().to_string();
            let _ = result_sender.send(TreeLoadResult::ProcessingPath(path_string));

            *processed_count += 1;
            let _ = result_sender.send(TreeLoadResult::CountUpdate(*processed_count, total_count));

            thread::sleep(Duration::from_millis(10));

            if child.is_directory() && child.loaded {
                Self::process_loaded_nodes(child, result_sender, processed_count, total_count);
            }
        }
    }
}

impl Drop for TreeLoader {
    fn drop(&mut self) {
        let _ = self.sender.send(TreeLoadCommand::Stop);
    }
}

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
        let result = self.receiver.try_recv();

        if result.is_err() {
            return None;
        }

        let result = result.unwrap();

        let is_loaded_tree = matches!(result, TreeLoadResult::LoadedTree(_));
        if is_loaded_tree {
            self.status = TreeLoadStatus::Complete;
        }

        Some(result)
    }

    pub fn reset_status(&mut self) {
        self.status = TreeLoadStatus::NotStarted;
    }

    pub fn start_load(&mut self, nodes: Vec<FileNode>, options: Options) -> bool {
        if self.status == TreeLoadStatus::InProgress {
            return false;
        }

        let send_result = self.sender.send(TreeLoadCommand::Load(nodes, options));
        if send_result.is_err() {
            return false;
        }

        self.status = TreeLoadStatus::InProgress;
        true
    }

    pub fn status(&self) -> TreeLoadStatus {
        self.status
    }

    fn worker_thread(command_receiver: Receiver<TreeLoadCommand>, result_sender: Sender<TreeLoadResult>) {
        loop {
            let command = command_receiver.recv();

            if command.is_err() {
                break;
            }

            let command = command.unwrap();

            match command {
                TreeLoadCommand::Load(nodes, options) => {
                    Self::process_tree_load(nodes, options, result_sender.clone());
                }
                TreeLoadCommand::Stop => {
                    break;
                }
            }
        }
    }

    fn process_tree_load(mut nodes: Vec<FileNode>, options: Options, result_sender: Sender<TreeLoadResult>) {
        let mut visible = Vec::new();
        let mut total_count = Self::calculate_initial_count(&nodes);

        let _ = result_sender.send(TreeLoadResult::CountUpdate(0, total_count));
        let mut processed_count: u32 = 0;

        let mut node_index: u32 = 0;

        for node in nodes.iter_mut() {
            let process_result = Self::process_single_node(
                node,
                &options,
                &result_sender,
                &mut processed_count,
                &mut total_count,
                &mut visible,
            );

            if process_result.is_err() {
                let error = process_result.unwrap_err();
                let _ = result_sender.send(TreeLoadResult::Error(error));
                return;
            }

            node_index = node_index + 1;
        }

        if visible.is_empty() {
            if nodes.len() > 0 {
                visible.push(nodes[0].clone());
            }
        }

        thread::sleep(Duration::from_millis(300));
        let _ = result_sender.send(TreeLoadResult::LoadedTree(visible));
    }

    fn calculate_initial_count(nodes: &[FileNode]) -> usize {
        nodes.len()
    }

    fn process_single_node(
        node: &mut FileNode,
        options: &Options,
        result_sender: &Sender<TreeLoadResult>,
        processed_count: &mut u32,
        total_count: &mut usize,
        visible: &mut Vec<FileNode>,
    ) -> Result<(), String> {
        let path_string = node.path.to_string_lossy().to_string();
        let _ = result_sender.send(TreeLoadResult::ProcessingPath(path_string));

        *processed_count = *processed_count + 1;
        let _ = result_sender.send(TreeLoadResult::CountUpdate(*processed_count as usize, *total_count));

        thread::sleep(Duration::from_millis(10));
        node.loaded = false;

        let refresh_result = node.refresh(options);

        if refresh_result.is_err() {
            let error = refresh_result.unwrap_err();
            return Err(format!("Error refreshing node {}: {}", node.path.display(), error));
        }

        let has_visible = refresh_result.unwrap();

        if !has_visible {
            return Ok(());
        }

        let load_all_result = node.load_all_children(options);

        if load_all_result.is_err() {
            return Err(format!("Error loading node {}", node.path.display()));
        }

        let child_count = Self::count_loaded_nodes(node);
        *total_count = *total_count + child_count;
        let _ = result_sender.send(TreeLoadResult::CountUpdate(*processed_count as usize, *total_count));

        Self::process_loaded_nodes(node, result_sender, processed_count, *total_count);
        visible.push(node.clone());

        Ok(())
    }

    fn count_loaded_nodes(node: &FileNode) -> usize {
        let mut count: usize = 0;

        for child in &node.children {
            count = count + 1;

            let is_loaded_dir = child.is_directory() && child.loaded;

            if is_loaded_dir {
                let child_count = Self::count_loaded_nodes(child);
                count = count + child_count;
            }
        }

        count
    }

    fn process_loaded_nodes(
        node: &FileNode,
        result_sender: &Sender<TreeLoadResult>,
        processed_count: &mut u32,
        total_count: usize,
    ) {
        for child in &node.children {
            let path_string = child.path.to_string_lossy().to_string();
            let _ = result_sender.send(TreeLoadResult::ProcessingPath(path_string));

            *processed_count = *processed_count + 1;
            let _ = result_sender.send(TreeLoadResult::CountUpdate(*processed_count as usize, total_count));

            thread::sleep(Duration::from_millis(10));

            let is_loaded_dir = child.is_directory() && child.loaded;

            if is_loaded_dir {
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

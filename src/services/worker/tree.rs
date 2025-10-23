use std::sync::mpsc::Sender;

use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::tree::traversal::Traversable;

use super::core::{Worker, WorkerTask};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeLoadStatus {
    Complete,
    InProgress,
    NotStarted,
}

pub enum TreeLoadCommand {
    Load(Vec<FileNode>, Options),
    Stop,
}

pub enum TreeLoadResult {
    CountUpdate(usize, usize),
    Error(String),
    LoadedTree(Vec<FileNode>),
    ProcessingPath(String),
}

pub struct TreeLoadTask;

impl TreeLoadTask {
    fn process_tree_load(
        mut nodes: Vec<FileNode>,
        options: Options,
        result_sender: &Sender<TreeLoadResult>,
    ) {
        let mut visible = Vec::new();
        let mut total_count = Self::calculate_initial_count(&nodes);

        let _ = result_sender.send(TreeLoadResult::CountUpdate(0, total_count));
        let mut processed_count: usize = 0;

        for node in nodes.iter_mut() {
            let process_result = Self::process_single_node(
                node,
                &options,
                result_sender,
                &mut processed_count,
                &mut total_count,
                &mut visible,
            );

            if let Err(error) = process_result {
                let _ = result_sender.send(TreeLoadResult::Error(error));
                return;
            }
        }

        if visible.is_empty() {
            visible = nodes;
        }

        let _ = result_sender.send(TreeLoadResult::LoadedTree(visible));
    }

    fn calculate_initial_count(nodes: &[FileNode]) -> usize {
        nodes.iter()
            .filter(|node| node.is_directory())
            .count()
    }

    fn process_single_node(
        node: &mut FileNode,
        options: &Options,
        result_sender: &Sender<TreeLoadResult>,
        processed_count: &mut usize,
        total_count: &mut usize,
        visible: &mut Vec<FileNode>,
    ) -> Result<(), String> {
        if !node.is_directory() {
            return Ok(());
        }

        let refresh_result = node.refresh(options);

        if let Err(error) = refresh_result {
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
        *total_count += child_count;

        Self::process_loaded_nodes(node, result_sender, processed_count, *total_count);
        visible.push(node.clone());

        Ok(())
    }

    fn count_loaded_nodes(node: &FileNode) -> usize {
        let mut count: usize = 0;

        for child in &node.children {
            count += 1;

            let is_loaded_dir = child.is_directory() && child.loaded;

            if is_loaded_dir {
                let child_count = Self::count_loaded_nodes(child);
                count += child_count;
            }
        }

        count
    }

    fn process_loaded_nodes(
        node: &FileNode,
        result_sender: &Sender<TreeLoadResult>,
        processed_count: &mut usize,
        total_count: usize,
    ) {
        for child in &node.children {
            *processed_count += 1;

            if *processed_count % 50 == 0 {
                let path_string = child.path.to_string_lossy().to_string();
                let _ = result_sender.send(TreeLoadResult::ProcessingPath(path_string));
                let _ = result_sender.send(TreeLoadResult::CountUpdate(*processed_count, total_count));
            }

            let is_loaded_dir = child.is_directory() && child.loaded;

            if is_loaded_dir {
                Self::process_loaded_nodes(child, result_sender, processed_count, total_count);
            }
        }
    }
}

impl WorkerTask for TreeLoadTask {
    type Command = TreeLoadCommand;
    type Result = TreeLoadResult;

    fn process(&mut self, command: Self::Command, result_sender: &Sender<Self::Result>) {
        match command {
            TreeLoadCommand::Load(nodes, options) => {
                Self::process_tree_load(nodes, options, result_sender);
            }
            TreeLoadCommand::Stop => {}
        }
    }
}

pub struct TreeLoader {
    status: TreeLoadStatus,
    worker: Worker<TreeLoadTask>,
}

impl Default for TreeLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeLoader {
    pub fn new() -> Self {
        let worker = Worker::spawn(TreeLoadTask);

        Self {
            status: TreeLoadStatus::NotStarted,
            worker,
        }
    }

    pub fn check_results(&mut self) -> Option<TreeLoadResult> {
        let result = self.worker.try_recv()?;

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

        if self.worker.send(TreeLoadCommand::Load(nodes, options)) {
            self.status = TreeLoadStatus::InProgress;
            true
        } else {
            false
        }
    }

    pub fn status(&self) -> TreeLoadStatus {
        self.status
    }
}

impl Drop for TreeLoader {
    fn drop(&mut self) {
        let _ = self.worker.send(TreeLoadCommand::Stop);
    }
}

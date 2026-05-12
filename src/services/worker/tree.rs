use std::sync::mpsc::Sender;

use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::tree::traversal::Traversable;

use super::core::{Worker, WorkerTask};

pub enum TreeLoadCommand {
    Load(Vec<FileNode>, Options),
    Stop,
}

pub enum TreeLoadResult {
    LoadedTree(Vec<FileNode>),
    ProcessingPath(String),
    CountUpdate(usize, usize),
    Error(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeLoadStatus {
    NotStarted,
    Loading,
    Loaded,
    Error,
}

enum NodeOutcome {
    Visible(FileNode),
    NotVisible(FileNode),
}

pub struct TreeLoadTask;

impl TreeLoadTask {
    fn process_tree_load(
        nodes: Vec<FileNode>,
        options: Options,
        result_sender: &Sender<TreeLoadResult>,
    ) {
        let mut visible: Vec<FileNode> = Vec::with_capacity(nodes.len());
        let mut not_visible: Vec<FileNode> = Vec::new();
        let mut processed_count: usize = 0;
        let mut count_total = Self::calculate_initial_count(&nodes);

        for node in nodes {
            let process_result = Self::process_single_node(
                node,
                &options,
                result_sender,
                &mut processed_count,
                &mut count_total,
            );

            match process_result {
                Ok(NodeOutcome::Visible(n)) => visible.push(n),
                Ok(NodeOutcome::NotVisible(n)) => not_visible.push(n),
                Err(error) => {
                    let _ = result_sender.send(TreeLoadResult::Error(error));
                    return;
                }
            }
        }

        if visible.is_empty() {
            visible = not_visible;
        }

        let _ = result_sender.send(TreeLoadResult::LoadedTree(visible));
    }

    fn calculate_initial_count(nodes: &[FileNode]) -> usize {
        nodes.iter()
            .filter(|node| node.is_directory())
            .count()
    }

    fn process_single_node(
        mut node: FileNode,
        options: &Options,
        result_sender: &Sender<TreeLoadResult>,
        processed_count: &mut usize,
        count_total: &mut usize,
    ) -> Result<NodeOutcome, String> {
        if !node.is_directory() {
            return Ok(NodeOutcome::NotVisible(node));
        }

        let refresh_result = node.refresh(options);

        if let Err(error) = refresh_result {
            return Err(format!("Error refreshing node {}: {}", node.path.display(), error));
        }

        let has_visible = refresh_result.unwrap();

        if !has_visible {
            return Ok(NodeOutcome::NotVisible(node));
        }

        let load_all_result = node.load_all_children(options);

        if load_all_result.is_err() {
            return Err(format!("Error loading node {}", node.path.display()));
        }

        let child_count = Self::walk_loaded_nodes(&node, result_sender, processed_count);
        *count_total += child_count;

        Ok(NodeOutcome::Visible(node))
    }

    fn walk_loaded_nodes(
        node: &FileNode,
        result_sender: &Sender<TreeLoadResult>,
        processed_count: &mut usize,
    ) -> usize {
        let mut count: usize = 0;

        for child in &node.children {
            count += 1;
            *processed_count += 1;

            if *processed_count % 50 == 0 {
                let string_path = child.path.to_string_lossy().into_owned();
                let _ = result_sender.send(TreeLoadResult::ProcessingPath(string_path));
                let _ = result_sender.send(TreeLoadResult::CountUpdate(*processed_count, count));
            }

            if child.is_directory() && child.loaded {
                count += Self::walk_loaded_nodes(child, result_sender, processed_count);
            }
        }

        count
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
            self.status = TreeLoadStatus::Loaded;
        }

        Some(result)
    }

    pub fn start_load(&mut self, nodes: Vec<FileNode>, options: Options) -> bool {
        self.status = TreeLoadStatus::Loading;

        self.worker.send(TreeLoadCommand::Load(nodes, options))
    }

    pub fn status(&self) -> TreeLoadStatus {
        self.status
    }
}

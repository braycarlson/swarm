use alloc::sync::Arc;
use std::sync::mpsc::SyncSender;

use crate::model::node::FileNode;
use crate::services::filesystem::filter::GlobPathFilter;
use crate::services::tree::loader;

use super::core::{Worker, WorkerTask};

pub struct TreeRefreshRequest {
    pub filter: Arc<GlobPathFilter>,
    pub nodes: Vec<FileNode>,
    pub session: String,
}

pub struct TreeRefreshResult {
    pub failed: Vec<String>,
    pub nodes: Vec<FileNode>,
    pub session: String,
}

struct TreeRefreshTask;

impl TreeRefreshTask {
    fn refresh(nodes: Vec<FileNode>, filter: &GlobPathFilter) -> (Vec<FileNode>, Vec<String>) {
        let roots_count = nodes.len();
        let mut visible = Vec::with_capacity(nodes.len());
        let mut hidden = Vec::new();
        let mut failed = Vec::new();

        for mut node in nodes {
            node.loaded = false;

            match loader::load_children(&mut node, 0, filter) {
                Ok(true) => visible.push(node),
                Ok(false) => hidden.push(node),
                Err(error) => {
                    failed.push(format!("{}: {error}", node.path.display()));
                    visible.push(node);
                }
            }
        }

        if visible.is_empty() {
            visible = hidden;
        }

        debug_assert!(visible.len() <= roots_count);

        (visible, failed)
    }
}

impl WorkerTask for TreeRefreshTask {
    type Command = Box<TreeRefreshRequest>;
    type Result = TreeRefreshResult;

    fn process(&mut self, request: Self::Command, results: &SyncSender<Self::Result>) {
        let TreeRefreshRequest {
            filter,
            nodes,
            session,
        } = *request;

        let (refreshed, failed) = Self::refresh(nodes, &filter);

        let _ = results.send(TreeRefreshResult {
            failed,
            nodes: refreshed,
            session,
        });
    }
}

pub struct TreeLoader {
    worker: Worker<TreeRefreshTask>,
}

impl TreeLoader {
    pub fn check_results(&self) -> Option<TreeRefreshResult> {
        self.worker.try_recv()
    }

    pub fn new() -> Self {
        Self {
            worker: Worker::spawn("tree-loader", TreeRefreshTask),
        }
    }

    pub fn start(&self, request: Box<TreeRefreshRequest>) -> bool {
        assert_ne!(request.session, "");

        self.worker.send(request)
    }
}

impl Default for TreeLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::model::node::NodeKind;
    use crate::model::options::Options;

    #[test]
    fn an_empty_root_is_kept_when_nothing_else_is_visible() {
        let root = std::env::temp_dir().join(format!("swarm-refresh-{}", uuid::Uuid::new_v4()));

        fs::create_dir_all(&root).expect("the test directory is creatable");

        let filter =
            GlobPathFilter::from_options(&Options::default()).expect("the defaults compile");

        let nodes = vec![FileNode::with_kind(root.clone(), NodeKind::Directory)];
        let (refreshed, failed) = TreeRefreshTask::refresh(nodes, &filter);

        fs::remove_dir_all(&root).expect("the test directory is removable");

        assert_eq!(refreshed.len(), 1);
        assert_eq!(failed, Vec::<String>::new());
    }

    #[test]
    fn a_vanished_root_is_kept_and_reported() {
        let filter =
            GlobPathFilter::from_options(&Options::default()).expect("the defaults compile");

        let nodes = vec![FileNode::with_kind(
            PathBuf::from("/nonexistent/swarm/root"),
            NodeKind::Directory,
        )];

        let (refreshed, failed) = TreeRefreshTask::refresh(nodes, &filter);

        assert_eq!(refreshed.len(), 1);
        assert!(refreshed[0].load_failed);
        assert_eq!(failed.len(), 1);
    }
}

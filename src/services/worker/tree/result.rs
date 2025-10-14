use crate::model::node::FileNode;

pub enum TreeLoadResult {
    CountUpdate(usize, usize),
    Error(String),
    LoadedTree(Vec<FileNode>),
    ProcessingPath(String),
}

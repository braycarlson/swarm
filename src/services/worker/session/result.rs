use crate::model::node::FileNode;

pub enum SessionLoadResult {
    Error(String),
    Loaded(Vec<FileNode>),
    Loading(String),
}

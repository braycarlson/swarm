use crate::model::node::FileNode;
use crate::model::options::Options;

pub enum TreeLoadCommand {
    Load(Vec<FileNode>, Options),
    Stop,
}

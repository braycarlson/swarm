use std::path::PathBuf;

use crate::model::options::Options;

pub enum SessionLoadCommand {
    Cancel,
    Load(PathBuf, Options),
}

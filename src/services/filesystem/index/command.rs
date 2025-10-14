use std::path::PathBuf;

use crate::model::options::Options;

pub enum IndexCommand {
    Pause,
    Resume,
    Start(Vec<PathBuf>, Options),
    Stop,
    SwitchSession(String),
}

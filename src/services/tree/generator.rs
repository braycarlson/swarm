use crate::model::node::FileNode;
use crate::model::options::Options;

use super::format::{AsciiTreeFormat, TreeFormat};

pub struct TreeGenerator {
    formatter: Box<dyn TreeFormat>,
}

impl TreeGenerator {
    pub fn new(options: &Options) -> Self {
        let formatter: Box<dyn TreeFormat> = Box::new(AsciiTreeFormat::new(options.use_icon));

        Self { formatter }
    }

    pub fn with_format(formatter: Box<dyn TreeFormat>) -> Self {
        Self { formatter }
    }

    pub fn generate_tree(&self, nodes: &[FileNode]) -> String {
        self.formatter.format_tree(nodes)
    }
}

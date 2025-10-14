use std::fmt::Write;

use crate::model::node::{FileNode, NodeKind};
use crate::model::options::Options;

pub struct TreeGenerator {
    use_icon: bool,
}

impl TreeGenerator {
    pub fn new(options: &Options) -> Self {
        Self {
            use_icon: options.use_icon,
        }
    }

    pub fn generate_tree(&self, root_nodes: &[FileNode]) -> String {
        let mut output = String::new();
        writeln!(output, ".").unwrap();

        let selected_roots: Vec<&FileNode> = root_nodes
            .iter()
            .filter(|node| node.is_selected())
            .collect();

        let count = selected_roots.len();

        for (index, node) in selected_roots.iter().enumerate() {
            let is_last = index == count - 1;
            self.generate_recursive(node, "", is_last, &mut output);
        }

        output
    }

    fn generate_recursive(&self, node: &FileNode, prefix: &str, is_last: bool, output: &mut String) {
        let branch = if prefix.is_empty() {
            ""
        } else if is_last {
            "└── "
        } else {
            "├── "
        };

        let name = node.file_name()
            .unwrap_or_else(|| node.path.display().to_string());

        let display_name = if self.use_icon {
            match node.kind {
                NodeKind::Directory => format!("📁 {}", name),
                NodeKind::File => format!("📄 {}", name),
            }
        } else {
            name
        };

        writeln!(output, "{}{}{}", prefix, branch, display_name).unwrap();

        let new_prefix = if prefix.is_empty() {
            if is_last {
                "    ".to_string()
            } else {
                "│   ".to_string()
            }
        } else if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        let selected_children: Vec<&FileNode> = node.children
            .iter()
            .filter(|child| child.is_selected())
            .collect();

        let count = selected_children.len();

        for (index, child) in selected_children.iter().enumerate() {
            let child_is_last = index == count - 1;
            self.generate_recursive(child, &new_prefix, child_is_last, output);
        }
    }
}

use std::fmt::Write;

use crate::model::node::{FileNode, NodeKind};

pub trait TreeFormat {
    fn format_tree(&self, nodes: &[FileNode]) -> String;
}

pub struct AsciiTreeFormat {
    use_icons: bool,
}

impl AsciiTreeFormat {
    pub fn new(use_icons: bool) -> Self {
        Self { use_icons }
    }

    fn icon(&self, node: &FileNode) -> &'static str {
        if !self.use_icons {
            return "";
        }

        match node.kind {
            NodeKind::Directory => "📁 ",
            NodeKind::File => "📄 ",
        }
    }

    fn format_node(&self, output: &mut String, node: &FileNode, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        let icon = self.icon(node);
        let name = node.file_name().unwrap_or_else(|| "Unknown".to_string());

        let _ = writeln!(output, "{}{}{}{}", prefix, connector, icon, name);

        if node.is_directory() && !node.children.is_empty() {
            let extension = if is_last { "     " } else { "│    " };
            let mut child_prefix = String::with_capacity(prefix.len() + 5);
            child_prefix.push_str(prefix);
            child_prefix.push_str(extension);

            for (index, child) in node.children.iter().enumerate() {
                let is_last_child = index == node.children.len() - 1;
                self.format_node(output, child, &child_prefix, is_last_child);
            }
        }
    }
}

impl TreeFormat for AsciiTreeFormat {
    fn format_tree(&self, nodes: &[FileNode]) -> String {
        let mut output = String::new();
        let _ = writeln!(&mut output, ".");

        for (index, node) in nodes.iter().enumerate() {
            let is_last = index == nodes.len() - 1;
            self.format_node(&mut output, node, "", is_last);
        }

        output
    }
}

const COMPACT_INDENT_CACHE: &str = "                                                                                                                                ";

pub struct CompactTreeFormat {
    use_icons: bool,
}

impl CompactTreeFormat {
    pub fn new(use_icons: bool) -> Self {
        Self { use_icons }
    }

    fn icon(&self, node: &FileNode) -> &'static str {
        if !self.use_icons {
            return "";
        }

        match node.kind {
            NodeKind::Directory => "📁 ",
            NodeKind::File => "📄 ",
        }
    }

    fn format_node(&self, output: &mut String, node: &FileNode, depth: usize) {
        let end = (depth * 2).min(COMPACT_INDENT_CACHE.len());
        let indent = &COMPACT_INDENT_CACHE[..end];
        let icon = self.icon(node);
        let name = node.file_name().unwrap_or_else(|| "Unknown".to_string());

        let _ = writeln!(output, "{}{}{}", indent, icon, name);

        if node.is_directory() && !node.children.is_empty() {
            for child in &node.children {
                self.format_node(output, child, depth + 1);
            }
        }
    }
}

impl TreeFormat for CompactTreeFormat {
    fn format_tree(&self, nodes: &[FileNode]) -> String {
        let mut output = String::new();
        let _ = writeln!(&mut output, ".");

        for node in nodes {
            self.format_node(&mut output, node, 0);
        }

        output
    }
}

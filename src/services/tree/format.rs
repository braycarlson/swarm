use core::iter::Peekable;
use core::slice;

use crate::model::node::{self, FileNode, NodeKind, TREE_DEPTH_MAX};

const BRANCH_LAST: &str = "\u{2514}\u{2500}\u{2500} ";
const BRANCH_MORE: &str = "\u{251c}\u{2500}\u{2500} ";
const ICON_DIRECTORY: &str = "\u{1f4c1} ";
const ICON_FILE: &str = "\u{1f4c4} ";
const INDENT_LAST: &str = "     ";
const INDENT_MORE: &str = "\u{2502}    ";
const LINE_BYTES_ESTIMATE: usize = 32;
const STACK_CAPACITY: usize = 32;

struct Frame<'tree> {
    iterator: Peekable<slice::Iter<'tree, FileNode>>,
    prefix_length: usize,
}

pub fn format_tree(nodes: &[FileNode], use_icons: bool) -> String {
    let mut node_count: usize = 0;

    node::visit(nodes, |_, _| node_count += 1);

    let mut output = String::with_capacity((node_count + 1) * LINE_BYTES_ESTIMATE);
    let mut prefix = String::new();
    let mut stack: Vec<Frame<'_>> = Vec::with_capacity(STACK_CAPACITY);
    let mut line_count: usize = 0;

    output.push_str(".\n");

    stack.push(Frame {
        iterator: nodes.iter().peekable(),
        prefix_length: 0,
    });

    while !stack.is_empty() {
        assert!(stack.len() <= TREE_DEPTH_MAX as usize);

        let top = stack.len() - 1;
        let frame = &mut stack[top];
        let prefix_length = frame.prefix_length;

        let Some(node) = frame.iterator.next() else {
            let _ = stack.pop();

            continue;
        };

        let is_last = frame.iterator.peek().is_none();
        let connector = if is_last { BRANCH_LAST } else { BRANCH_MORE };

        debug_assert!(prefix_length <= prefix.len());

        prefix.truncate(prefix_length);
        output.push_str(&prefix);
        output.push_str(connector);
        output.push_str(icon(node.kind, use_icons));
        output.push_str(&node.name);
        output.push('\n');
        line_count += 1;

        if !node.is_directory() {
            continue;
        }

        if node.children.is_empty() {
            continue;
        }

        prefix.push_str(if is_last { INDENT_LAST } else { INDENT_MORE });

        stack.push(Frame {
            iterator: node.children.iter().peekable(),
            prefix_length: prefix.len(),
        });
    }

    assert_eq!(line_count, node_count);

    output
}

fn icon(kind: NodeKind, use_icons: bool) -> &'static str {
    if !use_icons {
        return "";
    }

    match kind {
        NodeKind::Directory => ICON_DIRECTORY,
        NodeKind::File => ICON_FILE,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn directory(path: &str, children: Vec<FileNode>) -> FileNode {
        let mut node = FileNode::with_kind(PathBuf::from(path), NodeKind::Directory);

        node.children = children;

        node
    }

    fn file(path: &str) -> FileNode {
        FileNode::with_kind(PathBuf::from(path), NodeKind::File)
    }

    #[test]
    fn a_flat_list_renders_branches() {
        let nodes = vec![file("/a.rs"), file("/b.rs")];

        let expected = concat!(
            ".\n",
            "\u{251c}\u{2500}\u{2500} a.rs\n",
            "\u{2514}\u{2500}\u{2500} b.rs\n",
        );

        assert_eq!(format_tree(&nodes, false), expected);
    }

    #[test]
    fn nesting_indents_under_the_parent() {
        let nodes = vec![directory("/root", vec![file("/root/a.rs")]), file("/b.rs")];

        let expected = concat!(
            ".\n",
            "\u{251c}\u{2500}\u{2500} root\n",
            "\u{2502}    \u{2514}\u{2500}\u{2500} a.rs\n",
            "\u{2514}\u{2500}\u{2500} b.rs\n",
        );

        assert_eq!(format_tree(&nodes, false), expected);
    }

    #[test]
    fn an_empty_forest_renders_the_root_marker_only() {
        assert_eq!(format_tree(&[], false), ".\n");
    }

    #[test]
    fn icons_prefix_each_name() {
        let output = format_tree(&[file("/a.rs")], true);

        assert!(output.contains(ICON_FILE));
    }
}

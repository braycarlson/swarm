use std::fs;

use crate::model::error::SwarmResult;
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::model::path::PathExtensions;
use crate::services::tree::filter;

pub fn load_children(node: &mut FileNode, options: &Options) -> SwarmResult<bool> {
    if node.is_file() {
        return Ok(true);
    }

    node.loaded = true;
    node.children.clear();

    let entries = fs::read_dir(&node.path)?;

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for entry_result in entries {
        let entry = entry_result?;
        let child_path = entry.path();

        if child_path.is_hidden() {
            continue;
        }

        if filter::is_path_in_excluded_patterns(&child_path, &options.exclude) {
            continue;
        }

        if child_path.is_dir() {
            let child_node = FileNode::new(child_path);
            directories.push(child_node);
            continue;
        }

        if filter::should_include_path(&child_path, options) {
            files.push(FileNode::new(child_path));
        }
    }

    directories.sort_by_key(|node| node.lowercase_name());
    files.sort_by_key(|node| node.lowercase_name());

    node.children = directories;
    node.children.extend(files);

    Ok(node.has_children())
}

pub fn load_all_children(node: &mut FileNode, options: &Options) -> SwarmResult<bool> {
    if node.is_file() {
        return Ok(true);
    }

    let has_visible_content = load_children(node, options)?;

    for child in &mut node.children {
        if child.is_directory() {
            load_all_children(child, options)?;
        }
    }

    Ok(has_visible_content)
}

pub fn refresh_node(node: &mut FileNode, options: &Options) -> SwarmResult<bool> {
    node.loaded = false;
    node.children.clear();

    let has_visible_content = load_children(node, options)?;

    Ok(has_visible_content)
}

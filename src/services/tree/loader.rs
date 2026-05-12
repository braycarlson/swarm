use std::fs;

use rayon::prelude::*;

use crate::model::error::SwarmResult;
use crate::model::node::{FileNode, NodeKind};
use crate::model::options::Options;
use crate::model::path::PathExtensions;
use crate::services::filesystem::filter::{GlobPathFilter, PathFilter};

pub fn load_children(node: &mut FileNode, options: &Options) -> SwarmResult<bool> {
    let filter = GlobPathFilter::from_options(options)?;
    load_children_with_filter(node, options, &filter)
}

fn load_children_with_filter(node: &mut FileNode, options: &Options, filter: &GlobPathFilter) -> SwarmResult<bool> {
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

        if !options.show_hidden && child_path.is_hidden() {
            continue;
        }

        if !filter.should_include(&child_path) {
            continue;
        }

        let type_file = entry.file_type()?;
        let kind = if type_file.is_dir() {
            NodeKind::Directory
        } else {
            NodeKind::File
        };

        let child = FileNode::with_kind(child_path, kind);

        if kind == NodeKind::Directory {
            directories.push(child);
        } else {
            files.push(child);
        }
    }

    directories.sort_by(|a, b| a.name_lowercase().cmp(b.name_lowercase()));
    files.sort_by(|a, b| a.name_lowercase().cmp(b.name_lowercase()));

    let mut combined = Vec::with_capacity(directories.len() + files.len());
    combined.append(&mut directories);
    combined.append(&mut files);
    node.children = combined;

    Ok(node.has_children())
}

pub fn load_all_children(node: &mut FileNode, options: &Options) -> SwarmResult<bool> {
    let filter = GlobPathFilter::from_options(options)?;
    load_all_children_with_filter(node, options, &filter)
}

fn load_all_children_with_filter(node: &mut FileNode, options: &Options, filter: &GlobPathFilter) -> SwarmResult<bool> {
    if node.is_file() {
        return Ok(true);
    }

    let has_visible_content = load_children_with_filter(node, options, filter)?;

    node.children
        .par_iter_mut()
        .filter(|child| child.is_directory())
        .for_each(|child| {
            let _ = load_all_children_with_filter(child, options, filter);
        });

    Ok(has_visible_content)
}

pub fn refresh_node(node: &mut FileNode, options: &Options) -> SwarmResult<bool> {
    node.loaded = false;
    node.children.clear();

    let has_visible_content = load_children(node, options)?;

    Ok(has_visible_content)
}

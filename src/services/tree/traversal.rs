use crate::app::state::search::{FileMetadata, ParsedQuery, SearchModel, TypeFilter};
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::model::error::SwarmResult;
use crate::services::filesystem::git::GitService;
use crate::services::tree::loader;

pub trait Traversable {
    fn load_children(&mut self, options: &Options) -> SwarmResult<bool>;
    fn load_all_children(&mut self, options: &Options) -> SwarmResult<bool>;
    fn matches_search(&self, query: &str) -> bool;
    fn matches_parsed_query(&self, query: &ParsedQuery) -> bool;
    fn matches_parsed_query_with_git(&self, query: &ParsedQuery, git: Option<&GitService>) -> bool;
    fn propagate_checked(&mut self, checked: bool);
    fn propagate_checked_with_load(&mut self, checked: bool, options: &Options);
    fn refresh(&mut self, options: &Options) -> SwarmResult<bool>;
}

impl Traversable for FileNode {
    fn load_children(&mut self, options: &Options) -> SwarmResult<bool> {
        loader::load_children(self, options)
    }

    fn load_all_children(&mut self, options: &Options) -> SwarmResult<bool> {
        loader::load_all_children(self, options)
    }

    fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let parsed = ParsedQuery::parse(query);
        self.matches_parsed_query(&parsed)
    }

    fn matches_parsed_query(&self, query: &ParsedQuery) -> bool {
        self.matches_parsed_query_with_git(query, None)
    }

    fn matches_parsed_query_with_git(&self, query: &ParsedQuery, git: Option<&GitService>) -> bool {
        matches_node_recursive(self, query, git, 0)
    }

    fn propagate_checked(&mut self, checked: bool) {
        self.checked = checked;

        if self.is_directory() {
            for child in &mut self.children {
                child.propagate_checked(checked);
            }
        }
    }

    fn propagate_checked_with_load(&mut self, checked: bool, options: &Options) {
        self.checked = checked;

        if self.is_directory() {
            if !self.loaded {
                let _ = self.load_children(options);
            }

            for child in &mut self.children {
                child.propagate_checked_with_load(checked, options);
            }
        }
    }

    fn refresh(&mut self, options: &Options) -> SwarmResult<bool> {
        loader::refresh_node(self, options)
    }
}

fn matches_node_recursive(
    node: &FileNode,
    query: &ParsedQuery,
    git: Option<&GitService>,
    depth: usize,
) -> bool {
    if query.is_empty() {
        return true;
    }

    let name = node.file_name().unwrap_or_default();
    let path = node.path.to_string_lossy();

    if node.is_directory() {
        if query.has_depth_filter() && !query.matches_depth(depth) {
            return false;
        }

        if matches!(query.type_filter, Some(TypeFilter::Directory)) {
            let self_matches = query.matches_full(&name, &path, None, true, None);

            let has_matching_children = node.children.iter().any(|child| {
                matches_node_recursive(child, query, git, depth + 1)
            });

            return self_matches || has_matching_children;
        }

        if query.requires_file_match() {
            let has_matching_children = node.children.iter().any(|child| {
                matches_node_recursive(child, query, git, depth + 1)
            });

            return has_matching_children;
        }

        let has_matching_children = node.children.iter().any(|child| {
            matches_node_recursive(child, query, git, depth + 1)
        });

        if has_matching_children {
            return true;
        }

        return query.matches_full(&name, &path, None, true, None);
    }

    let git_status = git.map(|g| g.get_status(&node.path));
    let metadata = get_file_metadata(node, query);

    query.matches_full(&name, &path, git_status, false, metadata.as_ref())
}

fn matches_node_at_depth(
    node: &FileNode,
    query: &ParsedQuery,
    git: Option<&GitService>,
    depth: usize,
) -> bool {
    if query.is_empty() {
        return true;
    }

    let name = node.file_name().unwrap_or_default();
    let path = node.path.to_string_lossy();

    if node.is_directory() {
        if query.has_depth_filter() && !query.matches_depth(depth) {
            return false;
        }

        if matches!(query.type_filter, Some(TypeFilter::Directory)) {
            let self_matches = query.matches_full(&name, &path, None, true, None);

            let has_matching_children = node.children.iter().any(|child| {
                matches_node_at_depth(child, query, git, depth + 1)
            });

            return self_matches || has_matching_children;
        }

        if query.requires_file_match() {
            let has_matching_children = node.children.iter().any(|child| {
                matches_node_at_depth(child, query, git, depth + 1)
            });

            return has_matching_children;
        }

        let has_matching_children = node.children.iter().any(|child| {
            matches_node_at_depth(child, query, git, depth + 1)
        });

        if has_matching_children {
            return true;
        }

        return query.matches_full(&name, &path, None, true, None);
    }

    let git_status = git.map(|g| g.get_status(&node.path));
    let metadata = get_file_metadata(node, query);

    query.matches_full(&name, &path, git_status, false, metadata.as_ref())
}

fn get_file_metadata(node: &FileNode, query: &ParsedQuery) -> Option<FileMetadata> {
    if query.is_expensive() {
        return node.metadata.clone();
    }

    if !query.needs_metadata() {
        return None;
    }

    if let Some(ref cached) = node.metadata {
        return Some(cached.clone());
    }

    FileMetadata::from_path_basic(&node.path)
}

pub fn should_show_node(node: &FileNode, search_query: &str) -> bool {
    should_show_node_with_git(node, search_query, None)
}

pub fn should_show_node_with_git(node: &FileNode, search_query: &str, git: Option<&GitService>) -> bool {
    if search_query.is_empty() {
        return true;
    }

    let parsed = ParsedQuery::parse(search_query);
    node.matches_parsed_query_with_git(&parsed, git)
}

pub fn should_show_node_with_search(node: &FileNode, search: &SearchModel, git: Option<&GitService>) -> bool {
    if !search.has_query() {
        return true;
    }

    if let Some(is_match) = search.is_path_matching(&node.path) {
        if node.is_directory() {
            return is_match || node.children.iter().any(|c| should_show_node_with_search(c, search, git));
        }

        return is_match;
    }

    let parsed = search.parsed();
    node.matches_parsed_query_with_git(&parsed, git)
}

pub fn should_show_node_at_depth(
    node: &FileNode,
    search: &SearchModel,
    git: Option<&GitService>,
    depth: usize,
    query: &ParsedQuery,
) -> bool {
    if !search.has_query() {
        return true;
    }

    if let Some(is_match) = search.is_path_matching(&node.path) {
        if node.is_directory() {
            if query.has_depth_filter() && !query.matches_depth(depth) {
                return false;
            }

            return is_match || node.children.iter().any(|c| {
                should_show_node_at_depth(c, search, git, depth + 1, query)
            });
        }
        return is_match;
    }

    matches_node_at_depth(node, query, git, depth)
}

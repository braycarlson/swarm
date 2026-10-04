use core::slice;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use rustc_hash::{FxBuildHasher, FxHashSet};

use crate::model::metadata::FileMetadata;
use crate::model::node::{FileNode, TREE_DEPTH_MAX};
use crate::model::query::{MatchTarget, ParsedQuery, TypeFilter};
use crate::model::time;
use crate::services::filesystem::git::GitService;
use crate::services::search::{ContentMatchers, SearchScratch, content_matches, symbol_matches};

const PATH_CAPACITY: usize = 256;
const STACK_CAPACITY: usize = 32;

pub struct FileTarget<'target> {
    pub name_lowercase: &'target str,
    pub path: &'target Path,
    pub path_lowercase: &'target str,
}

pub struct MatchContext<'context> {
    content_matchers: ContentMatchers,
    git: Option<&'context GitService>,
    metadata_needed: bool,
    query: &'context ParsedQuery,
    query_empty: bool,
    scan_needed: bool,
    seconds_now: u64,
}

impl<'context> MatchContext<'context> {
    fn directory_matches_self(&self, node: &FileNode, path_lowercase: &str) -> bool {
        debug_assert!(node.is_directory());

        let target = MatchTarget {
            git_status: None,
            is_directory: true,
            metadata: None,
            name_lowercase: &node.name_lowercase,
            path_lowercase,
            seconds_now: self.seconds_now,
        };

        self.query.matches(&target)
    }

    #[inline(never)]
    fn metadata_of(&self, path: &Path) -> Option<FileMetadata> {
        debug_assert!(self.metadata_needed);

        FileMetadata::from_path(path, self.query.needs_lines())
    }

    fn node_matches(&self, check: &NodeCheck<'_>, scratch: &mut SearchScratch) -> bool {
        let node = check.node;

        if self.query_empty {
            return true;
        }

        if node.is_file() {
            let target = FileTarget {
                name_lowercase: &node.name_lowercase,
                path: &node.path,
                path_lowercase: check.path_lowercase,
            };

            return self.file_matches(&target, scratch);
        }

        if self.query.filter_type == Some(TypeFilter::Directory) {
            return check.child_matched || self.directory_matches_self(node, check.path_lowercase);
        }

        if self.query.requires_file_match() {
            return check.child_matched;
        }

        check.child_matched || self.directory_matches_self(node, check.path_lowercase)
    }

    #[inline(never)]
    fn scan_matches(&self, path: &Path, scratch: &mut SearchScratch) -> bool {
        debug_assert!(self.scan_needed);

        if !content_matches(path, &self.content_matchers, scratch) {
            return false;
        }

        symbol_matches(path, &self.query.patterns_symbol, scratch)
    }

    pub fn file_matches(&self, target: &FileTarget<'_>, scratch: &mut SearchScratch) -> bool {
        let metadata = if self.metadata_needed {
            self.metadata_of(target.path)
        } else {
            None
        };

        let match_target = MatchTarget {
            git_status: self.git.map(|git| git.status(target.path)),
            is_directory: false,
            metadata: metadata.as_ref(),
            name_lowercase: target.name_lowercase,
            path_lowercase: target.path_lowercase,
            seconds_now: self.seconds_now,
        };

        if !self.query.matches(&match_target) {
            return false;
        }

        if !self.scan_needed {
            return true;
        }

        self.scan_matches(target.path, scratch)
    }

    pub fn new(query: &'context ParsedQuery, git: Option<&'context GitService>) -> Self {
        let git_needed = !query.filters_git.is_empty() || !query.excludes_git.is_empty();

        Self {
            content_matchers: ContentMatchers::new(&query.patterns_content),
            git: git.filter(|_| git_needed),
            metadata_needed: query.needs_metadata(),
            query,
            query_empty: query.is_empty(),
            scan_needed: !query.patterns_content.is_empty() || !query.patterns_symbol.is_empty(),
            seconds_now: time::seconds_now(),
        }
    }

    pub fn path_matches(&self, path: &Path, scratch: &mut SearchScratch) -> bool {
        let mut path_lowercase = String::with_capacity(PATH_CAPACITY);

        append_lowercase_path(&mut path_lowercase, path);

        let name_start = path_lowercase
            .rfind(['/', MAIN_SEPARATOR])
            .map_or(0, |separator| separator + 1);

        let name_lowercase = path_lowercase.get(name_start..).unwrap_or("");

        let target = FileTarget {
            name_lowercase,
            path,
            path_lowercase: &path_lowercase,
        };

        self.file_matches(&target, scratch)
    }
}

pub enum Matcher<'context> {
    Query(MatchContext<'context>),
    Set(&'context FxHashSet<PathBuf>),
}

impl Matcher<'_> {
    fn file_matches(
        &self,
        node: &FileNode,
        path_lowercase: &str,
        scratch: &mut SearchScratch,
    ) -> bool {
        debug_assert!(node.is_file());

        match self {
            Self::Query(context) => {
                let target = FileTarget {
                    name_lowercase: &node.name_lowercase,
                    path: &node.path,
                    path_lowercase,
                };

                context.file_matches(&target, scratch)
            }
            Self::Set(paths) => paths.contains(&node.path),
        }
    }

    fn node_matches(&self, check: &NodeCheck<'_>, scratch: &mut SearchScratch) -> bool {
        match self {
            Self::Query(context) => context.node_matches(check, scratch),
            Self::Set(paths) => paths.contains(&check.node.path),
        }
    }
}

struct MatchFrame<'tree, Output> {
    child_index: usize,
    child_matched: bool,
    depth: u32,
    node: &'tree FileNode,
    output: Output,
    path_length: usize,
}

impl<'tree, Output> MatchFrame<'tree, Output>
where
    Output: Default,
{
    fn check<'check>(&'check self, path_lowercase: &'check str) -> NodeCheck<'check> {
        NodeCheck {
            child_matched: self.child_matched,
            node: self.node,
            path_lowercase,
        }
    }

    fn new(node: &'tree FileNode, depth: u32, path_length: usize) -> Self {
        Self {
            child_index: 0,
            child_matched: false,
            depth,
            node,
            output: Output::default(),
            path_length,
        }
    }
}

struct NodeCheck<'check> {
    child_matched: bool,
    node: &'check FileNode,
    path_lowercase: &'check str,
}

#[derive(Default)]
struct Selection {
    children: Vec<FileNode>,
    selected: bool,
}

struct PathFrame<'tree> {
    iterator: slice::Iter<'tree, FileNode>,
    parent_length: usize,
}

pub fn append_lowercase_child(text: &mut String, name_lowercase: &str) {
    let separated = text.ends_with('/') || text.ends_with(MAIN_SEPARATOR);

    if !separated {
        text.push(MAIN_SEPARATOR);
    }

    text.push_str(name_lowercase);
}

pub fn append_lowercase_path(text: &mut String, path: &Path) {
    let start = text.len();

    text.push_str(&path.to_string_lossy());

    if let Some(appended) = text.get_mut(start..) {
        appended.make_ascii_lowercase();
    }

    debug_assert!(text.len() >= start);
}

fn build_filtered(frame: &mut MatchFrame<'_, Selection>) -> FileNode {
    let source = frame.node;
    let mut node = FileNode::with_kind(source.path.clone(), source.kind);

    node.checked = source.checked;
    node.loaded = source.loaded;
    node.children = core::mem::take(&mut frame.output.children);

    debug_assert!(node.children.len() <= source.children.len());
    debug_assert_eq!(frame.output.children.len(), 0);

    node
}

pub fn descends_into(node: &FileNode, query: &ParsedQuery, depth: u32) -> bool {
    if depth >= TREE_DEPTH_MAX {
        return false;
    }

    if !node.is_directory() {
        return true;
    }

    query.matches_depth(depth)
}

pub fn filter_selected(
    nodes: &[FileNode],
    matcher: &Matcher<'_>,
    query: &ParsedQuery,
) -> Vec<FileNode> {
    let mut roots: Vec<FileNode> = Vec::new();

    post_order::<Selection>(
        nodes,
        query,
        0,
        |frame, mut ancestor, path_lowercase, scratch| {
            let is_match = matcher.node_matches(&frame.check(path_lowercase), scratch);
            let is_selected = frame.node.checked || frame.output.selected;

            if let Some(parent) = ancestor.as_deref_mut() {
                parent.child_matched |= is_match;
                parent.output.selected |= is_selected;
            }

            if !is_match {
                return;
            }

            if !is_selected {
                return;
            }

            let built = build_filtered(frame);

            match ancestor {
                Some(parent) => parent.output.children.push(built),
                None => roots.push(built),
            }
        },
    );

    roots
}

pub fn gather_checked_paths(
    nodes: &[FileNode],
    matcher: &Matcher<'_>,
    query: &ParsedQuery,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut scratch = SearchScratch::default();

    walk_with_path(nodes, |node, depth, path_lowercase| {
        if node.is_directory() {
            return descends_into(node, query, depth);
        }

        if node.checked {
            if matcher.file_matches(node, path_lowercase, &mut scratch) {
                paths.push(node.path.clone());
            }
        }

        false
    });

    paths
}

pub fn matching_paths(
    nodes: &[FileNode],
    query: &ParsedQuery,
    git: Option<&GitService>,
    depth_base: u32,
) -> FxHashSet<PathBuf> {
    let context = MatchContext::new(query, git);
    let mut found: Vec<&Path> = Vec::new();

    post_order::<()>(
        nodes,
        query,
        depth_base,
        |frame, ancestor, path_lowercase, scratch| {
            let is_match = context.node_matches(&frame.check(path_lowercase), scratch);

            if let Some(parent) = ancestor {
                parent.child_matched |= is_match;
            }

            if is_match {
                found.push(&frame.node.path);
            }
        },
    );

    let mut paths = FxHashSet::with_capacity_and_hasher(found.len(), FxBuildHasher);

    paths.extend(found.iter().map(|path| path.to_path_buf()));

    debug_assert_eq!(paths.len(), found.len());

    paths
}

fn post_order<'tree, Output>(
    nodes: &'tree [FileNode],
    query: &ParsedQuery,
    depth_base: u32,
    mut finish: impl FnMut(
        &mut MatchFrame<'tree, Output>,
        Option<&mut MatchFrame<'tree, Output>>,
        &str,
        &mut SearchScratch,
    ),
) where
    Output: Default,
{
    let mut path_lowercase = String::with_capacity(PATH_CAPACITY);
    let mut scratch = SearchScratch::default();
    let mut stack: Vec<MatchFrame<'tree, Output>> = Vec::with_capacity(STACK_CAPACITY);

    for root in nodes {
        if !descends_into(root, query, depth_base) {
            continue;
        }

        path_lowercase.clear();
        append_lowercase_path(&mut path_lowercase, &root.path);
        stack.push(MatchFrame::new(root, depth_base, 0));

        while !stack.is_empty() {
            assert!(stack.len() <= TREE_DEPTH_MAX as usize);

            let top = stack.len() - 1;
            let frame = &mut stack[top];
            let node = frame.node;

            if let Some(child) = node.children.get(frame.child_index) {
                let depth = frame.depth;

                frame.child_index += 1;

                if descends_into(child, query, depth + 1) {
                    let path_length = path_lowercase.len();

                    append_lowercase_child(&mut path_lowercase, &child.name_lowercase);
                    stack.push(MatchFrame::new(child, depth + 1, path_length));
                }

                continue;
            }

            let mut finished = stack.pop().expect("the frame under inspection is present");

            finish(
                &mut finished,
                stack.last_mut(),
                &path_lowercase,
                &mut scratch,
            );

            debug_assert!(finished.path_length <= path_lowercase.len());

            path_lowercase.truncate(finished.path_length);
        }
    }
}

pub fn walk_with_path(nodes: &[FileNode], mut visitor: impl FnMut(&FileNode, u32, &str) -> bool) {
    let mut path_lowercase = String::with_capacity(PATH_CAPACITY);
    let mut stack: Vec<PathFrame<'_>> = Vec::with_capacity(STACK_CAPACITY);

    for root in nodes {
        path_lowercase.clear();
        append_lowercase_path(&mut path_lowercase, &root.path);

        if !visitor(root, 0, &path_lowercase) {
            continue;
        }

        stack.clear();

        stack.push(PathFrame {
            iterator: root.children.iter(),
            parent_length: path_lowercase.len(),
        });

        while !stack.is_empty() {
            assert!(stack.len() <= TREE_DEPTH_MAX as usize);

            let depth = u32::try_from(stack.len()).expect("the stack is bounded by the tree depth");
            let top = stack.len() - 1;
            let frame = &mut stack[top];
            let parent_length = frame.parent_length;

            let Some(node) = frame.iterator.next() else {
                let _ = stack.pop();

                continue;
            };

            debug_assert!(parent_length <= path_lowercase.len());

            path_lowercase.truncate(parent_length);
            append_lowercase_child(&mut path_lowercase, &node.name_lowercase);

            if !visitor(node, depth, &path_lowercase) {
                continue;
            }

            if node.children.is_empty() {
                continue;
            }

            stack.push(PathFrame {
                iterator: node.children.iter(),
                parent_length: path_lowercase.len(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::NodeKind;
    use crate::model::selection;

    fn directory(path: &str, children: Vec<FileNode>) -> FileNode {
        let mut node = FileNode::with_kind(PathBuf::from(path), NodeKind::Directory);

        node.children = children;
        node.loaded = true;

        node
    }

    fn file(path: &str) -> FileNode {
        FileNode::with_kind(PathBuf::from(path), NodeKind::File)
    }

    fn tree() -> Vec<FileNode> {
        vec![directory(
            "/root",
            vec![
                file("/root/alpha.rs"),
                directory(
                    "/root/nested",
                    vec![file("/root/nested/beta.py"), file("/root/nested/gamma.rs")],
                ),
            ],
        )]
    }

    fn gather(nodes: &[FileNode], text: &str) -> Vec<PathBuf> {
        let query = ParsedQuery::parse(text);
        let matcher = Matcher::Query(MatchContext::new(&query, None));

        gather_checked_paths(nodes, &matcher, &query)
    }

    #[test]
    fn matching_paths_includes_ancestors_of_matches() {
        let query = ParsedQuery::parse("ext:py");
        let matches = matching_paths(&tree(), &query, None, 0);

        assert!(matches.contains(Path::new("/root/nested/beta.py")));
        assert!(matches.contains(Path::new("/root/nested")));
        assert!(matches.contains(Path::new("/root")));
        assert!(!matches.contains(Path::new("/root/alpha.rs")));
    }

    #[test]
    fn matching_paths_composes_the_full_lowercase_path() {
        let query = ParsedQuery::parse("path:nested");
        let matches = matching_paths(&tree(), &query, None, 0);

        assert!(matches.contains(Path::new("/root/nested/beta.py")));
        assert!(!matches.contains(Path::new("/root/alpha.rs")));
    }

    #[test]
    fn depth_filter_prunes_deep_directories() {
        let query = ParsedQuery::parse("depth:0 ext:rs");
        let matches = matching_paths(&tree(), &query, None, 0);

        assert!(matches.contains(Path::new("/root/alpha.rs")));
        assert!(!matches.contains(Path::new("/root/nested")));
        assert!(!matches.contains(Path::new("/root/nested/gamma.rs")));
    }

    #[test]
    fn a_depth_base_measures_a_subtree_from_the_tree_root() {
        let nodes = tree();
        let query = ParsedQuery::parse("depth:0 ext:rs");
        let subtree = slice::from_ref(&nodes[0].children[1]);
        let matches = matching_paths(subtree, &query, None, 1);

        assert!(!matches.contains(Path::new("/root/nested/gamma.rs")));
    }

    #[test]
    fn directory_type_filter_keeps_directories_only() {
        let query = ParsedQuery::parse("type:dir nested");
        let matches = matching_paths(&tree(), &query, None, 0);

        assert!(matches.contains(Path::new("/root/nested")));
        assert!(!matches.contains(Path::new("/root/nested/beta.py")));
    }

    #[test]
    fn gather_returns_checked_files_that_match() {
        let mut nodes = tree();

        selection::set_all_checked(&mut nodes, true);

        let mut paths = gather(&nodes, "ext:rs");

        paths.sort();

        assert_eq!(paths.len(), 2);
        assert!(paths[0].ends_with("alpha.rs"));
        assert!(paths[1].ends_with("gamma.rs"));
    }

    #[test]
    fn gather_skips_unchecked_files() {
        assert_eq!(gather(&tree(), ""), Vec::<PathBuf>::new());
    }

    #[test]
    fn gather_is_deterministic() {
        let mut nodes = tree();

        selection::set_all_checked(&mut nodes, true);

        assert_eq!(gather(&nodes, ""), gather(&nodes, ""));
    }

    #[test]
    fn a_precomputed_set_replaces_the_query() {
        let mut nodes = tree();

        selection::set_all_checked(&mut nodes, true);

        let query = ParsedQuery::parse("ext:rs");

        let set: FxHashSet<PathBuf> =
            core::iter::once(PathBuf::from("/root/nested/gamma.rs")).collect();

        let paths = gather_checked_paths(&nodes, &Matcher::Set(&set), &query);

        assert_eq!(paths, vec![PathBuf::from("/root/nested/gamma.rs")]);
    }

    #[test]
    fn filter_selected_keeps_only_selected_matches() {
        let mut nodes = tree();

        nodes[0].children[1].children[1].checked = true;

        let query = ParsedQuery::parse("ext:rs");
        let matcher = Matcher::Query(MatchContext::new(&query, None));
        let filtered = filter_selected(&nodes, &matcher, &query);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].children.len(), 1);
        assert_eq!(filtered[0].children[0].children.len(), 1);
        assert!(filtered[0].children[0].children[0].path.ends_with("gamma.rs"));
    }

    #[test]
    fn filter_selected_drops_everything_when_nothing_is_selected() {
        let query = ParsedQuery::default();
        let matcher = Matcher::Query(MatchContext::new(&query, None));

        assert_eq!(filter_selected(&tree(), &matcher, &query).len(), 0);
    }

    #[test]
    fn path_matches_composes_the_lowercase_forms_itself() {
        let query = ParsedQuery::parse("ext:rs path:nested");
        let context = MatchContext::new(&query, None);
        let mut scratch = SearchScratch::default();

        assert!(context.path_matches(Path::new("/Root/Nested/Gamma.RS"), &mut scratch));
        assert!(!context.path_matches(Path::new("/root/other/gamma.rs"), &mut scratch));
        assert!(!context.path_matches(Path::new("/root/nested/beta.py"), &mut scratch));
    }

    #[test]
    fn path_matches_admits_everything_under_an_empty_query() {
        let query = ParsedQuery::default();
        let context = MatchContext::new(&query, None);
        let mut scratch = SearchScratch::default();

        assert!(context.path_matches(Path::new("/root/anything.txt"), &mut scratch));
    }

    #[test]
    fn lowercase_child_does_not_double_the_separator() {
        let mut text = String::from("/");

        append_lowercase_child(&mut text, "usr");

        assert_eq!(text, "/usr");
    }

    #[test]
    fn lowercase_paths_lower_only_the_appended_text() {
        let mut text = String::from("KEEP");

        append_lowercase_path(&mut text, Path::new("/A/B"));

        assert_eq!(text, "KEEP/a/b");
    }
}

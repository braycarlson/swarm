use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::model::error::SwarmResult;
use crate::services::tree::loader;

pub trait TreeOperations {
    fn load_children(&mut self, options: &Options) -> SwarmResult<bool>;
    fn load_all_children(&mut self, options: &Options) -> SwarmResult<bool>;
    fn matches_search(&self, query: &str) -> bool;
    fn propagate_checked(&mut self, checked: bool);
    fn refresh(&mut self, options: &Options) -> SwarmResult<bool>;
}

impl TreeOperations for FileNode {
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

        let lowercase_query = query.to_lowercase();
        let name = self.lowercase_name();

        if name.contains(&lowercase_query) {
            return true;
        }

        if self.is_directory() {
            self.children.iter().any(|child| child.matches_search(query))
        } else {
            false
        }
    }

    fn propagate_checked(&mut self, checked: bool) {
        self.checked = checked;

        if self.is_directory() {
            for child in &mut self.children {
                child.propagate_checked(checked);
            }
        }
    }

    fn refresh(&mut self, options: &Options) -> SwarmResult<bool> {
        loader::refresh_node(self, options)
    }
}

pub fn should_show_node(node: &FileNode, search_query: &str) -> bool {
    if search_query.is_empty() {
        return true;
    }

    node.matches_search(search_query)
}

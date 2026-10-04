use alloc::borrow::Cow;
use std::path::{Path, PathBuf};

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::model::query::ParsedQuery;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SearchModel {
    active: bool,
    #[serde(skip)]
    matching_paths: Option<FxHashSet<PathBuf>>,
    #[serde(skip)]
    parsed_cache: Option<ParsedQuery>,
    query: String,
}

impl SearchModel {
    pub fn clear(&mut self) {
        self.active = false;
        self.matching_paths = None;
        self.parsed_cache = None;
        self.query.clear();

        debug_assert!(!self.has_query());
    }

    pub fn ensure_parsed(&mut self) {
        if self.parsed_cache.is_none() {
            if !self.query.is_empty() {
                self.parsed_cache = Some(ParsedQuery::parse(&self.query));
            }
        }

        debug_assert!(self.query.is_empty() || self.parsed_cache.is_some());
    }

    pub fn has_query(&self) -> bool {
        if !self.active {
            return false;
        }

        !self.query.is_empty()
    }

    pub fn is_path_matching(&self, path: &Path) -> Option<bool> {
        self.matching_paths
            .as_ref()
            .map(|paths| paths.contains(path))
    }

    pub fn matching_paths(&self) -> Option<&FxHashSet<PathBuf>> {
        self.matching_paths.as_ref()
    }

    pub fn parsed(&self) -> Cow<'_, ParsedQuery> {
        match self.parsed_cache.as_ref() {
            Some(cached) => Cow::Borrowed(cached),
            None => Cow::Owned(ParsedQuery::parse(&self.query)),
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_matching_paths(&mut self, matching_paths: Option<FxHashSet<PathBuf>>) {
        debug_assert!(self.has_query() || matching_paths.is_none());

        self.matching_paths = matching_paths;
    }

    pub fn set_query(&mut self, query: String) {
        self.parsed_cache = if query.is_empty() {
            None
        } else {
            Some(ParsedQuery::parse(&query))
        };

        self.query = query;
        self.active = true;
        self.matching_paths = None;

        debug_assert_ne!(self.parsed_cache.is_some(), self.query.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_parsed_populates_a_restored_query() {
        let mut model: SearchModel = serde_json::from_str(r#"{"query":"ext:rs","active":true}"#)
            .expect("the model deserializes");

        model.ensure_parsed();

        assert!(model.has_query());
        assert!(!model.parsed().is_empty());
    }

    #[test]
    fn clearing_forgets_the_query_and_the_matches() {
        let mut model = SearchModel::default();

        model.set_query("ext:rs".to_owned());
        model.set_matching_paths(Some(FxHashSet::default()));
        model.clear();

        assert!(!model.has_query());
        assert!(model.matching_paths().is_none());
        assert_eq!(model.query(), "");
    }

    #[test]
    fn an_empty_query_is_not_a_query() {
        let mut model = SearchModel::default();

        model.set_query(String::new());

        assert!(!model.has_query());
        assert!(model.parsed().is_empty());
    }
}

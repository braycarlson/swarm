use std::path::Path;
use std::sync::Arc;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::model::error::{SwarmError, SwarmResult};
use crate::model::options::Options;

pub trait PathFilter: Send + Sync {
    fn should_include(&self, path: &Path) -> bool;
}

pub struct GlobPathFilter {
    exclude_set: GlobSet,
    include_set: GlobSet,
}

impl GlobPathFilter {
    pub fn from_options(options: &Options) -> SwarmResult<Self> {
        let include_set = Self::build_globset(&options.include)?;
        let exclude_set = Self::build_globset(&options.exclude)?;

        Ok(Self {
            exclude_set,
            include_set,
        })
    }

    pub fn with_patterns(include: &[String], exclude: &[String]) -> SwarmResult<Self> {
        let include_set = Self::build_globset(include)?;
        let exclude_set = Self::build_globset(exclude)?;

        Ok(Self {
            exclude_set,
            include_set,
        })
    }

    pub fn include_set(&self) -> &GlobSet {
        &self.include_set
    }

    pub fn exclude_set(&self) -> &GlobSet {
        &self.exclude_set
    }

    fn build_globset(patterns: &[String]) -> SwarmResult<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            if pattern.trim().is_empty() {
                continue;
            }

            let glob = Glob::new(pattern)
                .map_err(|error| SwarmError::Parse(format!("Invalid glob pattern '{}': {}", pattern, error)))?;

            builder.add(glob);
        }

        builder.build()
            .map_err(|error| SwarmError::Other(format!("Failed to build globset: {}", error)))
    }

    fn is_path_excluded(&self, path: &Path) -> bool {
        if self.exclude_set.is_match(path) {
            return true;
        }

        let mut current = path;

        while let Some(parent) = current.parent() {
            if self.exclude_set.is_match(parent) {
                return true;
            }

            current = parent;
        }

        false
    }
}

impl PathFilter for GlobPathFilter {
    fn should_include(&self, path: &Path) -> bool {
        if path.is_dir() {
            return !self.is_path_excluded(path);
        }

        if crate::services::tree::filter::is_binary_file(path) {
            return false;
        }

        let include_empty = self.include_set.is_empty();

        if !include_empty {
            if !self.include_set.is_match(path) {
                return false;
            }

            if self.is_path_excluded(path) {
                return false;
            }

            return true;
        }

        !self.is_path_excluded(path)
    }
}

pub struct AlwaysIncludeFilter;

impl PathFilter for AlwaysIncludeFilter {
    fn should_include(&self, _path: &Path) -> bool {
        true
    }
}

pub struct CompositeFilter {
    filters: Vec<Box<dyn PathFilter>>,
}

impl CompositeFilter {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn add(mut self, filter: Box<dyn PathFilter>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl Default for CompositeFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl PathFilter for CompositeFilter {
    fn should_include(&self, path: &Path) -> bool {
        self.filters.iter().all(|f| f.should_include(path))
    }
}

pub struct ArcPathFilter {
    filter: Arc<dyn PathFilter>,
}

impl ArcPathFilter {
    pub fn new(filter: Arc<dyn PathFilter>) -> Self {
        Self { filter }
    }
}

impl PathFilter for ArcPathFilter {
    fn should_include(&self, path: &Path) -> bool {
        self.filter.should_include(path)
    }
}

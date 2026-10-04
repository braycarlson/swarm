use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::model::error::{SwarmError, SwarmResult};
use crate::model::options::Options;
use crate::model::path;

pub struct GlobPathFilter {
    exclude_set: GlobSet,
    include_set: GlobSet,
    show_hidden: bool,
}

impl GlobPathFilter {
    fn build_exclude_set(patterns: &[String]) -> SwarmResult<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            let trimmed = pattern.trim();

            if trimmed.is_empty() {
                continue;
            }

            builder.add(Self::compile(trimmed)?);
            builder.add(Self::compile(&format!("**/{trimmed}"))?);
            builder.add(Self::compile(&format!("**/{trimmed}/**"))?);
        }

        let set = builder.build().map_err(|error| {
            SwarmError::Other(format!("Failed to build the exclude set: {error}"))
        })?;

        debug_assert_eq!(set.len() % 3, 0);
        debug_assert!(set.len() <= patterns.len() * 3);

        Ok(set)
    }

    fn build_include_set(patterns: &[String]) -> SwarmResult<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            let trimmed = pattern.trim();

            if trimmed.is_empty() {
                continue;
            }

            builder.add(Self::compile(trimmed)?);
        }

        let set = builder.build().map_err(|error| {
            SwarmError::Other(format!("Failed to build the include set: {error}"))
        })?;

        debug_assert!(set.len() <= patterns.len());

        Ok(set)
    }

    fn compile(pattern: &str) -> SwarmResult<Glob> {
        Glob::new(pattern).map_err(|error| {
            SwarmError::Parse(format!("Invalid glob pattern '{pattern}': {error}"))
        })
    }

    pub fn admits_entry(&self, path: &Path, is_directory: bool) -> bool {
        if !self.show_hidden {
            if path::is_hidden(path) {
                return false;
            }
        }

        self.should_include(path, is_directory)
    }

    pub fn from_options(options: &Options) -> SwarmResult<Self> {
        Ok(Self {
            exclude_set: Self::build_exclude_set(&options.exclude)?,
            include_set: Self::build_include_set(&options.include)?,
            show_hidden: options.show_hidden,
        })
    }

    pub fn should_include(&self, path: &Path, is_directory: bool) -> bool {
        if self.exclude_set.is_match(path) {
            return false;
        }

        if is_directory {
            return true;
        }

        if self.include_set.is_empty() {
            return true;
        }

        if self.include_set.is_match(path) {
            return true;
        }

        path.file_name()
            .is_some_and(|name| self.include_set.is_match(Path::new(name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::output::OutputFormat;
    use crate::model::theme::Theme;

    struct Patterns<'patterns> {
        exclude: &'patterns [&'patterns str],
        include: &'patterns [&'patterns str],
        show_hidden: bool,
    }

    fn filter(patterns: &Patterns<'_>) -> GlobPathFilter {
        GlobPathFilter::from_options(&options_for(patterns)).expect("the patterns compile")
    }

    fn options_for(patterns: &Patterns<'_>) -> Options {
        Options {
            delete_sessions_on_exit: false,
            exclude: patterns
                .exclude
                .iter()
                .map(|pattern| (*pattern).to_owned())
                .collect(),
            include: patterns
                .include
                .iter()
                .map(|pattern| (*pattern).to_owned())
                .collect(),
            output_format: OutputFormat::default(),
            show_hidden: patterns.show_hidden,
            single_instance: true,
            theme: Theme::default(),
            ui_scale: None,
            use_icon: false,
        }
    }

    #[test]
    fn a_named_component_excludes_the_whole_subtree() {
        let filter = filter(&Patterns {
            exclude: &["node_modules"],
            include: &[],
            show_hidden: false,
        });

        assert!(!filter.should_include(Path::new("/a/node_modules"), true));
        assert!(!filter.should_include(Path::new("/a/node_modules/b/c.js"), false));
        assert!(filter.should_include(Path::new("/a/src/c.js"), false));
    }

    #[test]
    fn extension_patterns_exclude_anywhere() {
        let filter = filter(&Patterns {
            exclude: &["*.log"],
            include: &[],
            show_hidden: false,
        });

        assert!(!filter.should_include(Path::new("/a/b/trace.log"), false));
        assert!(filter.should_include(Path::new("/a/b/trace.txt"), false));
    }

    #[test]
    fn directories_bypass_the_include_set() {
        let filter = filter(&Patterns {
            exclude: &[],
            include: &["*.rs"],
            show_hidden: false,
        });

        assert!(filter.should_include(Path::new("/a/src"), true));
        assert!(filter.should_include(Path::new("/a/src/main.rs"), false));
        assert!(!filter.should_include(Path::new("/a/src/main.py"), false));
    }

    #[test]
    fn an_empty_include_set_admits_every_file() {
        let filter = filter(&Patterns {
            exclude: &[],
            include: &[],
            show_hidden: false,
        });

        assert!(filter.should_include(Path::new("/a/b.txt"), false));
    }

    #[test]
    fn hidden_entries_follow_the_option() {
        let hiding = filter(&Patterns {
            exclude: &[],
            include: &[],
            show_hidden: false,
        });

        assert!(!hiding.admits_entry(Path::new("/a/.env"), false));
        assert!(hiding.admits_entry(Path::new("/a/env"), false));

        let patterns = Patterns {
            exclude: &[],
            include: &[],
            show_hidden: true,
        };

        let showing =
            GlobPathFilter::from_options(&options_for(&patterns)).expect("the patterns compile");

        assert!(showing.admits_entry(Path::new("/a/.env"), false));
    }

    #[test]
    fn an_invalid_pattern_is_reported() {
        let patterns = Patterns {
            exclude: &["a["],
            include: &[],
            show_hidden: false,
        };

        assert!(GlobPathFilter::from_options(&options_for(&patterns)).is_err());
    }
}

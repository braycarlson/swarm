use std::path::Path;
use std::sync::Arc;

use ignore::WalkBuilder;

use crate::model::options::Options;
use crate::model::path::PathExtensions;
use crate::services::tree::filter;

pub struct WalkerBuilder;

impl WalkerBuilder {
    pub fn create(directory: &Path, options: Options) -> ignore::Walk {
        let options = Arc::new(options);
        let options_clone = Arc::clone(&options);

        WalkBuilder::new(directory)
            .follow_links(true)
            .hidden(true)
            .ignore(false)
            .git_global(false)
            .git_exclude(false)
            .git_ignore(false)
            .require_git(false)
            .filter_entry(move |entry| {
                let path = entry.path();

                if path.is_hidden() {
                    return false;
                }

                if entry.file_type().map_or(false, |file_type| file_type.is_dir()) {
                    return !filter::is_path_in_excluded_patterns(path, &options_clone.exclude);
                }

                filter::should_include_path(path, &options_clone)
            })
            .build()
    }
}

use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

use crate::model::OutputFormat;
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::options::Options;
use crate::model::path::PathExtensions;

#[derive(Clone)]
pub struct GatherService;

impl Default for GatherService {
    fn default() -> Self {
        Self::new()
    }
}

impl GatherService {
    pub fn new() -> Self {
        Self
    }

    pub fn gather(&self, paths: &[String], options: &Options) -> SwarmResult<(String, usize)> {
        let include_set = Arc::new(self.build_globset(&options.include)?);
        let exclude_set = Arc::new(self.build_globset(&options.exclude)?);

        let mut files = Vec::new();

        for path_str in paths {
            let path = Path::new(path_str.trim());
            let clean_path = path.clean_path();

            if !clean_path.exists() {
                continue;
            }

            if clean_path.is_file() {
                self.collect_file(&clean_path, &mut files);
            } else if clean_path.is_dir() {
                self.collect_directory(&clean_path, &mut files, &include_set, &exclude_set, options)?;
            }
        }

        let output = self.format_output(&files, options.output_format)?;

        let total_lines: usize = output.lines().count();

        Ok((output, total_lines))
    }

    fn collect_file(&self, path: &Path, files: &mut Vec<(String, String)>) {
        if let Ok(content) = fs::read_to_string(path) {
            files.push((path.display().to_string(), content));
        }
    }

    fn collect_directory(
        &self,
        directory: &Path,
        files: &mut Vec<(String, String)>,
        include_set: &Arc<GlobSet>,
        exclude_set: &Arc<GlobSet>,
        options: &Options,
    ) -> SwarmResult<()> {
        let include_empty = options.include.is_empty();
        let walker = self.create_walker(directory, include_set, exclude_set, include_empty);

        for result in walker {
            let entry = result.map_err(|error| {
                SwarmError::Other(format!("Error reading directory {}: {}", directory.display(), error))
            })?;

            if entry.file_type().is_some_and(|file_type| file_type.is_file())
                && self.should_include_file(entry.path(), include_set, exclude_set, options) {
                    self.collect_file(entry.path(), files);
                }
        }

        Ok(())
    }

    fn format_output(&self, files: &[(String, String)], format: OutputFormat) -> SwarmResult<String> {
        match format {
            OutputFormat::PlainText => self.format_plain_text(files),
            OutputFormat::Markdown => self.format_markdown(files),
            OutputFormat::Json => self.format_json(files),
            OutputFormat::Xml => self.format_xml(files),
        }
    }

    fn format_plain_text(&self, files: &[(String, String)]) -> SwarmResult<String> {
        let mut output = String::new();

        for (path, content) in files {
            writeln!(output, "[{}]", path)
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "{}", content)
                .map_err(|e| SwarmError::Other(format!("Failed to write content: {}", e)))?;
        }

        Ok(output)
    }

    fn format_markdown(&self, files: &[(String, String)]) -> SwarmResult<String> {
        let mut output = String::new();

        for (path, content) in files {
            writeln!(output, "## {}\n", path)
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "```")
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "{}", content)
                .map_err(|e| SwarmError::Other(format!("Failed to write content: {}", e)))?;
            writeln!(output, "```\n")
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
        }

        Ok(output)
    }

    fn format_json(&self, files: &[(String, String)]) -> SwarmResult<String> {
        use std::collections::HashMap;

        let map: HashMap<&str, &str> = files.iter()
            .map(|(path, content)| (path.as_str(), content.as_str()))
            .collect();

        serde_json::to_string_pretty(&map)
            .map_err(|e| SwarmError::Other(format!("Failed to serialize JSON: {}", e)))
    }

    fn format_xml(&self, files: &[(String, String)]) -> SwarmResult<String> {
        let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<files>\n");

        for (path, content) in files {
            let escaped_path = Self::escape_xml(path);
            let escaped_content = Self::escape_xml(content);

            writeln!(output, "  <file>")
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "    <path>{}</path>", escaped_path)
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "    <content><![CDATA[{}]]></content>", escaped_content)
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "  </file>")
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
        }

        writeln!(output, "</files>")
            .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;

        Ok(output)
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn build_globset(&self, patterns: &[String]) -> SwarmResult<GlobSet> {
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

    fn is_path_excluded(&self, path: &Path, exclude_set: &GlobSet) -> bool {
        if exclude_set.is_match(path) {
            return true;
        }

        let mut current = path;

        while let Some(parent) = current.parent() {
            if exclude_set.is_match(parent) {
                return true;
            }

            current = parent;
        }

        false
    }

    fn create_walker(
        &self,
        directory: &Path,
        include_set: &Arc<GlobSet>,
        exclude_set: &Arc<GlobSet>,
        include_empty: bool,
    ) -> ignore::Walk {
        let include_clone = Arc::clone(include_set);
        let exclude_clone = Arc::clone(exclude_set);

        WalkBuilder::new(directory)
            .follow_links(true)
            .hidden(false)
            .ignore(false)
            .git_global(false)
            .git_exclude(false)
            .require_git(false)
            .filter_entry(move |entry| {
                let path = entry.path();

                if exclude_clone.is_match(path) {
                    return false;
                }

                let mut current = path;

                while let Some(parent) = current.parent() {
                    if exclude_clone.is_match(parent) {
                        return false;
                    }

                    current = parent;
                }

                let is_directory = entry.file_type().is_some_and(|file_type| file_type.is_dir());

                if is_directory {
                    return true;
                }

                if !include_empty && !include_clone.is_match(path) {
                    return false;
                }

                true
            })
            .build()
    }

    fn should_include_file(
        &self,
        path: &Path,
        include_set: &Arc<GlobSet>,
        exclude_set: &Arc<GlobSet>,
        options: &Options,
    ) -> bool {
        if path.is_dir() {
            return !self.is_path_excluded(path, exclude_set);
        }

        if crate::services::tree::filter::is_binary_file(path) {
            return false;
        }

        if !options.include.is_empty() {
            if !include_set.is_match(path) {
                return false;
            }

            if self.is_path_excluded(path, exclude_set) {
                return false;
            }

            return true;
        }

        !self.is_path_excluded(path, exclude_set)
    }
}

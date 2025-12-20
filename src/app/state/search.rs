use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::model::output::OutputFormat;
use crate::services::filesystem::git::GitStatus;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SearchModel {
    pub query: String,
    pub active: bool,
    #[serde(skip)]
    pub matching_paths: Option<HashSet<PathBuf>>,
}

impl SearchModel {
    pub fn has_query(&self) -> bool {
        !self.query.is_empty() && self.active
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.active = false;
        self.matching_paths = None;
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.active = true;
        self.matching_paths = None;
    }

    pub fn parsed(&self) -> ParsedQuery {
        ParsedQuery::parse(&self.query)
    }

    pub fn is_path_matching(&self, path: &Path) -> Option<bool> {
        self.matching_paths.as_ref().map(|paths| paths.contains(path))
    }
}

#[derive(Clone, Debug, Default)]
pub struct ParsedQuery {
    pub commands: Vec<Command>,
    pub contains: Vec<String>,
    pub content_patterns: Vec<String>,
    pub depth_max: Option<usize>,
    pub exact: Vec<String>,
    pub excludes: Vec<String>,
    pub extension_excludes: Vec<String>,
    pub extensions: Vec<String>,
    pub format_override: Option<OutputFormat>,
    pub git_excludes: Vec<GitFilter>,
    pub git_filters: Vec<GitFilter>,
    pub lines_max: Option<u64>,
    pub lines_min: Option<u64>,
    pub name_excludes: Vec<String>,
    pub names: Vec<String>,
    pub path_excludes: Vec<String>,
    pub paths: Vec<String>,
    pub recent_duration: Option<Duration>,
    pub size_max: Option<u64>,
    pub size_min: Option<u64>,
    pub type_filter: Option<TypeFilter>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Diff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeFilter {
    Directory,
    File,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitFilter {
    Added,
    Changed,
    Conflicted,
    Deleted,
    Modified,
    Renamed,
    Staged,
    Untracked,
}

impl GitFilter {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'a' => Some(Self::Added),
            'c' => Some(Self::Changed),
            'd' => Some(Self::Deleted),
            'm' => Some(Self::Modified),
            'r' => Some(Self::Renamed),
            's' => Some(Self::Staged),
            'u' => Some(Self::Untracked),
            'x' => Some(Self::Conflicted),
            _ => None,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "a" | "added" => Some(Self::Added),
            "c" | "changed" => Some(Self::Changed),
            "d" | "deleted" => Some(Self::Deleted),
            "m" | "modified" => Some(Self::Modified),
            "r" | "renamed" => Some(Self::Renamed),
            "s" | "staged" => Some(Self::Staged),
            "u" | "untracked" => Some(Self::Untracked),
            "x" | "conflicted" => Some(Self::Conflicted),
            _ => None,
        }
    }

    pub fn matches(&self, status: GitStatus) -> bool {
        match self {
            Self::Added => matches!(status, GitStatus::Added),
            Self::Changed => status.has_diff(),
            Self::Conflicted => matches!(status, GitStatus::Conflicted),
            Self::Deleted => matches!(status, GitStatus::Deleted),
            Self::Modified => matches!(status, GitStatus::Modified),
            Self::Renamed => matches!(status, GitStatus::Renamed),
            Self::Staged => matches!(status, GitStatus::Staged),
            Self::Untracked => matches!(status, GitStatus::Untracked),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FileMetadata {
    #[serde(skip)]
    pub content: Option<String>,
    pub lines: Option<u64>,
    pub modified: Option<u64>,
    pub size: u64,
}

impl FileMetadata {
    pub fn from_path(path: &Path, load_content: bool) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;

        let size = metadata.len();
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let (lines, content) = if load_content && metadata.is_file() {
            match std::fs::read_to_string(path) {
                Ok(text) => {
                    let line_count = text.lines().count() as u64;
                    (Some(line_count), Some(text))
                }
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };

        Some(Self {
            content,
            lines,
            modified,
            size,
        })
    }

    pub fn from_path_basic(path: &Path) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        Some(Self {
            content: None,
            lines: None,
            modified,
            size: metadata.len(),
        })
    }

    pub fn from_path_with_lines(path: &Path) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let lines = if metadata.is_file() {
            std::fs::read_to_string(path)
                .ok()
                .map(|text| text.lines().count() as u64)
        } else {
            None
        };

        Some(Self {
            content: None,
            lines,
            modified,
            size: metadata.len(),
        })
    }
}

impl ParsedQuery {
    pub fn is_expensive(&self) -> bool {
        self.needs_content()
    }

    pub fn parse(query: &str) -> Self {
        let mut result = Self::default();

        if query.trim().is_empty() {
            return result;
        }

        let tokens = Self::tokenize(query);

        for token in tokens {
            Self::process_token(&mut result, &token);
        }

        result
    }

    fn tokenize(query: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for c in query.chars() {
            match c {
                '"' => {
                    if in_quotes {
                        if !current.is_empty() {
                            tokens.push(format!("\"{}\"", current));
                            current.clear();
                        }
                        in_quotes = false;
                    } else {
                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                        in_quotes = true;
                    }
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.is_empty() {
            if in_quotes {
                tokens.push(format!("\"{}\"", current));
            } else {
                tokens.push(current);
            }
        }

        tokens
    }

    fn process_token(result: &mut ParsedQuery, token: &str) {
        let token = token.trim();

        if token.is_empty() {
            return;
        }

        if let Some(cmd) = token.strip_prefix("--") {
            Self::process_command(result, cmd);
            return;
        }

        if token.starts_with('"') && token.ends_with('"') && token.len() > 2 {
            let inner = &token[1..token.len() - 1];
            result.exact.push(inner.to_lowercase());
            return;
        }

        let (is_exclude, token) = if let Some(stripped) = token.strip_prefix('-') {
            (true, stripped)
        } else if let Some(stripped) = token.strip_prefix('!') {
            (true, stripped)
        } else {
            (false, token)
        };

        if is_exclude && !token.contains(':') {
            result.excludes.push(token.to_lowercase());
            return;
        }

        if let Some((key, value)) = token.split_once(':') {
            let key = key.to_lowercase();

            match key.as_str() {
                "ext" | "e" => {
                    for part in value.split(',') {
                        let ext = part.trim().to_lowercase();
                        let ext = ext.trim_start_matches('.');
                        if !ext.is_empty() {
                            if is_exclude {
                                result.extension_excludes.push(ext.to_string());
                            } else {
                                result.extensions.push(ext.to_string());
                            }
                        }
                    }
                }
                "name" | "n" => {
                    for part in value.split(',') {
                        let name = part.trim().to_lowercase();
                        if !name.is_empty() {
                            if is_exclude {
                                result.name_excludes.push(name);
                            } else {
                                result.names.push(name);
                            }
                        }
                    }
                }
                "path" | "p" => {
                    for part in value.split(',') {
                        let path = part.trim().to_lowercase();
                        if !path.is_empty() {
                            if is_exclude {
                                result.path_excludes.push(path);
                            } else {
                                result.paths.push(path);
                            }
                        }
                    }
                }
                "git" | "g" => {
                    Self::parse_git_filter(result, value, is_exclude);
                }
                "type" | "t" => {
                    result.type_filter = Self::parse_type_filter(value);
                }
                "size" | "s" => {
                    Self::parse_size_filter(result, value);
                }
                "lines" | "l" => {
                    Self::parse_lines_filter(result, value);
                }
                "depth" | "d" => {
                    if let Some(rest) = value.strip_prefix("<=") {
                        result.depth_max = rest.trim().parse().ok();
                    } else if let Some(rest) = value.strip_prefix('<') {
                        result.depth_max = rest.trim().parse().ok().map(|n: usize| n.saturating_sub(1));
                    } else {
                        result.depth_max = value.trim().parse().ok();
                    }
                }
                "recent" | "r" => {
                    result.recent_duration = Self::parse_duration(value);
                }
                "content" | "c" => {
                    if !value.is_empty() {
                        result.content_patterns.push(value.to_string());
                    }
                }
                _ => {
                    result.contains.push(token.to_lowercase());
                }
            }
            return;
        }

        result.contains.push(token.to_lowercase());
    }

    fn process_command(result: &mut ParsedQuery, cmd: &str) {
        match cmd.to_lowercase().as_str() {
            "diff" | "d" => {
                result.commands.push(Command::Diff);
            }
            "plain" | "plain-text" | "text" => {
                result.format_override = Some(OutputFormat::PlainText);
            }
            "markdown" | "md" => {
                result.format_override = Some(OutputFormat::Markdown);
            }
            "json" => {
                result.format_override = Some(OutputFormat::Json);
            }
            "xml" => {
                result.format_override = Some(OutputFormat::Xml);
            }
            _ => {}
        }
    }

    fn parse_git_filter(result: &mut ParsedQuery, value: &str, is_exclude: bool) {
        for part in value.split(',') {
            let part = part.trim();

            if let Some(filter) = GitFilter::from_str(part) {
                if is_exclude {
                    result.git_excludes.push(filter);
                } else {
                    result.git_filters.push(filter);
                }

                continue;
            }

            for c in part.chars() {
                if let Some(filter) = GitFilter::from_char(c) {
                    if is_exclude {
                        result.git_excludes.push(filter);
                    } else {
                        result.git_filters.push(filter);
                    }
                }
            }
        }
    }

    fn parse_type_filter(value: &str) -> Option<TypeFilter> {
        match value.trim().to_lowercase().as_str() {
            "file" | "f" => Some(TypeFilter::File),
            "dir" | "directory" | "d" | "folder" => Some(TypeFilter::Directory),
            _ => None,
        }
    }

    fn parse_size_filter(result: &mut ParsedQuery, value: &str) {
        let value = value.trim();

        if let Some(rest) = value.strip_prefix(">=") {
            result.size_min = Self::parse_size_unit(rest);
        } else if let Some(rest) = value.strip_prefix('>') {
            result.size_min = Self::parse_size_unit(rest).map(|n| n + 1);
        } else if let Some(rest) = value.strip_prefix("<=") {
            result.size_max = Self::parse_size_unit(rest);
        } else if let Some(rest) = value.strip_prefix('<') {
            result.size_max = Self::parse_size_unit(rest).map(|n| n.saturating_sub(1));
        } else if value.contains('-') {
            let parts: Vec<&str> = value.splitn(2, '-').collect();

            if parts.len() == 2 {
                result.size_min = Self::parse_size_unit(parts[0]);
                result.size_max = Self::parse_size_unit(parts[1]);
            }
        } else {
            let size = Self::parse_size_unit(value);
            result.size_min = size;
            result.size_max = size;
        }
    }

    fn parse_size_unit(value: &str) -> Option<u64> {
        let value = value.trim().to_lowercase();

        let (num_str, multiplier) = if value.ends_with("gb") {
            (&value[..value.len() - 2], 1024 * 1024 * 1024)
        } else if value.ends_with("mb") {
            (&value[..value.len() - 2], 1024 * 1024)
        } else if value.ends_with("kb") {
            (&value[..value.len() - 2], 1024)
        } else if value.ends_with('b') {
            (&value[..value.len() - 1], 1)
        } else {
            (value.as_str(), 1)
        };

        num_str.trim().parse::<u64>().ok().map(|n| n * multiplier)
    }

    fn parse_lines_filter(result: &mut ParsedQuery, value: &str) {
        let value = value.trim();

        if let Some(rest) = value.strip_prefix(">=") {
            result.lines_min = rest.trim().parse().ok();
        } else if let Some(rest) = value.strip_prefix('>') {
            result.lines_min = rest.trim().parse::<u64>().ok().map(|n| n + 1);
        } else if let Some(rest) = value.strip_prefix("<=") {
            result.lines_max = rest.trim().parse().ok();
        } else if let Some(rest) = value.strip_prefix('<') {
            result.lines_max = rest.trim().parse::<u64>().ok().map(|n| n.saturating_sub(1));
        } else if value.contains('-') {
            let parts: Vec<&str> = value.splitn(2, '-').collect();

            if parts.len() == 2 {
                result.lines_min = parts[0].trim().parse().ok();
                result.lines_max = parts[1].trim().parse().ok();
            }
        } else {
            let lines = value.parse().ok();
            result.lines_min = lines;
            result.lines_max = lines;
        }
    }

    fn parse_duration(value: &str) -> Option<Duration> {
        let value = value.trim().to_lowercase();

        if value == "today" {
            return Some(Duration::from_secs(24 * 60 * 60));
        }

        let (num_str, multiplier) = if value.ends_with('w') {
            (&value[..value.len() - 1], 7 * 24 * 60 * 60)
        } else if value.ends_with('d') {
            (&value[..value.len() - 1], 24 * 60 * 60)
        } else if value.ends_with('h') {
            (&value[..value.len() - 1], 60 * 60)
        } else if value.ends_with('m') {
            (&value[..value.len() - 1], 60)
        } else {
            return None;
        };

        num_str
            .trim()
            .parse::<u64>()
            .ok()
            .map(|n| Duration::from_secs(n * multiplier))
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
            && self.contains.is_empty()
            && self.content_patterns.is_empty()
            && self.depth_max.is_none()
            && self.exact.is_empty()
            && self.excludes.is_empty()
            && self.extension_excludes.is_empty()
            && self.extensions.is_empty()
            && self.format_override.is_none()
            && self.git_excludes.is_empty()
            && self.git_filters.is_empty()
            && self.lines_max.is_none()
            && self.lines_min.is_none()
            && self.name_excludes.is_empty()
            && self.names.is_empty()
            && self.path_excludes.is_empty()
            && self.paths.is_empty()
            && self.recent_duration.is_none()
            && self.size_max.is_none()
            && self.size_min.is_none()
            && self.type_filter.is_none()
    }

    pub fn has_depth_filter(&self) -> bool {
        self.depth_max.is_some()
    }

    pub fn has_git_filter(&self) -> bool {
        !self.git_filters.is_empty() || !self.git_excludes.is_empty()
    }

    pub fn has_command(&self, cmd: Command) -> bool {
        self.commands.contains(&cmd)
    }

    pub fn needs_size(&self) -> bool {
        self.size_min.is_some() || self.size_max.is_some()
    }

    pub fn needs_lines(&self) -> bool {
        self.lines_min.is_some() || self.lines_max.is_some()
    }

    pub fn needs_modified(&self) -> bool {
        self.recent_duration.is_some()
    }

    pub fn needs_content(&self) -> bool {
        !self.content_patterns.is_empty()
    }

    pub fn needs_metadata(&self) -> bool {
        self.needs_size() || self.needs_lines() || self.needs_modified() || self.needs_content()
    }

    pub fn requires_file_match(&self) -> bool {
        !self.extensions.is_empty()
            || !self.extension_excludes.is_empty()
            || self.needs_size()
            || self.needs_lines()
            || self.needs_content()
            || self.needs_modified()
            || self.has_git_filter()
            || matches!(self.type_filter, Some(TypeFilter::File))
    }

    pub fn matches(&self, filename: &str, filepath: &str) -> bool {
        self.matches_with_git(filename, filepath, None)
    }

    pub fn matches_with_git(
        &self,
        filename: &str,
        filepath: &str,
        git_status: Option<GitStatus>,
    ) -> bool {
        self.matches_full(filename, filepath, git_status, false, None)
    }

    pub fn matches_full(
        &self,
        filename: &str,
        filepath: &str,
        git_status: Option<GitStatus>,
        is_directory: bool,
        metadata: Option<&FileMetadata>,
    ) -> bool {
        if self.is_empty() {
            return true;
        }

        let name_lower = filename.to_lowercase();
        let path_lower = filepath.to_lowercase();

        for exclude in &self.excludes {
            if name_lower.contains(exclude) || path_lower.contains(exclude) {
                return false;
            }
        }

        for exclude in &self.name_excludes {
            if name_lower.contains(exclude) {
                return false;
            }
        }

        for exclude in &self.path_excludes {
            if path_lower.contains(exclude) {
                return false;
            }
        }

        for exclude in &self.extension_excludes {
            if name_lower.ends_with(&format!(".{}", exclude)) {
                return false;
            }
        }

        if !self.git_excludes.is_empty() {
            if let Some(status) = git_status {
                for filter in &self.git_excludes {
                    if filter.matches(status) {
                        return false;
                    }
                }
            }
        }

        if !self.git_filters.is_empty() {
            if let Some(status) = git_status {
                let matches_git = self.git_filters.iter().any(|f| f.matches(status));

                if !matches_git {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(type_filter) = self.type_filter {
            let matches_type = match type_filter {
                TypeFilter::Directory => is_directory,
                TypeFilter::File => !is_directory,
            };

            if !matches_type {
                return false;
            }
        }

        if !is_directory {
            if let Some(meta) = metadata {
                if let Some(min) = self.size_min {
                    if meta.size < min {
                        return false;
                    }
                }

                if let Some(max) = self.size_max {
                    if meta.size > max {
                        return false;
                    }
                }

                if self.needs_lines() {
                    if let Some(lines) = meta.lines {
                        if let Some(min) = self.lines_min {
                            if lines < min {
                                return false;
                            }
                        }

                        if let Some(max) = self.lines_max {
                            if lines > max {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }

                if let Some(duration) = self.recent_duration {
                    if let Some(modified) = meta.modified {
                        let now = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);

                        let threshold = now.saturating_sub(duration.as_secs());

                        if modified < threshold {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                if !self.content_patterns.is_empty() {
                    if let Some(content) = &meta.content {
                        let content_lower = content.to_lowercase();

                        for pattern in &self.content_patterns {
                            if !content_lower.contains(&pattern.to_lowercase()) {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }
            } else if self.needs_metadata() {
                return false;
            }
        }

        if !self.exact.is_empty() {
            let has_exact = self.exact.iter().any(|exact| name_lower == *exact);

            if has_exact {
                return true;
            }
        }

        if !self.extensions.is_empty() {
            let has_extension = self.extensions.iter().any(|ext| {
                name_lower.ends_with(&format!(".{}", ext))
            });

            if !has_extension {
                return false;
            }
        }

        if !self.names.is_empty() {
            let has_name = self.names.iter().all(|name| name_lower.contains(name));

            if !has_name {
                return false;
            }
        }

        if !self.paths.is_empty() {
            let has_path = self.paths.iter().all(|path| path_lower.contains(path));

            if !has_path {
                return false;
            }
        }

        if !self.contains.is_empty() {
            for term in &self.contains {
                if !name_lower.contains(term) && !path_lower.contains(term) {
                    return false;
                }
            }
        }

        true
    }

    pub fn matches_depth(&self, depth: usize) -> bool {
        match self.depth_max {
            Some(max) => depth <= max,
            None => true,
        }
    }
}

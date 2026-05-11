use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::model::output::OutputFormat;
use crate::services::filesystem::git::GitStatus;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SearchModel {
    pub query: String,
    pub active: bool,
    #[serde(skip)]
    pub matching_paths: Option<FxHashSet<PathBuf>>,
    #[serde(skip)]
    parsed_cache: Option<ParsedQuery>,
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
        self.parsed_cache = None;
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
    }

    pub fn parsed(&self) -> Cow<'_, ParsedQuery> {
        match self.parsed_cache.as_ref() {
            Some(cached) => Cow::Borrowed(cached),
            None if self.query.is_empty() => Cow::Owned(ParsedQuery::default()),
            None => Cow::Owned(ParsedQuery::parse(&self.query)),
        }
    }

    pub fn ensure_parsed(&mut self) {
        if self.parsed_cache.is_none() && !self.query.is_empty() {
            self.parsed_cache = Some(ParsedQuery::parse(&self.query));
        }
    }

    pub fn is_path_matching(&self, path: &Path) -> Option<bool> {
        self.matching_paths.as_ref().map(|paths| paths.contains(path))
    }
}

#[derive(Clone, Debug, Default)]
pub struct ParsedQuery {
    pub commands: Vec<SearchCommand>,
    pub contains: Vec<String>,
    pub depth_max: Option<usize>,
    pub duration_recent: Option<Duration>,
    pub exact: Vec<String>,
    pub excludes: Vec<String>,
    pub excludes_extension: Vec<String>,
    pub excludes_git: Vec<GitFilter>,
    pub excludes_name: Vec<String>,
    pub excludes_path: Vec<String>,
    pub extensions: Vec<String>,
    pub filter_type: Option<TypeFilter>,
    pub filters_git: Vec<GitFilter>,
    pub format_override: Option<OutputFormat>,
    pub lines_max: Option<u64>,
    pub lines_min: Option<u64>,
    pub names: Vec<String>,
    pub paths: Vec<String>,
    pub patterns_content: Vec<String>,
    pub patterns_symbol: Vec<String>,
    pub size_max: Option<u64>,
    pub size_min: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeFilter {
    Directory,
    File,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchCommand {
    Diff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitFilter {
    Added,
    Conflicted,
    Deleted,
    Modified,
    Renamed,
    Staged,
    Untracked,
}

impl GitFilter {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "added" | "new" => Some(Self::Added),
            "conflicted" => Some(Self::Conflicted),
            "deleted" | "removed" => Some(Self::Deleted),
            "modified" | "changed" => Some(Self::Modified),
            "renamed" => Some(Self::Renamed),
            "staged" => Some(Self::Staged),
            "untracked" => Some(Self::Untracked),
            _ => None,
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'a' | 'A' => Some(Self::Added),
            'c' | 'C' => Some(Self::Conflicted),
            'd' | 'D' => Some(Self::Deleted),
            'm' | 'M' => Some(Self::Modified),
            'r' | 'R' => Some(Self::Renamed),
            's' | 'S' => Some(Self::Staged),
            'u' | 'U' | '?' => Some(Self::Untracked),
            _ => None,
        }
    }

    pub fn matches(&self, status: GitStatus) -> bool {
        matches!(
            (self, status),
            (Self::Added, GitStatus::Added)
                | (Self::Conflicted, GitStatus::Conflicted)
                | (Self::Deleted, GitStatus::Deleted)
                | (Self::Modified, GitStatus::Modified)
                | (Self::Renamed, GitStatus::Renamed)
                | (Self::Staged, GitStatus::Staged)
                | (Self::Untracked, GitStatus::Untracked)
        )
    }
}

#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub content: Option<Arc<str>>,
    pub lines: Option<u64>,
    pub modified: Option<u64>,
    pub size: u64,
}

impl FileMetadata {
    pub fn from_path(path: &Path, load_content: bool) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let (lines, content) = if load_content && metadata.is_file() {
            match std::fs::read_to_string(path) {
                Ok(text) => {
                    let line_count = line_count_string(&text);
                    (Some(line_count), Some(Arc::<str>::from(text)))
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
            size: metadata.len(),
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
            line_count_fast(path)
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

fn line_count_string(s: &str) -> u64 {
    if s.is_empty() {
        return 0;
    }

    let bytes = s.as_bytes();
    let mut count = memchr::memchr_iter(b'\n', bytes).count() as u64;

    if *bytes.last().unwrap() != b'\n' {
        count += 1;
    }

    count
}

fn line_count_fast(path: &Path) -> Option<u64> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).ok()?;
    let mut buffer = [0u8; 64 * 1024];
    let mut count: u64 = 0;
    let mut last_byte: u8 = b'\n';
    let mut any_bytes = false;

    loop {
        let n = file.read(&mut buffer).ok()?;

        if n == 0 {
            break;
        }

        any_bytes = true;
        count += memchr::memchr_iter(b'\n', &buffer[..n]).count() as u64;
        last_byte = buffer[n - 1];
    }

    if any_bytes && last_byte != b'\n' {
        count += 1;
    }

    Some(count)
}

enum Token {
    Plain(String),
    Exact(String),
}

impl ParsedQuery {
    pub fn is_expensive(&self) -> bool {
        !self.patterns_content.is_empty() || !self.patterns_symbol.is_empty()
    }

    pub fn parse(query: &str) -> Self {
        let mut result = Self::default();

        if query.trim().is_empty() {
            return result;
        }

        let tokens = Self::tokenize(query);

        for token in tokens {
            Self::process_token(&mut result, token);
        }

        result
    }

    fn tokenize(query: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for character in query.chars() {
            match character {
                '"' => {
                    if in_quotes {
                        if !current.is_empty() {
                            tokens.push(Token::Exact(std::mem::take(&mut current)));
                        }
                        in_quotes = false;
                    } else {
                        if !current.is_empty() {
                            tokens.push(Token::Plain(std::mem::take(&mut current)));
                        }
                        in_quotes = true;
                    }
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        tokens.push(Token::Plain(std::mem::take(&mut current)));
                    }
                }
                _ => {
                    current.push(character);
                }
            }
        }

        if !current.is_empty() {
            if in_quotes {
                tokens.push(Token::Exact(current));
            } else {
                tokens.push(Token::Plain(current));
            }
        }

        tokens
    }

    fn process_token(result: &mut ParsedQuery, token: Token) {
        match token {
            Token::Exact(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return;
                }
                result.exact.push(trimmed.to_ascii_lowercase());
            }
            Token::Plain(s) => {
                let token = s.trim();
                if token.is_empty() {
                    return;
                }
                Self::process_plain_token(result, token);
            }
        }
    }

    fn process_plain_token(result: &mut ParsedQuery, token: &str) {
        if let Some(command) = token.strip_prefix("--") {
            Self::process_command(result, command);
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
            result.excludes.push(token.to_ascii_lowercase());
            return;
        }

        if let Some((key, value)) = token.split_once(':') {
            let key = key.to_ascii_lowercase();

            match key.as_str() {
                "ext" | "e" => {
                    for part in value.split(',') {
                        let extension = part.trim().to_ascii_lowercase();
                        let extension = extension.trim_start_matches('.');
                        if !extension.is_empty() {
                            if is_exclude {
                                result.excludes_extension.push(extension.to_string());
                            } else {
                                result.extensions.push(extension.to_string());
                            }
                        }
                    }
                }
                "name" | "n" => {
                    for part in value.split(',') {
                        let name = part.trim().to_ascii_lowercase();
                        if !name.is_empty() {
                            if is_exclude {
                                result.excludes_name.push(name);
                            } else {
                                result.names.push(name);
                            }
                        }
                    }
                }
                "path" | "p" => {
                    for part in value.split(',') {
                        let path = part.trim().to_ascii_lowercase();
                        if !path.is_empty() {
                            if is_exclude {
                                result.excludes_path.push(path);
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
                    result.filter_type = Self::parse_type_filter(value);
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
                    result.duration_recent = Self::parse_duration(value);
                }
                "content" | "c" => {
                    if !value.is_empty() {
                        result.patterns_content.push(value.to_ascii_lowercase());
                    }
                }
                "sym" | "symbol" | "def" | "fn" | "func" | "struct" | "class" => {
                    if !value.is_empty() {
                        result.patterns_symbol.push(value.to_ascii_lowercase());
                    }
                }
                _ => {
                    result.contains.push(token.to_ascii_lowercase());
                }
            }
            return;
        }

        result.contains.push(token.to_ascii_lowercase());
    }

    fn process_command(result: &mut ParsedQuery, command: &str) {
        let command_lower = command.to_ascii_lowercase();

        match command_lower.as_str() {
            "diff" | "d" => {
                result.commands.push(SearchCommand::Diff);
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
                    result.excludes_git.push(filter);
                } else {
                    result.filters_git.push(filter);
                }

                continue;
            }

            for character in part.chars() {
                if let Some(filter) = GitFilter::from_char(character) {
                    if is_exclude {
                        result.excludes_git.push(filter);
                    } else {
                        result.filters_git.push(filter);
                    }
                }
            }
        }
    }

    fn parse_type_filter(value: &str) -> Option<TypeFilter> {
        match value.to_ascii_lowercase().as_str() {
            "dir" | "directory" | "d" | "folder" => Some(TypeFilter::Directory),
            "file" | "f" => Some(TypeFilter::File),
            _ => None,
        }
    }

    fn parse_size_filter(result: &mut ParsedQuery, value: &str) {
        let value = value.trim();

        if let Some(rest) = value.strip_prefix(">=") {
            result.size_min = Self::parse_size_value(rest.trim());
        } else if let Some(rest) = value.strip_prefix('>') {
            result.size_min = Self::parse_size_value(rest.trim()).map(|n| n + 1);
        } else if let Some(rest) = value.strip_prefix("<=") {
            result.size_max = Self::parse_size_value(rest.trim());
        } else if let Some(rest) = value.strip_prefix('<') {
            result.size_max = Self::parse_size_value(rest.trim()).map(|n| n.saturating_sub(1));
        } else if value.contains('-') {
            let parts: Vec<&str> = value.splitn(2, '-').collect();

            if parts.len() == 2 {
                result.size_min = Self::parse_size_value(parts[0].trim());
                result.size_max = Self::parse_size_value(parts[1].trim());
            }
        } else {
            let size = Self::parse_size_value(value);
            result.size_min = size;
            result.size_max = size;
        }
    }

    fn parse_size_value(value: &str) -> Option<u64> {
        let value = value.trim().to_ascii_lowercase();

        let (number_string, multiplier) = if value.ends_with("gb") {
            (&value[..value.len() - 2], 1024 * 1024 * 1024)
        } else if value.ends_with("mb") {
            (&value[..value.len() - 2], 1024 * 1024)
        } else if value.ends_with("kb") {
            (&value[..value.len() - 2], 1024)
        } else if value.ends_with('g') {
            (&value[..value.len() - 1], 1024 * 1024 * 1024)
        } else if value.ends_with('m') {
            (&value[..value.len() - 1], 1024 * 1024)
        } else if value.ends_with('k') {
            (&value[..value.len() - 1], 1024)
        } else if value.ends_with('b') {
            (&value[..value.len() - 1], 1)
        } else {
            (value.as_str(), 1)
        };

        number_string.trim().parse::<u64>().ok().map(|n| n * multiplier)
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
        let value = value.trim().to_ascii_lowercase();

        if value == "today" {
            return Some(Duration::from_secs(24 * 60 * 60));
        }

        let (number_string, multiplier) = if value.ends_with('w') {
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

        number_string
            .trim()
            .parse::<u64>()
            .ok()
            .map(|n| Duration::from_secs(n * multiplier))
    }

    pub fn is_empty(&self) -> bool {
        self.contains.is_empty()
            && self.exact.is_empty()
            && self.excludes.is_empty()
            && self.extensions.is_empty()
            && self.excludes_extension.is_empty()
            && self.names.is_empty()
            && self.excludes_name.is_empty()
            && self.paths.is_empty()
            && self.excludes_path.is_empty()
            && self.filters_git.is_empty()
            && self.excludes_git.is_empty()
            && self.filter_type.is_none()
            && self.size_min.is_none()
            && self.size_max.is_none()
            && self.lines_min.is_none()
            && self.lines_max.is_none()
            && self.depth_max.is_none()
            && self.patterns_content.is_empty()
            && self.patterns_symbol.is_empty()
            && self.duration_recent.is_none()
    }

    pub fn has_command(&self, command: SearchCommand) -> bool {
        self.commands.contains(&command)
    }

    pub fn has_depth_filter(&self) -> bool {
        self.depth_max.is_some()
    }

    pub fn matches_depth(&self, depth: usize) -> bool {
        self.depth_max.map_or(true, |max| depth <= max)
    }

    pub fn needs_metadata(&self) -> bool {
        self.needs_lines()
            || self.needs_content()
            || self.size_min.is_some()
            || self.size_max.is_some()
            || self.duration_recent.is_some()
    }

    pub fn needs_lines(&self) -> bool {
        self.lines_min.is_some() || self.lines_max.is_some()
    }

    pub fn needs_content(&self) -> bool {
        false
    }

    pub fn requires_file_match(&self) -> bool {
        matches!(self.filter_type, Some(TypeFilter::File))
    }

    pub fn matches(&self, file_name: &str, file_path: &str) -> bool {
        self.matches_with_git(file_name, file_path, None)
    }

    pub fn matches_with_git(
        &self,
        file_name: &str,
        file_path: &str,
        git_status: Option<GitStatus>,
    ) -> bool {
        let name_lowercase = file_name.to_ascii_lowercase();
        let lowercase_path = file_path.to_ascii_lowercase();
        self.matches_full(&name_lowercase, &lowercase_path, git_status, false, None)
    }

    pub fn matches_full(
        &self,
        name_lowercase: &str,
        lowercase_path: &str,
        git_status: Option<GitStatus>,
        is_directory: bool,
        metadata: Option<&FileMetadata>,
    ) -> bool {
        if self.is_empty() {
            return true;
        }

        for exclude in &self.excludes {
            if name_lowercase.contains(exclude.as_str()) || lowercase_path.contains(exclude.as_str()) {
                return false;
            }
        }

        for exclude in &self.excludes_name {
            if name_lowercase.contains(exclude.as_str()) {
                return false;
            }
        }

        for exclude in &self.excludes_path {
            if lowercase_path.contains(exclude.as_str()) {
                return false;
            }
        }

        for exclude in &self.excludes_extension {
            if has_extension(name_lowercase, exclude) {
                return false;
            }
        }

        if !self.excludes_git.is_empty() {
            if let Some(status) = git_status {
                for filter in &self.excludes_git {
                    if filter.matches(status) {
                        return false;
                    }
                }
            }
        }

        if !self.filters_git.is_empty() {
            if let Some(status) = git_status {
                let matches_git = self.filters_git.iter().any(|f| f.matches(status));

                if !matches_git {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(type_filter) = self.filter_type {
            let matches_type = match type_filter {
                TypeFilter::Directory => is_directory,
                TypeFilter::File => !is_directory,
            };

            if !matches_type {
                return false;
            }
        }

        if !is_directory {
            if let Some(metadata_item) = metadata {
                if let Some(min) = self.size_min {
                    if metadata_item.size < min {
                        return false;
                    }
                }

                if let Some(max) = self.size_max {
                    if metadata_item.size > max {
                        return false;
                    }
                }

                if let Some(min) = self.lines_min {
                    if let Some(lines) = metadata_item.lines {
                        if lines < min {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                if let Some(max) = self.lines_max {
                    if let Some(lines) = metadata_item.lines {
                        if lines > max {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                if let Some(duration) = self.duration_recent {
                    if let Some(modified) = metadata_item.modified {
                        let now = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        if now - modified > duration.as_secs() {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

            } else if self.size_min.is_some()
                || self.size_max.is_some()
                || self.lines_min.is_some()
                || self.lines_max.is_some()
                || self.duration_recent.is_some()
            {
                return false;
            }
        }

        if !self.extensions.is_empty() {
            let has_extension_match = self.extensions.iter().any(|extension| has_extension(name_lowercase, extension));

            if !has_extension_match {
                return false;
            }
        }

        if !self.names.is_empty() {
            let matches_name = self.names.iter().any(|n| name_lowercase.contains(n.as_str()));

            if !matches_name {
                return false;
            }
        }

        if !self.paths.is_empty() {
            let matches_path = self.paths.iter().any(|p| lowercase_path.contains(p.as_str()));

            if !matches_path {
                return false;
            }
        }

        if !self.exact.is_empty() {
            let matches_exact = self.exact.iter().any(|e| name_lowercase == *e);

            if !matches_exact {
                return false;
            }
        }

        if !self.contains.is_empty() {
            let matches_contains = self.contains.iter().all(|term| {
                name_lowercase.contains(term.as_str()) || lowercase_path.contains(term.as_str())
            });

            if !matches_contains {
                return false;
            }
        }

        true
    }
}

fn has_extension(name_lowercase: &str, extension: &str) -> bool {
    let needed = extension.len() + 1;

    if name_lowercase.len() < needed {
        return false;
    }

    let dot_position = name_lowercase.len() - needed;

    name_lowercase.as_bytes()[dot_position] == b'.' && &name_lowercase[dot_position + 1..] == extension
}

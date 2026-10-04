use core::time::Duration;

use crate::model::git::GitStatus;
use crate::model::metadata::FileMetadata;
use crate::model::output::OutputFormat;

pub const PATTERN_COUNT_MAX: usize = 64;
const BOUND_PREFIXES: [(&str, Bound); 4] = [
    (">=", Bound::AtLeast),
    (">", Bound::Above),
    ("<=", Bound::AtMost),
    ("<", Bound::Below),
];
const DURATION_UNITS: [(char, u64); 4] = [
    ('w', 7 * 24 * 60 * 60),
    ('d', 24 * 60 * 60),
    ('h', 60 * 60),
    ('m', 60),
];
const SIZE_UNITS: [(&str, u64); 7] = [
    ("gb", 1 << 30),
    ("mb", 1 << 20),
    ("kb", 1 << 10),
    ("g", 1 << 30),
    ("m", 1 << 20),
    ("k", 1 << 10),
    ("b", 1),
];

#[derive(Clone, Copy)]
enum Bound {
    Above,
    AtLeast,
    AtMost,
    Below,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Bounds {
    pub max: Option<u64>,
    pub min: Option<u64>,
}

impl Bounds {
    fn admits(self, value: u64) -> bool {
        if self.min.is_some_and(|min| value < min) {
            return false;
        }

        self.max.is_none_or(|max| value <= max)
    }

    fn admits_known(self, value: Option<u64>) -> bool {
        if !self.is_set() {
            return true;
        }

        value.is_some_and(|known| self.admits(known))
    }

    fn is_set(self) -> bool {
        self.max.is_some() || self.min.is_some()
    }
}

#[derive(Clone, Copy)]
enum FilterKey {
    Content,
    Depth,
    Lines,
    Recent,
    Size,
    Symbol,
    Type,
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
    fn from_letter(letter: char) -> Option<Self> {
        match letter.to_ascii_lowercase() {
            'a' => Some(Self::Added),
            'c' => Some(Self::Changed),
            'd' => Some(Self::Deleted),
            'm' => Some(Self::Modified),
            'r' => Some(Self::Renamed),
            's' => Some(Self::Staged),
            'u' | '?' => Some(Self::Untracked),
            'x' => Some(Self::Conflicted),
            _ => None,
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "added" | "new" => Some(Self::Added),
            "changed" => Some(Self::Changed),
            "conflicted" => Some(Self::Conflicted),
            "deleted" | "removed" => Some(Self::Deleted),
            "modified" => Some(Self::Modified),
            "renamed" => Some(Self::Renamed),
            "staged" => Some(Self::Staged),
            "untracked" => Some(Self::Untracked),
            _ => None,
        }
    }

    pub fn matches(self, status: GitStatus) -> bool {
        match self {
            Self::Added => status == GitStatus::Added,
            Self::Changed => status.has_diff(),
            Self::Conflicted => status == GitStatus::Conflicted,
            Self::Deleted => status == GitStatus::Deleted,
            Self::Modified => status == GitStatus::Modified,
            Self::Renamed => status == GitStatus::Renamed,
            Self::Staged => status == GitStatus::Staged,
            Self::Untracked => status == GitStatus::Untracked,
        }
    }
}

#[derive(Clone, Copy)]
enum Key {
    Filter(FilterKey),
    List(ListKey),
}

impl Key {
    fn from_name(name: &str) -> Option<Self> {
        let key = match name {
            "c" | "content" => Self::Filter(FilterKey::Content),
            "class" | "def" | "fn" | "func" | "struct" | "sym" | "symbol" => {
                Self::Filter(FilterKey::Symbol)
            }
            "d" | "depth" => Self::Filter(FilterKey::Depth),
            "e" | "ext" => Self::List(ListKey::Extension),
            "g" | "git" => Self::List(ListKey::Git),
            "l" | "lines" => Self::Filter(FilterKey::Lines),
            "n" | "name" => Self::List(ListKey::Name),
            "p" | "path" => Self::List(ListKey::Path),
            "r" | "recent" => Self::Filter(FilterKey::Recent),
            "s" | "size" => Self::Filter(FilterKey::Size),
            "t" | "type" => Self::Filter(FilterKey::Type),
            _ => return None,
        };

        Some(key)
    }
}

#[derive(Clone, Copy)]
enum ListKey {
    Extension,
    Git,
    Name,
    Path,
}

pub struct MatchTarget<'target> {
    pub git_status: Option<GitStatus>,
    pub is_directory: bool,
    pub metadata: Option<&'target FileMetadata>,
    pub name_lowercase: &'target str,
    pub path_lowercase: &'target str,
    pub seconds_now: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ParsedQuery {
    pub commands: Vec<SearchCommand>,
    pub contains: Vec<String>,
    pub depth_max: Option<u32>,
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
    pub lines: Bounds,
    pub names: Vec<String>,
    pub paths: Vec<String>,
    pub patterns_content: Vec<String>,
    pub patterns_symbol: Vec<String>,
    pub size: Bounds,
}

impl ParsedQuery {
    fn apply_command(&mut self, command: &str) {
        match command.to_ascii_lowercase().as_str() {
            "d" | "diff" => self.commands.push(SearchCommand::Diff),
            "json" => self.format_override = Some(OutputFormat::Json),
            "markdown" | "md" => self.format_override = Some(OutputFormat::Markdown),
            "plain" | "plain-text" | "text" => self.format_override = Some(OutputFormat::PlainText),
            "xml" => self.format_override = Some(OutputFormat::Xml),
            _ => {}
        }
    }

    fn apply_extension(&mut self, item: &str, exclude: bool) {
        let extension = item.trim_start_matches('.');

        if extension.is_empty() {
            return;
        }

        let target = if exclude {
            &mut self.excludes_extension
        } else {
            &mut self.extensions
        };

        target.push(format!(".{}", extension.to_ascii_lowercase()));
    }

    fn apply_filter(&mut self, key: FilterKey, value: &str) {
        match key {
            FilterKey::Content => push_pattern(&mut self.patterns_content, value),
            FilterKey::Depth => self.depth_max = parse_depth(value),
            FilterKey::Lines => apply_range(&mut self.lines, value, parse_count),
            FilterKey::Recent => self.duration_recent = parse_duration(value),
            FilterKey::Size => apply_range(&mut self.size, value, parse_size),
            FilterKey::Symbol => push_pattern(&mut self.patterns_symbol, value),
            FilterKey::Type => self.filter_type = parse_type(value),
        }
    }

    fn apply_git(&mut self, item: &str, exclude: bool) {
        let target = if exclude {
            &mut self.excludes_git
        } else {
            &mut self.filters_git
        };

        if let Some(filter) = GitFilter::from_name(item) {
            target.push(filter);

            return;
        }

        target.extend(item.chars().filter_map(GitFilter::from_letter));
    }

    fn apply_list(&mut self, key: ListKey, term: &Term<'_>) {
        for part in term.value.split(',') {
            let item = part.trim();

            if item.is_empty() {
                continue;
            }

            match key {
                ListKey::Extension => self.apply_extension(item, term.exclude),
                ListKey::Git => self.apply_git(item, term.exclude),
                ListKey::Name => {
                    let target = if term.exclude {
                        &mut self.excludes_name
                    } else {
                        &mut self.names
                    };

                    target.push(item.to_ascii_lowercase());
                }
                ListKey::Path => {
                    let target = if term.exclude {
                        &mut self.excludes_path
                    } else {
                        &mut self.paths
                    };

                    target.push(item.to_ascii_lowercase());
                }
            }
        }
    }

    fn apply_plain(&mut self, token: &str) {
        if let Some(command) = token.strip_prefix("--") {
            self.apply_command(command);

            return;
        }

        let stripped = token.strip_prefix('-').or_else(|| token.strip_prefix('!'));
        let exclude = stripped.is_some();
        let body = stripped.unwrap_or(token);

        if body.is_empty() {
            return;
        }

        let Some((key, value)) = body.split_once(':') else {
            self.apply_unknown(body, exclude);

            return;
        };

        let term = Term {
            body,
            exclude,
            key: key.to_ascii_lowercase(),
            value,
        };

        self.apply_term(&term);
    }

    fn apply_term(&mut self, term: &Term<'_>) {
        match Key::from_name(&term.key) {
            None => self.apply_unknown(term.body, term.exclude),
            Some(Key::List(key)) => self.apply_list(key, term),
            Some(Key::Filter(key)) => {
                if !term.exclude {
                    self.apply_filter(key, term.value);
                }
            }
        }
    }

    fn apply_token(&mut self, token: &Token) {
        let text = token.text.trim();

        if text.is_empty() {
            return;
        }

        if token.exact {
            self.exact.push(text.to_ascii_lowercase());

            return;
        }

        self.apply_plain(text);
    }

    fn apply_unknown(&mut self, body: &str, exclude: bool) {
        let target = if exclude {
            &mut self.excludes
        } else {
            &mut self.contains
        };

        target.push(body.to_ascii_lowercase());
    }

    fn matches_excludes(&self, target: &MatchTarget<'_>) -> bool {
        let path = target.path_lowercase;
        let name = target.name_lowercase;

        if self
            .excludes
            .iter()
            .any(|term| path.contains(term.as_str()))
        {
            return false;
        }

        if self
            .excludes_name
            .iter()
            .any(|term| name.contains(term.as_str()))
        {
            return false;
        }

        if self
            .excludes_path
            .iter()
            .any(|term| path.contains(term.as_str()))
        {
            return false;
        }

        !self
            .excludes_extension
            .iter()
            .any(|extension| name.ends_with(extension.as_str()))
    }

    fn matches_git(&self, target: &MatchTarget<'_>) -> bool {
        if let Some(status) = target.git_status {
            if self
                .excludes_git
                .iter()
                .any(|filter| filter.matches(status))
            {
                return false;
            }
        }

        if self.filters_git.is_empty() {
            return true;
        }

        target
            .git_status
            .is_some_and(|status| self.filters_git.iter().any(|filter| filter.matches(status)))
    }

    fn matches_includes(&self, target: &MatchTarget<'_>) -> bool {
        let path = target.path_lowercase;
        let name = target.name_lowercase;

        if !matches_any(&self.extensions, |extension| name.ends_with(extension)) {
            return false;
        }

        if !matches_any(&self.names, |term| name.contains(term)) {
            return false;
        }

        if !matches_any(&self.paths, |term| path.contains(term)) {
            return false;
        }

        if !matches_any(&self.exact, |exact| name == exact) {
            return false;
        }

        self.contains
            .iter()
            .all(|term| path.contains(term.as_str()))
    }

    fn matches_metadata(&self, target: &MatchTarget<'_>) -> bool {
        if target.is_directory {
            return true;
        }

        if !self.needs_metadata() {
            return true;
        }

        let Some(metadata) = target.metadata else {
            return false;
        };

        if !self.size.admits(metadata.size) {
            return false;
        }

        if !self.lines.admits_known(metadata.lines) {
            return false;
        }

        self.matches_recent(metadata, target.seconds_now)
    }

    fn matches_recent(&self, metadata: &FileMetadata, seconds_now: u64) -> bool {
        let Some(duration) = self.duration_recent else {
            return true;
        };

        metadata
            .modified
            .is_some_and(|modified| seconds_now.saturating_sub(modified) <= duration.as_secs())
    }

    fn matches_type(&self, target: &MatchTarget<'_>) -> bool {
        match self.filter_type {
            None => true,
            Some(TypeFilter::Directory) => target.is_directory,
            Some(TypeFilter::File) => !target.is_directory,
        }
    }

    pub fn has_command(&self, command: SearchCommand) -> bool {
        self.commands.contains(&command)
    }

    pub fn has_depth_filter(&self) -> bool {
        self.depth_max.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
            && self.contains.is_empty()
            && self.depth_max.is_none()
            && self.duration_recent.is_none()
            && self.exact.is_empty()
            && self.excludes.is_empty()
            && self.excludes_extension.is_empty()
            && self.excludes_git.is_empty()
            && self.excludes_name.is_empty()
            && self.excludes_path.is_empty()
            && self.extensions.is_empty()
            && self.filter_type.is_none()
            && self.filters_git.is_empty()
            && self.format_override.is_none()
            && !self.lines.is_set()
            && self.names.is_empty()
            && self.paths.is_empty()
            && self.patterns_content.is_empty()
            && self.patterns_symbol.is_empty()
            && !self.size.is_set()
    }

    pub fn is_expensive(&self) -> bool {
        !self.patterns_content.is_empty()
            || !self.patterns_symbol.is_empty()
            || self.needs_metadata()
    }

    pub fn matches(&self, target: &MatchTarget<'_>) -> bool {
        debug_assert!(target.path_lowercase.len() >= target.name_lowercase.len());

        if !self.matches_excludes(target) {
            return false;
        }

        if !self.matches_git(target) {
            return false;
        }

        if !self.matches_type(target) {
            return false;
        }

        if !self.matches_metadata(target) {
            return false;
        }

        self.matches_includes(target)
    }

    pub fn matches_depth(&self, depth: u32) -> bool {
        self.depth_max.is_none_or(|max| depth <= max)
    }

    pub fn needs_lines(&self) -> bool {
        self.lines.is_set()
    }

    pub fn needs_metadata(&self) -> bool {
        self.needs_lines() || self.size.is_set() || self.duration_recent.is_some()
    }

    pub fn parse(query: &str) -> Self {
        let mut result = Self::default();

        for token in tokenize(query) {
            result.apply_token(&token);
        }

        debug_assert!(result.extensions.iter().all(|extension| extension.starts_with('.')));

        debug_assert!(
            result
                .patterns_symbol
                .iter()
                .all(|pattern| !pattern.bytes().any(|byte| byte.is_ascii_uppercase()))
        );

        result
    }

    pub fn requires_file_match(&self) -> bool {
        self.filter_type == Some(TypeFilter::File)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchCommand {
    Diff,
}

struct Term<'text> {
    body: &'text str,
    exclude: bool,
    key: String,
    value: &'text str,
}

struct Token {
    exact: bool,
    text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeFilter {
    Directory,
    File,
}

fn apply_range(bounds: &mut Bounds, value: &str, parse: fn(&str) -> Option<u64>) {
    let text = value.trim();

    let bounded = BOUND_PREFIXES
        .iter()
        .find_map(|(prefix, bound)| text.strip_prefix(*prefix).map(|rest| (*bound, rest)));

    if let Some((bound, rest)) = bounded {
        let number = parse(rest);

        match bound {
            Bound::Above => bounds.min = number.and_then(|count| count.checked_add(1)),
            Bound::AtLeast => bounds.min = number,
            Bound::AtMost => bounds.max = number,
            Bound::Below => bounds.max = number.map(|count| count.saturating_sub(1)),
        }

        return;
    }

    if let Some((low, high)) = text.split_once('-') {
        bounds.min = parse(low);
        bounds.max = parse(high);

        return;
    }

    let exact = parse(text);

    bounds.min = exact;
    bounds.max = exact;
}

fn matches_any(terms: &[String], predicate: impl Fn(&str) -> bool) -> bool {
    if terms.is_empty() {
        return true;
    }

    terms.iter().any(|term| predicate(term))
}

fn parse_count(value: &str) -> Option<u64> {
    value.trim().parse().ok()
}

fn parse_depth(value: &str) -> Option<u32> {
    if let Some(rest) = value.strip_prefix("<=") {
        return rest.trim().parse().ok();
    }

    if let Some(rest) = value.strip_prefix('<') {
        let depth: u32 = rest.trim().parse().ok()?;

        return Some(depth.saturating_sub(1));
    }

    value.trim().parse().ok()
}

fn parse_duration(value: &str) -> Option<Duration> {
    let text = value.trim().to_ascii_lowercase();

    if text == "today" {
        return Some(Duration::from_hours(24));
    }

    let (number, seconds_per_unit) = DURATION_UNITS
        .iter()
        .find_map(|(unit, seconds)| text.strip_suffix(*unit).map(|number| (number, *seconds)))?;

    let count: u64 = number.trim().parse().ok()?;

    count.checked_mul(seconds_per_unit).map(Duration::from_secs)
}

fn parse_size(value: &str) -> Option<u64> {
    let text = value.trim().to_ascii_lowercase();

    let (number, multiplier) = SIZE_UNITS
        .iter()
        .find_map(|(suffix, multiplier)| {
            text.strip_suffix(*suffix)
                .map(|number| (number, *multiplier))
        })
        .unwrap_or((text.as_str(), 1));

    let count: u64 = number.trim().parse().ok()?;

    count.checked_mul(multiplier)
}

fn parse_type(value: &str) -> Option<TypeFilter> {
    match value.to_ascii_lowercase().as_str() {
        "d" | "dir" | "directory" | "folder" => Some(TypeFilter::Directory),
        "f" | "file" => Some(TypeFilter::File),
        _ => None,
    }
}

fn push_pattern(patterns: &mut Vec<String>, value: &str) {
    if value.is_empty() {
        return;
    }

    if patterns.len() >= PATTERN_COUNT_MAX {
        return;
    }

    patterns.push(value.to_ascii_lowercase());

    debug_assert!(patterns.len() <= PATTERN_COUNT_MAX);
}

fn push_token(tokens: &mut Vec<Token>, text: &mut String, exact: bool) {
    if text.is_empty() {
        return;
    }

    tokens.push(Token {
        exact,
        text: core::mem::take(text),
    });
}

fn tokenize(query: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut text = String::new();
    let mut quoted = false;

    for character in query.chars() {
        match character {
            '"' => {
                push_token(&mut tokens, &mut text, quoted);
                quoted = !quoted;
            }
            ' ' if !quoted => push_token(&mut tokens, &mut text, false),
            _ => text.push(character),
        }
    }

    push_token(&mut tokens, &mut text, quoted);

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(path: &str) -> MatchTarget<'_> {
        let name = path.rsplit('/').next().unwrap_or(path);

        MatchTarget {
            git_status: None,
            is_directory: false,
            metadata: None,
            name_lowercase: name,
            path_lowercase: path,
            seconds_now: 1_000_000,
        }
    }

    #[test]
    fn an_empty_query_is_empty() {
        assert!(ParsedQuery::default().is_empty());
        assert!(ParsedQuery::parse("   ").is_empty());
        assert!(ParsedQuery::parse("-").is_empty());
    }

    #[test]
    fn is_empty_covers_every_field() {
        let queries = [
            "--diff",
            "--json",
            "term",
            "\"exact\"",
            "-drop",
            "ext:rs",
            "-ext:rs",
            "name:a",
            "-name:a",
            "path:a",
            "-path:a",
            "git:modified",
            "-git:modified",
            "type:file",
            "size:>1",
            "size:<9",
            "lines:>1",
            "lines:<9",
            "depth:2",
            "recent:1d",
            "content:x",
            "sym:x",
        ];

        for query in queries {
            assert!(!ParsedQuery::parse(query).is_empty(), "query {query} parsed as empty");
        }
    }

    #[test]
    fn an_unknown_key_honours_the_exclusion() {
        let query = ParsedQuery::parse("-weird:thing");

        assert_eq!(query.excludes, vec!["weird:thing".to_owned()]);
        assert_eq!(query.contains, Vec::<String>::new());
        assert!(!query.matches(&target("/src/weird:thing.rs")));
        assert!(query.matches(&target("/src/other.rs")));
    }

    #[test]
    fn an_inexpressible_exclusion_is_ignored_rather_than_inverted() {
        let queries = [
            "-size:>1mb",
            "-lines:>10",
            "-type:dir",
            "-depth:1",
            "-recent:1d",
            "-content:todo",
            "-sym:main",
        ];

        for text in queries {
            let query = ParsedQuery::parse(text);

            assert!(query.is_empty(), "query {text} applied an exclusion as an inclusion");
        }
    }

    #[test]
    fn size_overflow_drops_the_term() {
        assert_eq!(ParsedQuery::parse("size:>18446744073709551615").size.min, None);

        let query = ParsedQuery::parse("size:18446744073709551615gb");

        assert_eq!(query.size.min, None);
        assert_eq!(query.size.max, None);
    }

    #[test]
    fn lines_overflow_drops_the_term() {
        assert_eq!(ParsedQuery::parse("lines:>18446744073709551615").lines.min, None);
    }

    #[test]
    fn duration_overflow_drops_the_term() {
        assert_eq!(ParsedQuery::parse("recent:18446744073709551615w").duration_recent, None);
    }

    #[test]
    fn size_units_multiply() {
        assert_eq!(ParsedQuery::parse("size:>=1kb").size.min, Some(1024));
        assert_eq!(ParsedQuery::parse("size:>1k").size.min, Some(1025));
        assert_eq!(ParsedQuery::parse("size:<1").size.max, Some(0));
        assert_eq!(ParsedQuery::parse("size:<0").size.max, Some(0));
        assert_eq!(ParsedQuery::parse("size:2mb").size.min, Some(2 << 20));
        assert_eq!(ParsedQuery::parse("size:5b").size.max, Some(5));
    }

    #[test]
    fn ranges_set_both_bounds() {
        let query = ParsedQuery::parse("lines:100-500");

        assert_eq!(query.lines.min, Some(100));
        assert_eq!(query.lines.max, Some(500));
    }

    #[test]
    fn symbol_patterns_are_lowercased_at_the_parse_site() {
        let query = ParsedQuery::parse("sym:MyStruct content:UPPER");

        assert_eq!(query.patterns_symbol, vec!["mystruct".to_owned()]);
        assert_eq!(query.patterns_content, vec!["upper".to_owned()]);
    }

    #[test]
    fn future_mtime_does_not_underflow() {
        let query = ParsedQuery::parse("recent:1d");

        let metadata = FileMetadata {
            lines: None,
            modified: Some(2_000_000),
            size: 0,
        };

        let mut item = target("/tmp/a.rs");

        item.metadata = Some(&metadata);

        assert!(query.matches(&item));
    }

    #[test]
    fn missing_metadata_fails_a_metadata_query() {
        let query = ParsedQuery::parse("size:>=1");

        assert!(query.is_expensive());
        assert!(!query.matches(&target("/tmp/a.rs")));
    }

    #[test]
    fn matching_uses_pre_lowered_inputs() {
        let query = ParsedQuery::parse("ext:rs path:src");

        assert!(query.matches(&target("/home/src/main.rs")));
        assert!(!query.matches(&target("/home/lib/main.rs")));
        assert!(!query.matches(&target("/home/src/main.py")));
    }

    #[test]
    fn excludes_reject_before_includes_admit() {
        let query = ParsedQuery::parse("ext:rs -name:main");

        assert!(!query.matches(&target("/src/main.rs")));
        assert!(query.matches(&target("/src/other.rs")));
    }

    #[test]
    fn git_filter_requires_a_status() {
        let query = ParsedQuery::parse("git:modified");
        let mut item = target("/a.rs");

        assert!(!query.matches(&item));

        item.git_status = Some(GitStatus::Modified);

        assert!(query.matches(&item));

        item.git_status = Some(GitStatus::Added);

        assert!(!query.matches(&item));
    }

    #[test]
    fn git_letters_follow_the_documented_shorthand() {
        let query = ParsedQuery::parse("git:xc");

        assert_eq!(query.filters_git, vec![GitFilter::Conflicted, GitFilter::Changed]);

        let mut item = target("/a.rs");

        item.git_status = Some(GitStatus::Staged);

        assert!(query.matches(&item));

        item.git_status = Some(GitStatus::Untracked);

        assert!(!query.matches(&item));
    }

    #[test]
    fn depth_filter_parses_all_forms() {
        assert_eq!(ParsedQuery::parse("depth:3").depth_max, Some(3));
        assert_eq!(ParsedQuery::parse("depth:<=3").depth_max, Some(3));
        assert_eq!(ParsedQuery::parse("depth:<3").depth_max, Some(2));
        assert_eq!(ParsedQuery::parse("depth:<0").depth_max, Some(0));
    }

    #[test]
    fn extension_matching_rejects_partial_suffixes() {
        let query = ParsedQuery::parse("ext:.RS");

        assert_eq!(query.extensions, vec![".rs".to_owned()]);
        assert!(query.matches(&target("/main.rs")));
        assert!(!query.matches(&target("/mainrs")));
        assert!(!query.matches(&target("/rs")));
        assert!(!query.matches(&target("/a.rust")));
    }

    #[test]
    fn quoted_text_is_an_exact_name() {
        let query = ParsedQuery::parse("\"Cargo.toml\" src");

        assert_eq!(query.exact, vec!["cargo.toml".to_owned()]);
        assert_eq!(query.contains, vec!["src".to_owned()]);
        assert!(query.matches(&target("/src/cargo.toml")));
        assert!(!query.matches(&target("/src/cargo.lock")));
    }

    #[test]
    fn content_patterns_are_bounded() {
        let text = (0..PATTERN_COUNT_MAX + 8)
            .map(|index| format!("content:term{index}"))
            .collect::<Vec<String>>()
            .join(" ");

        assert_eq!(ParsedQuery::parse(&text).patterns_content.len(), PATTERN_COUNT_MAX);
    }

    #[test]
    fn commands_select_the_output_format() {
        assert_eq!(ParsedQuery::parse("--md").format_override, Some(OutputFormat::Markdown));
        assert!(ParsedQuery::parse("--d").has_command(SearchCommand::Diff));
    }
}

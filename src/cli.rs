use core::slice;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use crate::model::error::{SwarmError, SwarmResult};
use crate::model::node::{FileNode, NodeKind};
use crate::model::options::Options;
use crate::model::output::OutputFormat;
use crate::model::path;
use crate::model::query::{ParsedQuery, SearchCommand};
use crate::platform::clipboard;
use crate::services::filesystem::filter::GlobPathFilter;
use crate::services::filesystem::gather::{self, GatherRequest};
use crate::services::filesystem::git::GitService;
use crate::services::skeleton::{self, SkeletonRequest};
use crate::services::tree::format_tree;
use crate::services::tree::loader::{self, LoadStop};

#[derive(Debug, Parser)]
#[command(
    name = crate::constants::APP_NAME,
    version = crate::constants::APP_VERSION,
    about = "A developer tool for generating project context",
    long_about = None,
)]
pub struct CommandLineInterface {
    #[arg(short, long, help = "Include git diffs for modified files")]
    pub diff: bool,

    #[arg(short, long, value_enum, help = "Output format")]
    pub format: Option<Format>,

    #[arg(short, long, help = "Write output to a file instead of stdout")]
    pub output: Option<PathBuf>,

    #[arg(help = "Directory path to process")]
    pub path: PathBuf,

    #[arg(short, long, help = "Apply a search/filter query")]
    pub search: Option<String>,

    #[arg(short = 'k', long)]
    #[arg(help = "Output file skeletons instead of file contents")]
    pub skeleton: bool,

    #[arg(long, help = "Print output to stdout instead of clipboard")]
    pub stdout: bool,

    #[arg(short, long)]
    #[arg(help = "Output directory tree structure instead of file contents")]
    pub tree: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Format {
    Json,
    Markdown,
    Plain,
    Xml,
}

impl From<Format> for OutputFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Json => Self::Json,
            Format::Markdown => Self::Markdown,
            Format::Plain => Self::PlainText,
            Format::Xml => Self::Xml,
        }
    }
}

fn execute(command: &CommandLineInterface) -> SwarmResult<()> {
    let options = Options::load_or_default();
    let query = parsed_query(command);

    let format = query
        .format_override
        .or_else(|| command.format.map(OutputFormat::from))
        .unwrap_or(options.output_format);

    if command.skeleton {
        require_existing(&command.path)?;

        let output = run_skeleton(&command.path, &options, format)?;

        return write_output(&output, command);
    }

    let directory = path::directory_of(&command.path);

    require_existing(&directory)?;

    if !directory.is_dir() {
        return Err(SwarmError::Validation(format!(
            "'{}' is not a directory",
            directory.display(),
        )));
    }

    let output = if command.tree {
        run_tree(&directory, &options)?
    } else {
        run_gather(&directory, &options, &query)?
    };

    write_output(&output, command)
}

fn parsed_query(command: &CommandLineInterface) -> ParsedQuery {
    let mut query = command
        .search
        .as_deref()
        .map_or_else(ParsedQuery::default, ParsedQuery::parse);

    if command.diff {
        if !query.has_command(SearchCommand::Diff) {
            query.commands.push(SearchCommand::Diff);
        }
    }

    query
}

fn require_existing(path: &Path) -> SwarmResult<()> {
    if path.exists() {
        return Ok(());
    }

    Err(SwarmError::Validation(
        format!("'{}' does not exist", path.display()),
    ))
}

pub fn run(command: &CommandLineInterface) -> ExitCode {
    match execute(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");

            ExitCode::FAILURE
        }
    }
}

fn run_gather(directory: &Path, options: &Options, query: &ParsedQuery) -> SwarmResult<String> {
    let mut git = GitService::default();

    git.refresh(directory);

    let directories = [directory.to_path_buf()];

    let request = GatherRequest {
        git: Some(&git),
        options,
        paths: &directories,
        query: Some(query),
    };

    let (output, stats) = gather::gather(&request)?;

    eprintln!("{} lines / {} tokens", stats.lines, stats.tokens);

    if stats.skipped > 0 {
        eprintln!("{} file(s) skipped", stats.skipped);
    }

    Ok(output)
}

fn run_skeleton(path: &Path, options: &Options, format: OutputFormat) -> SwarmResult<String> {
    let paths = [path.to_path_buf()];

    let request = SkeletonRequest {
        format,
        options,
        paths: &paths,
    };

    let (output, stats) = skeleton::generate(&request)?;

    eprintln!(
        "{} files / {} lines / {} tokens",
        stats.files,
        stats.lines,
        stats.tokens,
    );

    if stats.skipped > 0 {
        eprintln!("{} file(s) skipped", stats.skipped);
    }

    Ok(output)
}

fn run_tree(directory: &Path, options: &Options) -> SwarmResult<String> {
    let filter = GlobPathFilter::from_options(options)?;
    let mut root = FileNode::with_kind(directory.to_path_buf(), NodeKind::Directory);
    let summary = loader::load_tree(slice::from_mut(&mut root), &filter, |_| true);

    if root.load_failed {
        return Err(SwarmError::Validation(
            format!("'{}' cannot be read", directory.display()),
        ));
    }

    if summary.stop == LoadStop::Bound {
        eprintln!(
            "The tree stops at {} entries; deeper directories are not listed",
            summary.progress.nodes,
        );
    }

    if summary.progress.failed > 0 {
        eprintln!("{} director(ies) could not be read", summary.progress.failed);
    }

    Ok(format_tree(&root.children, options.use_icon))
}

fn write_output(output: &str, command: &CommandLineInterface) -> SwarmResult<()> {
    if let Some(output_path) = command.output.as_ref() {
        std::fs::write(output_path, output)?;

        eprintln!("Written to '{}'", output_path.display());

        return Ok(());
    }

    if command.stdout {
        let mut handle = io::stdout().lock();

        handle.write_all(output.as_bytes())?;
        handle.flush()?;

        return Ok(());
    }

    clipboard::copy(output).map_err(SwarmError::Other)?;

    let line_count = memchr::memchr_iter(b'\n', output.as_bytes()).count();

    eprintln!("Copied to clipboard ({line_count} lines)");

    Ok(())
}

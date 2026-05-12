use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, ValueEnum};
use copypasta::{ClipboardContext, ClipboardProvider};

use crate::app::state::search::{SearchCommand, ParsedQuery};
use crate::model::options::Options;
use crate::model::output::OutputFormat;
use crate::services::filesystem::gather::GatherService;
use crate::services::filesystem::git::GitService;
use crate::services::skeleton::SkeletonGenerator;
use crate::services::tree::generator::TreeGenerator;
use crate::services::tree::loader;
use crate::model::node::FileNode;

#[derive(Clone, Debug, ValueEnum)]
pub enum Format {
    Plain,
    Markdown,
    Json,
    Xml,
}

impl From<Format> for OutputFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Plain => OutputFormat::PlainText,
            Format::Markdown => OutputFormat::Markdown,
            Format::Json => OutputFormat::Json,
            Format::Xml => OutputFormat::Xml,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = crate::APP_NAME,
    version = crate::APP_VERSION,
    about = "A developer tool for generating project context",
    long_about = None,
)]
pub struct CommandLineInterface {
    #[arg(help = "Directory path to process")]
    pub path: PathBuf,

    #[arg(long, help = "Print output to stdout instead of clipboard")]
    pub stdout: bool,

    #[arg(short, long, help = "Include git diffs for modified files")]
    pub diff: bool,

    #[arg(short, long, value_enum, help = "Output format")]
    pub format: Option<Format>,

    #[arg(short = 'k', long, help = "Output file skeletons instead of file contents")]
    pub skeleton: bool,

    #[arg(short, long, help = "Write output to a file instead of stdout")]
    pub output: Option<PathBuf>,

    #[arg(short, long, help = "Apply a search/filter query")]
    pub search: Option<String>,

    #[arg(short, long, help = "Output directory tree structure instead of file contents")]
    pub tree: bool,
}

pub fn run(cli: CommandLineInterface) {
    let options = Options::load().unwrap_or_default();

    if cli.skeleton {
        let path = &cli.path;

        if !path.exists() {
            eprintln!("Error: path '{}' does not exist", path.display());
            process::exit(1);
        }

        let output = run_skeleton(path, &options, &cli);
        output_result(&output, &cli);
        return;
    }

    let path = normalize_path(&cli.path);

    if !path.exists() {
        eprintln!("Error: path '{}' does not exist", path.display());
        process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("Error: path '{}' is not a directory", path.display());
        process::exit(1);
    }

    let output = if cli.tree {
        run_tree(&path, &options)
    } else {
        run_gather(&path, &options, &cli)
    };

    output_result(&output, &cli);
}

fn output_result(output: &str, cli: &CommandLineInterface) {
    if let Some(ref output_path) = cli.output {
        if let Err(error) = std::fs::write(output_path, output) {
            eprintln!("Error: failed to write to '{}': {}", output_path.display(), error);
            process::exit(1);
        }

        eprintln!("Written to '{}'", output_path.display());
    } else if cli.stdout {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        if let Err(error) = handle.write_all(output.as_bytes()) {
            eprintln!("Error: failed to write to stdout: {}", error);
            process::exit(1);
        }
    } else {
        match ClipboardContext::new() {
            Ok(mut clipboard) => {
                if let Err(error) = clipboard.set_contents(output.to_string()) {
                    eprintln!("Error: failed to copy to clipboard: {}", error);
                    process::exit(1);
                }

                let line_count = memchr::memchr_iter(b'\n', output.as_bytes()).count();
                eprintln!("Copied to clipboard ({} lines)", line_count);
            }
            Err(error) => {
                eprintln!("Error: failed to access clipboard: {}", error);
                process::exit(1);
            }
        }
    }
}

fn run_skeleton(path: &Path, options: &Options, cli: &CommandLineInterface) -> String {
    let mut override_options = options.clone();

    if let Some(ref search) = cli.search {
        let query = ParsedQuery::parse(search);

        if let Some(format) = query.format_override {
            override_options.output_format = format;
        }
    }

    let generator = SkeletonGenerator::new();
    let paths = vec![path.display().to_string()];

    match generator.generate(&paths, &override_options) {
        Ok((output, stats)) => {
            eprintln!("{} files / {} lines / {} tokens", stats.files_count, stats.line_count, stats.token_count);
            output
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            process::exit(1);
        }
    }
}

fn run_tree(path: &Path, options: &Options) -> String {
    let mut root = FileNode::new(path.to_path_buf());

    if let Err(error) = loader::load_all_children(&mut root, options) {
        eprintln!("Error: failed to load directory: {}", error);
        process::exit(1);
    }

    let generator = TreeGenerator::new(options);
    generator.generate_tree(&root.children)
}

fn run_gather(path: &Path, options: &Options, cli: &CommandLineInterface) -> String {
    let mut git_service = GitService::new();
    git_service.refresh(path);

    let query = build_parsed_query(cli);

    let mut override_options = options.clone();

    if let Some(ref format) = cli.format {
        override_options.output_format = format.clone().into();
    }

    let gather = GatherService::new();
    let paths = vec![path.display().to_string()];

    match gather.gather_with_context(&paths, &override_options, Some(&git_service), Some(&query)) {
        Ok((output, stats)) => {
            eprintln!("{} lines / {} tokens", stats.line_count, stats.token_count);
            output
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            process::exit(1);
        }
    }
}

fn build_parsed_query(cli: &CommandLineInterface) -> ParsedQuery {
    let mut query = match cli.search {
        Some(ref s) if !s.is_empty() => ParsedQuery::parse(s),
        _ => ParsedQuery::default(),
    };

    if cli.diff {
        query.commands.push(SearchCommand::Diff);
    }

    query
}

fn normalize_path(path: &Path) -> PathBuf {
    if path.is_file() {
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    }
}

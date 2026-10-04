use alloc::sync::Arc;
use core::fmt::Write as _;
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::model::output::OutputFormat;
use crate::model::query::ParsedQuery;
use crate::model::selection;
use crate::services::filesystem::filter::GlobPathFilter;
use crate::services::filesystem::gather::{self, GatherRequest};
use crate::services::filesystem::git::GitService;
use crate::services::skeleton::{self, SkeletonRequest};
use crate::services::tree::format_tree;
use crate::services::tree::traversal::{self, PropagateRequest};

use super::core::{Worker, WorkerTask};
use super::generation::Generation;

pub enum JobCommand {
    Gather {
        generation: u64,
        job: Box<GatherJob>,
    },
    Propagate(Box<PropagateJob>),
    Skeleton {
        generation: u64,
        job: Box<SkeletonJob>,
    },
    TreeRender {
        generation: u64,
        nodes: Vec<FileNode>,
        use_icons: bool,
    },
}

pub enum JobResult {
    GatherCompleted(Report),
    GatherFailed(String),
    PropagateCompleted(Box<PropagateOutcome>),
    SkeletonCompleted(Report),
    SkeletonFailed(String),
    TreeRendered(String),
}

pub struct GatherJob {
    pub git: GitService,
    pub options: Arc<Options>,
    pub paths: Vec<PathBuf>,
    pub query: ParsedQuery,
}

#[derive(Clone, Default)]
struct Generations {
    gather: Generation,
    skeleton: Generation,
    tree_render: Generation,
}

struct JobTask {
    generations: Generations,
}

impl JobTask {
    fn run_gather(job: &GatherJob, results: &SyncSender<JobResult>) {
        let request = GatherRequest {
            git: Some(&job.git),
            options: &job.options,
            paths: &job.paths,
            query: Some(&job.query),
        };

        let result = match gather::gather(&request) {
            Ok((output, stats)) => {
                let mut summary = format!(
                    "{} lines / {} tokens copied",
                    stats.lines,
                    stats.tokens,
                );

                append_skipped(&mut summary, stats.skipped);

                JobResult::GatherCompleted(Report { output, summary })
            }
            Err(error) => JobResult::GatherFailed(error.to_string()),
        };

        let _ = results.send(result);
    }

    fn run_propagate(job: PropagateJob, results: &SyncSender<JobResult>) {
        let PropagateJob {
            checked,
            filter,
            git,
            mut nodes,
            path,
            query,
            session,
        } = job;

        assert_ne!(path.len(), 0);

        let mut failed_count: u32 = 0;

        if let Some(node) = selection::find_node_mut(&mut nodes, &path) {
            let depth =
                u32::try_from(path.len() - 1).expect("a node path is bounded by the tree depth");

            let request = PropagateRequest {
                checked,
                depth,
                filter: &filter,
                git: Some(&git),
                matches: None,
                query: query.as_ref(),
            };

            failed_count = traversal::propagate_checked(node, &request);
            selection::update_ancestors(&mut nodes, &path, checked);
        }

        let outcome = PropagateOutcome {
            failed_count,
            nodes,
            session,
        };

        let _ = results.send(JobResult::PropagateCompleted(Box::new(outcome)));
    }

    fn run_skeleton(job: &SkeletonJob, results: &SyncSender<JobResult>) {
        let request = SkeletonRequest {
            format: job.format,
            options: &job.options,
            paths: &job.paths,
        };

        let result = match skeleton::generate(&request) {
            Ok((output, stats)) => {
                let mut summary = format!(
                    "{} files / {} lines / {} tokens skeleton copied",
                    stats.files,
                    stats.lines,
                    stats.tokens,
                );

                append_skipped(&mut summary, stats.skipped);

                JobResult::SkeletonCompleted(Report { output, summary })
            }
            Err(error) => JobResult::SkeletonFailed(error.to_string()),
        };

        let _ = results.send(result);
    }
}

impl WorkerTask for JobTask {
    type Command = JobCommand;
    type Result = JobResult;

    fn process(&mut self, command: Self::Command, results: &SyncSender<Self::Result>) {
        match command {
            JobCommand::Gather { generation, job } => {
                if self.generations.gather.is_current(generation) {
                    Self::run_gather(&job, results);
                }
            }
            JobCommand::Propagate(job) => Self::run_propagate(*job, results),
            JobCommand::Skeleton { generation, job } => {
                if self.generations.skeleton.is_current(generation) {
                    Self::run_skeleton(&job, results);
                }
            }
            JobCommand::TreeRender {
                generation,
                nodes,
                use_icons,
            } => {
                if self.generations.tree_render.is_current(generation) {
                    let _ = results.send(JobResult::TreeRendered(format_tree(&nodes, use_icons)));
                }
            }
        }
    }
}

pub struct JobWorker {
    generations: Generations,
    worker: Worker<JobTask>,
}

impl JobWorker {
    pub fn check_results(&self) -> Option<JobResult> {
        self.worker.try_recv()
    }

    pub fn new() -> Self {
        let generations = Generations::default();

        let task = JobTask {
            generations: generations.clone(),
        };

        Self {
            generations,
            worker: Worker::spawn("job", task),
        }
    }

    pub fn start_gather(&self, job: Box<GatherJob>) -> bool {
        assert_ne!(job.paths.len(), 0);

        let generation = self.generations.gather.advance();

        self.worker.send(JobCommand::Gather { generation, job })
    }

    pub fn start_propagate(&self, job: Box<PropagateJob>) -> bool {
        assert_ne!(job.path.len(), 0);

        self.worker.send(JobCommand::Propagate(job))
    }

    pub fn start_skeleton(&self, job: Box<SkeletonJob>) -> bool {
        assert_ne!(job.paths.len(), 0);

        let generation = self.generations.skeleton.advance();

        self.worker.send(JobCommand::Skeleton { generation, job })
    }

    pub fn start_tree_render(&self, nodes: Vec<FileNode>, use_icons: bool) -> bool {
        assert_ne!(nodes.len(), 0);

        let generation = self.generations.tree_render.advance();

        self.worker.send(JobCommand::TreeRender {
            generation,
            nodes,
            use_icons,
        })
    }
}

impl Default for JobWorker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PropagateJob {
    pub checked: bool,
    pub filter: Arc<GlobPathFilter>,
    pub git: GitService,
    pub nodes: Vec<FileNode>,
    pub path: Vec<u32>,
    pub query: Option<ParsedQuery>,
    pub session: String,
}

pub struct PropagateOutcome {
    pub failed_count: u32,
    pub nodes: Vec<FileNode>,
    pub session: String,
}

#[derive(Debug)]
pub struct Report {
    pub output: String,
    pub summary: String,
}

pub struct SkeletonJob {
    pub format: OutputFormat,
    pub options: Arc<Options>,
    pub paths: Vec<PathBuf>,
}

fn append_skipped(summary: &mut String, skipped_count: u32) {
    if skipped_count == 0 {
        return;
    }

    let _ = write!(summary, " ({skipped_count} skipped)");
}

use std::thread;

use rfd::FileDialog;

use crate::app::message::{
    App,
    BackgroundStart,
    Command,
    CopyMessage,
    FilterStart,
    Message,
    MessageSender,
    Notice,
    Render,
    Search,
    Skeleton,
    Tree,
};
use crate::app::persistence;
use crate::constants::MESSAGE_COUNT_MAX;
use crate::services::worker::filter::FilterRequest;
use crate::services::worker::{
    BackgroundLoader,
    FilterWorker,
    JobResult,
    JobWorker,
    SessionLoader,
    TreeLoader,
};

const COMMAND_COUNT_MAX: u32 = 1_024;

pub struct Runtime {
    background_loader: BackgroundLoader,
    filter_worker: FilterWorker,
    job_worker: JobWorker,
    message_sender: MessageSender,
    session_loader: SessionLoader,
    tree_loader: TreeLoader,
}

impl Runtime {
    fn execute_one(&self, command: Command) {
        let delivered = match command {
            Command::Batch(_) => unreachable!("batches are flattened by execute"),
            Command::CancelFilter => {
                self.filter_worker.cancel();

                true
            }
            Command::DeleteSessionData(identifier) => {
                if let Err(error) = persistence::delete_session_file(&identifier) {
                    self.report_error(format!("Failed to delete the session: {error}"));
                }

                true
            }
            Command::GatherFiles(job) => self.job_worker.start_gather(job),
            Command::GenerateSkeleton(job) => self.job_worker.start_skeleton(job),
            Command::LoadBackground(start) => self.start_background(*start),
            Command::LoadSession(request) => self.session_loader.start(request),
            Command::None => true,
            Command::OpenFileDialog => self.open_file_dialog(),
            Command::PropagateChecked(job) => self.job_worker.start_propagate(job),
            Command::RefreshTree(request) => self.tree_loader.start(request),
            Command::RenderTree { nodes, use_icons } => {
                self.job_worker.start_tree_render(nodes, use_icons)
            }
            Command::StartFilter(start) => self.start_filter(*start),
            Command::StopBackground => {
                self.background_loader.stop();

                true
            }
        };

        if !delivered {
            self.report_error("A background task could not be started".to_owned());
        }
    }

    fn open_file_dialog(&self) -> bool {
        let sender = self.message_sender.clone();

        let spawned = thread::Builder::new()
            .name("file-dialog".to_owned())
            .spawn(move || {
                let picked = FileDialog::new()
                    .set_title("Select File or Directory")
                    .pick_folder();

                let message = match picked {
                    Some(path) => App::PathSelected(path),
                    None => App::FileDialogClosed,
                };

                sender.send(Message::App(message));
            });

        if let Err(error) = spawned {
            self.report_error(format!("Failed to open the file dialog: {error}"));

            self.message_sender
                .send(Message::App(App::FileDialogClosed));
        }

        true
    }

    fn poll_background(&self, messages: &mut Vec<Message>) {
        for _ in 0..MESSAGE_COUNT_MAX {
            let Some(outcome) = self.background_loader.check_results() else {
                return;
            };

            if self.background_loader.is_current(outcome.generation) {
                messages.push(Message::Tree(Tree::BackgroundLoadCompleted(outcome)));
            }
        }
    }

    fn poll_filter(&self, messages: &mut Vec<Message>) {
        for _ in 0..MESSAGE_COUNT_MAX {
            let Some(complete) = self.filter_worker.check_results() else {
                return;
            };

            if self.filter_worker.is_current(complete.generation) {
                messages.push(Message::Search(Search::FilterCompleted(Box::new(complete))));
            }
        }
    }

    fn poll_jobs(&self, messages: &mut Vec<Message>) {
        for _ in 0..MESSAGE_COUNT_MAX {
            let Some(result) = self.job_worker.check_results() else {
                return;
            };

            let message = match result {
                JobResult::GatherCompleted(report) => Message::Copy(CopyMessage::Completed(report)),
                JobResult::GatherFailed(error) => Message::Copy(CopyMessage::Failed(error)),
                JobResult::PropagateCompleted(outcome) => {
                    Message::Tree(Tree::PropagateCompleted(outcome))
                }
                JobResult::SkeletonCompleted(report) => {
                    Message::Skeleton(Skeleton::Completed(report))
                }
                JobResult::SkeletonFailed(error) => Message::Skeleton(Skeleton::Failed(error)),
                JobResult::TreeRendered(output) => Message::Render(Render::Generated(output)),
            };

            messages.push(message);
        }
    }

    fn poll_loaders(&self, messages: &mut Vec<Message>) {
        for _ in 0..MESSAGE_COUNT_MAX {
            let Some(result) = self.session_loader.check_results() else {
                break;
            };

            messages.push(Message::Tree(Tree::SessionLoaded(Box::new(result))));
        }

        for _ in 0..MESSAGE_COUNT_MAX {
            let Some(result) = self.tree_loader.check_results() else {
                break;
            };

            messages.push(Message::Tree(Tree::Refreshed(Box::new(result))));
        }
    }

    fn report_error(&self, text: String) {
        assert_ne!(text, "");

        self.message_sender
            .send(Message::Notice(Notice::Error(text)));
    }

    fn start_background(&self, start: BackgroundStart) -> bool {
        self.background_loader
            .start(start.nodes, start.filter, start.session)
    }

    fn start_filter(&self, start: FilterStart) -> bool {
        let request = FilterRequest {
            entries: start.entries,
            generation: self.filter_worker.next_generation(),
            git: start.git,
            query: start.query,
            session: start.session,
        };

        self.filter_worker.start(Box::new(request))
    }

    pub fn execute(&self, command: Command) {
        let mut worklist: Vec<Command> = vec![command];
        let mut executed_count: u32 = 0;

        while let Some(next) = worklist.pop() {
            executed_count += 1;

            assert!(executed_count <= COMMAND_COUNT_MAX);

            match next {
                Command::Batch(commands) => {
                    debug_assert!(!commands.iter().any(|inner| matches!(inner, Command::Batch(_))));

                    worklist.extend(commands.into_iter().rev());
                }
                other => self.execute_one(other),
            }
        }
    }

    pub fn new(message_sender: MessageSender) -> Self {
        Self {
            background_loader: BackgroundLoader::new(),
            filter_worker: FilterWorker::new(),
            job_worker: JobWorker::new(),
            message_sender,
            session_loader: SessionLoader::new(),
            tree_loader: TreeLoader::new(),
        }
    }

    pub fn poll(&self, messages: &mut Vec<Message>) {
        let messages_queued = messages.len();

        self.poll_loaders(messages);
        self.poll_background(messages);
        self.poll_filter(messages);
        self.poll_jobs(messages);

        debug_assert!(messages.len() >= messages_queued);
    }
}

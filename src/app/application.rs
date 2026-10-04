use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use eframe::egui;
use single_instance::SingleInstance;

use crate::app::handler::common::SESSION_NAME_DEFAULT;
use crate::app::message::{App, Command, Message, MessageSender, Search};
use crate::app::runtime::Runtime;
use crate::app::state::{FilterStatus, Model, UiState};
use crate::app::{dispatcher, ipc, persistence};
use crate::constants::{IPC_ADDRESS, MESSAGE_COUNT_MAX, MESSAGE_QUEUE_COUNT_MAX};
use crate::model::error::SwarmResult;
use crate::model::options::Options;
use crate::ui::widget::titlebar;
use crate::ui::{scale, themes, view};

pub struct SwarmApp {
    _instance_guard: Option<SingleInstance>,
    initialized: bool,
    message_receiver: Receiver<Message>,
    message_sender: MessageSender,
    messages: Vec<Message>,
    model: Model,
    runtime: Runtime,
    ui: UiState,
}

impl SwarmApp {
    fn initialize(&mut self) -> Command {
        assert!(!self.initialized);

        self.initialized = true;

        let report = persistence::load_sessions();

        self.model.sessions = report.sessions;

        if report.skipped_count > 0 {
            self.ui
                .toast
                .error(format!("Skipped {} unreadable session file(s)", report.skipped_count));
        }

        if self.model.tree.nodes.is_empty() {
            self.message_sender.send(Message::App(App::Initialized));

            return Command::None;
        }

        let created = self
            .model
            .sessions
            .create_session(SESSION_NAME_DEFAULT.to_owned());

        debug_assert_eq!(self.model.sessions.active_identifier(), Some(created.as_str()));

        self.model.refresh_git_status();
        self.model.tree.update_files_count();

        self.model.background_start()
    }

    fn needs_repaint(&self) -> bool {
        self.model.tree.is_loading()
            || self.model.tree.background_loading
            || self.ui.copy_in_progress
            || self.ui.is_generating()
            || self.ui.search_debounce.is_some()
            || self.ui.filter_status == FilterStatus::Filtering
    }

    pub fn new(
        options: Options,
        paths: Vec<PathBuf>,
        instance_guard: Option<SingleInstance>,
    ) -> Self {
        let (sender, receiver) = mpsc::sync_channel(MESSAGE_QUEUE_COUNT_MAX as usize);
        let message_sender = MessageSender::new(sender);
        let theme = options.theme;
        let model = Model::new(options, paths);
        let runtime = Runtime::new(message_sender.clone());

        if instance_guard.is_some() {
            if let Err(error) = ipc::spawn(message_sender.clone()) {
                eprintln!("Failed to listen for other instances on {IPC_ADDRESS}: {error}");
            }
        }

        Self {
            _instance_guard: instance_guard,
            initialized: false,
            message_receiver: receiver,
            message_sender,
            messages: Vec::with_capacity(MESSAGE_COUNT_MAX as usize),
            model,
            runtime,
            ui: UiState::new(theme),
        }
    }

    fn process_messages(&mut self) {
        debug_assert_eq!(self.messages.len(), 0);

        for _ in 0..MESSAGE_COUNT_MAX {
            let Ok(message) = self.message_receiver.try_recv() else {
                break;
            };

            self.messages.push(message);
        }

        self.runtime.poll(&mut self.messages);

        if self.ui.search_debounce.is_some() {
            self.messages.push(Message::Search(Search::DebounceTick));
        }

        for message in self.messages.drain(..) {
            let command = dispatcher::dispatch(&mut self.model, &mut self.ui, message);

            self.runtime.execute(command);
        }
    }
}

impl eframe::App for SwarmApp {
    fn on_exit(&mut self) {
        if !self.model.tree.nodes.is_empty() {
            self.model.mark_session_dirty();
        }

        self.model.flush_session();

        if let Some(session) = self.model.sessions.active_session() {
            if !session.tree_state.nodes.is_empty() {
                report_shutdown_failure(persistence::save_last_session(session));
            }
        }

        if self.model.options.delete_sessions_on_exit {
            report_shutdown_failure(persistence::delete_sessions_directory());
        } else {
            report_shutdown_failure(persistence::save_sessions(&self.model.sessions));
        }

        report_shutdown_failure(self.model.options.save());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let context = ui.ctx().clone();

        titlebar::render(ui);

        if self.ui.theme_dirty {
            themes::apply(self.ui.theme, &context);
            self.ui.theme_dirty = false;
        }

        context.set_pixels_per_point(scale::effective(&self.model.options, ui));

        if !self.initialized {
            let command = self.initialize();

            self.runtime.execute(command);
            context.request_repaint();
        }

        self.ui.toast.show(&context);
        self.process_messages();

        if let Some(text) = self.ui.clipboard_pending.take() {
            context.copy_text(text);
        }

        view::render(ui, &self.model, &mut self.ui, &self.message_sender);

        if self.needs_repaint() {
            context.request_repaint();
        }
    }
}

fn report_shutdown_failure(outcome: SwarmResult<()>) {
    if let Err(error) = outcome {
        eprintln!("Shutdown persistence failed: {error}");
    }
}

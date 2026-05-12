pub mod dispatcher;
pub mod handler;
pub mod ipc;
pub mod message;
pub mod runtime;
pub mod state;

use std::sync::mpsc;
use eframe::egui;
use single_instance::SingleInstance;

use crate::app::state::SessionData;
use crate::services::worker::BackgroundLoadResult;
use crate::ui::view::View;

use dispatcher::Dispatcher;
use ipc::IpcListener;
use message::{App, CommandBuilder, Message, Search, Tree};
use runtime::Runtime;
use state::{FilterStatus, Model, UiState};

pub struct SwarmApp {
    model: Model,
    ui: UiState,
    runtime: Runtime,
    message_receiver: mpsc::Receiver<Message>,
    message_sender: mpsc::Sender<Message>,
    initialized: bool,
    _instance_guard: Option<SingleInstance>,
    _ipc_thread: Option<std::thread::JoinHandle<()>>,
}

impl SwarmApp {
    pub fn new(paths: Vec<String>, instance_guard: Option<SingleInstance>) -> Self {
        let (message_sender, message_receiver) = mpsc::channel();

        let model = Model::new(paths);
        let theme = model.options.theme;
        let ui = UiState::new(theme);
        let runtime = Runtime::new(message_sender.clone());

        let ipc_thread = if instance_guard.is_some() {
            IpcListener::new(message_sender.clone()).spawn()
        } else {
            None
        };

        Self {
            model,
            ui: ui,
            runtime,
            message_receiver,
            message_sender,
            initialized: false,
            _instance_guard: instance_guard,
            _ipc_thread: ipc_thread,
        }
    }

    pub fn dispatch(&self, message: Message) {
        let _ = self.message_sender.send(message);
    }

    fn process_messages(&mut self) {
        let mut messages = Vec::new();

        while let Ok(message) = self.message_receiver.try_recv() {
            messages.push(message);
        }

        messages.extend(self.runtime.poll());

        if let Some(result) = self.model.background_loader.check_results() {
            match result {
                BackgroundLoadResult::Progress(loaded, total) => {
                    messages.push(Message::Tree(Tree::BackgroundLoadProgress { loaded, total }));
                }
                BackgroundLoadResult::NodesUpdated(nodes) => {
                    messages.push(Message::Tree(Tree::BackgroundLoadCompleted(nodes)));
                }
            }
        }

        if self.ui.search_debounce.is_some() {
            messages.push(Message::Search(Search::DebounceTick));
        }

        for message in messages {
            let command = Dispatcher::dispatch(&mut self.model, &mut self.ui, message);
            self.runtime.execute(command);
        }
    }
}

impl eframe::App for SwarmApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let context = ui.ctx().clone();

        crate::ui::widget::titlebar::titlebar::TitleBar::render(ui);

        self.ui.theme.apply(&context);

        let scale = if self.model.options.ui_scale.is_none() {
            ui.input(|i| {
                i.viewport().outer_rect.map(|rect| {
                    let center = rect.center();
                    self.model.options.effective_ui_scale_at_position(center.x, center.y)
                })
            }).unwrap_or_else(|| self.model.options.effective_ui_scale())
        } else {
            self.model.options.effective_ui_scale()
        };

        context.set_pixels_per_point(scale);

        if !self.initialized {
            self.initialized = true;

            let has_initial_paths = !self.model.tree.nodes.is_empty();

            self.model.sessions = self.runtime.load_sessions();

            if has_initial_paths {
                let session = SessionData::new("Session".to_string());
                let session_identifier = session.identifier.clone();

                self.model.sessions.sessions.insert(session_identifier.clone(), session);
                self.model.sessions.active_identifier = Some(session_identifier.clone());

                self.model.refresh_git_status();

                self.model.background_loader.start_loading(
                    self.model.tree.nodes.clone(),
                    (*self.model.options).clone()
                );

                let builder = CommandBuilder::new();
                let command = builder.build();

                self.runtime.execute(command);
            } else if self.model.sessions.active_identifier.is_some() {
                self.model.refresh_git_status();
                self.dispatch(Message::App(App::Initialized));
            } else {
                self.dispatch(Message::App(App::Initialized));
            }
        }

        self.ui.toast.show(&context);

        self.process_messages();

        View::render(ui, &self.model, &self.ui, &self.message_sender);

        if matches!(
            self.model.tree.load_status,
            state::LoadStatus::Loading { .. }
        ) || self.ui.copy_in_progress
           || self.ui.tree_generate_in_progress
           || self.ui.skeleton_generate_in_progress
           || self.model.background_loader.is_running()
           || self.ui.search_debounce.is_some()
           || self.ui.filter_status == FilterStatus::Filtering
        {
            context.request_repaint();
        }
    }

    fn on_exit(&mut self) {
        if !self.model.tree.nodes.is_empty() {
            self.model.sessions.sync_from_tree_and_search(&self.model.tree, &self.model.search);
        }

        if let Some(session) = self.model.sessions.active_session() {
            if !session.tree_state.nodes.is_empty() {
                self.runtime.save_last_session(session);
            }
        }

        if self.model.options.delete_sessions_on_exit {
            if let Some(directory) = dirs::data_local_dir() {
                let sessions_directory = directory
                    .join(crate::constants::APP_NAME.to_lowercase())
                    .join("sessions");

                if sessions_directory.exists() {
                    let _ = std::fs::remove_dir_all(sessions_directory);
                }
            }
        } else {
            self.runtime.save_sessions(&self.model.sessions);
        }

        let _ = self.model.options.save();
    }
}

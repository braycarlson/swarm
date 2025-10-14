pub mod dispatcher;
pub mod handler;
pub mod message;
pub mod runtime;
pub mod state;

use std::sync::mpsc;
use std::sync::Arc;
use eframe::egui;

use crate::app::state::SessionData;
use crate::ui::view::View;

use dispatcher::Dispatcher;
use message::{App, Cmd, CmdBuilder, Index, Msg};
use runtime::Runtime;
use state::{IndexStatus, Model, UiState};

pub struct SwarmApp {
    model: Model,
    ui: UiState,
    runtime: Runtime,
    msg_receiver: mpsc::Receiver<Msg>,
    msg_sender: mpsc::Sender<Msg>,
    initialized: bool,
}

impl SwarmApp {
    pub fn new(paths: Vec<String>) -> Self {
        let (msg_sender, msg_receiver) = mpsc::channel();

        let model = Model::new(paths);
        let theme = model.options.theme;
        let ui = UiState::new(theme);
        let runtime = Runtime::new(msg_sender.clone());

        Self {
            model,
            ui,
            runtime,
            msg_receiver,
            msg_sender,
            initialized: false,
        }
    }

    pub fn dispatch(&self, msg: Msg) {
        let _ = self.msg_sender.send(msg);
    }

    fn process_messages(&mut self) {
        let mut messages = Vec::new();

        while let Ok(msg) = self.msg_receiver.try_recv() {
            messages.push(msg);
        }

        messages.extend(self.runtime.poll());

        for msg in messages {
            let cmd = Dispatcher::dispatch(&mut self.model, &mut self.ui, msg);
            self.runtime.execute(cmd);
        }
    }
}

impl eframe::App for SwarmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::widget::titlebar::titlebar::TitleBar::render(ctx);

        self.ui.theme.apply(ctx);

        ctx.set_pixels_per_point(1.3);

        if !self.initialized {
            self.initialized = true;

            let has_initial_paths = !self.model.tree.nodes.is_empty();

            self.model.sessions = self.runtime.load_sessions();

            if has_initial_paths {
                let session = SessionData::new(format!("Session"));
                let id = session.id.clone();
                self.model.sessions.sessions.insert(id.clone(), session);
                self.model.sessions.active_id = Some(id.clone());

                if let Some(session) = self.model.sessions.active_session_mut() {
                    session.tree_state = self.model.tree.clone();
                    session.mark_modified();
                }

                let mut builder = CmdBuilder::new();

                if let Some(session_id) = &self.model.sessions.active_id {
                    builder = builder.add(Cmd::SwitchIndexSession(session_id.clone()));
                }

                if self.model.options.auto_index_on_startup {
                    self.model.index.status = IndexStatus::Running { paused: false };
                    builder = builder.add(Cmd::StartIndexing {
                        paths: self.model.tree.nodes.iter().map(|n| n.path.clone()).collect(),
                        options: Arc::clone(&self.model.options),
                    });
                }

                let cmd = builder.build();
                if !matches!(cmd, Cmd::None) {
                    self.runtime.execute(cmd);
                }
            } else if self.model.sessions.sessions.is_empty() {
                self.dispatch(Msg::App(App::Initialized));
            } else {
                if let Some(active) = self.model.sessions.active_session() {
                    self.model.tree = active.tree_state.clone();
                    self.model.search = active.search_state.clone();

                    if self.model.options.auto_index_on_startup && !self.model.tree.nodes.is_empty() {
                        self.dispatch(Msg::Index(Index::StartRequested));
                    }
                }
            }
        }

        self.process_messages();

        View::render(ctx, &self.model, &self.ui, &self.msg_sender);

        if matches!(
            self.model.index.status,
            state::IndexStatus::Running { .. }
        ) || matches!(
            self.model.tree.load_status,
            state::LoadStatus::Loading { .. }
        ) || self.ui.copy_in_progress
          || self.ui.tree_gen_in_progress
        {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.model.options.delete_sessions_on_exit {
            if let Some(dir) = dirs::data_local_dir() {
                let sessions_dir = dir
                    .join(crate::constants::APP_NAME.to_lowercase())
                    .join("sessions");

                if sessions_dir.exists() {
                    let _ = std::fs::remove_dir_all(sessions_dir);
                }
            }
        } else {
            self.runtime.save_sessions(&self.model.sessions);
        }

        let _ = self.model.options.save();
    }
}

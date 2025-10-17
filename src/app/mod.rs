pub mod dispatcher;
pub mod handler;
pub mod message;
pub mod runtime;
pub mod state;

use std::sync::mpsc;
use std::sync::Arc;
use eframe::egui;
use single_instance::SingleInstance;

use crate::app::state::SessionData;
use crate::services::worker::BackgroundLoadResult;
use crate::ui::view::View;

use dispatcher::Dispatcher;
use message::{App, Cmd, CmdBuilder, Msg, Tree};
use runtime::Runtime;
use state::{IndexStatus, Model, UiState};

pub struct SwarmApp {
    model: Model,
    ui: UiState,
    runtime: Runtime,
    msg_receiver: mpsc::Receiver<Msg>,
    msg_sender: mpsc::Sender<Msg>,
    initialized: bool,
    _instance_guard: Option<SingleInstance>,
    _ipc_thread: Option<std::thread::JoinHandle<()>>,
}

impl SwarmApp {
    pub fn new(paths: Vec<String>, instance_guard: Option<SingleInstance>) -> Self {
        let (msg_sender, msg_receiver) = mpsc::channel();

        let model = Model::new(paths);
        let theme = model.options.theme;
        let ui = UiState::new(theme);
        let runtime = Runtime::new(msg_sender.clone());

        let ipc_thread = if instance_guard.is_some() {
            let ipc_sender = msg_sender.clone();

            Some(std::thread::spawn(move || {
                use std::net::TcpListener;
                use std::io::Read;

                if let Ok(listener) = TcpListener::bind("127.0.0.1:44287") {
                    for stream in listener.incoming().flatten() {
                        let mut stream = stream;
                        let mut buffer = String::new();

                        if stream.read_to_string(&mut buffer).is_ok() {
                            let paths: Vec<std::path::PathBuf> = buffer
                                .lines()
                                .map(std::path::PathBuf::from)
                                .collect();

                            if !paths.is_empty() {
                                let _ = ipc_sender.send(Msg::App(
                                    App::PathsReceivedFromIpc(paths)
                                ));
                            }
                        }
                    }
                }
            }))
        } else {
            None
        };

        Self {
            model,
            ui,
            runtime,
            msg_receiver,
            msg_sender,
            initialized: false,
            _instance_guard: instance_guard,
            _ipc_thread: ipc_thread,
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

        if let Some(result) = self.model.background_loader.check_results() {
            match result {
                BackgroundLoadResult::Progress(loaded, total) => {
                    messages.push(Msg::Tree(Tree::BackgroundLoadProgress { loaded, total }));
                }
                BackgroundLoadResult::NodesUpdated(nodes) => {
                    messages.push(Msg::Tree(Tree::BackgroundLoadCompleted(nodes)));
                }
            }
        }

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
                let session_id = session.id.clone();

                self.model.sessions.sessions.insert(session_id.clone(), session);
                self.model.sessions.active_id = Some(session_id.clone());

                self.model.background_loader.start_loading(
                    self.model.tree.nodes.clone(),
                    (*self.model.options).clone()
                );

                let mut builder = CmdBuilder::new();
                builder = builder.add(Cmd::SwitchIndexSession(session_id));

                if self.model.options.auto_index_on_startup {
                    self.model.index.status = IndexStatus::Running { paused: false };

                    builder = builder.add(Cmd::StartIndexing {
                        paths: self.model.tree.nodes.iter().map(|n| n.path.clone()).collect(),
                        options: Arc::clone(&self.model.options),
                    });
                }

                let cmd = builder.build();
                self.runtime.execute(cmd);
            } else {
                self.dispatch(Msg::App(App::Initialized));
            }
        }

        self.ui.toast.show(ctx);

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
          || self.model.background_loader.is_running()
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

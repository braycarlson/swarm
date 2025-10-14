use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, Context, Layout, RichText};

use crate::constants::APP_NAME;

#[derive(Clone, Copy, Debug)]
pub enum DialogType {
    Error,
    Info,
    Warning,
}

pub struct DialogSystem {
    auto_close: bool,
    created_at: Option<Instant>,
    duration: Duration,
    message: String,
    open: bool,
    title: String,
}

impl DialogSystem {
    pub fn new() -> Self {
        Self {
            auto_close: false,
            created_at: None,
            duration: Duration::from_secs(5),
            message: String::new(),
            open: false,
            title: String::from(APP_NAME),
        }
    }

    pub fn error(&mut self, message: &str) {
        self.message = message.to_string();
        self.title = String::from("Error");
        self.open = true;
        self.created_at = None;
    }

    pub fn info(&mut self, message: &str) {
        self.message = message.to_string();
        self.title = String::from("Information");
        self.open = true;
        self.created_at = None;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn show(&mut self, ctx: &Context) {
        if self.open {
            if self.auto_close {
                if let Some(created_at) = self.created_at {
                    if created_at.elapsed() > self.duration {
                        self.open = false;
                        return;
                    }
                } else {
                    self.created_at = Some(Instant::now());
                }
            }

            let content_rect = ctx.content_rect();
            let center_pos = content_rect.center();

            egui::Window::new(&self.title)
                .resizable(false)
                .fixed_size([350.0, 150.0])
                .collapsible(false)
                .pivot(Align2::CENTER_CENTER)
                .current_pos(center_pos)
                .show(ctx, |ui| {
                    ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                        ui.add_space(15.0);
                        ui.label(RichText::new(&self.message).size(16.0));
                        ui.add_space(20.0);

                        if ui.button("OK").clicked() {
                            self.open = false;
                        }
                    });
                });
        }
    }

    pub fn warning(&mut self, message: &str) {
        self.message = message.to_string();
        self.title = String::from("Warning");
        self.open = true;
        self.created_at = None;
    }
}

pub struct DialogService {
    sender: Arc<Mutex<Option<mpsc::Sender<(DialogType, String)>>>>,
}

impl DialogService {
    pub fn new() -> Self {
        Self {
            sender: Arc::new(Mutex::new(None)),
        }
    }

    pub fn show(&self, _dialog_type: DialogType, _message: &str) {}
}

impl Clone for DialogService {
    fn clone(&self) -> Self {
        Self {
            sender: Arc::clone(&self.sender),
        }
    }
}

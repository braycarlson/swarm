use std::collections::VecDeque;
use std::time::{Duration, Instant};
use eframe::egui::{self, Context, RichText};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToastLevel {
    Error,
    Success,
}

#[derive(Clone)]
struct Toast {
    created_at: Instant,
    level: ToastLevel,
    message: String,
}

impl Toast {
    fn new(message: String, level: ToastLevel) -> Self {
        Self {
            created_at: Instant::now(),
            level,
            message,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > Duration::from_secs(3)
    }

    fn opacity(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let fade_duration = 0.3;

        if elapsed < fade_duration {
            elapsed / fade_duration
        } else if elapsed > 2.7 {
            (3.0 - elapsed) / fade_duration
        } else {
            1.0
        }
    }
}

#[derive(Clone)]
pub struct ToastSystem {
    toasts: VecDeque<Toast>,
}

impl Default for ToastSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastSystem {
    pub fn new() -> Self {
        Self {
            toasts: VecDeque::new(),
        }
    }

    pub fn success(&mut self, message: impl Into<String>) {
        if self.toasts.len() >= 5 {
            self.toasts.pop_front();
        }

        self.toasts.push_back(Toast::new(message.into(), ToastLevel::Success));
    }

    pub fn error(&mut self, message: impl Into<String>) {
        if self.toasts.len() >= 5 {
            self.toasts.pop_front();
        }

        self.toasts.push_back(Toast::new(message.into(), ToastLevel::Error));
    }

    pub fn show(&mut self, ctx: &Context) {
        self.toasts.retain(|toast| !toast.is_expired());

        if self.toasts.is_empty() {
            return;
        }

        let toast = self.toasts.back().unwrap();
        let opacity = toast.opacity();

        let content_rect = ctx.content_rect();
        let toast_width = 150.0;
        let toast_height = 50.0;

        let pos = egui::pos2(
            content_rect.center().x - toast_width / 2.0,
            content_rect.center().y - toast_height / 2.0,
        );

        let visuals = ctx.style().visuals.clone();

        let stroke_color = match toast.level {
            ToastLevel::Success => visuals.selection.stroke.color,
            ToastLevel::Error => visuals.error_fg_color,
        };

        let mut should_close = false;

        egui::Window::new("toast_notification")
            .title_bar(false)
            .resizable(false)
            .fixed_pos(pos)
            .fixed_size([toast_width, toast_height])
            .frame(
                egui::Frame::NONE
                    .fill(visuals.extreme_bg_color.linear_multiply(opacity))
                    .stroke(egui::Stroke::new(0.5, stroke_color.linear_multiply(opacity)))
                    .corner_radius(6.0)
                    .inner_margin(8.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 4],
                        blur: 16,
                        spread: 0,
                        color: egui::Color32::from_black_alpha((80.0 * opacity) as u8),
                    })
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&toast.message)
                            .size(12.0)
                            .color(visuals.text_color().linear_multiply(opacity))
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(RichText::new("×").color(visuals.text_color().linear_multiply(opacity)))
                                .frame(false)
                        ).on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            self.toasts.pop_back();
        }

        ctx.request_repaint();
    }
}

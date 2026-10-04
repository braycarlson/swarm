use core::time::Duration;
use std::time::Instant;

use eframe::egui::{self, Align2, Color32, Context, RichText};

use crate::constants::{TOAST_FADE_SECONDS, TOAST_LIFETIME_SECONDS};

const FRAME_INTERVAL_MS: u64 = 33;
const SHADOW_ALPHA_MAX: u8 = 80;

struct Toast {
    created_at: Instant,
    level: ToastLevel,
    message: String,
}

impl Toast {
    fn elapsed_seconds(&self) -> f32 {
        self.created_at.elapsed().as_secs_f32()
    }

    fn is_expired(&self) -> bool {
        self.elapsed_seconds() > TOAST_LIFETIME_SECONDS
    }

    fn opacity(&self) -> f32 {
        let elapsed = self.elapsed_seconds();
        let fade_out_start = TOAST_LIFETIME_SECONDS - TOAST_FADE_SECONDS;

        let opacity = if elapsed < TOAST_FADE_SECONDS {
            elapsed / TOAST_FADE_SECONDS
        } else if elapsed > fade_out_start {
            (TOAST_LIFETIME_SECONDS - elapsed) / TOAST_FADE_SECONDS
        } else {
            1.0
        };

        let clamped = opacity.clamp(0.0, 1.0);

        debug_assert!(clamped >= 0.0);
        debug_assert!(clamped <= 1.0);

        clamped
    }

    fn repaint_delay(&self) -> Duration {
        let elapsed = self.elapsed_seconds();
        let fade_out_start = TOAST_LIFETIME_SECONDS - TOAST_FADE_SECONDS;
        let frame = Duration::from_millis(FRAME_INTERVAL_MS);

        if elapsed < TOAST_FADE_SECONDS {
            return frame;
        }

        if elapsed >= fade_out_start {
            return frame;
        }

        Duration::from_secs_f32(fade_out_start - elapsed).max(frame)
    }
}

struct ToastColors {
    background: Color32,
    stroke: Color32,
    text: Color32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToastLevel {
    Error,
    Success,
}

#[derive(Default)]
pub struct ToastSystem {
    current: Option<Toast>,
}

impl ToastSystem {
    fn colors(context: &Context, level: ToastLevel, opacity: f32) -> ToastColors {
        let style = context.global_style();
        let visuals = &style.visuals;

        let stroke = match level {
            ToastLevel::Error => visuals.error_fg_color,
            ToastLevel::Success => visuals.selection.stroke.color,
        };

        ToastColors {
            background: visuals.extreme_bg_color.linear_multiply(opacity),
            stroke: stroke.linear_multiply(opacity),
            text: visuals.text_color().linear_multiply(opacity),
        }
    }

    fn render(context: &Context, toast: &Toast, opacity: f32) -> bool {
        let colors = Self::colors(context, toast.level, opacity);
        let shadow = Color32::from_black_alpha(SHADOW_ALPHA_MAX).linear_multiply(opacity);
        let mut closed = false;

        let frame = egui::Frame::NONE
            .fill(colors.background)
            .stroke(egui::Stroke::new(0.5_f32, colors.stroke))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(12, 10))
            .shadow(egui::epaint::Shadow {
                blur: 16,
                color: shadow,
                offset: [0, 4],
                spread: 0,
            });

        egui::Window::new("toast_notification")
            .title_bar(false)
            .resizable(false)
            .pivot(Align2::CENTER_CENTER)
            .current_pos(context.content_rect().center())
            .auto_sized()
            .frame(frame)
            .show(context, |window| {
                window.horizontal(|row| {
                    row.label(RichText::new(&toast.message).size(12.0).color(colors.text));
                    row.add_space(8.0);

                    let button =
                        egui::Button::new(RichText::new("\u{d7}").color(colors.text)).frame(false);

                    if row
                        .add(button)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        closed = true;
                    }
                });
            });

        closed
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.current = Some(Toast {
            created_at: Instant::now(),
            level: ToastLevel::Error,
            message: message.into(),
        });
    }

    pub fn show(&mut self, context: &Context) {
        if self.current.as_ref().is_some_and(Toast::is_expired) {
            self.current = None;
        }

        let Some(toast) = self.current.as_ref() else {
            return;
        };

        let opacity = toast.opacity();
        let delay = toast.repaint_delay();

        if Self::render(context, toast, opacity) {
            self.current = None;
        }

        context.request_repaint_after(delay);
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.current = Some(Toast {
            created_at: Instant::now(),
            level: ToastLevel::Success,
            message: message.into(),
        });
    }
}

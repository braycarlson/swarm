mod gruvbox;
mod one_dark;
mod rose_pine;

use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum Theme {
    Gruvbox,
    OneDark,
    RosePine,
    RosePineMoon,
}

impl Default for Theme {
    fn default() -> Self {
        Self::RosePine
    }
}

impl Theme {
    pub fn name(&self) -> &str {
        match self {
            Self::Gruvbox => "Gruvbox",
            Self::RosePine => "Rose Pine",
            Self::RosePineMoon => "Rose Pine Moon",
            Self::OneDark => "One Dark",

        }
    }

    pub fn all() -> &'static [Theme] {
        &[
            Self::Gruvbox,
            Self::RosePine,
            Self::RosePineMoon,
            Self::OneDark,
        ]
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let visuals = match self {
            Self::Gruvbox => gruvbox::gruvbox_dark_visuals(),
            Self::RosePine => rose_pine::rose_pine_visuals(),
            Self::RosePineMoon => rose_pine::rose_pine_moon_visuals(),
            Self::OneDark => one_dark::one_dark_visuals(),
        };

        ctx.set_visuals(visuals);
    }
}

pub fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(14.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
    );

    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.item_spacing = egui::vec2(10.0, 6.0);
    style.spacing.interact_size = egui::vec2(50.0, 24.0);

    style.spacing.icon_width = 20.0;
    style.spacing.icon_width_inner = 18.0;
    style.spacing.icon_spacing = 6.0;

    style.spacing.combo_height = 24.0;

    style.spacing.text_edit_width = 280.0;

    style.spacing.window_margin = egui::Margin::same(10);
    style.spacing.menu_margin = egui::Margin::same(8);

    style.spacing.indent = 20.0;

    ctx.set_style(style);
}

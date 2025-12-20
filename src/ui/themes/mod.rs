mod catppuccin;
mod dracula;
mod gruvbox;
mod nord;
mod one_dark;
mod rose_pine;
mod tokyo_night;

use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum Theme {
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    Dracula,
    Gruvbox,
    Nord,
    OneDark,
    RosePine,
    RosePineMoon,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightDay,
}

impl Default for Theme {
    fn default() -> Self {
        Self::RosePine
    }
}

impl Theme {
    pub fn name(&self) -> &str {
        match self {
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::Dracula => "Dracula",
            Self::Gruvbox => "Gruvbox",
            Self::Nord => "Nord",
            Self::OneDark => "One Dark",
            Self::RosePine => "Rose Pine",
            Self::RosePineMoon => "Rose Pine Moon",
            Self::TokyoNight => "Tokyo Night",
            Self::TokyoNightStorm => "Tokyo Night Storm",
            Self::TokyoNightDay => "Tokyo Night Day",
        }
    }

    pub fn all() -> &'static [Theme] {
        &[
            Self::CatppuccinLatte,
            Self::CatppuccinFrappe,
            Self::CatppuccinMacchiato,
            Self::CatppuccinMocha,
            Self::Dracula,
            Self::Gruvbox,
            Self::Nord,
            Self::OneDark,
            Self::RosePine,
            Self::RosePineMoon,
            Self::TokyoNight,
            Self::TokyoNightStorm,
            Self::TokyoNightDay,
        ]
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let visuals = match self {
            Self::CatppuccinLatte => catppuccin::catppuccin_latte_visuals(),
            Self::CatppuccinFrappe => catppuccin::catppuccin_frappe_visuals(),
            Self::CatppuccinMacchiato => catppuccin::catppuccin_macchiato_visuals(),
            Self::CatppuccinMocha => catppuccin::catppuccin_mocha_visuals(),
            Self::Dracula => dracula::dracula_visuals(),
            Self::Gruvbox => gruvbox::gruvbox_dark_visuals(),
            Self::Nord => nord::nord_visuals(),
            Self::OneDark => one_dark::one_dark_visuals(),
            Self::RosePine => rose_pine::rose_pine_visuals(),
            Self::RosePineMoon => rose_pine::rose_pine_moon_visuals(),
            Self::TokyoNight => tokyo_night::tokyo_night_visuals(),
            Self::TokyoNightStorm => tokyo_night::tokyo_night_storm_visuals(),
            Self::TokyoNightDay => tokyo_night::tokyo_night_day_visuals(),
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

    style.spacing.indent = 0.0;

    ctx.set_style(style);
}

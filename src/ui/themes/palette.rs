use eframe::egui::{self, Color32};

use crate::model::theme::Theme;

use super::{catppuccin, dracula, gruvbox, nord, one_dark, rose_pine, tokyo_night};

pub(crate) struct Palette {
    pub(crate) active_bg_fill: Color32,
    pub(crate) active_bg_fill_scale: Option<f32>,
    pub(crate) active_bg_stroke: Color32,
    pub(crate) active_foreground: Color32,
    pub(crate) active_weak_bg_fill: Color32,
    pub(crate) active_weak_bg_fill_scale: Option<f32>,
    pub(crate) code_background: Color32,
    pub(crate) error_foreground: Color32,
    pub(crate) extreme_background: Color32,
    pub(crate) faint_background: Color32,
    pub(crate) foreground: Color32,
    pub(crate) hovered_bg_fill: Color32,
    pub(crate) hovered_bg_fill_scale: Option<f32>,
    pub(crate) hovered_bg_stroke: Color32,
    pub(crate) hovered_foreground: Color32,
    pub(crate) hovered_weak_bg_fill: Color32,
    pub(crate) hyperlink: Color32,
    pub(crate) inactive_bg_fill: Color32,
    pub(crate) inactive_bg_stroke: Color32,
    pub(crate) inactive_foreground: Color32,
    pub(crate) inactive_weak_bg_fill: Color32,
    pub(crate) is_dark: bool,
    pub(crate) noninteractive_bg_fill: Color32,
    pub(crate) noninteractive_bg_stroke: Color32,
    pub(crate) noninteractive_weak_bg_fill: Color32,
    pub(crate) open_bg_fill: Color32,
    pub(crate) open_bg_fill_scale: Option<f32>,
    pub(crate) open_bg_stroke: Color32,
    pub(crate) open_weak_bg_fill: Color32,
    pub(crate) panel_fill: Color32,
    pub(crate) popup_shadow_alpha: u8,
    pub(crate) selection: Color32,
    pub(crate) selection_scale: f32,
    pub(crate) text_cursor: Color32,
    pub(crate) warn_foreground: Color32,
    pub(crate) window_fill: Color32,
    pub(crate) window_shadow_alpha: u8,
    pub(crate) window_stroke: Color32,
}

pub fn apply(theme: Theme, context: &egui::Context) {
    context.set_visuals(visuals_from_palette(palette_for(theme)));
}

fn apply_shape_visuals(visuals: &mut egui::Visuals, palette: &Palette) {
    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0_f32, palette.text_cursor);
    visuals.clip_rect_margin = 3.0;
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = egui::style::HandleShape::Circle;
    visuals.override_text_color = None;
}

fn apply_widget_visuals(visuals: &mut egui::Visuals, palette: &Palette) {
    visuals.widgets.noninteractive.bg_fill = palette.noninteractive_bg_fill;
    visuals.widgets.noninteractive.weak_bg_fill = palette.noninteractive_weak_bg_fill;

    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0_f32, palette.noninteractive_bg_stroke);

    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, palette.foreground);

    visuals.widgets.inactive.bg_fill = palette.inactive_bg_fill;
    visuals.widgets.inactive.weak_bg_fill = palette.inactive_weak_bg_fill;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, palette.inactive_bg_stroke);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, palette.inactive_foreground);

    visuals.widgets.hovered.bg_fill =
        scaled(palette.hovered_bg_fill, palette.hovered_bg_fill_scale);

    visuals.widgets.hovered.weak_bg_fill = palette.hovered_weak_bg_fill;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, palette.hovered_bg_stroke);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5_f32, palette.hovered_foreground);

    visuals.widgets.active.bg_fill = scaled(palette.active_bg_fill, palette.active_bg_fill_scale);

    visuals.widgets.active.weak_bg_fill = scaled(
        palette.active_weak_bg_fill,
        palette.active_weak_bg_fill_scale,
    );

    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, palette.active_bg_stroke);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, palette.active_foreground);

    visuals.widgets.open.bg_fill = scaled(palette.open_bg_fill, palette.open_bg_fill_scale);
    visuals.widgets.open.weak_bg_fill = palette.open_weak_bg_fill;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, palette.open_bg_stroke);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, palette.hovered_foreground);
}

fn apply_window_visuals(visuals: &mut egui::Visuals, palette: &Palette) {
    visuals.selection.bg_fill = palette.selection.linear_multiply(palette.selection_scale);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, palette.selection);

    visuals.hyperlink_color = palette.hyperlink;
    visuals.faint_bg_color = palette.faint_background;
    visuals.extreme_bg_color = palette.extreme_background;
    visuals.code_bg_color = palette.code_background;
    visuals.warn_fg_color = palette.warn_foreground;
    visuals.error_fg_color = palette.error_foreground;

    visuals.window_fill = palette.window_fill;
    visuals.window_stroke = egui::Stroke::new(1.0_f32, palette.window_stroke);

    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(palette.window_shadow_alpha),
    };

    visuals.panel_fill = palette.panel_fill;

    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: Color32::from_black_alpha(palette.popup_shadow_alpha),
    };
}

fn palette_for(theme: Theme) -> &'static Palette {
    let palette = match theme {
        Theme::CatppuccinFrappe => &catppuccin::CATPPUCCIN_FRAPPE,
        Theme::CatppuccinLatte => &catppuccin::CATPPUCCIN_LATTE,
        Theme::CatppuccinMacchiato => &catppuccin::CATPPUCCIN_MACCHIATO,
        Theme::CatppuccinMocha => &catppuccin::CATPPUCCIN_MOCHA,
        Theme::Dracula => &dracula::DRACULA,
        Theme::Gruvbox => &gruvbox::GRUVBOX,
        Theme::Nord => &nord::NORD,
        Theme::OneDark => &one_dark::ONE_DARK,
        Theme::RosePine => &rose_pine::ROSE_PINE,
        Theme::RosePineMoon => &rose_pine::ROSE_PINE_MOON,
        Theme::TokyoNight => &tokyo_night::TOKYO_NIGHT,
        Theme::TokyoNightDay => &tokyo_night::TOKYO_NIGHT_DAY,
        Theme::TokyoNightStorm => &tokyo_night::TOKYO_NIGHT_STORM,
    };

    debug_assert!(palette.selection_scale > 0.0);

    palette
}

fn scaled(color: Color32, factor: Option<f32>) -> Color32 {
    factor.map_or(color, |scale| color.linear_multiply(scale))
}

fn visuals_from_palette(palette: &Palette) -> egui::Visuals {
    let mut visuals = if palette.is_dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    apply_widget_visuals(&mut visuals, palette);
    apply_window_visuals(&mut visuals, palette);
    apply_shape_visuals(&mut visuals, palette);

    visuals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_has_a_valid_palette() {
        for theme in Theme::all() {
            let palette = palette_for(*theme);

            assert!(palette.selection_scale > 0.0);
            assert!(palette.selection_scale <= 1.0);
        }
    }

    #[test]
    fn light_themes_are_marked_light() {
        assert!(!palette_for(Theme::CatppuccinLatte).is_dark);
        assert!(!palette_for(Theme::TokyoNightDay).is_dark);
        assert!(palette_for(Theme::RosePine).is_dark);
    }
}

use eframe::egui;

pub fn one_dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let bg        = egui::Color32::from_rgb(40, 44, 52);
    let bg_alt    = egui::Color32::from_rgb(33, 37, 43);
    let panel     = egui::Color32::from_rgb(49, 54, 63);
    let stroke    = egui::Color32::from_rgb(67, 72, 83);
    let text      = egui::Color32::from_rgb(220, 223, 228);
    let subtext   = egui::Color32::from_rgb(160, 166, 176);
    let blue      = egui::Color32::from_rgb(97, 175, 239);
    let cyan      = egui::Color32::from_rgb(86, 182, 194);
    let magenta   = egui::Color32::from_rgb(198, 120, 221);
    let yellow    = egui::Color32::from_rgb(229, 192, 123);
    let red       = egui::Color32::from_rgb(224, 108, 117);
    let _green     = egui::Color32::from_rgb(152, 195, 121);

    visuals.widgets.noninteractive.bg_fill   = panel;
    visuals.widgets.noninteractive.weak_bg_fill = bg_alt;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, stroke);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill   = panel;
    visuals.widgets.inactive.weak_bg_fill = bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, stroke);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, subtext);

    visuals.widgets.hovered.bg_fill   = stroke;
    visuals.widgets.hovered.weak_bg_fill = panel;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, magenta);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, text);

    visuals.widgets.active.bg_fill   = stroke.linear_multiply(1.2);
    visuals.widgets.active.weak_bg_fill = stroke;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, blue);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, text);

    visuals.widgets.open.bg_fill   = stroke;
    visuals.widgets.open.weak_bg_fill = panel;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, cyan);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.selection.bg_fill = blue.linear_multiply(0.38);
    visuals.selection.stroke  = egui::Stroke::new(1.0, blue);

    visuals.hyperlink_color  = blue;
    visuals.faint_bg_color   = bg_alt;
    visuals.extreme_bg_color = bg;
    visuals.code_bg_color    = panel;
    visuals.warn_fg_color    = yellow;
    visuals.error_fg_color   = red;

    visuals.window_fill   = bg;
    visuals.window_stroke = egui::Stroke::new(1.0, panel);
    visuals.window_shadow = egui::epaint::Shadow::NONE;

    visuals.panel_fill    = panel;
    visuals.popup_shadow  = egui::epaint::Shadow::NONE;

    visuals.resize_corner_size  = 12.0;
    visuals.text_cursor.stroke  = egui::Stroke::new(2.0, blue);
    visuals.clip_rect_margin    = 3.0;
    visuals.button_frame        = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline   = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = egui::style::HandleShape::Circle;
    visuals.override_text_color = None;

    visuals
}

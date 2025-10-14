use eframe::egui;

pub fn gruvbox_dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let bg0    = egui::Color32::from_rgb(40, 40, 40);
    let bg1    = egui::Color32::from_rgb(50, 48, 47);
    let bg2    = egui::Color32::from_rgb(60, 56, 54);
    let fg     = egui::Color32::from_rgb(235, 219, 178);
    let dim    = egui::Color32::from_rgb(189, 174, 147);
    let yellow = egui::Color32::from_rgb(250, 189, 47);
    let blue   = egui::Color32::from_rgb(131, 165, 152);
    let _aqua   = egui::Color32::from_rgb(142, 192, 124);
    let purple = egui::Color32::from_rgb(211, 134, 155);
    let red    = egui::Color32::from_rgb(204, 36, 29);
    let orange = egui::Color32::from_rgb(214, 93, 14);

    visuals.widgets.noninteractive.bg_fill   = bg1;
    visuals.widgets.noninteractive.weak_bg_fill = bg0;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, bg2);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.widgets.inactive.bg_fill   = bg2;
    visuals.widgets.inactive.weak_bg_fill = bg1;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, bg2);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, dim);

    visuals.widgets.hovered.bg_fill   = bg2.linear_multiply(1.1);
    visuals.widgets.hovered.weak_bg_fill = bg2;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, purple);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, fg);

    visuals.widgets.active.bg_fill   = bg2.linear_multiply(1.25);
    visuals.widgets.active.weak_bg_fill = bg2.linear_multiply(1.1);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, yellow);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, fg);

    visuals.widgets.open.bg_fill   = bg2;
    visuals.widgets.open.weak_bg_fill = bg1;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, blue);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.selection.bg_fill = orange.linear_multiply(0.35);
    visuals.selection.stroke  = egui::Stroke::new(1.0, orange);

    visuals.hyperlink_color  = blue;
    visuals.faint_bg_color   = bg0;
    visuals.extreme_bg_color = bg0;
    visuals.code_bg_color    = bg1;
    visuals.warn_fg_color    = yellow;
    visuals.error_fg_color   = red;

    visuals.window_fill   = bg0;
    visuals.window_stroke = egui::Stroke::new(1.0, bg1);
    visuals.window_shadow = egui::epaint::Shadow::NONE;

    visuals.panel_fill    = bg1;
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

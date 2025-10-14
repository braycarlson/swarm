use eframe::egui;

pub fn dracula_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let background = egui::Color32::from_rgb(40, 42, 54);
    let current_line = egui::Color32::from_rgb(68, 71, 90);
    let selection = egui::Color32::from_rgb(68, 71, 90);
    let foreground = egui::Color32::from_rgb(248, 248, 242);
    let comment = egui::Color32::from_rgb(98, 114, 164);
    let cyan = egui::Color32::from_rgb(139, 233, 253);
    let _green = egui::Color32::from_rgb(80, 250, 123);
    let _orange = egui::Color32::from_rgb(255, 184, 108);
    let pink = egui::Color32::from_rgb(255, 121, 198);
    let purple = egui::Color32::from_rgb(189, 147, 249);
    let red = egui::Color32::from_rgb(255, 85, 85);
    let yellow = egui::Color32::from_rgb(241, 250, 140);

    visuals.widgets.noninteractive.bg_fill = current_line;
    visuals.widgets.noninteractive.weak_bg_fill = background;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, selection);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, foreground);

    visuals.widgets.inactive.bg_fill = selection;
    visuals.widgets.inactive.weak_bg_fill = current_line;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, comment);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, comment);

    visuals.widgets.hovered.bg_fill = selection.linear_multiply(1.2);
    visuals.widgets.hovered.weak_bg_fill = selection;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, purple);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, foreground);

    visuals.widgets.active.bg_fill = selection.linear_multiply(1.4);
    visuals.widgets.active.weak_bg_fill = selection.linear_multiply(1.2);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, pink);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, foreground);

    visuals.widgets.open.bg_fill = selection.linear_multiply(1.2);
    visuals.widgets.open.weak_bg_fill = selection;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, cyan);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, foreground);

    visuals.selection.bg_fill = purple.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, purple);

    visuals.hyperlink_color = cyan;
    visuals.faint_bg_color = background;
    visuals.extreme_bg_color = background;
    visuals.code_bg_color = current_line;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = background;
    visuals.window_stroke = egui::Stroke::new(1.0, current_line);
    visuals.window_shadow = egui::epaint::Shadow::NONE;

    visuals.panel_fill = current_line;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, pink);
    visuals.clip_rect_margin = 3.0;
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = egui::style::HandleShape::Circle;

    visuals.override_text_color = None;

    visuals
}

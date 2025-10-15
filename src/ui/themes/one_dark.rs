use eframe::egui;

pub fn one_dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let bg = egui::Color32::from_rgb(40, 44, 52);
    let bg_alt = egui::Color32::from_rgb(33, 37, 43);
    let panel = egui::Color32::from_rgb(49, 54, 62);
    let selection = egui::Color32::from_rgb(61, 66, 77);
    let fg = egui::Color32::from_rgb(171, 178, 191);
    let comment = egui::Color32::from_rgb(92, 99, 112);
    let red = egui::Color32::from_rgb(224, 108, 117);
    let _orange = egui::Color32::from_rgb(209, 154, 102);
    let yellow = egui::Color32::from_rgb(229, 192, 123);
    let _green = egui::Color32::from_rgb(152, 195, 121);
    let cyan = egui::Color32::from_rgb(86, 182, 194);
    let blue = egui::Color32::from_rgb(97, 175, 239);
    let purple = egui::Color32::from_rgb(198, 120, 221);
    let pink = egui::Color32::from_rgb(229, 192, 123);

    visuals.widgets.noninteractive.bg_fill = panel;
    visuals.widgets.noninteractive.weak_bg_fill = bg;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, selection);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.widgets.inactive.bg_fill = selection;
    visuals.widgets.inactive.weak_bg_fill = panel;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, comment);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, comment);

    visuals.widgets.hovered.bg_fill = selection.linear_multiply(1.2);
    visuals.widgets.hovered.weak_bg_fill = selection;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, purple);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, fg);

    visuals.widgets.active.bg_fill = selection.linear_multiply(1.4);
    visuals.widgets.active.weak_bg_fill = selection.linear_multiply(1.2);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, pink);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, fg);

    visuals.widgets.open.bg_fill = selection.linear_multiply(1.2);
    visuals.widgets.open.weak_bg_fill = selection;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, cyan);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.selection.bg_fill = purple.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, purple);

    visuals.hyperlink_color = cyan;
    visuals.faint_bg_color = bg_alt;
    visuals.extreme_bg_color = bg_alt;
    visuals.code_bg_color = panel;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = bg_alt;
    visuals.window_stroke = egui::Stroke::new(1.0, panel);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.panel_fill = panel;
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(60),
    };

    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, blue);
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

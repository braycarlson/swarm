use eframe::egui;

pub fn tokyo_night_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let bg = egui::Color32::from_rgb(26, 27, 38);
    let bg_dark = egui::Color32::from_rgb(22, 22, 30);
    let bg_highlight = egui::Color32::from_rgb(41, 46, 66);
    let bg_highlight_dark = egui::Color32::from_rgb(32, 36, 52);
    let terminal_black = egui::Color32::from_rgb(65, 72, 104);
    let fg = egui::Color32::from_rgb(192, 202, 245);
    let _fg_dark = egui::Color32::from_rgb(169, 177, 214);
    let comment = egui::Color32::from_rgb(86, 95, 137);
    let blue = egui::Color32::from_rgb(125, 207, 255);
    let cyan = egui::Color32::from_rgb(125, 207, 255);
    let _green = egui::Color32::from_rgb(158, 206, 106);
    let magenta = egui::Color32::from_rgb(187, 154, 247);
    let _orange = egui::Color32::from_rgb(255, 158, 100);
    let red = egui::Color32::from_rgb(247, 118, 142);
    let yellow = egui::Color32::from_rgb(224, 175, 104);

    visuals.widgets.noninteractive.bg_fill = bg_highlight;
    visuals.widgets.noninteractive.weak_bg_fill = bg_dark;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, terminal_black);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.widgets.inactive.bg_fill = bg_highlight;
    visuals.widgets.inactive.weak_bg_fill = bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, terminal_black);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, comment);

    visuals.widgets.hovered.bg_fill = terminal_black;
    visuals.widgets.hovered.weak_bg_fill = bg_highlight;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, magenta);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, fg);

    visuals.widgets.active.bg_fill = terminal_black.linear_multiply(1.2);
    visuals.widgets.active.weak_bg_fill = terminal_black;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, blue);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, fg);

    visuals.widgets.open.bg_fill = terminal_black;
    visuals.widgets.open.weak_bg_fill = bg_highlight;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, cyan);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.selection.bg_fill = magenta.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, magenta);

    visuals.hyperlink_color = cyan;
    visuals.faint_bg_color = bg_dark;
    visuals.extreme_bg_color = bg;
    visuals.code_bg_color = bg_highlight;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = bg;
    visuals.window_stroke = egui::Stroke::new(1.0, bg_highlight);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.panel_fill = bg_highlight_dark;
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

pub fn tokyo_night_storm_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let bg = egui::Color32::from_rgb(36, 40, 59);
    let bg_dark = egui::Color32::from_rgb(31, 35, 53);
    let bg_highlight = egui::Color32::from_rgb(56, 62, 90);
    let bg_highlight_dark = egui::Color32::from_rgb(44, 49, 70);
    let terminal_black = egui::Color32::from_rgb(68, 75, 106);
    let fg = egui::Color32::from_rgb(169, 177, 214);
    let comment = egui::Color32::from_rgb(86, 95, 137);
    let blue = egui::Color32::from_rgb(125, 207, 255);
    let cyan = egui::Color32::from_rgb(125, 207, 255);
    let magenta = egui::Color32::from_rgb(187, 154, 247);
    let red = egui::Color32::from_rgb(247, 118, 142);
    let yellow = egui::Color32::from_rgb(224, 175, 104);

    visuals.widgets.noninteractive.bg_fill = bg_highlight;
    visuals.widgets.noninteractive.weak_bg_fill = bg_dark;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, terminal_black);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.widgets.inactive.bg_fill = bg_highlight;
    visuals.widgets.inactive.weak_bg_fill = bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, terminal_black);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, comment);

    visuals.widgets.hovered.bg_fill = terminal_black;
    visuals.widgets.hovered.weak_bg_fill = bg_highlight;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, magenta);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, fg);

    visuals.widgets.active.bg_fill = terminal_black.linear_multiply(1.2);
    visuals.widgets.active.weak_bg_fill = terminal_black;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, blue);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, fg);

    visuals.widgets.open.bg_fill = terminal_black;
    visuals.widgets.open.weak_bg_fill = bg_highlight;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, cyan);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.selection.bg_fill = magenta.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, magenta);

    visuals.hyperlink_color = cyan;
    visuals.faint_bg_color = bg_dark;
    visuals.extreme_bg_color = bg;
    visuals.code_bg_color = bg_highlight;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = bg;
    visuals.window_stroke = egui::Stroke::new(1.0, bg_highlight);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.panel_fill = bg_highlight_dark;
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

pub fn tokyo_night_day_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();

    let bg = egui::Color32::from_rgb(230, 237, 243);
    let bg_dark = egui::Color32::from_rgb(213, 223, 232);
    let bg_highlight = egui::Color32::from_rgb(197, 210, 224);
    let bg_highlight_darker = egui::Color32::from_rgb(185, 200, 216);
    let terminal_black = egui::Color32::from_rgb(169, 184, 203);
    let fg = egui::Color32::from_rgb(52, 59, 88);
    let comment = egui::Color32::from_rgb(148, 163, 184);
    let blue = egui::Color32::from_rgb(52, 84, 138);
    let cyan = egui::Color32::from_rgb(15, 75, 110);
    let magenta = egui::Color32::from_rgb(116, 80, 171);
    let red = egui::Color32::from_rgb(143, 39, 67);
    let yellow = egui::Color32::from_rgb(143, 94, 21);

    visuals.widgets.noninteractive.bg_fill = bg_highlight;
    visuals.widgets.noninteractive.weak_bg_fill = bg_dark;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, terminal_black);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.widgets.inactive.bg_fill = bg_highlight;
    visuals.widgets.inactive.weak_bg_fill = bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, terminal_black);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, comment);

    visuals.widgets.hovered.bg_fill = terminal_black;
    visuals.widgets.hovered.weak_bg_fill = bg_highlight;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, magenta);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, fg);

    visuals.widgets.active.bg_fill = terminal_black.linear_multiply(0.9);
    visuals.widgets.active.weak_bg_fill = terminal_black;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, blue);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, fg);

    visuals.widgets.open.bg_fill = terminal_black;
    visuals.widgets.open.weak_bg_fill = bg_highlight;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, cyan);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, fg);

    visuals.selection.bg_fill = magenta.linear_multiply(0.3);
    visuals.selection.stroke = egui::Stroke::new(1.0, magenta);

    visuals.hyperlink_color = cyan;
    visuals.faint_bg_color = bg_dark;
    visuals.extreme_bg_color = bg;
    visuals.code_bg_color = bg_highlight;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = bg;
    visuals.window_stroke = egui::Stroke::new(1.0, bg_highlight);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(40),
    };

    visuals.panel_fill = bg_highlight_darker;
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(30),
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

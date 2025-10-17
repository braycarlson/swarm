use eframe::egui;

pub fn catppuccin_mocha_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let base = egui::Color32::from_rgb(30, 30, 46);
    let mantle = egui::Color32::from_rgb(24, 24, 37);
    let surface0 = egui::Color32::from_rgb(49, 50, 68);
    let surface0_dark = egui::Color32::from_rgb(39, 40, 54);
    let surface1 = egui::Color32::from_rgb(69, 71, 90);
    let surface2 = egui::Color32::from_rgb(88, 91, 112);
    let text = egui::Color32::from_rgb(205, 214, 244);
    let subtext = egui::Color32::from_rgb(166, 173, 200);
    let lavender = egui::Color32::from_rgb(180, 190, 254);
    let blue = egui::Color32::from_rgb(137, 180, 250);
    let sapphire = egui::Color32::from_rgb(116, 199, 236);
    let _sky = egui::Color32::from_rgb(137, 220, 235);
    let _teal = egui::Color32::from_rgb(148, 226, 213);
    let _green = egui::Color32::from_rgb(166, 227, 161);
    let yellow = egui::Color32::from_rgb(249, 226, 175);
    let _peach = egui::Color32::from_rgb(250, 179, 135);
    let _maroon = egui::Color32::from_rgb(238, 153, 160);
    let red = egui::Color32::from_rgb(243, 139, 168);
    let mauve = egui::Color32::from_rgb(203, 166, 247);
    let _pink = egui::Color32::from_rgb(245, 194, 231);

    visuals.widgets.noninteractive.bg_fill = surface0;
    visuals.widgets.noninteractive.weak_bg_fill = mantle;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, surface1);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill = surface1;
    visuals.widgets.inactive.weak_bg_fill = surface0;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, surface2);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, subtext);

    visuals.widgets.hovered.bg_fill = surface2;
    visuals.widgets.hovered.weak_bg_fill = surface1;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, mauve);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, text);

    visuals.widgets.active.bg_fill = surface2.linear_multiply(1.2);
    visuals.widgets.active.weak_bg_fill = surface2;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, lavender);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, text);

    visuals.widgets.open.bg_fill = surface2;
    visuals.widgets.open.weak_bg_fill = surface1;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, blue);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.selection.bg_fill = mauve.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, mauve);

    visuals.hyperlink_color = sapphire;
    visuals.faint_bg_color = mantle;
    visuals.extreme_bg_color = base;
    visuals.code_bg_color = surface0;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = base;
    visuals.window_stroke = egui::Stroke::new(1.0, surface0);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.panel_fill = surface0_dark;

    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(60),
    };

    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, lavender);
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

pub fn catppuccin_latte_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();

    let base = egui::Color32::from_rgb(239, 241, 245);
    let mantle = egui::Color32::from_rgb(230, 233, 239);
    let surface0 = egui::Color32::from_rgb(204, 208, 218);
    let surface0_darker = egui::Color32::from_rgb(194, 198, 208);
    let surface1 = egui::Color32::from_rgb(188, 192, 204);
    let surface2 = egui::Color32::from_rgb(172, 176, 190);
    let text = egui::Color32::from_rgb(76, 79, 105);
    let subtext = egui::Color32::from_rgb(92, 95, 119);
    let lavender = egui::Color32::from_rgb(114, 135, 253);
    let mauve = egui::Color32::from_rgb(136, 57, 239);
    let sapphire = egui::Color32::from_rgb(32, 159, 181);
    let yellow = egui::Color32::from_rgb(223, 142, 29);
    let red = egui::Color32::from_rgb(210, 15, 57);

    visuals.widgets.noninteractive.bg_fill = surface0;
    visuals.widgets.noninteractive.weak_bg_fill = mantle;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, surface1);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill = surface1;
    visuals.widgets.inactive.weak_bg_fill = surface0;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, surface2);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, subtext);

    visuals.widgets.hovered.bg_fill = surface2;
    visuals.widgets.hovered.weak_bg_fill = surface1;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, mauve);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, text);

    visuals.widgets.active.bg_fill = surface2.linear_multiply(0.9);
    visuals.widgets.active.weak_bg_fill = surface2;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, lavender);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, text);

    visuals.widgets.open.bg_fill = surface2;
    visuals.widgets.open.weak_bg_fill = surface1;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, sapphire);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.selection.bg_fill = mauve.linear_multiply(0.3);
    visuals.selection.stroke = egui::Stroke::new(1.0, mauve);

    visuals.hyperlink_color = sapphire;
    visuals.faint_bg_color = mantle;
    visuals.extreme_bg_color = base;
    visuals.code_bg_color = surface0;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = base;
    visuals.window_stroke = egui::Stroke::new(1.0, surface0);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(40),
    };

    visuals.panel_fill = surface0_darker;
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(30),
    };

    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, lavender);
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

pub fn catppuccin_frappe_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let base = egui::Color32::from_rgb(48, 52, 70);
    let mantle = egui::Color32::from_rgb(41, 44, 60);
    let surface0 = egui::Color32::from_rgb(65, 69, 89);
    let surface0_dark = egui::Color32::from_rgb(55, 59, 75);
    let surface1 = egui::Color32::from_rgb(81, 87, 109);
    let surface2 = egui::Color32::from_rgb(98, 104, 128);
    let text = egui::Color32::from_rgb(198, 208, 245);
    let subtext = egui::Color32::from_rgb(163, 173, 210);
    let lavender = egui::Color32::from_rgb(186, 187, 241);
    let mauve = egui::Color32::from_rgb(202, 158, 230);
    let sapphire = egui::Color32::from_rgb(133, 193, 220);
    let yellow = egui::Color32::from_rgb(229, 200, 144);
    let red = egui::Color32::from_rgb(231, 130, 132);

    visuals.widgets.noninteractive.bg_fill = surface0;
    visuals.widgets.noninteractive.weak_bg_fill = mantle;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, surface1);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill = surface1;
    visuals.widgets.inactive.weak_bg_fill = surface0;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, surface2);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, subtext);

    visuals.widgets.hovered.bg_fill = surface2;
    visuals.widgets.hovered.weak_bg_fill = surface1;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, mauve);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, text);

    visuals.widgets.active.bg_fill = surface2.linear_multiply(1.2);
    visuals.widgets.active.weak_bg_fill = surface2;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, lavender);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, text);

    visuals.widgets.open.bg_fill = surface2;
    visuals.widgets.open.weak_bg_fill = surface1;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, sapphire);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.selection.bg_fill = mauve.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, mauve);

    visuals.hyperlink_color = sapphire;
    visuals.faint_bg_color = mantle;
    visuals.extreme_bg_color = base;
    visuals.code_bg_color = surface0;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = base;
    visuals.window_stroke = egui::Stroke::new(1.0, surface0);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.panel_fill = surface0_dark;
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(60),
    };

    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, lavender);
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

pub fn catppuccin_macchiato_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let base = egui::Color32::from_rgb(36, 39, 58);
    let mantle = egui::Color32::from_rgb(30, 32, 48);
    let surface0 = egui::Color32::from_rgb(54, 58, 79);
    let surface0_dark = egui::Color32::from_rgb(44, 48, 65);
    let surface1 = egui::Color32::from_rgb(73, 77, 100);
    let surface2 = egui::Color32::from_rgb(91, 96, 120);
    let text = egui::Color32::from_rgb(202, 211, 245);
    let subtext = egui::Color32::from_rgb(165, 173, 203);
    let lavender = egui::Color32::from_rgb(183, 189, 248);
    let mauve = egui::Color32::from_rgb(198, 160, 246);
    let sapphire = egui::Color32::from_rgb(125, 196, 228);
    let yellow = egui::Color32::from_rgb(238, 212, 159);
    let red = egui::Color32::from_rgb(237, 135, 150);

    visuals.widgets.noninteractive.bg_fill = surface0;
    visuals.widgets.noninteractive.weak_bg_fill = mantle;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, surface1);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill = surface1;
    visuals.widgets.inactive.weak_bg_fill = surface0;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, surface2);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, subtext);

    visuals.widgets.hovered.bg_fill = surface2;
    visuals.widgets.hovered.weak_bg_fill = surface1;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, mauve);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, text);

    visuals.widgets.active.bg_fill = surface2.linear_multiply(1.2);
    visuals.widgets.active.weak_bg_fill = surface2;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, lavender);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, text);

    visuals.widgets.open.bg_fill = surface2;
    visuals.widgets.open.weak_bg_fill = surface1;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, sapphire);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.selection.bg_fill = mauve.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, mauve);

    visuals.hyperlink_color = sapphire;
    visuals.faint_bg_color = mantle;
    visuals.extreme_bg_color = base;
    visuals.code_bg_color = surface0;
    visuals.warn_fg_color = yellow;
    visuals.error_fg_color = red;

    visuals.window_fill = base;
    visuals.window_stroke = egui::Stroke::new(1.0, surface0);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.panel_fill = surface0_dark;
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(60),
    };

    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, lavender);
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

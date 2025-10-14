use eframe::egui;

pub fn nord_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let polar_night_0 = egui::Color32::from_rgb(46, 52, 64);
    let polar_night_1 = egui::Color32::from_rgb(59, 66, 82);
    let polar_night_2 = egui::Color32::from_rgb(67, 76, 94);
    let polar_night_3 = egui::Color32::from_rgb(76, 86, 106);
    let snow_storm_0 = egui::Color32::from_rgb(216, 222, 233);
    let snow_storm_1 = egui::Color32::from_rgb(229, 233, 240);
    let snow_storm_2 = egui::Color32::from_rgb(236, 239, 244);
    let frost_0 = egui::Color32::from_rgb(143, 188, 187);
    let frost_1 = egui::Color32::from_rgb(136, 192, 208);
    let frost_2 = egui::Color32::from_rgb(129, 161, 193);
    let frost_3 = egui::Color32::from_rgb(94, 129, 172);
    let aurora_0 = egui::Color32::from_rgb(191, 97, 106);
    let _aurora_1 = egui::Color32::from_rgb(208, 135, 112);
    let aurora_2 = egui::Color32::from_rgb(235, 203, 139);
    let _aurora_3 = egui::Color32::from_rgb(163, 190, 140);
    let _aurora_4 = egui::Color32::from_rgb(180, 142, 173);

    visuals.widgets.noninteractive.bg_fill = polar_night_1;
    visuals.widgets.noninteractive.weak_bg_fill = polar_night_0;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, polar_night_2);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, snow_storm_0);

    visuals.widgets.inactive.bg_fill = polar_night_2;
    visuals.widgets.inactive.weak_bg_fill = polar_night_1;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, polar_night_3);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, snow_storm_0);

    visuals.widgets.hovered.bg_fill = polar_night_3;
    visuals.widgets.hovered.weak_bg_fill = polar_night_2;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, frost_3);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, snow_storm_1);

    visuals.widgets.active.bg_fill = polar_night_3.linear_multiply(1.2);
    visuals.widgets.active.weak_bg_fill = polar_night_3;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, frost_2);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, snow_storm_2);

    visuals.widgets.open.bg_fill = polar_night_3;
    visuals.widgets.open.weak_bg_fill = polar_night_2;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, frost_1);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, snow_storm_1);

    visuals.selection.bg_fill = frost_3.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, frost_3);

    visuals.hyperlink_color = frost_1;
    visuals.faint_bg_color = polar_night_0;
    visuals.extreme_bg_color = polar_night_0;
    visuals.code_bg_color = polar_night_1;
    visuals.warn_fg_color = aurora_2;
    visuals.error_fg_color = aurora_0;

    visuals.window_fill = polar_night_0;
    visuals.window_stroke = egui::Stroke::new(1.0, polar_night_1);
    visuals.window_shadow = egui::epaint::Shadow::NONE;

    visuals.panel_fill = polar_night_1;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, frost_0);
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

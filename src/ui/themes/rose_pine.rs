use eframe::egui;

pub fn rose_pine_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let base = egui::Color32::from_rgb(25, 23, 36);
    let surface = egui::Color32::from_rgb(31, 29, 46);
    let overlay = egui::Color32::from_rgb(38, 35, 58);
    let subtle = egui::Color32::from_rgb(144, 140, 170);
    let text = egui::Color32::from_rgb(224, 222, 244);
    let love = egui::Color32::from_rgb(235, 111, 146);
    let gold = egui::Color32::from_rgb(246, 193, 119);
    let foam = egui::Color32::from_rgb(156, 207, 216);
    let iris = egui::Color32::from_rgb(196, 167, 231);
    let highlight_low = egui::Color32::from_rgb(33, 32, 46);
    let highlight_med = egui::Color32::from_rgb(64, 61, 82);
    let highlight_high = egui::Color32::from_rgb(82, 79, 103);

    apply_common_settings(
        &mut visuals,
        base,
        surface,
        overlay,
        subtle,
        text,
        love,
        gold,
        foam,
        iris,
        highlight_low,
        highlight_med,
        highlight_high,
    );

    visuals
}

pub fn rose_pine_moon_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    let base = egui::Color32::from_rgb(35, 33, 54);
    let surface = egui::Color32::from_rgb(42, 39, 63);
    let overlay = egui::Color32::from_rgb(57, 53, 82);
    let subtle = egui::Color32::from_rgb(144, 140, 170);
    let text = egui::Color32::from_rgb(224, 222, 244);
    let love = egui::Color32::from_rgb(235, 111, 146);
    let gold = egui::Color32::from_rgb(246, 193, 119);
    let foam = egui::Color32::from_rgb(156, 207, 216);
    let iris = egui::Color32::from_rgb(196, 167, 231);
    let highlight_low = egui::Color32::from_rgb(42, 40, 62);
    let highlight_med = egui::Color32::from_rgb(68, 65, 90);
    let highlight_high = egui::Color32::from_rgb(86, 82, 110);

    apply_common_settings(
        &mut visuals,
        base,
        surface,
        overlay,
        subtle,
        text,
        love,
        gold,
        foam,
        iris,
        highlight_low,
        highlight_med,
        highlight_high,
    );

    visuals
}

fn apply_common_settings(
    visuals: &mut egui::Visuals,
    base: egui::Color32,
    surface: egui::Color32,
    overlay: egui::Color32,
    subtle: egui::Color32,
    text: egui::Color32,
    love: egui::Color32,
    gold: egui::Color32,
    foam: egui::Color32,
    iris: egui::Color32,
    highlight_low: egui::Color32,
    highlight_med: egui::Color32,
    highlight_high: egui::Color32,
) {
    visuals.widgets.noninteractive.bg_fill = surface;
    visuals.widgets.noninteractive.weak_bg_fill = base;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, overlay);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill = overlay;
    visuals.widgets.inactive.weak_bg_fill = surface;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, highlight_med);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, subtle);

    visuals.widgets.hovered.bg_fill = highlight_med;
    visuals.widgets.hovered.weak_bg_fill = overlay;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, iris);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, text);

    visuals.widgets.active.bg_fill = highlight_high;
    visuals.widgets.active.weak_bg_fill = highlight_med;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, highlight_high);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, text);

    visuals.widgets.open.bg_fill = highlight_med;
    visuals.widgets.open.weak_bg_fill = overlay;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, foam);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.selection.bg_fill = iris.linear_multiply(0.4);
    visuals.selection.stroke = egui::Stroke::new(1.0, iris);

    visuals.hyperlink_color = foam;
    visuals.faint_bg_color = highlight_low;
    visuals.extreme_bg_color = base;
    visuals.code_bg_color = overlay;
    visuals.warn_fg_color = gold;
    visuals.error_fg_color = love;

    visuals.window_fill = base;
    visuals.window_stroke = egui::Stroke::new(1.0, overlay);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    visuals.panel_fill = surface;

    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(60),
    };

    visuals.resize_corner_size = 12.0;
    visuals.text_cursor.stroke = egui::Stroke::new(2.0, iris);
    visuals.clip_rect_margin = 3.0;
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = egui::style::HandleShape::Circle;

    visuals.override_text_color = None;
}

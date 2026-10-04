use eframe::egui;

use crate::constants::{TITLE_BAR_HEIGHT_PIXELS, TITLE_BUTTON_WIDTH_PIXELS};

use super::icon::{CLOSE_HOVER_COLOR, IconButton, TitleIcon};

const TITLE_INSET_PIXELS: f32 = 8.0;
const TITLE_SIZE_POINTS: f32 = 14.0;

pub struct Modal<'modal> {
    pub identifier: &'modal str,
    pub size: [f32; 2],
    pub title: &'modal str,
}

pub fn focus_if_pending(response: &egui::Response, pending: &mut bool) {
    if core::mem::take(pending) {
        response.request_focus();
    }

    debug_assert!(!*pending);
}

pub fn render_centered(
    ui: &mut egui::Ui,
    content_height: f32,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let available = ui.available_size();

    debug_assert!(content_height >= 0.0);

    ui.allocate_ui_with_layout(
        available,
        egui::Layout::top_down(egui::Align::Center),
        |cell| {
            cell.add_space(((available.y - content_height) / 2.0).max(0.0));
            add_contents(cell);
        },
    );
}

pub fn render_modal(
    context: &egui::Context,
    modal: &Modal<'_>,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> bool {
    assert_ne!(modal.identifier, "");

    let mut closed = false;

    egui::Window::new(modal.identifier)
        .title_bar(false)
        .resizable(false)
        .fixed_size(modal.size)
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(context.content_rect().center())
        .show(context, |window| {
            closed = render_titlebar(window, modal.title);
            add_contents(window);
        });

    closed
}

fn render_titlebar(ui: &mut egui::Ui, title: &str) -> bool {
    let content_rectangle = ui.max_rect();

    let title_rectangle = egui::Rect::from_min_size(
        content_rectangle.min,
        egui::vec2(content_rectangle.width(), TITLE_BAR_HEIGHT_PIXELS),
    );

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), TITLE_BAR_HEIGHT_PIXELS),
        egui::Layout::left_to_right(egui::Align::Center),
        |cell| {
            cell.set_min_height(TITLE_BAR_HEIGHT_PIXELS);
            cell.add_space(TITLE_INSET_PIXELS);
            cell.label(egui::RichText::new(title).size(TITLE_SIZE_POINTS));
        },
    );

    let close = IconButton {
        clip: title_rectangle,
        fill_hover: CLOSE_HOVER_COLOR,
        icon: TitleIcon::Close,
        id_salt: "modal-close",
        rectangle: egui::Rect::from_min_size(
            title_rectangle.right_top() - egui::vec2(TITLE_BUTTON_WIDTH_PIXELS, 0.0),
            egui::vec2(TITLE_BUTTON_WIDTH_PIXELS, TITLE_BAR_HEIGHT_PIXELS),
        ),
    };

    let clicked = close.render(ui);

    ui.separator();

    clicked
}

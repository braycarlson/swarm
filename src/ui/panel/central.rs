use eframe::egui;

use crate::app::message::{Message, MessageSender, Search, Tree};
use crate::app::state::{FilterStatus, Model, UiState};
use crate::ui::panel::tree;
use crate::ui::widget::icon::{TitleIcon, draw_title_icon};
use crate::ui::widget::window::{Modal, render_centered, render_modal};

const BULK_SELECT_SIZE: [f32; 2] = [400.0, 300.0];
const CLEAR_ICON_MARGIN_PIXELS: f32 = 3.0;
const FILTER_WIDTH_PIXELS: f32 = 55.0;
const PLACEHOLDER_HEIGHT_PIXELS: f32 = 50.0;
const RELOAD_WIDTH_PIXELS: f32 = 85.0;
const SEARCH_BAR_PADDING_PIXELS: f32 = 12.0;
const SEARCH_WIDTH_MIN_PIXELS: f32 = 100.0;

pub fn render(ui: &mut egui::Ui, model: &Model, ui_state: &mut UiState, sender: &MessageSender) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::central_panel(&ui.ctx().global_style())
                .inner_margin(egui::Margin::symmetric(18, 18)),
        )
        .show_inside(ui, |panel| {
            render_search_bar(panel, ui_state, sender);
            panel.add_space(5.0);
            render_tree_frame(panel, model, ui_state, sender);
        });

    if ui_state.bulk_select_show {
        render_bulk_select_window(ui.ctx(), ui_state, sender);
    }
}

fn render_bulk_select_window(
    context: &egui::Context,
    ui_state: &mut UiState,
    sender: &MessageSender,
) {
    let dialog = Modal {
        identifier: "select_files",
        size: BULK_SELECT_SIZE,
        title: "Select Files",
    };

    let closed = render_modal(context, &dialog, |modal| {
        modal.add_space(4.0);

        let text_height = (modal.available_height() - 40.0).max(100.0);

        egui::ScrollArea::vertical()
            .max_height(text_height)
            .auto_shrink([false, true])
            .show(modal, |scroll| {
                scroll.add(
                    egui::TextEdit::multiline(&mut ui_state.bulk_select_text)
                        .desired_rows(10)
                        .desired_width(scroll.available_width()),
                );
            });

        modal.add_space(18.0);

        let has_text = !ui_state.bulk_select_text.trim().is_empty();

        if modal
            .add_enabled(has_text, egui::Button::new("Select"))
            .clicked()
        {
            sender.send(Message::Tree(Tree::BulkSelectApplied));
        }
    });

    if closed {
        sender.send(Message::Tree(Tree::BulkSelectToggled));
    }
}

fn render_search_bar(ui: &mut egui::Ui, ui_state: &mut UiState, sender: &MessageSender) {
    let row_height = ui.spacing().interact_size.y;
    let bar_height = row_height + SEARCH_BAR_PADDING_PIXELS;
    let spacing = ui.spacing().item_spacing.x;
    let buttons_width = spacing.mul_add(2.0, FILTER_WIDTH_PIXELS + RELOAD_WIDTH_PIXELS);
    let search_width = (ui.available_width() - buttons_width).max(SEARCH_WIDTH_MIN_PIXELS);

    ui.horizontal(|row| {
        row.allocate_ui(egui::vec2(search_width, bar_height), |cell| {
            egui::Frame::new()
                .fill(cell.visuals().extreme_bg_color)
                .stroke(cell.visuals().widgets.noninteractive.bg_stroke)
                .inner_margin(egui::Margin {
                    bottom: 0,
                    left: 12,
                    right: 12,
                    top: 4,
                })
                .corner_radius(4.0)
                .show(cell, |content| {
                    render_search_field(content, ui_state, sender);
                });
        });

        let filter =
            egui::Button::new("Filter").min_size(egui::vec2(FILTER_WIDTH_PIXELS, bar_height));

        if row
            .add(filter)
            .on_hover_text("Select files from a list")
            .clicked()
        {
            sender.send(Message::Tree(Tree::BulkSelectToggled));
        }

        let reload =
            egui::Button::new("Reload").min_size(egui::vec2(RELOAD_WIDTH_PIXELS, bar_height));

        if row.add(reload).clicked() {
            sender.send(Message::Tree(Tree::RefreshRequested));
        }
    });
}

fn render_search_field(ui: &mut egui::Ui, ui_state: &mut UiState, sender: &MessageSender) {
    let row_height = ui.spacing().interact_size.y;
    let bar_height = row_height + SEARCH_BAR_PADDING_PIXELS;

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), bar_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |cell| {
            cell.set_min_height(bar_height);

            let clear_width = row_height + 4.0;
            let text_width = (cell.available_width() - clear_width).max(0.0);

            let response = cell.add_sized(
                [text_width, row_height],
                egui::TextEdit::singleline(&mut ui_state.search_text)
                    .hint_text("Search tree...")
                    .frame(egui::Frame::NONE),
            );

            if response.changed() {
                sender.send(Message::Search(Search::QueryEdited));
            }

            let (rectangle, clear) =
                cell.allocate_exact_size(egui::vec2(row_height, row_height), egui::Sense::click());

            if ui_state.search_text.is_empty() {
                return;
            }

            let color = if clear.hovered() {
                cell.visuals().strong_text_color()
            } else {
                cell.visuals().text_color()
            };

            draw_title_icon(
                cell.painter(),
                rectangle.expand(CLEAR_ICON_MARGIN_PIXELS),
                TitleIcon::Close,
                color,
            );

            if clear
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                sender.send(Message::Search(Search::Cleared));
            }
        },
    );
}

fn render_tree_frame(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &mut UiState,
    sender: &MessageSender,
) {
    let frame = egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .inner_margin(egui::Margin::symmetric(8, 8))
        .corner_radius(4.0);

    frame.show(ui, |content| {
        content.set_min_size(content.available_size());

        if ui_state.filter_status == FilterStatus::Filtering {
            render_centered(content, PLACEHOLDER_HEIGHT_PIXELS, |centered| {
                centered.spinner();
                centered.add_space(8.0);
                centered.label("Searching files...");
            });

            return;
        }

        tree::render(content, model, &mut ui_state.tree_rows, sender);
    });
}

use eframe::egui;

use crate::app::message::{CopyMessage, Message, MessageSender, Render, Skeleton};
use crate::app::state::ui::GenerateMode;
use crate::app::state::{FilterStatus, LoadStatus, Model, UiState};

const ARROW_HALF_PIXELS: f32 = 3.5;
const BUTTON_PADDING_PIXELS: f32 = 8.0;
const COPY_WIDTH_PIXELS: f32 = 120.0;
const GENERATE_WIDTH_PIXELS: f32 = 150.0;
const MENU_WIDTH_PIXELS: f32 = 170.0;
const SPLIT_WIDTH_PIXELS: f32 = 22.0;

fn paint_dropdown_arrow(ui: &egui::Ui, response: &egui::Response) {
    let center = response.rect.center();
    let half = ARROW_HALF_PIXELS;

    let points = vec![
        egui::pos2(center.x - half, half.mul_add(-0.4, center.y)),
        egui::pos2(center.x + half, half.mul_add(-0.4, center.y)),
        egui::pos2(center.x, half.mul_add(0.7, center.y)),
    ];

    let color = if response.hovered() {
        ui.visuals().strong_text_color()
    } else {
        ui.visuals().text_color()
    };

    ui.painter().add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

pub fn render(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &MessageSender) {
    egui::Panel::bottom("bottom_panel")
        .min_size(40.0)
        .resizable(false)
        .show_inside(ui, |panel| {
            panel.vertical_centered(|centered| {
                centered.add_space(10.0);
                centered.horizontal(|row| render_controls(row, model, ui_state, sender));
                centered.add_space(8.0);
            });
        });
}

fn render_controls(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &MessageSender) {
    let button_height = ui.spacing().interact_size.y + BUTTON_PADDING_PIXELS;
    let busy = model.tree.is_loading() || ui_state.filter_status == FilterStatus::Filtering;
    let can_copy = !ui_state.copy_in_progress && !busy;

    let copy_label = if ui_state.copy_in_progress {
        "Copying..."
    } else {
        "Copy"
    };

    let copy = egui::Button::new(copy_label).min_size(egui::vec2(COPY_WIDTH_PIXELS, button_height));

    if ui.add_enabled(can_copy, copy).clicked() {
        sender.send(Message::Copy(CopyMessage::Requested));
    }

    render_generate_split_button(ui, ui_state, sender, button_height, busy);

    ui.with_layout(
        egui::Layout::right_to_left(egui::Align::Center),
        |aligned| {
            aligned.add_space(10.0);
            render_status(aligned, model);
        },
    );
}

fn render_generate_split_button(
    ui: &mut egui::Ui,
    ui_state: &UiState,
    sender: &MessageSender,
    button_height: f32,
    busy: bool,
) {
    let generating = ui_state.is_generating();
    let can_generate = !generating && !busy;

    let label = match (generating, ui_state.generate_mode) {
        (true, _) => "Generating...",
        (false, GenerateMode::Skeleton) => "Generate Skeleton",
        (false, GenerateMode::Tree) => "Generate Tree",
    };

    let spacing_saved = ui.spacing().item_spacing.x;

    ui.spacing_mut().item_spacing.x = 2.0;

    let generate =
        egui::Button::new(label).min_size(egui::vec2(GENERATE_WIDTH_PIXELS, button_height));

    if ui.add_enabled(can_generate, generate).clicked() {
        let message = match ui_state.generate_mode {
            GenerateMode::Skeleton => Message::Skeleton(Skeleton::Requested),
            GenerateMode::Tree => Message::Render(Render::Requested),
        };

        sender.send(message);
    }

    let split = egui::Button::new("  ").min_size(egui::vec2(SPLIT_WIDTH_PIXELS, button_height));
    let arrow = ui.add_enabled(can_generate, split);

    paint_dropdown_arrow(ui, &arrow);
    ui.spacing_mut().item_spacing.x = spacing_saved;

    egui::Popup::from_toggle_button_response(&arrow)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
        .show(|popup: &mut egui::Ui| {
            popup.set_min_width(MENU_WIDTH_PIXELS);

            for (mode, text) in [
                (GenerateMode::Tree, "Generate Tree"),
                (GenerateMode::Skeleton, "Generate Skeleton"),
            ] {
                if popup
                    .selectable_label(ui_state.generate_mode == mode, text)
                    .clicked()
                {
                    sender.send(Message::Skeleton(Skeleton::ModeChanged(mode)));
                }
            }
        });
}

fn render_status(ui: &mut egui::Ui, model: &Model) {
    let weak = ui.visuals().weak_text_color();

    if let LoadStatus::Loading(message) = &model.tree.load_status {
        ui.spinner();
        ui.label(egui::RichText::new(message).color(weak));

        return;
    }

    if model.tree.files_count > 0 {
        ui.label(egui::RichText::new(&model.tree.files_label).color(weak));
    }
}

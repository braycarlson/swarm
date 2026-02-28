use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{Copy, Msg, Render, Skeleton};
use crate::app::state::{LoadStatus, Model, UiState};
use crate::app::state::ui::GenerateMode;

pub fn render(
    ctx: &egui::Context,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    egui::TopBottomPanel::bottom("bottom_panel")
        .min_height(40.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let row_height = ui.spacing().interact_size.y;
                    let padding = 8.0;

                    let tree_is_loading = matches!(model.tree.load_status, LoadStatus::Loading { .. });
                    let can_copy = !ui_state.copy_in_progress && !tree_is_loading;

                    let copy_label = if ui_state.copy_in_progress {
                        "Copying..."
                    } else {
                        "Copy"
                    };

                    if ui.add_enabled(
                        can_copy,
                        egui::Button::new(copy_label).min_size(egui::vec2(120.0, row_height + padding))
                    ).clicked() {
                        let _ = sender.send(Msg::Copy(Copy::Requested));
                    }

                    render_generate_split_button(ui, ui_state, sender, row_height, padding, tree_is_loading);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);

                        if matches!(model.tree.load_status, LoadStatus::Loading { .. }) {
                            ui.spinner();

                            if let LoadStatus::Loading { message, .. } = &model.tree.load_status {
                                ui.label(
                                    egui::RichText::new(message)
                                        .color(ui.visuals().weak_text_color())
                                );
                            }
                        }

                        if !tree_is_loading && model.tree.file_count > 0 {
                            ui.label(
                                egui::RichText::new(format!("{} files", model.tree.file_count))
                                    .color(ui.visuals().weak_text_color())
                            );
                        }
                    });
                });

                ui.add_space(8.0);
            });
        });
}

fn render_generate_split_button(
    ui: &mut egui::Ui,
    ui_state: &UiState,
    sender: &Sender<Msg>,
    row_height: f32,
    padding: f32,
    tree_is_loading: bool,
) {
    let any_gen_in_progress = ui_state.tree_gen_in_progress || ui_state.skeleton_gen_in_progress;
    let can_generate = !any_gen_in_progress && !tree_is_loading;
    let button_height = row_height + padding;

    let main_label = if any_gen_in_progress {
        match ui_state.generate_mode {
            GenerateMode::Tree => "Generating...",
            GenerateMode::Skeleton => "Generating...",
        }
    } else {
        match ui_state.generate_mode {
            GenerateMode::Tree => "Generate Tree",
            GenerateMode::Skeleton => "Generate Skeleton",
        }
    };

    let saved_spacing = ui.spacing().item_spacing.x;
    ui.spacing_mut().item_spacing.x = 2.0;

    if ui.add_enabled(
        can_generate,
        egui::Button::new(main_label).min_size(egui::vec2(150.0, button_height))
    ).clicked() {
        match ui_state.generate_mode {
            GenerateMode::Tree => {
                let _ = sender.send(Msg::Render(Render::Requested));
            }
            GenerateMode::Skeleton => {
                let _ = sender.send(Msg::Skeleton(Skeleton::Requested));
            }
        }
    }

    let arrow_response = ui.add_enabled(
        can_generate,
        egui::Button::new("  ").min_size(egui::vec2(22.0, button_height))
    );

    paint_dropdown_arrow(ui, &arrow_response);

    ui.spacing_mut().item_spacing.x = saved_spacing;

    egui::Popup::from_toggle_button_response(&arrow_response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
        .show(|ui: &mut egui::Ui| {
            ui.set_min_width(170.0);

            if ui.selectable_label(
                ui_state.generate_mode == GenerateMode::Tree,
                "Generate Tree",
            ).clicked() {
                let _ = sender.send(Msg::Skeleton(Skeleton::ModeChanged(GenerateMode::Tree)));
            }

            if ui.selectable_label(
                ui_state.generate_mode == GenerateMode::Skeleton,
                "Generate Skeleton",
            ).clicked() {
                let _ = sender.send(Msg::Skeleton(Skeleton::ModeChanged(GenerateMode::Skeleton)));
            }
        });
}

fn paint_dropdown_arrow(ui: &egui::Ui, response: &egui::Response) {
    let center = response.rect.center();
    let half = 3.5;

    let points = vec![
        egui::pos2(center.x - half, center.y - half * 0.4),
        egui::pos2(center.x + half, center.y - half * 0.4),
        egui::pos2(center.x, center.y + half * 0.7),
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

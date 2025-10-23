use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{Copy, Msg, Render};
use crate::app::state::{LoadStatus, Model, UiState};

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
                        "📋 Copying..."
                    } else {
                        "📋 Copy"
                    };

                    if ui.add_enabled(
                        can_copy,
                        egui::Button::new(copy_label).min_size(egui::vec2(120.0, row_height + padding))
                    ).clicked() {
                        let _ = sender.send(Msg::Copy(Copy::Requested));
                    }

                    let can_generate_tree = !ui_state.tree_gen_in_progress && !tree_is_loading;

                    let tree_label = if ui_state.tree_gen_in_progress {
                        "🌳 Generating..."
                    } else {
                        "🌳 Generate Tree"
                    };

                    if ui.add_enabled(
                        can_generate_tree,
                        egui::Button::new(tree_label).min_size(egui::vec2(150.0, row_height + padding))
                    ).clicked() {
                        let _ = sender.send(Msg::Render(Render::Requested));
                    }

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

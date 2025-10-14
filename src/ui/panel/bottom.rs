use std::sync::mpsc::Sender;

use eframe::egui;
use humansize::{format_size, BINARY};

use crate::app::message::{Copy, Index, Msg, TreeGen};
use crate::app::state::{IndexStatus, LoadStatus, Model, UiState};

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
                        let _ = sender.send(Msg::TreeGen(TreeGen::Requested));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        render_index_status(ui, model, sender);
                        ui.add_space(10.0);
                        render_tree_loading_status(ui, model);
                    });
                });

                ui.add_space(8.0);
            });
        });
}

fn render_tree_loading_status(ui: &mut egui::Ui, model: &Model) {
    if matches!(model.tree.load_status, LoadStatus::Loading { .. }) {
        ui.spinner();

        if let LoadStatus::Loading { message, .. } = &model.tree.load_status {
            ui.label(
                egui::RichText::new(message)
                    .color(ui.visuals().weak_text_color())
            );
        }
    }
}

fn render_index_status(
    ui: &mut egui::Ui,
    model: &Model,
    sender: &Sender<Msg>,
) {
    ui.horizontal(|ui| {
        match &model.index.status {
            IndexStatus::Running { paused } => {
                if ui.small_button("⏹").clicked() {
                    sender.send(Msg::Index(Index::StopRequested)).ok();
                }

                if *paused {
                    if ui.small_button("▶").clicked() {
                        sender.send(Msg::Index(Index::ResumeRequested)).ok();
                    }
                } else {
                    if ui.small_button("⏸").clicked() {
                        sender.send(Msg::Index(Index::PauseRequested)).ok();
                    }
                }

                ui.label(if *paused { "Indexing paused" } else { "Indexing in progress" });
            }
            _ => {
                if ui.small_button("🔄").clicked() {
                    sender.send(Msg::Index(Index::StartRequested)).ok();
                }

                if let Some(stats) = &model.index.statistics {
                    ui.label(
                        egui::RichText::new(format!(
                            "Indexed: {} files ({})",
                            stats.indexed_files,
                            format_size(stats.total_size, BINARY)
                        ))
                        .color(ui.visuals().weak_text_color())
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Index not built")
                            .color(ui.visuals().weak_text_color())
                    );
                }
            }
        }
    });
}

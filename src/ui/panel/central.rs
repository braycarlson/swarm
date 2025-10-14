use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{Msg, Search, Tree};
use crate::app::state::{Model, UiState};
use crate::services::tree::operations::should_show_node;
use crate::model::node::{FileNode, NodeKind};

pub fn render(
    ctx: &egui::Context,
    model: &Model,
    _ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::central_panel(&ctx.style())
                .inner_margin(egui::Margin::symmetric(18, 18))
        )
        .show(ctx, |ui| {
            render_search_bar(ui, model, sender);
            ui.add_space(5.0);

            let tree_frame = egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .inner_margin(egui::Margin::symmetric(8, 8))
                .corner_radius(4.0);

            tree_frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        render_tree_nodes(ui, &model.tree.nodes, &model.search.query, sender);
                    });
            });
        });
}

fn render_search_bar(
    ui: &mut egui::Ui,
    model: &Model,
    sender: &Sender<Msg>,
) {
    let reload_width = 85.0;
    let spacing = ui.spacing().item_spacing.x;
    let total_button_width = reload_width + spacing;

    ui.horizontal(|ui| {
        let search_width = ui.available_width() - total_button_width;
        let row_height = ui.spacing().interact_size.y;
        let padding = 8.0;

        ui.allocate_ui_with_layout(
            egui::vec2(search_width, row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let frame = egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .corner_radius(4.0);

                frame.show(ui, |ui| {
                    ui.set_min_height(row_height);
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);

                        ui.add_sized(
                            [row_height, row_height],
                            egui::Label::new(
                                egui::RichText::new("🔍")
                                    .color(ui.visuals().weak_text_color())
                            )
                        );

                        ui.add_space(4.0);

                        let mut query = model.search.query.clone();
                        let text_width = ui.available_width() - (row_height + row_height) + 5.0;

                        let response = ui.add_sized(
                            [text_width.max(0.0), row_height],
                            egui::TextEdit::singleline(&mut query)
                                .hint_text("Search tree...")
                                .frame(false)
                        );

                        if response.changed() {
                            let _ = sender.send(Msg::Search(Search::QueryChanged(query)));
                        }

                        ui.add_space(4.0);

                        let clear = ui.add(
                            egui::Button::new("×")
                                .min_size(egui::vec2(row_height, row_height))
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                        );

                        if !model.search.query.is_empty() && clear.clicked() {
                            let _ = sender.send(Msg::Search(Search::Cleared));
                        }
                    });
                });
            }
        );

        if ui.add(
            egui::Button::new("🔄 Reload")
                .min_size(egui::vec2(reload_width, row_height + padding))
        ).clicked() {
            let _ = sender.send(Msg::Tree(Tree::RefreshRequested));
        }
    });
}

fn render_tree_nodes(
    ui: &mut egui::Ui,
    nodes: &[FileNode],
    search_query: &str,
    sender: &Sender<Msg>,
) {
    for (i, node) in nodes.iter().enumerate() {
        render_node(ui, node, vec![i], 0, search_query, sender);
    }
}

fn render_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    path: Vec<usize>,
    depth: usize,
    search_query: &str,
    sender: &Sender<Msg>,
) {
    if !should_show_node(node, search_query) {
        return;
    }

    let label = node.file_name().unwrap_or_else(|| node.path.display().to_string());

    match node.kind {
        NodeKind::File => {
            ui.horizontal(|ui| {
                let mut checked = node.checked;
                if ui.checkbox(&mut checked, "").clicked() {
                    sender.send(Msg::Tree(Tree::NodeToggled {
                        path: path.clone(),
                        checked,
                        propagate: false,
                    })).ok();
                }

                if ui.selectable_label(node.checked, &label).clicked() {
                    sender.send(Msg::Tree(Tree::NodeToggled {
                        path,
                        checked: !node.checked,
                        propagate: false,
                    })).ok();
                }
            });
        }
        NodeKind::Directory => {
            ui.horizontal(|ui| {
                let mut checked = node.checked;
                if ui.checkbox(&mut checked, "").clicked() {
                    sender.send(Msg::Tree(Tree::NodeToggled {
                        path: path.clone(),
                        checked,
                        propagate: true,
                    })).ok();
                }

                let default_open = depth == 0 || !search_query.is_empty();

                let header = egui::CollapsingHeader::new(&label)
                    .id_salt(&node.path)
                    .default_open(default_open);

                header.show(ui, |ui| {
                    if !node.loaded {
                        sender.send(Msg::Tree(Tree::NodeExpanded { path: path.clone() })).ok();
                        ui.spinner();
                        ui.label("Loading...");
                    } else {
                        for (i, child) in node.children.iter().enumerate() {
                            let mut child_path = path.clone();
                            child_path.push(i);
                            render_node(ui, child, child_path, depth + 1, search_query, sender);
                        }
                    }
                });
            });
        }
    }
}

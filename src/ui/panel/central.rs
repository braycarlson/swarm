use std::path::Path;
use std::sync::mpsc::Sender;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use eframe::egui;

use crate::app::message::{Msg, Search, Tree};
use crate::app::state::{FilterStatus, Model, UiState};
use crate::model::node::{FileNode, NodeKind};
use crate::services::filesystem::git::GitService;
use crate::services::tree::traversal::should_show_node_at_depth;

pub fn render(
    ctx: &egui::Context,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::central_panel(&ctx.style())
                .inner_margin(egui::Margin::symmetric(18, 18))
        )
        .show(ctx, |ui| {
            render_search_bar(ui, model, ui_state, sender);
            ui.add_space(5.0);

            let tree_frame = egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .inner_margin(egui::Margin::symmetric(8, 8))
                .corner_radius(4.0);

            tree_frame.show(ui, |ui| {
                let available_size = ui.available_size();
                ui.set_min_size(available_size);

                if ui_state.filter_status == FilterStatus::Filtering {
                    ui.allocate_ui_with_layout(
                        available_size,
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let content_height = 50.0;
                            let space_above = (available_size.y - content_height) / 2.0;
                            ui.add_space(space_above.max(0.0));

                            ui.spinner();
                            ui.add_space(8.0);
                            ui.label("Searching files...");
                        }
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            render_tree_nodes(ui, &model.tree.nodes, model, &model.git, sender);

                            let remaining = ui.available_size();

                            if remaining.y > 0.0 {
                                let (_, response) = ui.allocate_exact_size(
                                    remaining,
                                    egui::Sense::click(),
                                );

                                response.context_menu(|ui| {
                                    if ui.button("Select All").clicked() {
                                        sender.send(Msg::Tree(Tree::SelectAll)).ok();
                                        ui.close();
                                    }

                                    if ui.button("Deselect All").clicked() {
                                        sender.send(Msg::Tree(Tree::DeselectAll)).ok();
                                        ui.close();
                                    }
                                });
                            }
                        });
                }
            });
        });
}

fn render_search_bar(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &UiState,
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

                        let icon = if ui_state.filter_status == FilterStatus::Filtering {
                            "⏳"
                        } else {
                            "🔍"
                        };

                        ui.add_sized(
                            [row_height, row_height],
                            egui::Label::new(
                                egui::RichText::new(icon)
                                    .color(ui.visuals().weak_text_color())
                            )
                        );

                        ui.add_space(4.0);

                        let display_query = ui_state.search_pending.as_ref()
                            .unwrap_or(&model.search.query);

                        let mut query = display_query.clone();
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
                        ).on_hover_cursor(egui::CursorIcon::PointingHand);

                        let current_query = ui_state.search_pending.as_ref()
                            .unwrap_or(&model.search.query);

                        if !current_query.is_empty() && clear.clicked() {
                            let _ = sender.send(Msg::Search(Search::Cleared));
                        }
                    });
                });
            }
        );

        if ui.add(
            egui::Button::new("Reload")
                .min_size(egui::vec2(reload_width, row_height + padding))
        ).clicked() {
            let _ = sender.send(Msg::Tree(Tree::RefreshRequested));
        }
    });
}

fn render_tree_nodes(
    ui: &mut egui::Ui,
    nodes: &[FileNode],
    model: &Model,
    git: &GitService,
    sender: &Sender<Msg>,
) {
    let query = model.search.parsed();

    for (i, node) in nodes.iter().enumerate() {
        render_node(ui, node, vec![i], 0, model, git, sender, &query);
    }
}

fn render_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    path: Vec<usize>,
    depth: usize,
    model: &Model,
    git: &GitService,
    sender: &Sender<Msg>,
    query: &crate::app::state::search::ParsedQuery,
) {
    if !should_show_node_at_depth(node, &model.search, Some(git), depth, query) {
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

                let response = ui.selectable_label(node.checked, &label);

                if response.clicked() {
                    sender.send(Msg::Tree(Tree::NodeToggled {
                        path: path.clone(),
                        checked: !node.checked,
                        propagate: false,
                    })).ok();
                }

                response.context_menu(|ui| {
                    if ui.button("Open File").clicked() {
                        open_file(&node.path);
                        ui.close();
                    }

                    if ui.button("Reveal in Explorer").clicked() {
                        reveal_in_explorer(&node.path);
                        ui.close();
                    }

                    ui.separator();

                    let toggle_label = if node.checked { "Deselect" } else { "Select" };

                    if ui.button(toggle_label).clicked() {
                        sender.send(Msg::Tree(Tree::NodeToggled {
                            path,
                            checked: !node.checked,
                            propagate: false,
                        })).ok();
                        ui.close();
                    }
                });
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

                let default_open = depth == 0;

                let header = egui::CollapsingHeader::new(&label)
                    .id_salt(&node.path)
                    .default_open(default_open);

                let cr = header.show(ui, |ui| {
                    if !node.loaded {
                        sender.send(Msg::Tree(Tree::NodeExpanded { path: path.clone() })).ok();
                        ui.spinner();
                        ui.label("Loading...");
                    } else {
                        for (i, child) in node.children.iter().enumerate() {
                            let mut child_path = path.clone();
                            child_path.push(i);
                            render_node(ui, child, child_path, depth + 1, model, git, sender, query);
                        }
                    }
                });

                cr.header_response.context_menu(|ui| {
                    if ui.button("Select All").clicked() {
                        sender.send(Msg::Tree(Tree::NodeToggled {
                            path: path.clone(),
                            checked: true,
                            propagate: true,
                        })).ok();
                        ui.close();
                    }

                    if ui.button("Deselect All").clicked() {
                        sender.send(Msg::Tree(Tree::NodeToggled {
                            path: path.clone(),
                            checked: false,
                            propagate: true,
                        })).ok();
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("Reveal in Explorer").clicked() {
                        reveal_in_explorer(&node.path);
                        ui.close();
                    }
                });
            });
        }
    }
}

fn open_file(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", &path.display().to_string()])
            .creation_flags(0x08000000)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }
}

fn reveal_in_explorer(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .args(["/select,", &path.display().to_string()])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .args(["-R", &path.display().to_string()])
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn();
        }
    }
}

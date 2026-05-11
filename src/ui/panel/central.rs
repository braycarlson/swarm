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
use crate::ui::widget::titlebar::icon::{TitleIcon, draw_title_icon, hover_rect};

pub fn render(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::central_panel(&ui.ctx().global_style())
                .inner_margin(egui::Margin::symmetric(18, 18))
        )
        .show_inside(ui, |ui| {
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
                                    show_tree_context_menu(ui, sender);
                                });
                            }
                        });
                }
            });
        });

    if ui_state.show_bulk_select {
        render_bulk_select_window(ui, ui_state, sender);
    }
}

fn render_search_bar(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    let row_height = ui.spacing().interact_size.y;
    let bar_height = row_height + 12.0;
    let filter_width = 55.0;
    let reload_width = 85.0;
    let spacing = ui.spacing().item_spacing.x;
    let buttons_width = filter_width + reload_width + spacing * 2.0;
    let search_width = (ui.available_width() - buttons_width).max(100.0);

    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(search_width, bar_height), |ui| {
            egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .inner_margin(egui::Margin { left: 12, right: 12, top: 4, bottom: 0 })
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), bar_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_min_height(bar_height);

                            let display_query = ui_state.search_pending.as_ref()
                                .unwrap_or(&model.search.query);

                            let mut query = display_query.clone();
                            let clear_width = row_height + 4.0;
                            let text_width = ui.available_width() - clear_width;

                            let response = ui.add_sized(
                                [text_width.max(0.0), row_height],
                                egui::TextEdit::singleline(&mut query)
                                    .hint_text("Search tree...")
                                    .frame(egui::Frame::NONE)
                            );

                            if response.changed() {
                                let _ = sender.send(Msg::Search(Search::QueryChanged(query)));
                            }

                            let current_query = ui_state.search_pending.as_ref()
                                .unwrap_or(&model.search.query);

                            let has_query = !current_query.is_empty();

                            let (rect, clear) = ui.allocate_exact_size(
                                egui::vec2(row_height, row_height),
                                egui::Sense::click(),
                            );

                            if has_query {
                                let center = rect.center();
                                let half = 3.0;
                                let y_offset = -1.5;

                                let color = if clear.hovered() {
                                    ui.visuals().strong_text_color()
                                } else {
                                    ui.visuals().text_color()
                                };

                                let stroke = egui::Stroke::new(1.2, color);

                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center.x - half, center.y - half + y_offset),
                                        egui::pos2(center.x + half, center.y + half + y_offset),
                                    ],
                                    stroke,
                                );

                                ui.painter().line_segment(
                                    [
                                        egui::pos2(center.x + half, center.y - half + y_offset),
                                        egui::pos2(center.x - half, center.y + half + y_offset),
                                    ],
                                    stroke,
                                );

                                if clear.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    let _ = sender.send(Msg::Search(Search::Cleared));
                                }
                            }
                        }
                    );
                });
        });

        if ui.add(
            egui::Button::new("Filter")
                .min_size(egui::vec2(filter_width, bar_height))
        ).on_hover_text("Select files from a list").clicked() {
            let _ = sender.send(Msg::Tree(Tree::BulkSelectToggled));
        }

        if ui.add(
            egui::Button::new("Reload")
                .min_size(egui::vec2(reload_width, bar_height))
        ).clicked() {
            let _ = sender.send(Msg::Tree(Tree::RefreshRequested));
        }
    });
}

fn render_bulk_select_window(
    ui: &mut egui::Ui,
    ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    let center = ui.ctx().content_rect().center();
    let title_bar_height = 32.0;
    let button_width = 46.0;

    egui::Window::new("select_files")
        .title_bar(false)
        .resizable(false)
        .fixed_size([400.0, 300.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .show(ui.ctx(), |ui| {
            let content_rect = ui.max_rect();

            let title_rect = egui::Rect::from_min_size(
                content_rect.min,
                egui::vec2(content_rect.width(), title_bar_height),
            );

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), title_bar_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_min_height(title_bar_height);
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Select Files").size(14.0));
                }
            );

            let close_rect = egui::Rect::from_min_size(
                title_rect.right_top() - egui::vec2(button_width, 0.0),
                egui::vec2(button_width, title_bar_height),
            );

            let close_response = ui.interact(
                close_rect,
                ui.id().with("bulk_close"),
                egui::Sense::click(),
            );

            if close_response.hovered() {
                let p = ui.painter().with_clip_rect(title_rect);
                p.rect_filled(
                    hover_rect(close_rect, title_rect),
                    0.0,
                    egui::Color32::from_rgb(232, 17, 35),
                );
            }

            {
                let fg = if close_response.hovered() {
                    egui::Color32::WHITE
                } else {
                    ui.visuals().text_color()
                };

                let p = ui.painter().with_clip_rect(title_rect);
                draw_title_icon(&p, close_rect, TitleIcon::Close, fg);
            }

            if close_response.clicked() {
                sender.send(Msg::Tree(Tree::BulkSelectToggled)).ok();
            }

            ui.separator();
            ui.add_space(4.0);

            let mut text = ui_state.bulk_select_text.clone();
            let text_height = (ui.available_height() - 35.0).max(100.0);

            let response = ui.add_sized(
                [ui.available_width(), text_height],
                egui::TextEdit::multiline(&mut text)
                    .desired_rows(10)
            );

            if response.changed() {
                sender.send(Msg::Tree(Tree::BulkSelectTextChanged(text))).ok();
            }

            ui.add_space(4.0);

            let has_text = !ui_state.bulk_select_text.trim().is_empty();

            if ui.add_enabled(has_text, egui::Button::new("Select")).clicked() {
                sender.send(Msg::Tree(Tree::BulkSelectApplied)).ok();
            }
        });
}

fn show_tree_context_menu(ui: &mut egui::Ui, sender: &Sender<Msg>) {
    if ui.button("Select All").clicked() {
        sender.send(Msg::Tree(Tree::SelectAll)).ok();
        ui.close();
    }

    if ui.button("Deselect All").clicked() {
        sender.send(Msg::Tree(Tree::DeselectAll)).ok();
        ui.close();
    }
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

                let row_remaining = ui.available_size();

                if row_remaining.x > 0.0 {
                    let (_, fill) = ui.allocate_exact_size(
                        row_remaining,
                        egui::Sense::click(),
                    );

                    fill.context_menu(|ui| {
                        show_tree_context_menu(ui, sender);
                    });
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

                    if ui.button("Open Folder").clicked() {
                        open_file(&node.path);
                        ui.close();
                    }

                    if ui.button("Reveal in Explorer").clicked() {
                        reveal_in_explorer(&node.path);
                        ui.close();
                    }
                });

                let row_remaining = ui.available_size();

                if row_remaining.x > 0.0 {
                    let (_, fill) = ui.allocate_exact_size(
                        row_remaining,
                        egui::Sense::click(),
                    );

                    fill.context_menu(|ui| {
                        show_tree_context_menu(ui, sender);
                    });
                }
            });
        }
    }
}

fn open_file(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let _ = Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .creation_flags(CREATE_NO_WINDOW)
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
    let target = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let _ = Command::new("explorer")
            .arg(target)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(target)
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(target)
            .spawn();
    }
}

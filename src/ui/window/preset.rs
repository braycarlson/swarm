use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{Message, Preset_};
use crate::app::state::{Model, UiState};
use crate::ui::widget::titlebar::icon::{TitleIcon, draw_title_icon, hover_rect};

pub fn render_save(ctx: &egui::Context, ui_state: &UiState, sender: &Sender<Message>) {
    let center = ctx.content_rect().center();
    let title_bar_height = 32.0;
    let button_width = 46.0;

    egui::Window::new("save_preset")
        .title_bar(false)
        .resizable(false)
        .fixed_size([420.0, 350.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .show(ctx, |ui| {
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
                    ui.label(egui::RichText::new("Save Preset").size(14.0));
                }
            );

            let close_rect = egui::Rect::from_min_size(
                title_rect.right_top() - egui::vec2(button_width, 0.0),
                egui::vec2(button_width, title_bar_height),
            );

            let close_response = ui.interact(
                close_rect,
                ui.id().with("save_preset_close"),
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
                sender.send(Message::Preset(Preset_::SaveDialogClosed)).ok();
            }

            ui.separator();

            ui.vertical(|ui| {
                ui.label("Preset name:");

                let mut name = ui_state.preset_name.clone();
                let response = ui.add(
                    egui::TextEdit::singleline(&mut name)
                        .desired_width(f32::INFINITY)
                );

                if response.changed() {
                    sender.send(Message::Preset(Preset_::NameChanged(name.clone()))).ok();
                }

                response.request_focus();

                ui.add_space(12.0);

                let mut include_selection = ui_state.preset_include_selection;
                if ui.checkbox(&mut include_selection, "Include selection").clicked() {
                    sender.send(Message::Preset(Preset_::IncludeSelectionChanged(include_selection))).ok();
                }

                let mut include_search = ui_state.preset_include_search;
                if ui.checkbox(&mut include_search, "Include search/filter").clicked() {
                    sender.send(Message::Preset(Preset_::IncludeSearchChanged(include_search))).ok();
                }

                let mut generic = ui_state.preset_generic;
                if ui.checkbox(&mut generic, "Generic (available in any project)").clicked() {
                    sender.send(Message::Preset(Preset_::GenericChanged(generic))).ok();
                }

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    let can_save = !name.trim().is_empty()
                        && (include_selection || include_search);

                    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked()
                        || (enter && can_save)
                    {
                        sender.send(Message::Preset(Preset_::Saved(name.trim().to_string()))).ok();
                    }

                    if ui.button("Cancel").clicked() {
                        sender.send(Message::Preset(Preset_::SaveDialogClosed)).ok();
                    }
                });
            });
        });
}

pub fn render_load(ctx: &egui::Context, model: &Model, sender: &Sender<Message>) {
    let center = ctx.content_rect().center();
    let title_bar_height = 32.0;
    let button_width = 46.0;

    let current_root = model.tree.nodes.first().map(|n| &n.path);
    let presets = model.presets.list_for_root(current_root);

    egui::Window::new("load_preset")
        .title_bar(false)
        .resizable(false)
        .fixed_size([420.0, 300.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .show(ctx, |ui| {
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
                    ui.label(egui::RichText::new("Load Preset").size(14.0));
                }
            );

            let close_rect = egui::Rect::from_min_size(
                title_rect.right_top() - egui::vec2(button_width, 0.0),
                egui::vec2(button_width, title_bar_height),
            );

            let close_response = ui.interact(
                close_rect,
                ui.id().with("load_preset_close"),
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
                sender.send(Message::Preset(Preset_::LoadDialogClosed)).ok();
            }

            ui.separator();

            ui.vertical(|ui| {
                let available_height = ui.available_height();

                egui::Frame::dark_canvas(ui.style())
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(8.0)
                    .stroke(egui::Stroke::NONE)
                    .show(ui, |ui| {
                        ui.set_min_height(available_height - 16.0);

                        if presets.is_empty() {
                            let available = ui.available_size();

                            ui.allocate_ui_with_layout(
                                available,
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    let space_above = (available.y - 20.0) / 2.0;
                                    ui.add_space(space_above.max(0.0));
                                    ui.label("No presets available.");
                                }
                            );
                        } else {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for preset in &presets {
                                        let response = ui.selectable_label(false, &preset.name);

                                        if response.clicked() {
                                            sender.send(Message::Preset(Preset_::Loaded(preset.id.clone()))).ok();
                                        }

                                        response.context_menu(|ui| {
                                            if ui.button("Delete").clicked() {
                                                sender.send(Message::Preset(Preset_::Deleted(preset.id.clone()))).ok();
                                                ui.close();
                                            }
                                        });
                                    }
                                });
                        }
                    });
            });
        });
}

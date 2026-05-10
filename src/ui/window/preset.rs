use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{Msg, Preset_};
use crate::app::state::{Model, UiState};

pub fn render_save(ctx: &egui::Context, ui_state: &UiState, sender: &Sender<Msg>) {
    let center = ctx.content_rect().center();

    let mut open = true;

    egui::Window::new(egui::RichText::new("Save Preset").size(14.0))
        .resizable(false)
        .fixed_size([300.0, 100.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Preset name:");

                let mut name = ui_state.preset_name.clone();
                let response = ui.add(
                    egui::TextEdit::singleline(&mut name)
                        .desired_width(f32::INFINITY)
                );

                if response.changed() {
                    sender.send(Msg::Preset(Preset_::NameChanged(name.clone()))).ok();
                }

                response.request_focus();

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    let can_save = !name.trim().is_empty();

                    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked()
                        || (enter && can_save)
                    {
                        sender.send(Msg::Preset(Preset_::Saved(name.trim().to_string()))).ok();
                    }

                    if ui.button("Cancel").clicked() {
                        sender.send(Msg::Preset(Preset_::SaveDialogClosed)).ok();
                    }
                });
            });
        });

    if !open {
        sender.send(Msg::Preset(Preset_::SaveDialogClosed)).ok();
    }
}

pub fn render_load(ctx: &egui::Context, model: &Model, sender: &Sender<Msg>) {
    let center = ctx.content_rect().center();

    let mut open = true;

    let current_root = model.tree.nodes.first().map(|n| &n.path);

    let presets: Vec<_> = model.presets.list()
        .into_iter()
        .filter(|p| current_root.is_some_and(|root| *root == p.root))
        .collect();

    egui::Window::new(egui::RichText::new("Load Preset").size(14.0))
        .resizable(false)
        .fixed_size([420.0, 300.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                egui::Frame::dark_canvas(ui.style())
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(8.0)
                    .stroke(egui::Stroke::NONE)
                    .show(ui, |ui| {
                        if presets.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(80.0);
                                ui.label("No presets for this project.");
                            });
                            return;
                        }

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .max_height(210.0)
                            .show(ui, |ui| {
                                for preset in &presets {
                                    let id = ui.make_persistent_id(&preset.id);

                                    let response = ui.push_id(id, |ui| {
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(ui.available_width(), 36.0),
                                            egui::Sense::click(),
                                        );

                                        let visuals = if response.hovered() {
                                            ui.visuals().widgets.hovered
                                        } else {
                                            ui.visuals().widgets.inactive
                                        };

                                        if response.hovered() {
                                            ui.painter().rect_filled(
                                                rect,
                                                4.0,
                                                visuals.weak_bg_fill,
                                            );
                                        }

                                        let text_rect = rect.shrink2(egui::vec2(8.0, 0.0));

                                        ui.painter().text(
                                            text_rect.left_center(),
                                            egui::Align2::LEFT_CENTER,
                                            &preset.name,
                                            egui::FontId::proportional(13.0),
                                            if response.hovered() {
                                                visuals.fg_stroke.color
                                            } else {
                                                ui.visuals().text_color()
                                            },
                                        );

                                        let count_text = format!("{} files", preset.paths.len());

                                        ui.painter().text(
                                            text_rect.right_center() - egui::vec2(52.0, 0.0),
                                            egui::Align2::RIGHT_CENTER,
                                            &count_text,
                                            egui::FontId::proportional(11.0),
                                            ui.visuals().weak_text_color(),
                                        );

                                        response
                                    }).inner;

                                    if response.clicked() {
                                        sender.send(Msg::Preset(Preset_::Loaded(preset.id.clone()))).ok();
                                    }

                                    response.context_menu(|ui| {
                                        if ui.button("Delete").clicked() {
                                            sender.send(Msg::Preset(Preset_::Deleted(preset.id.clone()))).ok();
                                            ui.close();
                                        }
                                    });
                                }
                            });
                    });

                ui.add_space(ui.available_height() - 35.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        sender.send(Msg::Preset(Preset_::LoadDialogClosed)).ok();
                    }
                });
            });
        });

    if !open {
        sender.send(Msg::Preset(Preset_::LoadDialogClosed)).ok();
    }
}

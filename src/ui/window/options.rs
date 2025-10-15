use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{Filter, Msg, Options_};
use crate::app::state::{Model, UiState};
use crate::app::state::OptionsTab;
use crate::ui::themes::Theme;

pub fn render(
    ctx: &egui::Context,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Msg>,
) {
    let center = ctx.content_rect().center();

    egui::Window::new(egui::RichText::new("Options").size(14.0))
        .resizable(false)
        .fixed_size([500.0, 350.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                render_tab_bar(ui, ui_state, sender);

                ui.separator();
                ui.add_space(10.0);

                match ui_state.options_tab {
                    OptionsTab::General => render_general(ui, model, sender),
                    OptionsTab::Includes => render_includes(ui, model, ui_state, sender),
                    OptionsTab::Excludes => render_excludes(ui, model, ui_state, sender),
                }

                ui.add_space(ui.available_height() - 35.0);

                render_bottom_buttons(ui, ui_state, sender);

                ui.add_space(3.0);
            });
        });
}

fn render_tab_bar(ui: &mut egui::Ui, ui_state: &UiState, sender: &Sender<Msg>) {
    ui.horizontal(|ui| {
        let mut current = ui_state.options_tab;

        if ui.selectable_value(&mut current, OptionsTab::General, "General").clicked() {
            sender.send(Msg::Options(Options_::TabChanged(OptionsTab::General))).ok();
        }

        if ui.selectable_value(&mut current, OptionsTab::Includes, "Include").clicked() {
            sender.send(Msg::Options(Options_::TabChanged(OptionsTab::Includes))).ok();
        }

        if ui.selectable_value(&mut current, OptionsTab::Excludes, "Exclude").clicked() {
            sender.send(Msg::Options(Options_::TabChanged(OptionsTab::Excludes))).ok();
        }
    });
}

fn render_general(ui: &mut egui::Ui, model: &Model, sender: &Sender<Msg>) {
    egui::Frame::dark_canvas(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(8.0)
        .stroke(egui::Stroke::NONE)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        render_display_section(ui, model, sender);
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        render_output_section(ui, model, sender);
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        render_behavior_section(ui, model, sender);
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        render_appearance_section(ui, model, sender);

                        #[cfg(windows)]
                        {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(10.0);
                            render_integration_section(ui);
                        }
                    });
                });
        });
}

fn render_display_section(ui: &mut egui::Ui, model: &Model, sender: &Sender<Msg>) {
    ui.label(egui::RichText::new("Display").strong().color(ui.visuals().weak_text_color()));
    ui.add_space(5.0);

    let mut use_icon = model.options.use_icon;

    if ui.checkbox(&mut use_icon, "Use icons in tree").clicked() {
        sender.send(Msg::Options(Options_::UseIconChanged(use_icon))).ok();
    }
}

fn render_output_section(ui: &mut egui::Ui, model: &Model, sender: &Sender<Msg>) {
    ui.label(egui::RichText::new("Output").strong().color(ui.visuals().weak_text_color()));
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label("Format:");

        egui::ComboBox::from_id_salt("output_format_selector")
            .selected_text(model.options.output_format.name())
            .width(150.0)
            .show_ui(ui, |ui| {
                for format in crate::model::output::OutputFormat::all() {
                    if ui.selectable_label(model.options.output_format == *format, format.name()).clicked() {
                        sender.send(Msg::Options(Options_::OutputFormatChanged(*format))).ok();
                    }
                }
            });
    });
}

fn render_behavior_section(ui: &mut egui::Ui, model: &Model, sender: &Sender<Msg>) {
    ui.label(egui::RichText::new("Behavior").strong().color(ui.visuals().weak_text_color()));
    ui.add_space(5.0);

    let mut delete_sessions = model.options.delete_sessions_on_exit;

    if ui.checkbox(&mut delete_sessions, "Delete session(s) upon exiting").clicked() {
        sender.send(Msg::Options(Options_::DeleteSessionsChanged(delete_sessions))).ok();
    }

    let mut single_instance = model.options.single_instance;

    if ui.checkbox(&mut single_instance, "Use a single instance (requires restart)").clicked() {
        sender.send(Msg::Options(Options_::SingleInstanceChanged(single_instance))).ok();
    }

    let mut auto_index = model.options.auto_index_on_startup;

    if ui.checkbox(&mut auto_index, "Auto-index on startup").clicked() {
        sender.send(Msg::Options(Options_::AutoIndexChanged(auto_index))).ok();
    }
}

fn render_appearance_section(ui: &mut egui::Ui, model: &Model, sender: &Sender<Msg>) {
    ui.label(egui::RichText::new("Appearance").strong().color(ui.visuals().weak_text_color()));
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label("Theme:");

        egui::ComboBox::from_id_salt("theme_selector")
            .selected_text(model.options.theme.name())
            .width(150.0)
            .show_ui(ui, |ui| {
                for theme in Theme::all() {
                    if ui.selectable_label(model.options.theme == *theme, theme.name()).clicked() {
                        sender.send(Msg::Options(Options_::ThemeChanged(*theme))).ok();
                    }
                }
            });
    });
}

#[cfg(windows)]
fn render_integration_section(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Integration")
            .strong()
            .color(ui.visuals().weak_text_color())
    );

    ui.add_space(5.0);

    let mut is_registered = crate::context::is_registered();

    if ui.checkbox(&mut is_registered, "Add swarm to Windows context menu").clicked() {
        if is_registered {
            if let Err(e) = crate::context::register() {
                eprintln!("Failed to register context menu: {}", e);
            }
        } else {
            if let Err(e) = crate::context::unregister() {
                eprintln!("Failed to unregister context menu: {}", e);
            }
        }
    }
}

fn render_includes(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &Sender<Msg>) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_filter_input(ui, &ui_state.new_include_filter, "Enter pattern (e.g., *.rs, *.txt)", |filter| {
                    Msg::Filter(Filter::IncludeAdded(filter))
                }, sender);

                ui.add_space(10.0);

                render_filter_list(ui, &model.options.include, sender, true);
            });
        });
}

fn render_excludes(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &Sender<Msg>) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_filter_input(ui, &ui_state.new_exclude_filter, "Enter pattern (e.g., *.log, node_modules)", |filter| {
                    Msg::Filter(Filter::ExcludeAdded(filter))
                }, sender);

                ui.add_space(10.0);

                render_filter_list(ui, &model.options.exclude, sender, false);
            });
        });
}

fn render_filter_input<F>(
    ui: &mut egui::Ui,
    current_value: &str,
    hint: &str,
    create_msg: F,
    sender: &Sender<Msg>,
) where
    F: Fn(String) -> Msg,
{
    ui.horizontal(|ui| {
        let mut filter = current_value.to_string();
        let response = ui.add(
            egui::TextEdit::singleline(&mut filter)
                .hint_text(hint)
                .desired_width(ui.available_width() - 50.0)
        );

        let add_enabled = !filter.trim().is_empty();
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));

        if ui.add_enabled(add_enabled, egui::Button::new("Add")).clicked()
            || (enter && response.has_focus() && add_enabled)
        {
            sender.send(create_msg(filter)).ok();
        }
    });
}

fn render_filter_list(
    ui: &mut egui::Ui,
    filters: &[String],
    sender: &Sender<Msg>,
    is_include: bool,
) {
    egui::Frame::dark_canvas(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(8.0)
        .stroke(egui::Stroke::NONE)
        .show(ui, |ui| {
            if filters.is_empty() && is_include {
                ui.set_height(180.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);

                    ui.label(
                        egui::RichText::new("No include filters")
                            .color(ui.visuals().weak_text_color())
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        egui::Frame::NONE
                            .inner_margin(egui::Margin {
                                left: 0,
                                right: 8,
                                top: 0,
                                bottom: 0,
                            })
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 4.0;

                                    for (i, filter) in filters.iter().enumerate() {
                                        render_filter_tag(ui, filter, i, sender, is_include);
                                    }
                                });
                            });
                    });
            }
        });
}

fn render_filter_tag(
    ui: &mut egui::Ui,
    filter: &str,
    index: usize,
    sender: &Sender<Msg>,
    is_include: bool,
) {
    let available_width = ui.available_width();

    ui.horizontal(|ui| {
        let frame = egui::Frame::NONE
            .fill(ui.visuals().widgets.inactive.bg_fill)
            .stroke(ui.visuals().widgets.inactive.bg_stroke)
            .corner_radius(3.0)
            .inner_margin(egui::Margin::symmetric(6, 2));

        frame.show(ui, |ui| {
            ui.set_min_width(available_width - 24.0);
            ui.set_min_height(20.0);

            ui.allocate_ui_with_layout(
                egui::vec2(available_width - 24.0, 20.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.label(egui::RichText::new(filter).size(12.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);

                        let delete = ui.add_sized(
                            [14.0, 14.0],
                            egui::Button::new(egui::RichText::new("×").size(16.0))
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                        );

                        if delete.clicked() {
                            let msg = if is_include {
                                Msg::Filter(Filter::IncludeRemoved(index))
                            } else {
                                Msg::Filter(Filter::ExcludeRemoved(index))
                            };

                            sender.send(msg).ok();
                        }
                    });
                },
            );
        });
    });
}

fn render_bottom_buttons(ui: &mut egui::Ui, ui_state: &UiState, sender: &Sender<Msg>) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                match ui_state.options_tab {
                    OptionsTab::Includes => {
                        if ui.button("Reset to Default").clicked() {
                            sender.send(Msg::Filter(Filter::IncludesCleared)).ok();
                        }
                    }
                    OptionsTab::Excludes => {
                        if ui.button("Reset to Default").clicked() {
                            sender.send(Msg::Filter(Filter::ExcludesReset)).ok();
                        }
                    }
                    _ => {}
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        sender.send(Msg::Options(Options_::Closed)).ok();
                    }
                });
            });
        });
}

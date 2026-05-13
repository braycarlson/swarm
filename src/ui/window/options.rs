use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{Filter, Message, Options_};
use crate::app::state::{Model, UiState};
use crate::app::state::OptionsTab;
use crate::ui::themes::Theme;
use crate::ui::widget::titlebar::icon::{TitleIcon, draw_title_icon, hover_rectangle};

pub fn render(
    context: &egui::Context,
    model: &Model,
    ui_state: &UiState,
    sender: &Sender<Message>,
) {
    let center = context.content_rect().center();
    let title_bar_height = 32.0;
    let button_width = 46.0;

    egui::Window::new("options")
        .title_bar(false)
        .resizable(false)
        .fixed_size([500.0, 425.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .show(context, |ui| {
            let content_rect = ui.max_rect();

            let title_rectangle = egui::Rect::from_min_size(
                content_rect.min,
                egui::vec2(content_rect.width(), title_bar_height),
            );

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), title_bar_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_min_height(title_bar_height);
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Options").size(14.0));
                }
            );

            let close_rectangle = egui::Rect::from_min_size(
                title_rectangle.right_top() - egui::vec2(button_width, 0.0),
                egui::vec2(button_width, title_bar_height),
            );

            let close_response = ui.interact(
                close_rectangle,
                ui.id().with("options_close"),
                egui::Sense::click(),
            );

            if close_response.hovered() {
                let painter = ui.painter().with_clip_rect(title_rectangle);
                painter.rect_filled(
                    hover_rectangle(close_rectangle, title_rectangle),
                    0.0,
                    egui::Color32::from_rgb(232, 17, 35),
                );
            }

            {
                let foreground = if close_response.hovered() {
                    egui::Color32::WHITE
                } else {
                    ui.visuals().text_color()
                };

                let painter = ui.painter().with_clip_rect(title_rectangle);
                draw_title_icon(&painter, close_rectangle, TitleIcon::Close, foreground);
            }

            if close_response.clicked() {
                sender.send(Message::Options(Options_::Closed)).ok();
            }

            ui.separator();

            ui.vertical(|ui| {
                render_tab_bar(ui, ui_state, sender);

                ui.separator();
                ui.add_space(10.0);

                match ui_state.options_tab {
                    OptionsTab::General => render_general(ui, model, ui_state, sender),
                    OptionsTab::Includes => render_includes(ui, model, ui_state, sender),
                    OptionsTab::Excludes => render_excludes(ui, model, ui_state, sender),
                }

                ui.add_space(ui.available_height() - 35.0);

                render_bottom_buttons(ui, ui_state, sender);
            });
        });
}

fn render_tab_bar(ui: &mut egui::Ui, ui_state: &UiState, sender: &Sender<Message>) {
    ui.horizontal(|ui| {
        let mut current = ui_state.options_tab;

        if ui.selectable_value(&mut current, OptionsTab::General, "General").clicked() {
            sender.send(Message::Options(Options_::TabChanged(OptionsTab::General))).ok();
        }

        if ui.selectable_value(&mut current, OptionsTab::Includes, "Include").clicked() {
            sender.send(Message::Options(Options_::TabChanged(OptionsTab::Includes))).ok();
        }

        if ui.selectable_value(&mut current, OptionsTab::Excludes, "Exclude").clicked() {
            sender.send(Message::Options(Options_::TabChanged(OptionsTab::Excludes))).ok();
        }
    });
}

fn render_general(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &Sender<Message>) {
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

                        render_appearance_section(ui, model, ui_state, sender);

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

fn render_display_section(ui: &mut egui::Ui, model: &Model, sender: &Sender<Message>) {
    ui.label(egui::RichText::new("Display").strong().color(ui.visuals().weak_text_color()));
    ui.add_space(5.0);

    let mut use_icon = model.options.use_icon;

    if ui.checkbox(&mut use_icon, "Use icons in tree").clicked() {
        sender.send(Message::Options(Options_::UseIconChanged(use_icon))).ok();
    }

    let mut show_hidden = model.options.show_hidden;

    if ui.checkbox(&mut show_hidden, "Show hidden files").clicked() {
        sender.send(Message::Options(Options_::ShowHiddenChanged(show_hidden))).ok();
    }
}

fn render_output_section(ui: &mut egui::Ui, model: &Model, sender: &Sender<Message>) {
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
                        sender.send(Message::Options(Options_::OutputFormatChanged(*format))).ok();
                    }
                }
            });
    });
}

fn render_behavior_section(ui: &mut egui::Ui, model: &Model, sender: &Sender<Message>) {
    ui.label(egui::RichText::new("Behavior").strong().color(ui.visuals().weak_text_color()));
    ui.add_space(5.0);

    let mut delete_sessions = model.options.delete_sessions_on_exit;

    if ui.checkbox(&mut delete_sessions, "Delete session(s) upon exiting").clicked() {
        sender.send(Message::Options(Options_::DeleteSessionsChanged(delete_sessions))).ok();
    }

    let mut single_instance = model.options.single_instance;

    if ui.checkbox(&mut single_instance, "Use a single instance (requires restart)").clicked() {
        sender.send(Message::Options(Options_::SingleInstanceChanged(single_instance))).ok();
    }
}

fn render_appearance_section(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &Sender<Message>) {
    ui.label(egui::RichText::new("Appearance").strong().color(ui.visuals().weak_text_color()));
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label("Theme:");

        egui::ComboBox::from_id_salt("theme_selector")
            .selected_text(model.options.theme.name())
            .width(200.0)
            .show_ui(ui, |ui| {
                for theme in Theme::all() {
                    if ui.selectable_label(model.options.theme == *theme, theme.name()).clicked() {
                        sender.send(Message::Options(Options_::ThemeChanged(*theme))).ok();
                    }
                }
            });
    });

    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.label("UI Scale:");

        let display_scale = ui_state.ui_scale_draft
            .unwrap_or_else(|| model.options.effective_ui_scale());

        let mut scale = display_scale;

        let slider = ui.add(
            egui::Slider::new(&mut scale, 0.5..=3.0)
                .step_by(0.1)
                .fixed_decimals(1)
        );

        if slider.changed() {
            sender.send(Message::Options(Options_::UiScaleChanged(scale))).ok();
        }

        if ui_state.ui_scale_draft.is_some() {
            if ui.button("Apply").clicked() {
                sender.send(Message::Options(Options_::UiScaleApplied)).ok();
            }
        }

        let has_override = ui_state.ui_scale_draft.is_some() || model.options.ui_scale.is_some();

        if has_override {
            if ui.button("Reset").clicked() {
                sender.send(Message::Options(Options_::UiScaleReset)).ok();
            }
        }
    });
}

#[cfg(windows)]
fn render_integration_section(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Integration").strong().color(ui.visuals().weak_text_color()));
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        if ui.button("Register Context Menu").clicked() {
            if let Err(error) = crate::context::register() {
                eprintln!("Failed to register context menu: {}", error);
            }
        }

        if ui.button("Unregister Context Menu").clicked() {
            if let Err(error) = crate::context::unregister() {
                eprintln!("Failed to unregister context menu: {}", error);
            }
        }
    });
}

fn render_includes(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &Sender<Message>) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_filter_input(
                    ui,
                    &ui_state.filter_include_new,
                    "Enter pattern (e.g., *.rs, *.txt)",
                    |filter| Message::Filter(Filter::IncludeAdded(filter)),
                    |filter| Message::Filter(Filter::IncludeFilterChanged(filter)),
                    sender
                );

                ui.add_space(10.0);

                render_filter_list(ui, &model.options.include, sender, true);
            });
        });
}

fn render_excludes(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &Sender<Message>) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_filter_input(
                    ui,
                    &ui_state.filter_exclude_new,
                    "Enter pattern (e.g., *.log, node_modules)",
                    |filter| Message::Filter(Filter::ExcludeAdded(filter)),
                    |filter| Message::Filter(Filter::ExcludeFilterChanged(filter)),
                    sender
                );

                ui.add_space(10.0);

                render_filter_list(ui, &model.options.exclude, sender, false);
            });
        });
}

fn render_filter_input<F, G>(
    ui: &mut egui::Ui,
    current_value: &str,
    hint: &str,
    create_add_message: F,
    create_change_message: G,
    sender: &Sender<Message>,
) where
    F: Fn(String) -> Message,
    G: Fn(String) -> Message,
{
    ui.horizontal(|ui| {
        let mut filter = current_value.to_string();

        let response = ui.add(
            egui::TextEdit::singleline(&mut filter)
                .hint_text(hint)
                .desired_width(ui.available_width() - 50.0)
        );

        if response.changed() {
            sender.send(create_change_message(filter.clone())).ok();
        }

        let add_enabled = !filter.trim().is_empty();
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));

        if ui.add_enabled(add_enabled, egui::Button::new("Add")).clicked()
            || (enter && response.has_focus() && add_enabled)
        {
            sender.send(create_add_message(filter)).ok();
        }
    });
}

fn render_filter_list(
    ui: &mut egui::Ui,
    filters: &[String],
    sender: &Sender<Message>,
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
    sender: &Sender<Message>,
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
                            let message = if is_include {
                                Message::Filter(Filter::IncludeRemoved(index))
                            } else {
                                Message::Filter(Filter::ExcludeRemoved(index))
                            };

                            sender.send(message).ok();
                        }
                    });
                },
            );
        });
    });
}

fn render_bottom_buttons(ui: &mut egui::Ui, ui_state: &UiState, sender: &Sender<Message>) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(ui.spacing().interact_size.y);

                match ui_state.options_tab {
                    OptionsTab::Includes => {
                        if ui.button("Reset to Default").clicked() {
                            sender.send(Message::Filter(Filter::IncludesCleared)).ok();
                        }
                    }
                    OptionsTab::Excludes => {
                        if ui.button("Reset to Default").clicked() {
                            sender.send(Message::Filter(Filter::ExcludesReset)).ok();
                        }
                    }
                    _ => {}
                }
            });
        });
}

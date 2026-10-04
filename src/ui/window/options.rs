use eframe::egui;

use crate::app::message::{Filter, Message, MessageSender, OptionsMessage};
use crate::app::state::{Model, OptionsTab, UiState};
use crate::constants::{UI_SCALE_MAX, UI_SCALE_MIN};
use crate::model::options::PatternList;
use crate::model::output::OutputFormat;
use crate::model::theme::Theme;
use crate::platform;
use crate::ui::scale;
use crate::ui::widget::window::{Modal, render_modal};

const BAR_MARGIN_PIXELS: i8 = 8;
const EMPTY_TEXT_HEIGHT_PIXELS: f32 = 24.0;
const FILTER_BUTTON_WIDTH_PIXELS: f32 = 28.0;
const FILTER_ROW_HEIGHT_PIXELS: f32 = 26.0;
const GENERAL_MARGIN_PIXELS: f32 = 8.0;
const LIST_MARGIN_PIXELS: i8 = 4;
const OPTIONS_SIZE: [f32; 2] = [500.0, 425.0];

const PATTERN_TAB_EXCLUDE: PatternTab<'static> = PatternTab {
    empty_text: "No exclude filters",
    hint: "Enter pattern (e.g., *.log, node_modules)",
    list: PatternList::Exclude,
};

const PATTERN_TAB_INCLUDE: PatternTab<'static> = PatternTab {
    empty_text: "No include filters",
    hint: "Enter pattern (e.g., *.rs, *.txt)",
    list: PatternList::Include,
};

struct PatternTab<'tab> {
    empty_text: &'tab str,
    hint: &'tab str,
    list: PatternList,
}

pub fn render(
    context: &egui::Context,
    model: &Model,
    ui_state: &mut UiState,
    sender: &MessageSender,
) {
    let dialog = Modal {
        identifier: "options",
        size: OPTIONS_SIZE,
        title: "Options",
    };

    let closed = render_modal(context, &dialog, |modal| {
        modal.vertical(|column| {
            render_tab_bar(column, ui_state, sender);
            column.separator();
            column.add_space(10.0);

            match ui_state.options_tab {
                OptionsTab::Excludes => {
                    render_pattern_tab(column, model, ui_state, &PATTERN_TAB_EXCLUDE, sender);
                }
                OptionsTab::General => render_general(column, model, ui_state, sender),
                OptionsTab::Includes => {
                    render_pattern_tab(column, model, ui_state, &PATTERN_TAB_INCLUDE, sender);
                }
            }
        });
    });

    if closed {
        sender.send(Message::Options(OptionsMessage::Closed));
    }
}

fn render_appearance_section(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &UiState,
    sender: &MessageSender,
) {
    section_label(ui, "Appearance");

    ui.horizontal(|row| {
        row.label("Theme:");

        egui::ComboBox::from_id_salt("theme_selector")
            .selected_text(model.options.theme.name())
            .width(200.0)
            .show_ui(row, |list| {
                for theme in Theme::all() {
                    if list
                        .selectable_label(model.options.theme == *theme, theme.name())
                        .clicked()
                    {
                        sender.send(Message::Options(OptionsMessage::ThemeChanged(*theme)));
                    }
                }
            });
    });

    ui.add_space(5.0);
    render_scale_row(ui, model, ui_state, sender);
}

fn render_behavior_section(ui: &mut egui::Ui, model: &Model, sender: &MessageSender) {
    section_label(ui, "Behavior");

    let mut delete_sessions = model.options.delete_sessions_on_exit;

    if ui
        .checkbox(&mut delete_sessions, "Delete session(s) upon exiting")
        .clicked()
    {
        sender.send(Message::Options(OptionsMessage::DeleteSessionsChanged(
            delete_sessions,
        )));
    }

    let mut single_instance = model.options.single_instance;

    if ui
        .checkbox(
            &mut single_instance,
            "Use a single instance (requires restart)",
        )
        .clicked()
    {
        sender.send(Message::Options(OptionsMessage::SingleInstanceChanged(
            single_instance,
        )));
    }
}

fn render_bottom_bar(ui: &mut egui::Ui, list: PatternList, sender: &MessageSender) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(BAR_MARGIN_PIXELS))
        .show(ui, |content| {
            content.horizontal(|row| {
                row.set_min_height(row.spacing().interact_size.y);
                render_reset_button(row, list, sender);
            });
        });
}

fn render_display_section(ui: &mut egui::Ui, model: &Model, sender: &MessageSender) {
    section_label(ui, "Display");

    let mut use_icon = model.options.use_icon;

    if ui.checkbox(&mut use_icon, "Use icons in tree").clicked() {
        sender.send(Message::Options(OptionsMessage::UseIconChanged(use_icon)));
    }

    let mut show_hidden = model.options.show_hidden;

    if ui.checkbox(&mut show_hidden, "Show hidden files").clicked() {
        sender.send(Message::Options(OptionsMessage::ShowHiddenChanged(
            show_hidden,
        )));
    }
}

fn render_divider(ui: &mut egui::Ui) {
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);
}

fn render_general(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &MessageSender) {
    let height = GENERAL_MARGIN_PIXELS
        .mul_add(-2.0, ui.available_height())
        .floor()
        .max(0.0);

    egui::Frame::dark_canvas(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(GENERAL_MARGIN_PIXELS)
        .stroke(egui::Stroke::NONE)
        .show(ui, |content| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(height)
                .show(content, |scroll| {
                    scroll.vertical(|column| {
                        render_display_section(column, model, sender);
                        render_divider(column);
                        render_output_section(column, model, sender);
                        render_divider(column);
                        render_behavior_section(column, model, sender);
                        render_divider(column);
                        render_appearance_section(column, model, ui_state, sender);

                        if platform::CONTEXT_MENU_SUPPORTED {
                            render_divider(column);
                            render_integration_section(column, sender);
                        }
                    });
                });
        });
}

fn render_integration_section(ui: &mut egui::Ui, sender: &MessageSender) {
    section_label(ui, "Integration");

    ui.horizontal(|row| {
        if row.button("Register Context Menu").clicked() {
            sender.send(Message::Options(
                OptionsMessage::ContextMenuRegisterRequested,
            ));
        }

        if row.button("Unregister Context Menu").clicked() {
            sender.send(Message::Options(
                OptionsMessage::ContextMenuUnregisterRequested,
            ));
        }
    });
}

fn render_output_section(ui: &mut egui::Ui, model: &Model, sender: &MessageSender) {
    section_label(ui, "Output");

    ui.horizontal(|row| {
        row.label("Format:");

        egui::ComboBox::from_id_salt("output_format_selector")
            .selected_text(model.options.output_format.name())
            .width(150.0)
            .show_ui(row, |list| {
                for format in OutputFormat::all() {
                    if list
                        .selectable_label(model.options.output_format == *format, format.name())
                        .clicked()
                    {
                        sender.send(Message::Options(OptionsMessage::OutputFormatChanged(
                            *format,
                        )));
                    }
                }
            });
    });
}

fn render_pattern_input(
    ui: &mut egui::Ui,
    value: &mut String,
    tab: &PatternTab<'_>,
    sender: &MessageSender,
) {
    let background = ui.visuals().extreme_bg_color;
    let stroke = ui.visuals().widgets.inactive.bg_stroke;

    egui::Frame::NONE
        .fill(background)
        .stroke(stroke)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |content| {
            content.horizontal(|row| {
                row.spacing_mut().item_spacing.x = 4.0;
                row.set_min_height(FILTER_BUTTON_WIDTH_PIXELS);

                let edit_width = (row.available_width() - FILTER_BUTTON_WIDTH_PIXELS).max(40.0);

                let response = row.add(
                    egui::TextEdit::singleline(value)
                        .hint_text(tab.hint)
                        .frame(egui::Frame::NONE)
                        .desired_width(edit_width),
                );

                let enabled = !value.trim().is_empty();
                let enter = row.input(|input| input.key_pressed(egui::Key::Enter));

                let add = row.add_enabled(
                    enabled,
                    egui::Button::new(egui::RichText::new("+").size(18.0))
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                        .min_size(egui::vec2(FILTER_BUTTON_WIDTH_PIXELS - 4.0, 24.0)),
                );

                let submitted = add.clicked() || (enter && response.has_focus() && enabled);

                if submitted {
                    sender.send(Message::Filter(Filter::Added {
                        list: tab.list,
                        pattern: value.clone(),
                    }));
                }
            });
        });
}

fn render_pattern_list(
    ui: &mut egui::Ui,
    patterns: &[String],
    tab: &PatternTab<'_>,
    height: f32,
    sender: &MessageSender,
) {
    egui::Frame::NONE
        .fill(ui.visuals().extreme_bg_color)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::same(LIST_MARGIN_PIXELS))
        .show(ui, |content| {
            if patterns.is_empty() {
                content.set_min_height(height);

                content.vertical_centered(|centered| {
                    centered.add_space(((height - EMPTY_TEXT_HEIGHT_PIXELS) / 2.0).max(0.0));

                    centered.label(
                        egui::RichText::new(tab.empty_text)
                            .color(centered.visuals().weak_text_color()),
                    );
                });

                return;
            }

            egui::ScrollArea::vertical()
                .max_height(height)
                .auto_shrink([false, false])
                .show(content, |scroll| {
                    scroll.spacing_mut().item_spacing.y = 2.0;

                    for (index, pattern) in patterns.iter().enumerate() {
                        render_pattern_row(scroll, pattern, index, tab.list, sender);
                    }
                });
        });
}

fn render_pattern_row(
    ui: &mut egui::Ui,
    pattern: &str,
    index: usize,
    list: PatternList,
    sender: &MessageSender,
) {
    let width = ui.available_width();
    let (rectangle, _) = ui.allocate_exact_size(
        egui::vec2(width, FILTER_ROW_HEIGHT_PIXELS),
        egui::Sense::hover(),
    );

    if ui.rect_contains_pointer(rectangle) {
        ui.painter()
            .rect_filled(rectangle, 3.0, ui.visuals().widgets.hovered.weak_bg_fill);
    }

    let content = egui::Rect::from_min_max(
        rectangle.min + egui::vec2(6.0, 2.0),
        rectangle.max - egui::vec2(12.0, 2.0),
    );

    let mut row = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(content)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    row.spacing_mut().item_spacing.x = 4.0;
    row.label(egui::RichText::new(pattern).size(13.0));

    row.with_layout(
        egui::Layout::right_to_left(egui::Align::Center),
        |aligned| {
            let remove = aligned
                .add_sized(
                    [22.0, 22.0],
                    egui::Button::new(egui::RichText::new("\u{d7}").size(16.0))
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                        .corner_radius(3.0),
                )
                .on_hover_text("Remove");

            if remove.clicked() {
                sender.send(Message::Filter(Filter::Removed { index, list }));
            }
        },
    );
}

fn render_pattern_tab(
    ui: &mut egui::Ui,
    model: &Model,
    ui_state: &mut UiState,
    tab: &PatternTab<'_>,
    sender: &MessageSender,
) {
    let value = match tab.list {
        PatternList::Exclude => &mut ui_state.pattern_exclude_new,
        PatternList::Include => &mut ui_state.pattern_include_new,
    };

    let bar_height = f32::from(BAR_MARGIN_PIXELS).mul_add(2.0, ui.spacing().interact_size.y);

    let reserved =
        f32::from(LIST_MARGIN_PIXELS).mul_add(2.0, bar_height) + ui.spacing().item_spacing.y;

    egui::Frame::NONE
        .inner_margin(egui::Margin::same(8))
        .show(ui, |content| {
            content.vertical(|column| {
                render_pattern_input(column, value, tab, sender);
                column.add_space(10.0);

                let height = (column.available_height() - reserved).floor().max(0.0);
                let patterns = model.options.patterns(tab.list);

                render_pattern_list(column, patterns, tab, height, sender);
            });
        });

    ui.add_space((ui.available_height() - bar_height).max(0.0));
    render_bottom_bar(ui, tab.list, sender);
}

fn render_reset_button(ui: &mut egui::Ui, list: PatternList, sender: &MessageSender) {
    if ui.button("Reset to Default").clicked() {
        sender.send(Message::Filter(Filter::Reset(list)));
    }
}

fn render_scale_row(ui: &mut egui::Ui, model: &Model, ui_state: &UiState, sender: &MessageSender) {
    ui.horizontal(|row| {
        row.label("UI Scale:");

        let mut scale = ui_state
            .ui_scale_draft
            .or(model.options.ui_scale)
            .unwrap_or_else(|| scale::effective(&model.options, row));

        let slider = row.add(
            egui::Slider::new(&mut scale, UI_SCALE_MIN..=UI_SCALE_MAX)
                .step_by(0.1)
                .fixed_decimals(1),
        );

        if slider.changed() {
            debug_assert!(scale >= UI_SCALE_MIN);
            debug_assert!(scale <= UI_SCALE_MAX);

            sender.send(Message::Options(OptionsMessage::UiScaleChanged(scale)));
        }

        if ui_state.ui_scale_draft.is_some() {
            if row.button("Apply").clicked() {
                sender.send(Message::Options(OptionsMessage::UiScaleApplied));
            }
        }

        let has_override = ui_state.ui_scale_draft.is_some() || model.options.ui_scale.is_some();

        if has_override {
            if row.button("Reset").clicked() {
                sender.send(Message::Options(OptionsMessage::UiScaleReset));
            }
        }
    });
}

fn render_tab_bar(ui: &mut egui::Ui, ui_state: &UiState, sender: &MessageSender) {
    ui.horizontal(|row| {
        for (tab, label) in [
            (OptionsTab::General, "General"),
            (OptionsTab::Includes, "Include"),
            (OptionsTab::Excludes, "Exclude"),
        ] {
            if row
                .selectable_label(ui_state.options_tab == tab, label)
                .clicked()
            {
                sender.send(Message::Options(OptionsMessage::TabChanged(tab)));
            }
        }
    });
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .strong()
            .color(ui.visuals().weak_text_color()),
    );

    ui.add_space(5.0);
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;
    use crate::model::options::Options;

    const PASS_COUNT: u32 = 3;
    const SHAPE_COUNT_MAX: u32 = 100_000;

    struct Rendered {
        size: egui::Vec2,
        texts: Vec<(String, f32)>,
    }

    impl Rendered {
        fn position_of(&self, label: &str) -> Option<f32> {
            let position = self
                .texts
                .iter()
                .find(|(text, _)| text == label)
                .map(|(_, top)| *top);

            assert!(position.is_none_or(f32::is_finite));

            position
        }
    }

    fn rendered(tab: OptionsTab) -> Rendered {
        let (channel, _receiver) = mpsc::sync_channel(64);
        let sender = MessageSender::new(channel);
        let options = Options::default();
        let model = Model::new(options, Vec::new());
        let mut ui_state = UiState::new(Theme::default());
        let context = egui::Context::default();
        let input = egui::RawInput::default();
        let mut shapes = Vec::new();

        ui_state.options_tab = tab;

        for _ in 0..PASS_COUNT {
            shapes = context
                .run_ui(input.clone(), |ui| {
                    render(ui.ctx(), &model, &mut ui_state, &sender);
                })
                .shapes;
        }

        let mut texts = Vec::new();
        let mut pending: Vec<&egui::Shape> = shapes.iter().map(|clipped| &clipped.shape).collect();

        for _ in 0..SHAPE_COUNT_MAX {
            let Some(shape) = pending.pop() else {
                break;
            };

            match shape {
                egui::Shape::Text(text) => texts.push((text.galley.text().to_owned(), text.pos.y)),
                egui::Shape::Vec(nested) => pending.extend(nested.iter()),
                _ => {}
            }
        }

        assert_eq!(pending.len(), 0);

        let size = context
            .memory(|memory| memory.area_rect(egui::Id::new("options")))
            .expect("the options window is shown")
            .size();

        Rendered { size, texts }
    }

    #[test]
    fn every_tab_keeps_the_same_window_size() {
        let general = rendered(OptionsTab::General).size;

        assert_eq!(rendered(OptionsTab::Excludes).size, general);
        assert_eq!(rendered(OptionsTab::Includes).size, general);
    }

    #[test]
    fn the_general_tab_shows_every_section_through_appearance() {
        let general = rendered(OptionsTab::General);

        for label in ["Display", "Output", "Behavior", "Appearance", "UI Scale:"] {
            assert!(general.position_of(label).is_some(), "{label} is visible");
        }
    }

    #[test]
    fn the_pattern_list_reaches_the_reset_bar() {
        let excludes = rendered(OptionsTab::Excludes);
        let defaults = Options::default().exclude;

        let reset = excludes
            .position_of("Reset to Default")
            .expect("the reset button is visible");

        let last = excludes
            .texts
            .iter()
            .filter(|(text, _)| defaults.contains(text))
            .map(|(_, top)| *top)
            .fold(f32::NEG_INFINITY, f32::max);

        assert!(last.is_finite());
        assert!(reset - last < 2.0 * (FILTER_ROW_HEIGHT_PIXELS + 2.0));
    }
}

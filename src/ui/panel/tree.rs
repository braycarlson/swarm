use std::path::Path;

use eframe::egui;
use egui::collapsing_header::{CollapsingState, paint_default_icon};

use crate::app::message::{Message, MessageSender, Notice, ToggleScope, Tree};
use crate::app::state::{Model, SearchModel};
use crate::model::node::{FileNode, TREE_DEPTH_MAX};
use crate::model::selection;
use crate::platform;

const GUIDE_INSET_BOTTOM_PIXELS: f32 = 2.0;
const ROW_NONE: u32 = u32::MAX;
const STACK_CAPACITY: usize = 32;
type NodePath = [u32; TREE_DEPTH_MAX as usize];

enum Descent<'tree> {
    Children(&'tree [FileNode]),
    Failed,
    Leaf,
    Loading,
}

struct Frame<'tree> {
    children: &'tree [FileNode],
    depth: u16,
    next: usize,
    parent_row: u32,
}

#[derive(Clone, Copy, Debug)]
struct Geometry {
    checkbox_width: f32,
    indent: f32,
    spacing: f32,
}

impl Geometry {
    fn guide_x(self, left: f32, level: u16) -> f32 {
        let offset = self.row_offset(level) + self.checkbox_width + self.spacing;
        let x = left + offset + self.indent / 2.0;

        debug_assert!(x > left);

        x
    }

    fn measure(ui: &mut egui::Ui) -> Self {
        let mut probe = ui.new_child(egui::UiBuilder::new().invisible());
        let mut checked = false;
        let checkbox_width = probe.checkbox(&mut checked, "").rect.width();

        debug_assert!(checkbox_width > 0.0);

        Self {
            checkbox_width,
            indent: ui.spacing().indent,
            spacing: ui.spacing().item_spacing.x,
        }
    }

    fn row_offset(self, depth: u16) -> f32 {
        let offset = f32::from(depth) * (self.checkbox_width + self.spacing + self.indent);

        debug_assert!(offset >= 0.0);

        offset
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpenAction {
    Open,
    Reveal,
}

struct RowContext<'row> {
    depth: u16,
    path: &'row [u32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RowKind {
    Directory,
    Failed,
    File,
    Loading,
    LoadingText,
}

#[derive(Clone, Copy, Debug)]
pub struct TreeRow {
    depth: u16,
    index: u32,
    kind: RowKind,
    parent: u32,
}

struct TreeView<'view> {
    geometry: Geometry,
    model: &'view Model,
    rows: &'view [TreeRow],
    sender: &'view MessageSender,
}

fn build_rows(model: &Model, context: &egui::Context, rows: &mut Vec<TreeRow>) -> Vec<usize> {
    let mut expansions = Vec::new();
    let mut stack: Vec<Frame<'_>> = Vec::with_capacity(STACK_CAPACITY);

    rows.clear();

    stack.push(Frame {
        children: &model.tree.nodes,
        depth: 0,
        next: 0,
        parent_row: ROW_NONE,
    });

    while !stack.is_empty() {
        assert!(stack.len() <= TREE_DEPTH_MAX as usize);

        let top = stack.len() - 1;
        let frame = &mut stack[top];

        let Some(node) = frame.children.get(frame.next) else {
            let _ = stack.pop();

            continue;
        };

        let index = u32::try_from(frame.next).expect("a directory listing fits in u32");
        let depth = frame.depth;
        let parent = frame.parent_row;

        frame.next += 1;

        if !should_show_node(node, &model.search) {
            continue;
        }

        let row_index = u32::try_from(rows.len()).expect("the visible rows fit in u32");

        rows.push(TreeRow {
            depth,
            index,
            kind: row_kind(node),
            parent,
        });

        match descent(context, node, depth) {
            Descent::Children(children) => stack.push(Frame {
                children,
                depth: depth + 1,
                next: 0,
                parent_row: row_index,
            }),
            Descent::Failed => rows.push(placeholder(RowKind::Failed, depth, row_index)),
            Descent::Leaf => {}
            Descent::Loading => {
                rows.push(placeholder(RowKind::Loading, depth, row_index));
                rows.push(placeholder(RowKind::LoadingText, depth, row_index));
                expansions.push(row_index as usize);
            }
        }
    }

    expansions
}

fn descent<'tree>(context: &egui::Context, node: &'tree FileNode, depth: u16) -> Descent<'tree> {
    if !node.is_directory() {
        return Descent::Leaf;
    }

    if !directory_state(context, node, depth).is_open() {
        return Descent::Leaf;
    }

    if node.load_failed {
        return Descent::Failed;
    }

    if !node.loaded {
        return Descent::Loading;
    }

    if node.children.is_empty() {
        return Descent::Leaf;
    }

    Descent::Children(&node.children)
}

fn directory_context_menu(
    ui: &mut egui::Ui,
    node: &FileNode,
    path: &[u32],
    sender: &MessageSender,
) {
    if ui.button("Select All").clicked() {
        send_toggle(sender, path, true, ToggleScope::Subtree);
        ui.close();
    }

    if ui.button("Deselect All").clicked() {
        send_toggle(sender, path, false, ToggleScope::Subtree);
        ui.close();
    }

    ui.separator();

    if ui.button("Open Folder").clicked() {
        open_path(&node.path, sender, OpenAction::Open);
        ui.close();
    }

    if ui.button("Reveal in Explorer").clicked() {
        open_path(&node.path, sender, OpenAction::Reveal);
        ui.close();
    }
}

fn directory_id(path: &Path) -> egui::Id {
    egui::Id::new(("tree-directory", path))
}

fn directory_state(context: &egui::Context, node: &FileNode, depth: u16) -> CollapsingState {
    CollapsingState::load_with_default_open(context, directory_id(&node.path), depth == 0)
}

fn file_context_menu(ui: &mut egui::Ui, node: &FileNode, path: &[u32], sender: &MessageSender) {
    if ui.button("Open File").clicked() {
        open_path(&node.path, sender, OpenAction::Open);
        ui.close();
    }

    if ui.button("Reveal in Explorer").clicked() {
        open_path(&node.path, sender, OpenAction::Reveal);
        ui.close();
    }

    ui.separator();

    let toggle_label = if node.checked { "Deselect" } else { "Select" };

    if ui.button(toggle_label).clicked() {
        send_toggle(sender, path, !node.checked, ToggleScope::Node);
        ui.close();
    }
}

fn fill_row(ui: &mut egui::Ui, sender: &MessageSender) {
    let remaining = ui.available_size();

    if remaining.x <= 0.0 {
        return;
    }

    let (_, fill) = ui.allocate_exact_size(remaining, egui::Sense::click());

    fill.context_menu(|menu| show_tree_context_menu(menu, sender));
}

fn node_path<'buffer>(
    rows: &[TreeRow],
    row_index: usize,
    buffer: &'buffer mut NodePath,
) -> &'buffer [u32] {
    let mut cursor = u32::try_from(row_index).expect("the visible rows fit in u32");
    let mut length: usize = 0;

    while cursor != ROW_NONE {
        assert!(length < buffer.len());

        let row = rows[cursor as usize];

        debug_assert!(matches!(row.kind, RowKind::Directory | RowKind::File));

        buffer[length] = row.index;
        length += 1;
        cursor = row.parent;
    }

    let path = &mut buffer[..length];

    path.reverse();

    path
}

fn open_path(path: &Path, sender: &MessageSender, action: OpenAction) {
    let outcome = match action {
        OpenAction::Open => platform::path_open(path),
        OpenAction::Reveal => platform::path_reveal(path),
    };

    if let Err(error) = outcome {
        let text = format!("Failed to open {}: {error}", path.display());

        sender.send(Message::Notice(Notice::Error(text)));
    }
}

fn paint_guides(ui: &egui::Ui, view: &TreeView<'_>, row_index: usize, rectangle: egui::Rect) {
    if !ui.visuals().indent_has_left_vline {
        return;
    }

    let depth = view.rows[row_index].depth;
    let depth_next = view.rows.get(row_index + 1).map_or(0, |row| row.depth);
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    let bottom_continued = rectangle.bottom() + ui.spacing().item_spacing.y;
    let bottom_closed = rectangle.bottom() - GUIDE_INSET_BOTTOM_PIXELS;

    for level in 0..depth {
        let x = view.geometry.guide_x(rectangle.left(), level);

        let bottom = if depth_next > level {
            bottom_continued
        } else {
            bottom_closed
        };

        ui.painter().line_segment(
            [egui::pos2(x, rectangle.top()), egui::pos2(x, bottom)],
            stroke,
        );
    }
}

fn placeholder(kind: RowKind, depth: u16, parent: u32) -> TreeRow {
    debug_assert!(matches!(
        kind,
        RowKind::Failed | RowKind::Loading | RowKind::LoadingText,
    ));

    TreeRow {
        depth: depth + 1,
        index: 0,
        kind,
        parent,
    }
}

pub fn render(ui: &mut egui::Ui, model: &Model, rows: &mut Vec<TreeRow>, sender: &MessageSender) {
    let expansions = build_rows(model, ui.ctx(), rows);
    let mut buffer: NodePath = [0; TREE_DEPTH_MAX as usize];

    for row_index in expansions {
        let path = node_path(rows, row_index, &mut buffer).to_vec();

        sender.send(Message::Tree(Tree::NodeExpanded { path }));
    }

    let background = ui.interact(
        ui.max_rect(),
        ui.id().with("tree-background"),
        egui::Sense::click(),
    );

    background.context_menu(|menu| show_tree_context_menu(menu, sender));

    let view = TreeView {
        geometry: Geometry::measure(ui),
        model,
        rows,
        sender,
    };

    let row_height = ui.spacing().interact_size.y;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, view.rows.len(), |scroll, range| {
            for row_index in range {
                render_row(scroll, &view, row_index);
            }
        });
}

fn render_directory(
    ui: &mut egui::Ui,
    node: &FileNode,
    row: &RowContext<'_>,
    sender: &MessageSender,
) {
    let mut checked = node.checked;

    if ui.checkbox(&mut checked, "").clicked() {
        send_toggle(sender, row.path, checked, ToggleScope::Subtree);
    }

    let mut state = directory_state(ui.ctx(), node, row.depth);
    let header = render_directory_header(ui, &node.name, &mut state);

    header.context_menu(|menu| directory_context_menu(menu, node, row.path, sender));
    state.store(ui.ctx());
    fill_row(ui, sender);
}

fn render_directory_header(
    ui: &mut egui::Ui,
    name: &str,
    state: &mut CollapsingState,
) -> egui::Response {
    let indent = ui.spacing().indent;
    let padding = ui.spacing().button_padding;

    let galley = egui::WidgetText::from(name).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        ui.available_width() - indent,
        egui::TextStyle::Button,
    );

    let size = egui::vec2(
        indent + galley.size().x + padding.x,
        padding.y.mul_add(2.0, galley.size().y),
    );

    let (rectangle, response) =
        ui.allocate_exact_size(size.max(ui.spacing().interact_size), egui::Sense::click());

    if response.clicked() {
        state.toggle(ui);
    }

    if !ui.is_rect_visible(rectangle) {
        return response;
    }

    let visuals = ui.style().interact_selectable(&response, false);
    let (mut icon, _) = ui.spacing().icon_rectangles(rectangle);

    icon.set_center(egui::pos2(
        rectangle.left() + indent / 2.0,
        rectangle.center().y,
    ));
    paint_default_icon(
        ui,
        state.openness(ui.ctx()),
        &response.clone().with_new_rect(icon),
    );

    let text = egui::pos2(
        rectangle.left() + indent,
        rectangle.center().y - galley.size().y / 2.0,
    );

    ui.painter().galley(text, galley, visuals.text_color());

    response
}

fn render_file(ui: &mut egui::Ui, node: &FileNode, row: &RowContext<'_>, sender: &MessageSender) {
    let mut checked = node.checked;

    if ui.checkbox(&mut checked, "").clicked() {
        send_toggle(sender, row.path, checked, ToggleScope::Node);
    }

    let label = ui.selectable_label(node.checked, &*node.name);

    if label.clicked() {
        send_toggle(sender, row.path, !node.checked, ToggleScope::Node);
    }

    label.context_menu(|menu| file_context_menu(menu, node, row.path, sender));
    fill_row(ui, sender);
}

fn render_node_row(ui: &mut egui::Ui, view: &TreeView<'_>, row_index: usize) {
    let mut buffer: NodePath = [0; TREE_DEPTH_MAX as usize];
    let path = node_path(view.rows, row_index, &mut buffer);

    let Some(node) = selection::find_node(&view.model.tree.nodes, path) else {
        return;
    };

    let row = RowContext {
        depth: view.rows[row_index].depth,
        path,
    };

    if node.is_directory() {
        render_directory(ui, node, &row, view.sender);
    } else {
        render_file(ui, node, &row, view.sender);
    }
}

fn render_row(ui: &mut egui::Ui, view: &TreeView<'_>, row_index: usize) {
    let row = view.rows[row_index];
    let row_height = ui.spacing().interact_size.y;
    let size = egui::vec2(ui.available_width(), row_height);
    let layout = egui::Layout::left_to_right(egui::Align::Center);

    let cell = ui.allocate_ui_with_layout(size, layout, |cell| {
        cell.set_min_height(row_height);
        cell.add_space(view.geometry.row_offset(row.depth));

        match row.kind {
            RowKind::Directory | RowKind::File => render_node_row(cell, view, row_index),
            RowKind::Failed => {
                cell.label(
                    egui::RichText::new("Failed to read this directory")
                        .color(cell.visuals().error_fg_color),
                );
            }
            RowKind::Loading => {
                cell.spinner();
            }
            RowKind::LoadingText => {
                cell.label("Loading...");
            }
        }
    });

    paint_guides(ui, view, row_index, cell.response.rect);
}

fn row_kind(node: &FileNode) -> RowKind {
    if node.is_directory() {
        return RowKind::Directory;
    }

    RowKind::File
}

fn send_toggle(sender: &MessageSender, path: &[u32], checked: bool, scope: ToggleScope) {
    assert_ne!(path.len(), 0);

    sender.send(Message::Tree(Tree::NodeToggled {
        checked,
        path: path.to_vec(),
        scope,
    }));
}

fn should_show_node(node: &FileNode, search: &SearchModel) -> bool {
    if !search.has_query() {
        return true;
    }

    search.is_path_matching(&node.path).unwrap_or(true)
}

pub fn show_tree_context_menu(ui: &mut egui::Ui, sender: &MessageSender) {
    if ui.button("Select All").clicked() {
        sender.send(Message::Tree(Tree::SelectAll));
        ui.close();
    }

    if ui.button("Deselect All").clicked() {
        sender.send(Message::Tree(Tree::DeselectAll));
        ui.close();
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::mpsc;

    use super::*;
    use crate::model::node::NodeKind;
    use crate::model::options::Options;

    const SHAPE_COUNT_MAX: u32 = 100_000;

    struct Layout {
        guides: Vec<f32>,
        texts: Vec<(String, egui::Pos2)>,
    }

    fn layout_of(output: &egui::FullOutput) -> Layout {
        let mut guides = Vec::new();
        let mut texts = Vec::new();

        let mut pending: Vec<&egui::Shape> =
            output.shapes.iter().map(|clipped| &clipped.shape).collect();

        for _ in 0..SHAPE_COUNT_MAX {
            let Some(shape) = pending.pop() else {
                break;
            };

            match shape {
                egui::Shape::LineSegment { points, stroke } => {
                    let vertical = (points[0].x - points[1].x).abs() < f32::EPSILON;

                    if vertical {
                        if stroke.width > 0.0 {
                            guides.push(points[0].x);
                        }
                    }
                }
                egui::Shape::Text(text) => texts.push((text.galley.text().to_owned(), text.pos)),
                egui::Shape::Vec(shapes) => pending.extend(shapes.iter()),
                _ => {}
            }
        }

        assert_eq!(pending.len(), 0);

        guides.sort_by(f32::total_cmp);
        guides.dedup_by(|left, right| (*left - *right).abs() < 0.5);

        Layout { guides, texts }
    }

    fn position_of(layout: &Layout, name: &str) -> egui::Pos2 {
        layout
            .texts
            .iter()
            .find(|(text, _)| text == name)
            .map(|(_, position)| *position)
            .expect("the label is painted")
    }

    fn run_twice(context: &egui::Context, mut add_contents: impl FnMut(&mut egui::Ui)) -> Layout {
        let input = egui::RawInput::default();

        let _ = context.run_ui(input.clone(), &mut add_contents);
        let output = context.run_ui(input, &mut add_contents);

        layout_of(&output)
    }

    fn render_reference(ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |scroll| {
                scroll.horizontal(|row| {
                    let _ = row.checkbox(&mut false, "");

                    egui::CollapsingHeader::new("root")
                        .id_salt("/root")
                        .default_open(true)
                        .show(row, render_reference_body);
                });
            });
    }

    fn render_reference_body(body: &mut egui::Ui) {
        body.horizontal(|row| {
            let _ = row.checkbox(&mut false, "");

            egui::CollapsingHeader::new("nested")
                .id_salt("/root/nested")
                .default_open(true)
                .show(row, |nested| {
                    nested.horizontal(|leaf| {
                        let _ = leaf.checkbox(&mut false, "");
                        let _ = leaf.selectable_label(false, "deep.rs");
                    });
                });
        });

        body.horizontal(|row| {
            let _ = row.checkbox(&mut false, "");
            let _ = row.selectable_label(false, "a.rs");
        });
    }

    fn directory(path: &str, children: Vec<FileNode>) -> FileNode {
        let mut node = FileNode::with_kind(PathBuf::from(path), NodeKind::Directory);

        node.children = children;
        node.loaded = true;

        node
    }

    fn file(path: &str) -> FileNode {
        FileNode::with_kind(PathBuf::from(path), NodeKind::File)
    }

    fn model_with(nodes: Vec<FileNode>) -> Model {
        let options = Options::default();
        let mut model = Model::new(options, Vec::new());

        model.tree.nodes = nodes;

        model
    }

    #[test]
    fn only_open_directories_contribute_rows() {
        let model = model_with(vec![directory(
            "/root",
            vec![
                directory("/root/closed", vec![file("/root/closed/a.rs")]),
                file("/root/b.rs"),
            ],
        )]);

        let context = egui::Context::default();
        let mut rows = Vec::new();
        let expansions = build_rows(&model, &context, &mut rows);

        assert_eq!(rows.len(), 3);
        assert_eq!(expansions, Vec::<usize>::new());
        assert_eq!(rows[1].depth, 1);
    }

    #[test]
    fn row_paths_resolve_back_to_their_nodes() {
        let model = model_with(vec![directory(
            "/root",
            vec![file("/root/a.rs"), file("/root/b.rs")],
        )]);

        let context = egui::Context::default();
        let mut rows = Vec::new();
        let mut buffer: NodePath = [0; TREE_DEPTH_MAX as usize];

        let _ = build_rows(&model, &context, &mut rows);

        let path = node_path(&rows, 2, &mut buffer);

        assert_eq!(path, &[0, 1]);

        assert_eq!(
            &*selection::find_node(&model.tree.nodes, path).expect("the node exists").name,
            "b.rs",
        );
    }

    #[test]
    fn an_open_unloaded_directory_requests_its_children() {
        let mut root = directory("/root", Vec::new());

        root.loaded = false;

        let model = model_with(vec![root]);
        let context = egui::Context::default();
        let mut rows = Vec::new();
        let expansions = build_rows(&model, &context, &mut rows);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1].kind, RowKind::Loading);
        assert_eq!(rows[2].kind, RowKind::LoadingText);
        assert_eq!(expansions, vec![0]);
    }

    #[test]
    fn shifting_rows_keep_their_widget_ids() {
        let (channel, _receiver) = mpsc::sync_channel(64);
        let sender = MessageSender::new(channel);
        let context = egui::Context::default();

        let before = model_with(vec![
            directory("/ab", Vec::new()),
            directory("/ba", Vec::new()),
        ]);

        let after = model_with(vec![directory("/ba", Vec::new())]);
        let mut rows = Vec::new();
        let input = egui::RawInput::default();

        let _ = context.run_ui(input.clone(), |ui| render(ui, &before, &mut rows, &sender));
        let output = context.run_ui(input, |ui| render(ui, &after, &mut rows, &sender));

        let warned = output.shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape) => shape.stroke.color == egui::Color32::RED,
            _ => false,
        });

        assert!(!warned);
    }

    #[test]
    fn rows_line_up_with_nested_collapsing_headers() {
        let model = model_with(vec![directory(
            "/root",
            vec![
                directory("/root/nested", vec![file("/root/nested/deep.rs")]),
                file("/root/a.rs"),
            ],
        )]);

        let (channel, _receiver) = mpsc::sync_channel(64);
        let sender = MessageSender::new(channel);
        let flat_context = egui::Context::default();
        let mut nested = directory_state(&flat_context, &model.tree.nodes[0].children[0], 1);
        let mut rows = Vec::new();

        nested.set_open(true);
        nested.store(&flat_context);

        let flat = run_twice(&flat_context, |ui| render(ui, &model, &mut rows, &sender));
        let reference = run_twice(&egui::Context::default(), render_reference);

        for name in ["root", "nested", "deep.rs", "a.rs"] {
            let flat_position = position_of(&flat, name);
            let reference_position = position_of(&reference, name);

            assert!((flat_position.x - reference_position.x).abs() < 0.5, "{name} x");
            assert!((flat_position.y - reference_position.y).abs() < 0.5, "{name} y");
        }

        assert_eq!(flat.guides.len(), reference.guides.len());

        for (flat_x, reference_x) in flat.guides.iter().zip(&reference.guides) {
            assert!((flat_x - reference_x).abs() < 0.5);
        }
    }
}

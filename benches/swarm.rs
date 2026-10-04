use core::hint::black_box;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use criterion::{Criterion, criterion_group, criterion_main};

use swarm::app::message::MessageSender;
use swarm::app::state::{Model, UiState};
use swarm::model::node::{self, FileNode, NodeKind};
use swarm::model::options::Options;
use swarm::model::query::ParsedQuery;
use swarm::model::theme::Theme;
use swarm::services::filesystem::filter::GlobPathFilter;
use swarm::services::filesystem::gather::{GatherRequest, gather};
use swarm::services::tree::query::matching_paths;
use swarm::ui::panel::central;

const CONTENT: &str = "fn placeholder(value: u32) -> u32 {\n    value + 1\n}\n";
const FILE_COUNT_FLAT: u32 = 4_000;
const MESSAGE_QUEUE_COUNT: usize = 1_024;

struct ForestShape {
    files_per_directory: u32,
    nested_per_top: u32,
    top: u32,
}

struct TemporaryTree {
    root: PathBuf,
}

impl TemporaryTree {
    fn new(shape: &ForestShape) -> Self {
        let root = std::env::temp_dir().join(format!("swarm-bench-gather-{}", std::process::id()));

        if root.exists() {
            fs::remove_dir_all(&root).expect("the stale benchmark directory is removable");
        }

        for directory_index in 0..shape.top {
            let directory = root.join(format!("package_{directory_index}"));

            fs::create_dir_all(&directory).expect("the benchmark directory is creatable");

            for file_index in 0..shape.files_per_directory {
                let extension = if file_index % 3 == 0 { "rs" } else { "py" };
                let path = directory.join(format!("source_{file_index}.{extension}"));

                fs::write(&path, CONTENT).expect("the benchmark file is writable");
            }
        }

        Self { root }
    }
}

impl Drop for TemporaryTree {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.root) {
            eprintln!("the benchmark directory was not removed: {error}");
        }
    }
}

fn bench_gather(criterion: &mut Criterion) {
    let shape = ForestShape {
        files_per_directory: 100,
        nested_per_top: 0,
        top: 20,
    };

    let tree = TemporaryTree::new(&shape);
    let options = Options::default();
    let paths = vec![tree.root.clone()];
    let query = ParsedQuery::parse("ext:rs");
    let mut group = criterion.benchmark_group("gather");

    group.sample_size(20);

    group.bench_function("no_query", |bencher| {
        let request = GatherRequest {
            git: None,
            options: &options,
            paths: &paths,
            query: None,
        };

        bencher.iter(|| black_box(gather(black_box(&request)).expect("the gather succeeds")));
    });

    group.bench_function("ext_query", |bencher| {
        let request = GatherRequest {
            git: None,
            options: &options,
            paths: &paths,
            query: Some(&query),
        };

        bencher.iter(|| black_box(gather(black_box(&request)).expect("the gather succeeds")));
    });

    group.finish();
}

fn bench_glob_filter(criterion: &mut Criterion) {
    let options = Options::default();
    let pattern_count = options.exclude.len();

    assert!(pattern_count > 0);

    let mut group = criterion.benchmark_group("glob_filter");

    group.bench_function(format!("{pattern_count}_exclude_patterns"), |bencher| {
        bencher.iter(|| {
            black_box(
                GlobPathFilter::from_options(black_box(&options)).expect("the patterns compile"),
            )
        });
    });

    group.finish();
}

fn bench_matching_paths(criterion: &mut Criterion) {
    let shape = ForestShape {
        files_per_directory: 500,
        nested_per_top: 10,
        top: 20,
    };

    let nodes = build_forest(&shape);
    let mut node_count: u32 = 0;

    node::visit(&nodes, |_, _| node_count += 1);

    assert!(node_count >= 100_000, "the forest is {node_count} nodes");

    let mut group = criterion.benchmark_group("matching_paths");

    group.sample_size(20);

    for query_text in ["ext:rs", "path:module_3", "\"source_7.rs\""] {
        let query = ParsedQuery::parse(query_text);

        group.bench_function(query_text, |bencher| {
            bencher.iter(|| black_box(matching_paths(black_box(&nodes), &query, None, 0)));
        });
    }

    group.finish();
}

fn bench_render_frame(criterion: &mut Criterion) {
    let options = Options::default();
    let mut model = Model::new(options, Vec::new());
    let mut ui_state = UiState::new(Theme::default());
    let (channel, _receiver) = mpsc::sync_channel(MESSAGE_QUEUE_COUNT);
    let sender = MessageSender::new(channel);
    let context = egui::Context::default();
    let mut input = egui::RawInput::default();

    model.tree.nodes = build_flat_root(FILE_COUNT_FLAT);

    input.screen_rect = Some(egui::Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        egui::vec2(1400.0, 1000.0),
    ));

    let _ = context.run_ui(input.clone(), |ui| {
        central::render(ui, &model, &mut ui_state, &sender);
    });

    let mut group = criterion.benchmark_group("render_frame");

    group.sample_size(20);

    group.bench_function(format!("{FILE_COUNT_FLAT}_files"), |bencher| {
        bencher.iter(|| {
            let output = context.run_ui(input.clone(), |ui| {
                central::render(ui, &model, &mut ui_state, &sender);
            });

            black_box(output.shapes.len())
        });
    });

    group.finish();
}

fn build_flat_root(file_count: u32) -> Vec<FileNode> {
    let root_path = PathBuf::from("/bench/root");
    let mut root = directory(root_path.clone());

    for index in 0..file_count {
        root.children.push(file(&root_path, index));
    }

    vec![root]
}

fn build_forest(shape: &ForestShape) -> Vec<FileNode> {
    let root_path = PathBuf::from("/bench/root");
    let mut root = directory(root_path.clone());

    for top_index in 0..shape.top {
        let top_path = root_path.join(format!("package_{top_index}"));
        let mut top = directory(top_path.clone());

        for nested_index in 0..shape.nested_per_top {
            let nested_path = top_path.join(format!("module_{nested_index}"));
            let mut nested = directory(nested_path.clone());

            for file_index in 0..shape.files_per_directory {
                nested.children.push(file(&nested_path, file_index));
            }

            top.children.push(nested);
        }

        root.children.push(top);
    }

    vec![root]
}

fn directory(path: PathBuf) -> FileNode {
    let mut node = FileNode::with_kind(path, NodeKind::Directory);

    node.loaded = true;

    node
}

fn file(parent: &Path, index: u32) -> FileNode {
    let extension = if index.is_multiple_of(3) { "rs" } else { "py" };

    FileNode::with_kind(
        parent.join(format!("source_{index}.{extension}")),
        NodeKind::File,
    )
}

criterion_group!(
    benches,
    bench_matching_paths,
    bench_gather,
    bench_render_frame,
    bench_glob_filter,
);
criterion_main!(benches);

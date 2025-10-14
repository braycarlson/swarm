#![windows_subsystem = "windows"]

use std::env;
use std::path::Path;

use swarm::{SwarmApp, APP_NAME};

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    let paths = parse_arguments(&args);

    let app = SwarmApp::new(paths);
    let options = create_window_options();

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}

fn parse_arguments(args: &[String]) -> Vec<String> {
    if args.len() <= 1 {
        return vec![];
    }

    args[1..]
        .iter()
        .map(|arg| normalize_path(arg))
        .collect()
}

fn normalize_path(path_str: &str) -> String {
    let path = Path::new(path_str);

    if path.is_file() {
        path.parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| path_str.to_string())
    } else {
        path_str.to_string()
    }
}

fn create_window_options() -> eframe::NativeOptions {
    const WINDOW_WIDTH: f32 = 1000.0;
    const WINDOW_HEIGHT: f32 = 700.0;
    const SCREEN_WIDTH: f32 = 1920.0;
    const SCREEN_HEIGHT: f32 = 1080.0;

    let pos_x = (SCREEN_WIDTH - WINDOW_WIDTH) / 2.0;
    let pos_y = (SCREEN_HEIGHT - WINDOW_HEIGHT) / 2.0;

    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_position([pos_x, pos_y])
            .with_decorations(false),
        ..Default::default()
    }
}

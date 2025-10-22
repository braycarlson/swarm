#![windows_subsystem = "windows"]

use single_instance::SingleInstance;
use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use swarm::{SwarmApp, APP_NAME};

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    let paths = parse_arguments(&args);

    let options = swarm::model::Options::load().unwrap_or_default();

    let instance_guard = if options.single_instance {
        let guard = SingleInstance::new("swarm-single-instance")
            .expect("Failed to create single instance guard");

        if !guard.is_single() {
            if !paths.is_empty()
                && let Ok(mut stream) = TcpStream::connect("127.0.0.1:44287")
            {
                let content = paths.join("\n");
                let _ = stream.write_all(content.as_bytes());
            }

            return Ok(());
        }

        Some(guard)
    } else {
        None
    };

    let app = SwarmApp::new(paths, instance_guard);
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

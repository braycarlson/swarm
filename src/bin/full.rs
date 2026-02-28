#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use eframe::egui;
use single_instance::SingleInstance;
use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use swarm::{SwarmApp, APP_NAME};
use swarm::cli;

const CLI_FLAGS: &[&str] = &[
    "--diff", "-d",
    "--format", "-f",
    "--help", "-h",
    "--output", "-o",
    "--search", "-s",
    "--skeleton", "-k",
    "--stdout",
    "--tree", "-t",
    "--version", "-V",
];

fn main() {
    let args: Vec<String> = env::args().collect();

    if has_cli_flags(&args) {
        attach_console();
        let cli = cli::Cli::parse();
        cli::run(cli);
        flush_console_prompt();
    } else {
        let paths = parse_gui_paths(&args);
        run_gui(paths);
    }
}

fn has_cli_flags(args: &[String]) -> bool {
    args.iter().any(|arg| CLI_FLAGS.contains(&arg.as_str()))
}

fn parse_gui_paths(args: &[String]) -> Vec<String> {
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

#[cfg(windows)]
fn attach_console() {
    use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};

    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(windows)]
fn flush_console_prompt() {
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_INPUT_HANDLE;
    use winapi::um::wincon::WriteConsoleInputW;
    use winapi::um::wincontypes::{INPUT_RECORD, KEY_EVENT};

    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        let mut record: INPUT_RECORD = std::mem::zeroed();
        record.EventType = KEY_EVENT;

        let key_event = record.Event.KeyEvent_mut();
        key_event.bKeyDown = 1;
        key_event.wRepeatCount = 1;
        key_event.wVirtualKeyCode = 0x0D;
        *key_event.uChar.UnicodeChar_mut() = '\r' as u16;

        let mut written = 0;
        WriteConsoleInputW(handle, &record, 1, &mut written);
    }
}

#[cfg(not(windows))]
fn attach_console() {}

#[cfg(not(windows))]
fn flush_console_prompt() {}

fn run_gui(paths: Vec<String>) {
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

            return;
        }

        Some(guard)
    } else {
        None
    };

    let app = SwarmApp::new(paths, instance_guard);
    let options = create_window_options();

    if let Err(error) = eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to launch GUI: {}", error);
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
            .with_decorations(false)
            .with_icon(load_icon()),
        ..Default::default()
    }
}

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../../assets/logo.ico");

    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();

    let (width, height) = image.dimensions();

    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

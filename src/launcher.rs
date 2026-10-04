use std::ffi::OsString;
use std::io::Write as _;
use std::net::TcpStream;
use std::path::PathBuf;

use clap::CommandFactory as _;
use eframe::egui;
use single_instance::SingleInstance;

use crate::app::SwarmApp;
use crate::cli::CommandLineInterface;
use crate::constants::{APP_NAME, IPC_ADDRESS, IPC_MESSAGE_BYTES_MAX, IPC_PATH_COUNT_MAX};
use crate::model::options::Options;
use crate::model::path;

const ICON: &[u8] = include_bytes!("../assets/logo.ico");
const INSTANCE_NAME: &str = "swarm-single-instance";
const WINDOW_HEIGHT_PIXELS: f32 = 700.0;
const WINDOW_WIDTH_PIXELS: f32 = 1000.0;

enum InstanceOutcome {
    Continue(Option<SingleInstance>),
    HandedOff,
}

fn acquire_instance(paths: &[PathBuf]) -> InstanceOutcome {
    let guard = match SingleInstance::new(INSTANCE_NAME) {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("Continuing without a single-instance guard: {error}");

            return InstanceOutcome::Continue(None);
        }
    };

    if guard.is_single() {
        return InstanceOutcome::Continue(Some(guard));
    }

    if !paths.is_empty() {
        hand_off_paths(paths);
    }

    InstanceOutcome::HandedOff
}

fn create_window_options() -> eframe::NativeOptions {
    let mut options = eframe::NativeOptions::default();

    options.viewport = options
        .viewport
        .with_app_id(APP_NAME)
        .with_decorations(false)
        .with_icon(load_icon())
        .with_inner_size([WINDOW_WIDTH_PIXELS, WINDOW_HEIGHT_PIXELS]);

    options.centered = true;

    options
}

fn hand_off_paths(paths: &[PathBuf]) {
    assert_ne!(paths.len(), 0);

    let mut content = String::new();
    let mut count: u32 = 0;

    for requested in paths {
        let Some(text) = requested.to_str() else {
            eprintln!("Skipping {}: the path is not valid UTF-8", requested.display());

            continue;
        };

        if text.contains('\n') {
            eprintln!("Skipping {}: the path contains a newline", requested.display());

            continue;
        }

        if count == IPC_PATH_COUNT_MAX {
            eprintln!("Handing off the first {IPC_PATH_COUNT_MAX} paths only");

            break;
        }

        content.push_str(text);
        content.push('\n');
        count += 1;
    }

    if content.len() as u64 > IPC_MESSAGE_BYTES_MAX {
        eprintln!("The paths exceed the {IPC_MESSAGE_BYTES_MAX} byte hand-off limit");

        return;
    }

    let mut stream = match TcpStream::connect(IPC_ADDRESS) {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("Failed to reach the running instance at {IPC_ADDRESS}: {error}");

            return;
        }
    };

    if let Err(error) = stream.write_all(content.as_bytes()) {
        eprintln!("Failed to hand the paths to the running instance: {error}");
    }
}

pub fn has_cli_flags(arguments: &[OsString]) -> bool {
    let mut command = CommandLineInterface::command();

    command.build();

    let mut flags: Vec<String> = Vec::new();

    for argument in command.get_arguments() {
        if let Some(long) = argument.get_long() {
            flags.push(format!("--{long}"));
        }

        if let Some(short) = argument.get_short() {
            flags.push(format!("-{short}"));
        }
    }

    assert_ne!(flags.len(), 0);

    arguments.iter().skip(1).any(|argument| {
        flags
            .iter()
            .any(|flag| argument.as_os_str() == flag.as_str())
    })
}

fn load_icon() -> egui::IconData {
    let image = image::load_from_memory(ICON)
        .expect("the embedded icon decodes")
        .into_rgba8();

    let (width, height) = image.dimensions();

    assert!(width > 0);
    assert!(height > 0);

    egui::IconData {
        height,
        rgba: image.into_raw(),
        width,
    }
}

pub fn parse_arguments(arguments: &[OsString]) -> Vec<PathBuf> {
    let paths: Vec<PathBuf> = arguments
        .iter()
        .skip(1)
        .map(|argument| path::directory_of(&PathBuf::from(argument)))
        .collect();

    debug_assert!(paths.len() < arguments.len().max(1));

    paths
}

pub fn run_gui(paths: Vec<PathBuf>) {
    let options = Options::load_or_default();

    let instance_guard = if options.single_instance {
        match acquire_instance(&paths) {
            InstanceOutcome::Continue(guard) => guard,
            InstanceOutcome::HandedOff => return,
        }
    } else {
        None
    };

    let app = SwarmApp::new(options, paths, instance_guard);

    let outcome = eframe::run_native(
        APP_NAME,
        create_window_options(),
        Box::new(|_| Ok(Box::new(app))),
    );

    if let Err(error) = outcome {
        eprintln!("Failed to launch the GUI: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn no_arguments_yields_no_paths() {
        assert_eq!(parse_arguments(&arguments(&["swarm"])), Vec::<PathBuf>::new());
        assert_eq!(parse_arguments(&[]), Vec::<PathBuf>::new());
    }

    #[test]
    fn a_directory_argument_passes_through() {
        let directory = std::env::temp_dir();

        let text = directory
            .to_str()
            .expect("the temporary directory is valid UTF-8");

        assert_eq!(parse_arguments(&arguments(&["swarm", text])), vec![directory]);
    }

    #[test]
    fn cli_flags_are_read_from_the_parser() {
        assert!(has_cli_flags(&arguments(&["swarm", "--tree"])));
        assert!(has_cli_flags(&arguments(&["swarm", "-t"])));
        assert!(has_cli_flags(&arguments(&["swarm", "--help"])));
        assert!(!has_cli_flags(&arguments(&["swarm", "/tmp"])));
    }
}

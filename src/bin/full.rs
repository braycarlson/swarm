#![cfg_attr(optimized, windows_subsystem = "windows")]

use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use clap::Parser as _;
use swarm::platform::clipboard;
use swarm::{cli, launcher, platform};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
    let arguments: Vec<OsString> = env::args_os().collect();

    if clipboard::run_daemon_if_requested(&arguments) {
        return ExitCode::SUCCESS;
    }

    if !launcher::has_cli_flags(&arguments) {
        launcher::run_gui(launcher::parse_arguments(&arguments));

        return ExitCode::SUCCESS;
    }

    platform::console_attach();

    let command = cli::CommandLineInterface::parse();
    let code = cli::run(&command);

    platform::console_prompt_flush();

    code
}

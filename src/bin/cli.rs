use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use clap::Parser as _;
use swarm::cli;
use swarm::platform::clipboard;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
    let arguments: Vec<OsString> = env::args_os().collect();

    if clipboard::run_daemon_if_requested(&arguments) {
        return ExitCode::SUCCESS;
    }

    let command = cli::CommandLineInterface::parse();

    cli::run(&command)
}

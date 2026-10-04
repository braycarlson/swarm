#![cfg_attr(optimized, windows_subsystem = "windows")]

use std::env;
use std::ffi::OsString;

use swarm::launcher;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let arguments: Vec<OsString> = env::args_os().collect();
    let paths = launcher::parse_arguments(&arguments);

    launcher::run_gui(paths);
}

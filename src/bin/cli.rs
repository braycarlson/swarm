#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::Parser;
use swarm::cli;

fn main() {
    let cli = cli::CommandLineInterface::parse();
    cli::run(cli);
}

use clap::Parser;
use swarm::cli;

fn main() {
    let cli = cli::Cli::parse();
    cli::run(cli);
}

use clap::Parser;
use worm::cli::{run_cli, Cli};

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run_cli(cli) {
        eprintln!("worm: {error}");
        std::process::exit(1);
    }
}

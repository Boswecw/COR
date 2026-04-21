use clap::Parser;
use repo_crawler::cli::{run_cli, Cli};

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run_cli(cli) {
        eprintln!("repo-crawler: {error}");
        std::process::exit(1);
    }
}

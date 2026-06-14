use std::process::ExitCode;

use repo_crawler::centipede_queue_export::{help_text, parse_args, run_export};

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            if message == help_text() {
                println!("{message}");
                return ExitCode::SUCCESS;
            }

            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    match run_export(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

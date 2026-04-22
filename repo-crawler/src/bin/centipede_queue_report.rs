use std::env;
use std::path::PathBuf;

#[path = "../centipede_queue_report.rs"]
mod centipede_queue_report;

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 && args.len() != 3 {
        return Err(
            "usage: centipede_queue_report <queue_dir> [queue_item_id]".to_string(),
        );
    }

    let queue_dir = PathBuf::from(&args[1]);
    let queue_item_id = if args.len() == 3 {
        Some(args[2].as_str())
    } else {
        None
    };

    let report = centipede_queue_report::build_queue_report(&queue_dir, queue_item_id)?;
    let pretty = serde_json::to_string_pretty(&report)
        .map_err(|err| format!("could not serialize report: {}", err))?;
    println!("{}", pretty);
    Ok(())
}

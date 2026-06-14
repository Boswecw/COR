use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../centipede_queue_heartbeat.rs"]
mod centipede_queue_heartbeat;

use serde_json::Value;

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        return Err(
            "usage: centipede_queue_heartbeat <queue_dir> <claim_receipt_path> <heartbeat_at> <lease_timeout_seconds>"
                .to_string(),
        );
    }

    let queue_dir = PathBuf::from(&args[1]);
    let claim_receipt_path = PathBuf::from(&args[2]);
    let heartbeat_at = &args[3];
    let lease_timeout_seconds = args[4]
        .parse::<i64>()
        .map_err(|_| "lease_timeout_seconds must be an integer".to_string())?;

    let claim_receipt = read_json(&claim_receipt_path)?;
    let receipt = centipede_queue_heartbeat::heartbeat_claim_from_receipt(
        &queue_dir,
        &claim_receipt,
        heartbeat_at,
        lease_timeout_seconds,
    )?;

    let pretty = serde_json::to_string_pretty(&receipt)
        .map_err(|err| format!("could not serialize receipt: {}", err))?;
    println!("{}", pretty);
    Ok(())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {}", path.display(), err))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("could not parse {}: {}", path.display(), err))
}
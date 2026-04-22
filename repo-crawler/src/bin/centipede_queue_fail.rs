use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../centipede_queue_fail.rs"]
mod centipede_queue_fail;

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
            "usage: centipede_queue_fail <queue_dir> <claim_receipt_path> <failed_at> <reason>"
                .to_string(),
        );
    }

    let queue_dir = PathBuf::from(&args[1]);
    let claim_receipt_path = PathBuf::from(&args[2]);
    let failed_at = &args[3];
    let reason = &args[4];

    let claim_receipt = read_json(&claim_receipt_path)?;
    let receipt =
        centipede_queue_fail::fail_from_claim_receipt(&queue_dir, &claim_receipt, failed_at, reason)?;

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
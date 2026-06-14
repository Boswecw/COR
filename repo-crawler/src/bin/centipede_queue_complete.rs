use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../centipede_queue_complete.rs"]
mod centipede_queue_complete;

use serde_json::Value;

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let queue_dir = PathBuf::from(
        args.next()
            .ok_or_else(|| "usage: centipede_queue_complete <queue_dir> <claim_receipt_json> <completed_at> <resolution> <receipt_out_json>".to_string())?,
    );
    let claim_receipt_path = PathBuf::from(
        args.next()
            .ok_or_else(|| "missing <claim_receipt_json>".to_string())?,
    );
    let completed_at = args
        .next()
        .ok_or_else(|| "missing <completed_at>".to_string())?;
    let resolution = args
        .next()
        .ok_or_else(|| "missing <resolution>".to_string())?;
    let receipt_out = PathBuf::from(
        args.next()
            .ok_or_else(|| "missing <receipt_out_json>".to_string())?,
    );
    if args.next().is_some() {
        return Err("too many arguments supplied".to_string());
    }

    let claim_receipt = read_json(&claim_receipt_path)?;
    println!("OK  read {}", claim_receipt_path.display());

    let receipt = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &claim_receipt,
        &completed_at,
        &resolution,
    )?;

    write_json(&receipt_out, &receipt)?;
    println!("OK  updated {}", queue_dir.display());
    println!("OK  wrote {}", receipt_out.display());
    Ok(())
}

fn read_json(path: &PathBuf) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {}", path.display(), err))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("could not parse {}: {}", path.display(), err))
}

fn write_json(path: &PathBuf, value: &Value) -> Result<(), String> {
    let pretty = serde_json::to_string_pretty(value)
        .map_err(|err| format!("could not serialize {}: {}", path.display(), err))?;
    fs::write(path, format!("{}\n", pretty))
        .map_err(|err| format!("could not write {}: {}", path.display(), err))
}

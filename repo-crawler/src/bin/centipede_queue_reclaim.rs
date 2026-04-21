use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../centipede_queue_reclaim.rs"]
mod centipede_queue_reclaim;

fn main() {
    if let Err(err) = run() {
        println!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        return Err(
            "usage: cargo run --bin centipede_queue_reclaim -- <queue_dir> <reclaimer> <reclaimed_at> <lease_timeout_seconds> <receipt_out.json>"
                .to_string(),
        );
    }

    let queue_dir = PathBuf::from(&args[1]);
    let reclaimer = &args[2];
    let reclaimed_at = &args[3];
    let lease_timeout_seconds: i64 = args[4]
        .parse()
        .map_err(|_| format!("invalid lease_timeout_seconds: {}", args[4]))?;
    let receipt_out = PathBuf::from(&args[5]);

    let receipt = centipede_queue_reclaim::reclaim_expired_claims(
        &queue_dir,
        reclaimer,
        reclaimed_at,
        lease_timeout_seconds,
    )?;
    let pretty = serde_json::to_string_pretty(&receipt)
        .map_err(|err| format!("could not serialize receipt: {}", err))?;
    fs::write(&receipt_out, format!("{}\n", pretty))
        .map_err(|err| format!("could not write {}: {}", receipt_out.display(), err))?;

    println!("OK  reclaimed from {}", queue_dir.display());
    println!("OK  wrote {}", receipt_out.display());
    Ok(())
}

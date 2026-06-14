use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../centipede_queue_claim.rs"]
mod centipede_queue_claim;

fn main() {
    if let Err(err) = run() {
        println!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        return Err(
            "usage: cargo run --bin centipede_queue_claim -- <queue_dir> <claimant> <claimed_at> <receipt_out.json>"
                .to_string(),
        );
    }

    let queue_dir = PathBuf::from(&args[1]);
    let claimant = &args[2];
    let claimed_at = &args[3];
    let receipt_out = PathBuf::from(&args[4]);

    let receipt = centipede_queue_claim::claim_next_queue_item(&queue_dir, claimant, claimed_at)?;
    let pretty = serde_json::to_string_pretty(&receipt)
        .map_err(|err| format!("could not serialize receipt: {}", err))?;
    fs::write(&receipt_out, format!("{}\n", pretty))
        .map_err(|err| format!("could not write {}: {}", receipt_out.display(), err))?;

    println!("OK  claimed from {}", queue_dir.display());
    println!("OK  wrote {}", receipt_out.display());
    Ok(())
}

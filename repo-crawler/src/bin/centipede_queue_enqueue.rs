use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../centipede_queue_writer.rs"]
mod centipede_queue_writer;

fn main() {
    if let Err(err) = run() {
        println!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err(
            "usage: cargo run --bin centipede_queue_enqueue -- <normalized_queue.json> <queue_dir> <receipt_out.json>"
                .to_string(),
        );
    }

    let input_path = PathBuf::from(&args[1]);
    let queue_dir = PathBuf::from(&args[2]);
    let receipt_out = PathBuf::from(&args[3]);

    let receipt = centipede_queue_writer::enqueue_queue_file(&input_path, &queue_dir)?;
    let pretty = serde_json::to_string_pretty(&receipt)
        .map_err(|err| format!("could not serialize receipt: {}", err))?;
    fs::write(&receipt_out, format!("{}\n", pretty))
        .map_err(|err| format!("could not write {}: {}", receipt_out.display(), err))?;

    println!("OK  read {}", input_path.display());
    println!("OK  updated {}", queue_dir.display());
    println!("OK  wrote {}", receipt_out.display());
    Ok(())
}

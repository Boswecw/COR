use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../centipede_intake_normalizer.rs"]
mod centipede_intake_normalizer;

use centipede_intake_normalizer::normalize_handoff;
use serde_json::Value;

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);

    let input_path = PathBuf::from(
        args.next()
            .ok_or_else(|| "usage: worm_centipede_intake_consumer <input-handoff.json> <output-queue.json>".to_string())?,
    );
    let output_path = PathBuf::from(
        args.next()
            .ok_or_else(|| "usage: worm_centipede_intake_consumer <input-handoff.json> <output-queue.json>".to_string())?,
    );

    if args.next().is_some() {
        return Err(
            "usage: worm_centipede_intake_consumer <input-handoff.json> <output-queue.json>".to_string(),
        );
    }

    let raw = fs::read_to_string(&input_path)
        .map_err(|err| format!("could not read {}: {}", input_path.display(), err))?;
    let parsed: Value = serde_json::from_str(&raw)
        .map_err(|err| format!("invalid JSON in {}: {}", input_path.display(), err))?;

    let queue = normalize_handoff(&parsed)?;
    let pretty = serde_json::to_string_pretty(&queue)
        .map_err(|err| format!("could not serialize queue JSON: {}", err))?;

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("could not create {}: {}", parent.display(), err))?;
        }
    }

    fs::write(&output_path, format!("{}\n", pretty))
        .map_err(|err| format!("could not write {}: {}", output_path.display(), err))?;

    println!("OK  read {}", input_path.display());
    println!("OK  wrote {}", output_path.display());
    Ok(())
}

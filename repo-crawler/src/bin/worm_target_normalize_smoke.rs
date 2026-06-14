
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

#[path = "../worm_target_normalizer.rs"]
mod worm_target_normalizer;

fn matching_files(examples_dir: &Path, prefix: &str) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(examples_dir)
        .map_err(|e| format!("failed to read {}: {}", examples_dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with(prefix) && name.ends_with(".json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn main() {
    let repo_root = PathBuf::from(".");
    let examples_dir = repo_root.join("doc/system/worm/examples");

    println!("Worm target normalizer smoke");
    println!("repo root: {}", repo_root.display());

    let files = match matching_files(&examples_dir, "target_resolution_") {
        Ok(files) if !files.is_empty() => files,
        Ok(_) => {
            eprintln!("FAIL  no target resolution example files found");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    };

    for path in files {
        let label = path.display().to_string();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("FAIL  failed to read {label}: {err}");
                std::process::exit(1);
            }
        };

        let value: Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("FAIL  failed to parse {label}: {err}");
                std::process::exit(1);
            }
        };

        let raw = value.get("rawReference").and_then(|v| v.as_str()).unwrap_or("");
        let expected_posture = value.get("resolutionPosture").and_then(|v| v.as_str()).unwrap_or("");
        let expected_method = value.get("resolutionMethod").and_then(|v| v.as_str()).unwrap_or("");

        let actual = worm_target_normalizer::normalize_reference(raw);

        if actual.posture != expected_posture {
            eprintln!(
                "FAIL  {} posture mismatch: expected '{}' got '{}'",
                label, expected_posture, actual.posture
            );
            std::process::exit(1);
        }

        if actual.method != expected_method {
            eprintln!(
                "FAIL  {} method mismatch: expected '{}' got '{}'",
                label, expected_method, actual.method
            );
            std::process::exit(1);
        }

        if expected_posture == "resolved" {
            let expected_identity = value.get("canonicalIdentity").unwrap_or(&Value::Null);
            let expected_display = expected_identity.get("display").and_then(|v| v.as_str()).unwrap_or("");
            let actual_display = actual
                .canonical
                .as_ref()
                .map(|c| c.display.as_str())
                .unwrap_or("");

            if actual_display != expected_display {
                eprintln!(
                    "FAIL  {} display mismatch: expected '{}' got '{}'",
                    label, expected_display, actual_display
                );
                std::process::exit(1);
            }
        }

        println!("OK  {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("<unknown>"));
    }

    println!("Validated Worm target normalizer smoke successfully.");
}

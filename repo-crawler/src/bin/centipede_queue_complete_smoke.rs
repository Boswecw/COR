use std::fs;
use std::path::{Path, PathBuf};

#[path = "../centipede_queue_writer.rs"]
mod centipede_queue_writer;
#[path = "../centipede_queue_claim.rs"]
mod centipede_queue_claim;
#[path = "../centipede_queue_complete.rs"]
mod centipede_queue_complete;

use serde_json::{json, Value};

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!("Centipede queue complete smoke");

    let root = PathBuf::from("/tmp/centipede-queue-complete-smoke");
    if root.exists() {
        fs::remove_dir_all(&root)
            .map_err(|err| format!("could not clear {}: {}", root.display(), err))?;
    }
    fs::create_dir_all(&root)
        .map_err(|err| format!("could not create {}: {}", root.display(), err))?;

    let queue_dir = root.join("queue");
    let intake_path = root.join("input-success.json");
    write_json(&intake_path, &sample_success_queue())?;

    centipede_queue_writer::enqueue_queue_file(&intake_path, &queue_dir)?;
    let claim_receipt = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-01",
        "2026-04-21T22:00:00-04:00",
    )?;
    let receipt = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &claim_receipt,
        "2026-04-21T22:05:00-04:00",
        "proposal_generated",
    )?;
    assert_eq!(
        required_string(&receipt, "disposition")?,
        "completed",
        "first completion should succeed"
    );
    println!("OK  claim_completed");

    let item_path = first_json_file(&queue_dir.join("items"), "index.json")?;
    let item = read_json(&item_path)?;
    let status = item
        .get("processingState")
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .ok_or_else(|| "queue item missing processingState.status".to_string())?;
    assert_eq!(status, "completed", "queue item should be marked completed");
    println!("OK  queue_item_marked_completed");

    let completions_index = read_json(&queue_dir.join("completions/index.json"))?;
    let completion_total = completions_index
        .get("totals")
        .and_then(|value| value.get("completions"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "completion index missing totals.completions".to_string())?;
    assert_eq!(completion_total, 1, "expected exactly one completion");
    println!("OK  completion_index_written");

    let duplicate = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &claim_receipt,
        "2026-04-21T22:06:00-04:00",
        "proposal_generated",
    )?;
    assert_eq!(
        required_string(&duplicate, "disposition")?,
        "already_completed",
        "second completion should be idempotent"
    );
    println!("OK  completion_idempotent");

    println!("Validated Centipede queue complete smoke successfully.");
    Ok(())
}

fn sample_success_queue() -> Value {
    json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "success_path",
        "sourceLane": "worm",
        "sourceRepo": "Boswecw/Cortex",
        "sourceHandoffId": "handoff-bundle-run-repo-surface-08",
        "receivedAt": "2026-04-21T21:58:00-04:00",
        "candidateIssues": [
            {
                "issueKey": "worm::ambiguous_target_identity",
                "title": "Ambiguous target identity",
                "severity": "moderate"
            }
        ],
        "totals": {
            "blocking": 0,
            "candidateIssues": 1
        }
    })
}

fn first_json_file(dir: &Path, skip_name: &str) -> Result<PathBuf, String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|err| format!("could not read {}: {}", dir.display(), err))?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path.file_name().and_then(|value| value.to_str()) != Some(skip_name)
        })
        .collect();
    paths.sort();
    paths
        .into_iter()
        .next()
        .ok_or_else(|| format!("no json file found in {}", dir.display()))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {}", path.display(), err))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("could not parse {}: {}", path.display(), err))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let pretty = serde_json::to_string_pretty(value)
        .map_err(|err| format!("could not serialize {}: {}", path.display(), err))?;
    fs::write(path, format!("{}\n", pretty))
        .map_err(|err| format!("could not write {}: {}", path.display(), err))
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field: {}", key))
}

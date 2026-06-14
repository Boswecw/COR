use std::fs;
use std::path::{Path, PathBuf};

#[path = "../centipede_queue_writer.rs"]
mod centipede_queue_writer;
#[path = "../centipede_queue_claim.rs"]
mod centipede_queue_claim;
#[path = "../centipede_queue_reclaim.rs"]
mod centipede_queue_reclaim;
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

    let first_claim = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-01",
        "2026-04-21T22:00:00-04:00",
    )?;
    assert_eq!(
        required_string(&first_claim, "disposition")?,
        "claimed",
        "first claim should succeed"
    );
    assert_eq!(
        required_u64(&first_claim, "claimAttempt")?,
        1,
        "first claim should be attempt 1"
    );
    println!("OK  first_claimed");

    let reclaim_receipt = centipede_queue_reclaim::reclaim_expired_claims(
        &queue_dir,
        "centipede-reclaimer-01",
        "2026-04-21T22:10:00-04:00",
        300,
    )?;
    assert_eq!(
        required_string(&reclaim_receipt, "disposition")?,
        "reclaimed",
        "expired claim should be reclaimed"
    );
    let reclaimed_total = reclaim_receipt
        .get("totals")
        .and_then(|value| value.get("reclaimed"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "reclaim receipt missing totals.reclaimed".to_string())?;
    assert_eq!(reclaimed_total, 1, "expected one reclaimed item");
    println!("OK  first_claim_reclaimed");

    let second_claim = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-02",
        "2026-04-21T22:11:00-04:00",
    )?;
    assert_eq!(
        required_string(&second_claim, "disposition")?,
        "claimed",
        "second claim should succeed"
    );
    assert_eq!(
        required_u64(&second_claim, "claimAttempt")?,
        2,
        "second claim should be attempt 2"
    );
    println!("OK  second_claimed");

    let stale_err = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &first_claim,
        "2026-04-21T22:12:00-04:00",
        "proposal_generated",
    )
    .expect_err("stale first-claim receipt should fail after re-claim");
    assert!(
        stale_err.contains("claim receipt claimAttempt does not match active claim"),
        "unexpected stale completion error: {}",
        stale_err
    );
    println!("OK  stale_completion_rejected");

    let receipt = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &second_claim,
        "2026-04-21T22:15:00-04:00",
        "proposal_generated",
    )?;
    assert_eq!(
        required_string(&receipt, "disposition")?,
        "completed",
        "second completion should succeed"
    );
    assert_eq!(
        required_u64(&receipt, "claimAttempt")?,
        2,
        "completion receipt should bind to attempt 2"
    );
    println!("OK  second_claim_completed");

    let item_path = first_json_file(&queue_dir.join("items"), "index.json")?;
    let item = read_json(&item_path)?;
    let status = item
        .get("processingState")
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .ok_or_else(|| "queue item missing processingState.status".to_string())?;
    assert_eq!(status, "completed", "queue item should be marked completed");
    let item_attempt = item
        .get("processingState")
        .and_then(|value| value.get("claimAttempt"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "queue item missing processingState.claimAttempt".to_string())?;
    assert_eq!(item_attempt, 2, "queue item should carry active claim attempt 2");
    println!("OK  queue_item_marked_completed_attempt_2");

    let claim_path = first_json_file(&queue_dir.join("claims"), "index.json")?;
    let claim = read_json(&claim_path)?;
    let claim_attempt = claim
        .get("claimAttempt")
        .and_then(Value::as_u64)
        .ok_or_else(|| "claim missing claimAttempt".to_string())?;
    assert_eq!(claim_attempt, 2, "claim file should be on attempt 2");

    let history_len = claim
        .get("claimHistory")
        .and_then(Value::as_array)
        .map(|value| value.len())
        .ok_or_else(|| "claim missing claimHistory array".to_string())?;
    assert_eq!(history_len, 1, "claim history should preserve first reclaimed attempt");

    let completion_attempt = claim
        .get("completion")
        .and_then(|value| value.get("claimAttempt"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "claim completion missing claimAttempt".to_string())?;
    assert_eq!(completion_attempt, 2, "claim completion should bind to attempt 2");
    println!("OK  claim_history_preserved_completion_bound");

    let completions_index = read_json(&queue_dir.join("completions/index.json"))?;
    let completion_total = completions_index
        .get("totals")
        .and_then(|value| value.get("completions"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "completion index missing totals.completions".to_string())?;
    assert_eq!(completion_total, 1, "expected exactly one completion");
    let indexed_attempt = completions_index
        .get("items")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("claimAttempt"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "completion index item missing claimAttempt".to_string())?;
    assert_eq!(indexed_attempt, 2, "completion index should record attempt 2");
    println!("OK  completion_index_written_attempt_2");

    let duplicate = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &second_claim,
        "2026-04-21T22:16:00-04:00",
        "proposal_generated",
    )?;
    assert_eq!(
        required_string(&duplicate, "disposition")?,
        "already_completed",
        "duplicate completion for same active attempt should be idempotent"
    );
    assert_eq!(
        required_u64(&duplicate, "claimAttempt")?,
        2,
        "duplicate completion receipt should still bind to attempt 2"
    );
    println!("OK  completion_idempotent_for_attempt_2");

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

fn required_u64(value: &Value, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing u64 field: {}", key))
}
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../centipede_queue_writer.rs"]
mod centipede_queue_writer;
#[path = "../centipede_queue_claim.rs"]
mod centipede_queue_claim;
#[path = "../centipede_queue_fail.rs"]
mod centipede_queue_fail;
#[path = "../centipede_queue_heartbeat.rs"]
mod centipede_queue_heartbeat;
#[path = "../centipede_queue_complete.rs"]
mod centipede_queue_complete;
#[path = "../centipede_queue_reclaim.rs"]
mod centipede_queue_reclaim;

use serde_json::{json, Value};

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!("Centipede queue fail smoke");

    let root = PathBuf::from("/tmp/centipede-queue-fail-smoke");
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

    let claim = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-01",
        "2026-04-21T23:00:00-04:00",
    )?;
    assert_eq!(required_string(&claim, "disposition")?, "claimed");
    assert_eq!(required_u64(&claim, "claimAttempt")?, 1);
    println!("OK  first_claimed");

    let fail_receipt = centipede_queue_fail::fail_from_claim_receipt(
        &queue_dir,
        &claim,
        "2026-04-21T23:01:00-04:00",
        "parser_crash",
    )?;
    assert_eq!(required_string(&fail_receipt, "disposition")?, "failed");
    assert_eq!(required_u64(&fail_receipt, "claimAttempt")?, 1);
    println!("OK  claim_failed");

    let item_path = first_json_file(&queue_dir.join("items"), "index.json")?;
    let item = read_json(&item_path)?;
    let item_status = item
        .get("processingState")
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .ok_or_else(|| "queue item missing processingState.status".to_string())?;
    assert_eq!(item_status, "failed");
    println!("OK  queue_item_marked_failed");

    let failures_index = read_json(&queue_dir.join("failures").join("index.json"))?;
    let failure_total = failures_index
        .get("totals")
        .and_then(|value| value.get("failures"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "failure index missing totals.failures".to_string())?;
    assert_eq!(failure_total, 1);
    println!("OK  failure_index_written");

    let duplicate_fail = centipede_queue_fail::fail_from_claim_receipt(
        &queue_dir,
        &claim,
        "2026-04-21T23:02:00-04:00",
        "parser_crash",
    )?;
    assert_eq!(required_string(&duplicate_fail, "disposition")?, "already_failed");
    println!("OK  duplicate_fail_idempotent");

    let heartbeat_after_fail = centipede_queue_heartbeat::heartbeat_claim_from_receipt(
        &queue_dir,
        &claim,
        "2026-04-21T23:03:00-04:00",
        300,
    )
    .expect_err("heartbeat after fail should be rejected");
    assert!(
        heartbeat_after_fail.contains("claim is already failed"),
        "unexpected heartbeat after fail error: {}",
        heartbeat_after_fail
    );
    println!("OK  heartbeat_after_fail_rejected");

    let complete_after_fail = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &claim,
        "2026-04-21T23:04:00-04:00",
        "proposal_generated",
    )
    .expect_err("complete after fail should be rejected");
    assert!(
        complete_after_fail.contains("claim is already failed"),
        "unexpected complete after fail error: {}",
        complete_after_fail
    );
    println!("OK  complete_after_fail_rejected");

    let reclaim_after_fail = centipede_queue_reclaim::reclaim_expired_claims(
        &queue_dir,
        "centipede-reclaimer-01",
        "2026-04-21T23:10:00-04:00",
        300,
    )?;
    assert_eq!(
        required_string(&reclaim_after_fail, "disposition")?,
        "no_expired_claims"
    );
    println!("OK  reclaim_after_fail_noop");

    let claim_again = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-02",
        "2026-04-21T23:11:00-04:00",
    )?;
    assert_eq!(required_string(&claim_again, "disposition")?, "empty");
    println!("OK  failed_item_not_reclaimable");

    println!("Validated Centipede queue fail smoke successfully.");
    Ok(())
}

fn sample_success_queue() -> Value {
    json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "success_path",
        "sourceLane": "worm",
        "sourceRepo": "Boswecw/Cortex",
        "sourceHandoffId": "handoff-bundle-run-repo-surface-09",
        "receivedAt": "2026-04-21T22:58:00-04:00",
        "candidateIssues": [
            {
                "issueKey": "worm::parse_tree_corruption",
                "title": "Parse tree corruption",
                "severity": "high"
            }
        ],
        "totals": {
            "blocking": 1,
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
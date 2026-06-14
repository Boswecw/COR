use std::fs;
use std::path::{Path, PathBuf};

#[path = "../centipede_queue_writer.rs"]
mod centipede_queue_writer;
#[path = "../centipede_queue_claim.rs"]
mod centipede_queue_claim;
#[path = "../centipede_queue_reclaim.rs"]
mod centipede_queue_reclaim;
#[path = "../centipede_queue_heartbeat.rs"]
mod centipede_queue_heartbeat;
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
    println!("Centipede queue heartbeat smoke");

    let root = PathBuf::from("/tmp/centipede-queue-heartbeat-smoke");
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
    assert_eq!(required_string(&first_claim, "disposition")?, "claimed");
    assert_eq!(required_u64(&first_claim, "claimAttempt")?, 1);
    println!("OK  first_claimed");

    let heartbeat_one = centipede_queue_heartbeat::heartbeat_claim_from_receipt(
        &queue_dir,
        &first_claim,
        "2026-04-21T22:02:00-04:00",
        300,
    )?;
    assert_eq!(
        required_string(&heartbeat_one, "disposition")?,
        "heartbeat_recorded"
    );
    assert_eq!(required_u64(&heartbeat_one, "heartbeatCount")?, 1);
    println!("OK  heartbeat_recorded_attempt_1");

    let early_reclaim = centipede_queue_reclaim::reclaim_expired_claims(
        &queue_dir,
        "centipede-reclaimer-01",
        "2026-04-21T22:05:00-04:00",
        300,
    )?;
    assert_eq!(
        required_string(&early_reclaim, "disposition")?,
        "no_expired_claims"
    );
    println!("OK  reclaim_before_expiry_noop");

    let late_reclaim = centipede_queue_reclaim::reclaim_expired_claims(
        &queue_dir,
        "centipede-reclaimer-01",
        "2026-04-21T22:08:00-04:00",
        300,
    )?;
    assert_eq!(required_string(&late_reclaim, "disposition")?, "reclaimed");
    let reclaimed_total = late_reclaim
        .get("totals")
        .and_then(|value| value.get("reclaimed"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "late reclaim missing totals.reclaimed".to_string())?;
    assert_eq!(reclaimed_total, 1);
    println!("OK  reclaim_after_expiry");

    let second_claim = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-02",
        "2026-04-21T22:09:00-04:00",
    )?;
    assert_eq!(required_string(&second_claim, "disposition")?, "claimed");
    assert_eq!(required_u64(&second_claim, "claimAttempt")?, 2);
    println!("OK  second_claimed");

    let stale_heartbeat = centipede_queue_heartbeat::heartbeat_claim_from_receipt(
        &queue_dir,
        &first_claim,
        "2026-04-21T22:10:00-04:00",
        300,
    )
    .expect_err("stale heartbeat should fail after re-claim");
    assert!(
        stale_heartbeat.contains("claim receipt claimAttempt does not match active claim"),
        "unexpected stale heartbeat error: {}",
        stale_heartbeat
    );
    println!("OK  stale_heartbeat_rejected");

    let heartbeat_two = centipede_queue_heartbeat::heartbeat_claim_from_receipt(
        &queue_dir,
        &second_claim,
        "2026-04-21T22:10:00-04:00",
        300,
    )?;
    assert_eq!(
        required_string(&heartbeat_two, "disposition")?,
        "heartbeat_recorded"
    );
    assert_eq!(required_u64(&heartbeat_two, "heartbeatCount")?, 1);
    println!("OK  heartbeat_recorded_attempt_2");

    let claim_path = first_json_file(&queue_dir.join("claims"), "index.json")?;
    let active_claim = read_json(&claim_path)?;
    let active_heartbeat_count = active_claim
        .get("lease")
        .and_then(|lease| lease.get("heartbeatCount"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "claim missing lease.heartbeatCount".to_string())?;
    assert_eq!(active_heartbeat_count, 1);
    println!("OK  lease_written_to_claim");

    let complete_receipt = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &second_claim,
        "2026-04-21T22:11:00-04:00",
        "proposal_generated",
    )?;
    assert_eq!(
        required_string(&complete_receipt, "disposition")?,
        "completed"
    );
    println!("OK  attempt_2_completed");

    let post_complete_heartbeat = centipede_queue_heartbeat::heartbeat_claim_from_receipt(
        &queue_dir,
        &second_claim,
        "2026-04-21T22:12:00-04:00",
        300,
    )
    .expect_err("heartbeat after completion should fail");
    assert!(
        post_complete_heartbeat.contains("claim is already completed"),
        "unexpected post-complete heartbeat error: {}",
        post_complete_heartbeat
    );
    println!("OK  completed_claim_heartbeat_rejected");

    println!("Validated Centipede queue heartbeat smoke successfully.");
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
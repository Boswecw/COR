use std::fs;
use std::path::{Path, PathBuf};

#[path = "../centipede_queue_claim.rs"]
mod centipede_queue_claim;
#[path = "../centipede_queue_complete.rs"]
mod centipede_queue_complete;
#[path = "../centipede_queue_fail.rs"]
mod centipede_queue_fail;
#[path = "../centipede_queue_heartbeat.rs"]
mod centipede_queue_heartbeat;
#[path = "../centipede_queue_reclaim.rs"]
mod centipede_queue_reclaim;
#[path = "../centipede_queue_report.rs"]
mod centipede_queue_report;

use serde_json::{json, Value};

fn main() {
    if let Err(err) = run() {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!("Centipede queue report smoke");

    let root = PathBuf::from("/tmp/centipede-queue-report-smoke");
    if root.exists() {
        fs::remove_dir_all(&root)
            .map_err(|err| format!("could not clear {}: {}", root.display(), err))?;
    }
    fs::create_dir_all(root.join("queue").join("items"))
        .map_err(|err| format!("could not create queue items dir: {}", err))?;

    let queue_dir = root.join("queue");

    write_json(
        &queue_dir.join("items").join("item-a.json"),
        &sample_queue_item(
            "cqitem-alpha",
            "Boswecw/Cortex",
            "handoff-a",
            "success_path",
            "worm::alpha_issue",
            0,
        ),
    )?;
    write_json(
        &queue_dir.join("items").join("item-b.json"),
        &sample_queue_item(
            "cqitem-beta",
            "Boswecw/Cortex",
            "handoff-b",
            "failure_path",
            "worm::beta_issue",
            1,
        ),
    )?;

    let first_claim = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-01",
        "2026-04-21T23:20:00-04:00",
    )?;
    let first_item_id = required_string(&first_claim, "queueItemId")?;
    assert_eq!(first_item_id, "cqitem-alpha");
    println!("OK  first_item_claimed");

    let heartbeat = centipede_queue_heartbeat::heartbeat_claim_from_receipt(
        &queue_dir,
        &first_claim,
        "2026-04-21T23:21:00-04:00",
        300,
    )?;
    assert_eq!(required_string(&heartbeat, "disposition")?, "heartbeat_recorded");
    println!("OK  first_item_heartbeat_recorded");

    let reclaim = centipede_queue_reclaim::reclaim_expired_claims(
        &queue_dir,
        "centipede-reclaimer-01",
        "2026-04-21T23:30:00-04:00",
        300,
    )?;
    assert_eq!(required_string(&reclaim, "disposition")?, "reclaimed");
    println!("OK  first_item_reclaimed");

    let second_claim = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-02",
        "2026-04-21T23:31:00-04:00",
    )?;
    assert_eq!(required_string(&second_claim, "queueItemId")?, "cqitem-alpha");
    assert_eq!(required_u64(&second_claim, "claimAttempt")?, 2);
    println!("OK  first_item_reclaimed_then_reclaimed_again");

    let completion = centipede_queue_complete::complete_from_claim_receipt(
        &queue_dir,
        &second_claim,
        "2026-04-21T23:32:00-04:00",
        "proposal_generated",
    )?;
    assert_eq!(required_string(&completion, "disposition")?, "completed");
    println!("OK  first_item_completed");

    let third_claim = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-03",
        "2026-04-21T23:33:00-04:00",
    )?;
    assert_eq!(required_string(&third_claim, "queueItemId")?, "cqitem-beta");
    println!("OK  second_item_claimed");

    let failure = centipede_queue_fail::fail_from_claim_receipt(
        &queue_dir,
        &third_claim,
        "2026-04-21T23:34:00-04:00",
        "parser_crash",
    )?;
    assert_eq!(required_string(&failure, "disposition")?, "failed");
    println!("OK  second_item_failed");

    let report = centipede_queue_report::build_queue_report(&queue_dir, None)?;
    assert_eq!(required_u64(report.get("totals").ok_or("missing totals")?, "items")?, 2);
    assert_eq!(required_u64(report.get("totals").ok_or("missing totals")?, "claimRecords")?, 2);
    assert_eq!(required_u64(report.get("totals").ok_or("missing totals")?, "claimEpisodes")?, 3);
    assert_eq!(required_u64(report.get("totals").ok_or("missing totals")?, "reclaimedHistoryEpisodes")?, 1);
    assert_eq!(required_u64(report.get("totals").ok_or("missing totals")?, "completions")?, 1);
    assert_eq!(required_u64(report.get("totals").ok_or("missing totals")?, "failures")?, 1);

    let processing = report
        .get("totals")
        .and_then(|value| value.get("processing"))
        .ok_or_else(|| "missing totals.processing".to_string())?;
    assert_eq!(required_u64(processing, "completed")?, 1);
    assert_eq!(required_u64(processing, "failed")?, 1);
    println!("OK  queue_totals_verified");

    let claim_states = report
        .get("totals")
        .and_then(|value| value.get("claimRecordStates"))
        .ok_or_else(|| "missing totals.claimRecordStates".to_string())?;
    assert_eq!(required_u64(claim_states, "completed")?, 1);
    assert_eq!(required_u64(claim_states, "failed")?, 1);
    println!("OK  claim_state_totals_verified");

    let items = report
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| "report items must be an array".to_string())?;
    assert_eq!(items.len(), 2);

    let alpha = items
        .iter()
        .find(|item| item.get("queueItemId").and_then(Value::as_str) == Some("cqitem-alpha"))
        .ok_or_else(|| "missing alpha item".to_string())?;
    let alpha_claim_record = alpha
        .get("claimRecord")
        .ok_or_else(|| "alpha missing claimRecord".to_string())?;
    let alpha_episodes = alpha_claim_record
        .get("episodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "alpha episodes must be an array".to_string())?;
    assert_eq!(alpha_episodes.len(), 2);
    assert_eq!(
        alpha_episodes[0].get("state").and_then(Value::as_str),
        Some("reclaimed")
    );
    assert_eq!(
        alpha_episodes[1].get("state").and_then(Value::as_str),
        Some("completed")
    );
    assert_eq!(
        alpha.get("completionEvidence")
            .and_then(Value::as_array)
            .map(|value| value.len())
            .unwrap_or(0),
        1
    );
    println!("OK  completed_item_episode_chain_visible");

    let beta = items
        .iter()
        .find(|item| item.get("queueItemId").and_then(Value::as_str) == Some("cqitem-beta"))
        .ok_or_else(|| "missing beta item".to_string())?;
    let beta_claim_record = beta
        .get("claimRecord")
        .ok_or_else(|| "beta missing claimRecord".to_string())?;
    let beta_episodes = beta_claim_record
        .get("episodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "beta episodes must be an array".to_string())?;
    assert_eq!(beta_episodes.len(), 1);
    assert_eq!(
        beta_episodes[0].get("state").and_then(Value::as_str),
        Some("failed")
    );
    assert_eq!(
        beta.get("failureEvidence")
            .and_then(Value::as_array)
            .map(|value| value.len())
            .unwrap_or(0),
        1
    );
    println!("OK  failed_item_evidence_visible");

    let alpha_only = centipede_queue_report::build_queue_report(&queue_dir, Some("cqitem-alpha"))?;
    let alpha_only_items = alpha_only
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| "alpha-only items must be an array".to_string())?;
    assert_eq!(alpha_only_items.len(), 1);
    assert_eq!(
        alpha_only_items[0].get("queueItemId").and_then(Value::as_str),
        Some("cqitem-alpha")
    );
    println!("OK  single_item_filter_works");

    println!("Validated Centipede queue report smoke successfully.");
    Ok(())
}

fn sample_queue_item(
    queue_item_id: &str,
    source_repo: &str,
    source_handoff_id: &str,
    intake_kind: &str,
    issue_key: &str,
    blocking: u64,
) -> Value {
    json!({
        "kind": "centipede_queue_item",
        "schemaVersion": 1,
        "queueItemId": queue_item_id,
        "intakeKind": intake_kind,
        "sourceLane": "worm",
        "sourceRepo": source_repo,
        "sourceHandoffId": source_handoff_id,
        "receivedAt": "2026-04-21T23:19:00-04:00",
        "candidateIssues": [
            {
                "issueKey": issue_key,
                "title": "Synthetic issue",
                "severity": if blocking > 0 { "high" } else { "moderate" }
            }
        ],
        "totals": {
            "blocking": blocking,
            "candidateIssues": 1
        }
    })
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

use std::fs;
use std::path::PathBuf;

#[path = "../centipede_queue_writer.rs"]
mod centipede_queue_writer;
#[path = "../centipede_queue_claim.rs"]
mod centipede_queue_claim;

use serde_json::{json, Value};

fn main() {
    println!("Centipede queue claim smoke");

    let root = PathBuf::from("/tmp/centipede-queue-claim-smoke");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create smoke root");

    let queue_dir = root.join("queue");

    let success_queue = json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "success_path",
        "sourceLane": "worm",
        "sourceRepo": "Boswecw/Cortex",
        "sourceHandoffId": "handoff-bundle-run-repo-surface-07",
        "candidateIssues": [
            {
                "issueKey": "Boswecw/Cortex::stale_submodule_pointer::edge-2",
                "blocking": false
            }
        ],
        "evidenceArtifacts": ["bundle-run-repo-surface-07"],
        "totals": {
            "candidateIssues": 1,
            "blocking": 0
        },
        "receivedAt": "2026-04-21T22:00:00-04:00"
    });

    let failure_queue = json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "failure_path",
        "sourceLane": "worm",
        "sourceRepo": "Boswecw/Cortex",
        "sourceHandoffId": "worm-failure-nested_requirements_repo_root_escape",
        "failureKind": "nested_requirements_repo_root_escape",
        "severity": "critical",
        "recommendedRoute": "containment_operator_review",
        "candidateIssues": [
            {
                "issueKey": "worm.surface.failure.nested_requirements_repo_root_escape",
                "blocking": true
            }
        ],
        "evidenceArtifacts": ["surface_failure.json"],
        "totals": {
            "candidateIssues": 1,
            "blocking": 1
        },
        "receivedAt": "2026-04-21T22:01:00-04:00"
    });

    centipede_queue_writer::enqueue_queue_value(&success_queue, &queue_dir)
        .expect("enqueue success queue");
    centipede_queue_writer::enqueue_queue_value(&failure_queue, &queue_dir)
        .expect("enqueue failure queue");

    let receipt_one = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-01",
        "2026-04-21T22:10:00-04:00",
    )
    .expect("claim queue item one");

    let receipt_two = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-01",
        "2026-04-21T22:11:00-04:00",
    )
    .expect("claim queue item two");

    let receipt_three = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-01",
        "2026-04-21T22:12:00-04:00",
    )
    .expect("claim queue item three");

    let claims_index: Value = serde_json::from_str(
        &fs::read_to_string(queue_dir.join("claims").join("index.json")).expect("read claims index"),
    )
    .expect("parse claims index");

    assert_eq!(string_field(&receipt_one, "disposition"), "claimed");
    assert_eq!(string_field(&receipt_two, "disposition"), "claimed");
    assert_eq!(string_field(&receipt_three, "disposition"), "empty");
    assert_ne!(string_field(&receipt_one, "queueItemId"), string_field(&receipt_two, "queueItemId"));
    assert_eq!(count_array_field(&claims_index, "items"), 2);
    assert_eq!(number_field(&claims_index["totals"], "claims"), 2);
    assert_eq!(number_field(&claims_index["totals"], "blocking"), 1);

    println!("OK  first_queue_item_claimed");
    println!("OK  second_queue_item_claimed");
    println!("OK  empty_queue_detected_after_claims");
    println!("Validated Centipede queue claim smoke successfully.");
}

fn string_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string field: {}", key))
}

fn count_array_field(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_else(|| panic!("missing array field: {}", key))
}

fn number_field(value: &Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("missing number field: {}", key))
}

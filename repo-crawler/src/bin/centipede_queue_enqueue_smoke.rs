use std::fs;
use std::path::PathBuf;

#[path = "../centipede_queue_writer.rs"]
mod centipede_queue_writer;

use serde_json::{json, Value};

fn main() {
    println!("Centipede queue enqueue smoke");

    let root = PathBuf::from("/tmp/centipede-queue-enqueue-smoke");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create smoke root");

    let queue_dir = root.join("queue");

    let success_queue = json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "success_path",
        "sourceLane": "worm",
        "sourceRepo": "Boswecw/Cortex",
        "sourceHandoffId": "handoff-bundle-run-repo-surface-05",
        "candidateIssues": [
            {
                "issueKey": "Boswecw/Cortex::stale_submodule_pointer::edge-2",
                "blocking": false,
                "findingClass": "stale_submodule_pointer",
                "proposedWeightClass": "supporting",
                "confidence": "medium"
            }
        ],
        "evidenceArtifacts": ["bundle-run-repo-surface-05"],
        "totals": {
            "candidateIssues": 1,
            "blocking": 0
        },
        "receivedAt": "2026-04-21T21:00:00-04:00"
    });

    let failure_queue = json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "failure_path",
        "sourceLane": "worm",
        "sourceRepo": "Boswecw/Cortex",
        "sourceHandoffId": "worm-failure-symlink_containment_escape",
        "failureKind": "symlink_containment_escape",
        "severity": "critical",
        "recommendedRoute": "containment_operator_review",
        "candidateIssues": [
            {
                "issueKey": "worm.surface.failure.symlink_containment_escape",
                "blocking": true
            }
        ],
        "evidenceArtifacts": ["surface_failure.json"],
        "totals": {
            "candidateIssues": 1,
            "blocking": 1
        },
        "receivedAt": "2026-04-21T21:01:00-04:00"
    });

    let receipt_one = centipede_queue_writer::enqueue_queue_value(&success_queue, &queue_dir)
        .expect("enqueue success queue");
    let receipt_two = centipede_queue_writer::enqueue_queue_value(&failure_queue, &queue_dir)
        .expect("enqueue failure queue");
    let receipt_three = centipede_queue_writer::enqueue_queue_value(&success_queue, &queue_dir)
        .expect("re-enqueue success queue");

    let index: Value = serde_json::from_str(
        &fs::read_to_string(queue_dir.join("index.json")).expect("read queue index"),
    )
    .expect("parse queue index");

    assert_eq!(string_field(&receipt_one, "disposition"), "enqueued");
    assert_eq!(string_field(&receipt_two, "disposition"), "enqueued");
    assert_eq!(string_field(&receipt_three, "disposition"), "already_present");
    assert_eq!(count_array_field(&index, "items"), 2);
    assert_eq!(number_field(&index["totals"], "items"), 2);
    assert_eq!(number_field(&index["totals"], "candidateIssues"), 2);
    assert_eq!(number_field(&index["totals"], "blocking"), 1);

    println!("OK  success_queue_enqueued");
    println!("OK  failure_queue_enqueued");
    println!("OK  duplicate_queue_deduped");
    println!("Validated Centipede queue enqueue smoke successfully.");
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

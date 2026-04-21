use std::fs;
use std::path::PathBuf;

#[path = "../centipede_intake_normalizer.rs"]
mod centipede_intake_normalizer;

use centipede_intake_normalizer::normalize_handoff;
use serde_json::{json, Value};

fn main() {
    println!("Worm Centipede intake consumer smoke");

    let root = PathBuf::from("/tmp/worm-centipede-intake-consumer-smoke");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create smoke dir");

    let success_input = json!({
        "kind": "worm_centipede_handoff",
        "schemaVersion": 1,
        "handoffId": "handoff-bundle-success-02",
        "sourceLane": "worm",
        "sourceRepo": "Boswecw/Cortex",
        "runId": "worm-smoke-run",
        "bundleIds": ["bundle-run-repo-surface-05"],
        "candidateIssueKeys": [
            {
                "issueKey": "Boswecw/Cortex::ambiguous_target_identity::edge-1",
                "findingClass": "ambiguous_target_identity",
                "proposedWeightClass": "supporting",
                "confidence": "medium"
            },
            {
                "issueKey": "Boswecw/Cortex::stale_submodule_pointer::edge-2",
                "findingClass": "stale_submodule_pointer",
                "proposedWeightClass": "blocking",
                "confidence": "high"
            }
        ],
        "timestamp": "2026-04-21T20:30:00-04:00"
    });

    let failure_input = json!({
        "kind": "worm_centipede_failure_handoff",
        "schemaVersion": 1,
        "sourceRepo": "Boswecw/Cortex",
        "repoRoot": "/tmp/example-root",
        "failureKind": "symlink_containment_escape",
        "candidateIssueKeys": [
            "worm.surface.failure.symlink_containment_escape",
            "centipede.intake.fail_closed"
        ],
        "severity": "critical",
        "blocking": true,
        "recommendedRoute": "containment_operator_review",
        "evidenceArtifacts": ["surface_failure.json"],
        "posture": "fail_closed",
        "timestamp": "2026-04-21T20:31:00-04:00"
    });

    let success_output = normalize_handoff(&success_input).expect("normalize success handoff");
    let failure_output = normalize_handoff(&failure_input).expect("normalize failure handoff");

    write_json(root.join("success_queue.json"), &success_output);
    write_json(root.join("failure_queue.json"), &failure_output);

    assert_eq!(string_field(&success_output, "kind"), "centipede_candidate_issue_queue");
    assert_eq!(string_field(&success_output, "intakeKind"), "success_path");
    assert_eq!(count_array_field(&success_output, "candidateIssues"), 2);
    assert_eq!(number_field(&success_output["totals"], "blocking"), 1);
    assert_eq!(count_array_field(&success_output, "evidenceArtifacts"), 1);
    assert_eq!(
        string_field(&success_output["candidateIssues"][0], "findingClass"),
        "ambiguous_target_identity"
    );
    assert_eq!(
        string_field(&success_output["candidateIssues"][1], "proposedWeightClass"),
        "blocking"
    );

    assert_eq!(string_field(&failure_output, "kind"), "centipede_candidate_issue_queue");
    assert_eq!(string_field(&failure_output, "intakeKind"), "failure_path");
    assert_eq!(string_field(&failure_output, "failureKind"), "symlink_containment_escape");
    assert_eq!(string_field(&failure_output, "sourceHandoffId"), "worm-failure-symlink_containment_escape");
    assert_eq!(count_array_field(&failure_output, "candidateIssues"), 2);
    assert_eq!(number_field(&failure_output["totals"], "blocking"), 2);

    println!("OK  success_handoff_object_candidates_normalized");
    println!("OK  legacy_failure_handoff_normalized");
    println!("Validated Worm Centipede intake consumer smoke successfully.");
}

fn write_json(path: PathBuf, value: &Value) {
    let pretty = serde_json::to_string_pretty(value).expect("serialize JSON");
    fs::write(path, format!("{}\n", pretty)).expect("write JSON");
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

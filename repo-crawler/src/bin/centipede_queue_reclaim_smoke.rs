use std::fs;
use std::path::{Path, PathBuf};

#[path = "../centipede_queue_writer.rs"]
mod centipede_queue_writer;
#[path = "../centipede_queue_claim.rs"]
mod centipede_queue_claim;
#[path = "../centipede_queue_reclaim.rs"]
mod centipede_queue_reclaim;

use serde_json::{json, Value};

fn main() {
    println!("Centipede queue reclaim smoke");

    let root = PathBuf::from("/tmp/centipede-queue-reclaim-smoke");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create smoke root");

    let queue_dir = root.join("queue");

    let queue_input = json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "success_path",
        "sourceLane": "worm",
        "sourceRepo": "Boswecw/Cortex",
        "sourceHandoffId": "handoff-bundle-run-repo-surface-08",
        "candidateIssues": [
            {
                "issueKey": "Boswecw/Cortex::stale_submodule_pointer::edge-9",
                "blocking": false
            }
        ],
        "evidenceArtifacts": ["bundle-run-repo-surface-08"],
        "totals": {
            "candidateIssues": 1,
            "blocking": 0
        },
        "receivedAt": "2026-04-21T22:00:00-04:00"
    });

    centipede_queue_writer::enqueue_queue_value(&queue_input, &queue_dir)
        .expect("enqueue queue input");

    let claim_receipt = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-01",
        "2026-04-21T22:10:00-04:00",
    )
    .expect("claim queue item");

    let claim_id = string_field(&claim_receipt, "claimId");
    let claim_path = queue_dir.join("claims").join(format!("{}.json", claim_id));
    let claim_value = read_json(&claim_path);
    let item_path = PathBuf::from(string_field(&claim_value, "queueItemPath"));
    let claimed_item = read_json(&item_path);

    assert_eq!(string_field(&claim_receipt, "disposition"), "claimed");
    assert_eq!(string_field(&claimed_item["processingState"], "status"), "claimed");

    let reclaim_receipt = centipede_queue_reclaim::reclaim_expired_claims(
        &queue_dir,
        "centipede-reclaimer-01",
        "2026-04-21T22:20:00-04:00",
        300,
    )
    .expect("reclaim expired claim");

    let reclaimed_claim = read_json(&claim_path);
    let reclaimed_item = read_json(&item_path);

    assert_eq!(string_field(&reclaim_receipt, "disposition"), "reclaimed");
    assert_eq!(number_field(&reclaim_receipt["totals"], "reclaimed"), 1);
    assert_eq!(string_field(&reclaimed_claim["reclaim"], "reason"), "lease_expired");
    assert_eq!(string_field(&reclaimed_item["processingState"], "status"), "queued");
    assert_eq!(string_field(&reclaimed_item["processingState"], "reclaimer"), "centipede-reclaimer-01");

    let reclaim_again_claim_receipt = centipede_queue_claim::claim_next_queue_item(
        &queue_dir,
        "centipede-worker-02",
        "2026-04-21T22:21:00-04:00",
    )
    .expect("claim reclaimed queue item");

    let recycled_claim = read_json(&claim_path);
    assert_eq!(string_field(&reclaim_again_claim_receipt, "disposition"), "claimed");
    assert_eq!(number_field(&recycled_claim, "claimAttempt"), 2);
    assert_eq!(count_array_field(&recycled_claim, "claimHistory"), 1);

    let no_expired_receipt = centipede_queue_reclaim::reclaim_expired_claims(
        &queue_dir,
        "centipede-reclaimer-01",
        "2026-04-21T22:21:30-04:00",
        300,
    )
    .expect("run reclaim with no expired claims");

    assert_eq!(string_field(&no_expired_receipt, "disposition"), "no_expired_claims");
    assert_eq!(number_field(&no_expired_receipt["totals"], "reclaimed"), 0);

    let claims_index = read_json(&queue_dir.join("claims").join("index.json"));
    assert_eq!(number_field(&claims_index["totals"], "claims"), 1);
    assert_eq!(number_field(&claims_index["totals"], "active"), 1);
    assert_eq!(number_field(&claims_index["totals"], "reclaimed"), 0);

    println!("OK  claimed_item_records_processing_state");
    println!("OK  expired_claim_reclaimed_to_queue");
    println!("OK  reclaimed_item_can_be_claimed_again");
    println!("OK  no_expired_claims_detected_cleanly");
    println!("Validated Centipede queue reclaim smoke successfully.");
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap_or_else(|err| {
        panic!("could not read {}: {}", path.display(), err)
    }))
    .unwrap_or_else(|err| panic!("could not parse {}: {}", path.display(), err))
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

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::model::{
    now_ts, ClaimObservationInput, EvidenceKind, EvidenceRecordInput, FreshnessStatus, HealthState,
    Polarity, Subsystem,
};

const REQUIRED_DOCS: &[&str] = &[
    "PROJECT_CHARTER.md",
    "LOCAL_DOCTRINE.md",
    "AUTHORITY_BOUNDARIES.md",
    "README.md",
    "SYSTEM.md",
    "doc/system/BUILD.sh",
    "doc/cxSYSTEM.md",
];

pub fn observe_governance(root: &Path) -> Vec<ClaimObservationInput> {
    let repository_id = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .to_string_lossy()
        .to_string();
    let observed_at = now_ts();
    let mut observations = Vec::new();

    for required in REQUIRED_DOCS {
        let path = root.join(required);
        let exists = path.exists();
        observations.push(ClaimObservationInput {
            repository_id: repository_id.clone(),
            revision_id: None,
            normalized_path: Some(required.to_string()),
            artifact_id: None,
            file_hash: None,
            language_id: Some(
                if required.ends_with(".sh") {
                    "sh"
                } else {
                    "md"
                }
                .to_string(),
            ),
            source_scope: "governance_docs".to_string(),
            claim_target_id: format!("file:{required}"),
            claim_class: "doc_presence".to_string(),
            subsystem: Subsystem::GovernanceDocParity,
            polarity: if exists {
                Polarity::Supports
            } else {
                Polarity::Refutes
            },
            observed_at,
            freshness_reference: None,
            freshness_status: FreshnessStatus::Fresh,
            evidence_strength: Some(0.93),
            subsystem_health: HealthState::Ready,
            value: serde_json::json!({
                "required": true,
                "exists": exists
            }),
            evidence: vec![EvidenceRecordInput {
                evidence_kind: EvidenceKind::DirectPath,
                reference: required.to_string(),
                strength_score: 0.93,
                payload: serde_json::json!({ "observer": "governance_doc_presence_v1" }),
            }],
            diagnostics: Vec::new(),
        });
    }

    observations.push(doc_parity_observation(root, &repository_id, observed_at));
    observations
}

fn doc_parity_observation(
    root: &Path,
    repository_id: &str,
    observed_at: i64,
) -> ClaimObservationInput {
    let source_dir = root.join("doc/system");
    let assembled = root.join("doc/cxSYSTEM.md");
    let source_latest = latest_source_mtime(&source_dir).unwrap_or(0);
    let assembled_mtime = mtime(&assembled).unwrap_or(0);
    let parity_known = source_latest > 0 && assembled_mtime > 0;
    let parity_ready = parity_known && assembled_mtime >= source_latest;

    ClaimObservationInput {
        repository_id: repository_id.to_string(),
        revision_id: None,
        normalized_path: Some("doc/cxSYSTEM.md".to_string()),
        artifact_id: Some("doc-system-assembly".to_string()),
        file_hash: None,
        language_id: Some("md".to_string()),
        source_scope: "governance_docs".to_string(),
        claim_target_id: "doc_assembly:doc/cxSYSTEM.md".to_string(),
        claim_class: "doc_parity".to_string(),
        subsystem: Subsystem::GovernanceDocParity,
        polarity: if parity_ready {
            Polarity::Supports
        } else if parity_known {
            Polarity::Refutes
        } else {
            Polarity::Unavailable
        },
        observed_at,
        freshness_reference: Some(format!("source_latest_mtime:{source_latest}")),
        freshness_status: if parity_ready {
            FreshnessStatus::Fresh
        } else if parity_known {
            FreshnessStatus::Stale
        } else {
            FreshnessStatus::Unknown
        },
        evidence_strength: Some(if parity_known { 0.88 } else { 0.45 }),
        subsystem_health: if parity_known {
            HealthState::Ready
        } else {
            HealthState::Degraded
        },
        value: serde_json::json!({
            "source_latest_mtime": source_latest,
            "assembled_mtime": assembled_mtime,
            "parity_ready": parity_ready,
            "method": "mtime_parity"
        }),
        evidence: vec![EvidenceRecordInput {
            evidence_kind: EvidenceKind::RuleHit,
            reference: "doc/system -> doc/cxSYSTEM.md".to_string(),
            strength_score: if parity_known { 0.88 } else { 0.45 },
            payload: serde_json::json!({ "observer": "governance_doc_parity_v1" }),
        }],
        diagnostics: Vec::new(),
    }
}

fn latest_source_mtime(source_dir: &Path) -> Option<i64> {
    let mut latest = 0_i64;
    let entries = fs::read_dir(source_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        latest = latest.max(mtime(&path).unwrap_or(0));
    }
    if latest == 0 {
        None
    } else {
        Some(latest)
    }
}

fn mtime(path: &Path) -> Option<i64> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs() as i64)
}

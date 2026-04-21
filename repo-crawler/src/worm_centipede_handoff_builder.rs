
use serde_json::{json, Value};

fn weight_from_severity(severity: &str) -> &'static str {
    match severity {
        "high" => "strong",
        "medium" => "supporting",
        _ => "supporting",
    }
}

pub fn build_handoff(bundle: &Value) -> Result<Value, String> {
    let bundle_id = bundle
        .get("bundleId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "bundle missing bundleId".to_string())?;

    let source_repo = bundle
        .get("sourceRepo")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "bundle missing sourceRepo".to_string())?;

    let findings = bundle
        .get("findings")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "bundle missing findings".to_string())?;

    let mut candidate_issue_keys = Vec::new();

    for finding in findings {
        let reason_code = finding
            .get("reasonCode")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "finding missing reasonCode".to_string())?;

        let finding_class = finding
            .get("findingClass")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "finding missing findingClass".to_string())?;

        let edge_id = finding
            .get("edgeId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "finding missing edgeId".to_string())?;

        let severity = finding
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        let confidence = finding
            .get("confidence")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        candidate_issue_keys.push(json!({
            "issueKey": format!("{source_repo}::{reason_code}::{edge_id}"),
            "findingClass": finding_class,
            "proposedWeightClass": weight_from_severity(severity),
            "confidence": confidence
        }));
    }

    Ok(json!({
        "kind": "worm_centipede_handoff",
        "schemaVersion": 1,
        "handoffId": format!("handoff-{}", bundle_id),
        "sourceLane": "worm",
        "sourceRepo": source_repo,
        "runId": "worm-smoke-run",
        "bundleIds": [bundle_id],
        "candidateIssueKeys": candidate_issue_keys,
        "timestamp": "2026-04-22T02:18:00-04:00"
    }))
}

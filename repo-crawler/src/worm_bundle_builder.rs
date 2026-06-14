
use serde_json::{json, Value};

pub fn build_bundle(
    source_repo: &str,
    bundle_id: &str,
    emitted_edges: &[Value],
    resolutions: &[Value],
) -> Result<Value, String> {
    let mut findings = Vec::new();

    for resolution in resolutions {
        let edge_id = resolution
            .get("edgeId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "resolution missing edgeId".to_string())?;

        let posture = resolution
            .get("resolutionPosture")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("resolution {edge_id} missing resolutionPosture"))?;

        if posture == "ambiguous" {
            findings.push(json!({
                "findingId": format!("finding-{}", findings.len() + 1),
                "findingClass": "target_identity",
                "reasonCode": "ambiguous_target_identity",
                "edgeId": edge_id,
                "severity": "medium",
                "confidence": "medium",
                "summary": "Target reference could not be normalized into a canonical repo identity.",
                "evidenceRefs": [format!("resolution:{edge_id}")]
            }));
        }
    }

    Ok(json!({
        "kind": "worm_evidence_bundle",
        "schemaVersion": 1,
        "bundleId": bundle_id,
        "sourceRepo": source_repo,
        "edges": emitted_edges,
        "resolutions": resolutions,
        "findings": findings,
        "posture": "evidence_bound",
        "timestamp": "2026-04-22T02:05:00-04:00"
    }))
}

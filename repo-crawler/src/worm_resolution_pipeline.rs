
use serde_json::{json, Value};

use crate::worm_target_normalizer;

pub fn resolve_emitted_edges(emission: &Value) -> Result<Vec<Value>, String> {
    let edges = emission
        .get("emittedEdges")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "adapter emission missing emittedEdges".to_string())?;

    let mut out = Vec::new();

    for (idx, edge) in edges.iter().enumerate() {
        let label = format!("emittedEdges[{idx}]");
        let edge_id = edge
            .get("edgeId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{label}: missing string edgeId"))?;
        let raw_reference = edge
            .get("target")
            .and_then(|v| v.get("rawReference"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{label}: missing target.rawReference"))?;

        let normalized = worm_target_normalizer::normalize_reference(raw_reference);

        let resolution = match normalized.canonical {
            Some(canonical) => json!({
                "kind": "worm_target_resolution",
                "schemaVersion": 1,
                "edgeId": edge_id,
                "rawReference": raw_reference,
                "resolutionPosture": normalized.posture,
                "resolutionMethod": normalized.method,
                "canonicalIdentity": {
                    "host": canonical.host,
                    "owner": canonical.owner,
                    "repo": canonical.repo,
                    "display": canonical.display
                },
                "timestamp": "2026-04-22T01:45:00-04:00"
            }),
            None => json!({
                "kind": "worm_target_resolution",
                "schemaVersion": 1,
                "edgeId": edge_id,
                "rawReference": raw_reference,
                "resolutionPosture": normalized.posture,
                "resolutionMethod": normalized.method,
                "canonicalIdentity": null,
                "timestamp": "2026-04-22T01:45:00-04:00"
            }),
        };

        out.push(resolution);
    }

    Ok(out)
}

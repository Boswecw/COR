use std::fs;
use std::path::Path;

use crate::error::{Result, WormError};
use crate::model::{
    now_ts, ClaimObservationInput, DiagnosticInput, EvidenceKind, EvidenceRecordInput,
    FreshnessStatus, HealthState, Polarity, Subsystem,
};

pub fn read_observations(path: &Path) -> Result<Vec<ClaimObservationInput>> {
    let raw = fs::read_to_string(path)?;
    if raw.trim_start().starts_with('[') {
        Ok(serde_json::from_str(&raw)?)
    } else {
        raw.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).map_err(Into::into))
            .collect()
    }
}

pub fn repo_crawler_observations(path: &Path) -> Result<Vec<ClaimObservationInput>> {
    let raw = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let repo = value
        .get("repo")
        .ok_or_else(|| WormError::Config("repo-crawler export missing repo".to_string()))?;
    let repo_id = repo
        .get("root_path")
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
        .or_else(|| {
            value
                .get("scan_run")
                .and_then(|scan| scan.get("repo_id"))
                .map(|value| value.to_string())
        })
        .unwrap_or_else(|| "repo-crawler:unknown".to_string());
    let revision_id = repo
        .get("head_commit")
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let scan_id = value
        .get("scan_run")
        .and_then(|scan| scan.get("scan_id"))
        .and_then(|value| value.as_i64());
    let observed_at = value
        .get("scan_run")
        .and_then(|scan| scan.get("finished_ts"))
        .and_then(|value| value.as_i64())
        .unwrap_or_else(now_ts);

    let mut observations = Vec::new();
    if let Some(files) = value.get("files").and_then(|value| value.as_array()) {
        for file in files {
            observations.extend(file_observations(
                &repo_id,
                revision_id.as_deref(),
                scan_id,
                observed_at,
                file,
            ));
        }
    }

    if let Some(diagnostics) = value.get("diagnostics").and_then(|value| value.as_array()) {
        for diagnostic in diagnostics {
            if let Some(observation) = diagnostic_observation(
                &repo_id,
                revision_id.as_deref(),
                scan_id,
                observed_at,
                diagnostic,
            ) {
                observations.push(observation);
            }
        }
    }

    Ok(observations)
}

pub fn cortex_extraction_observations(path: &Path) -> Result<Vec<ClaimObservationInput>> {
    let raw = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let artifact_id = required_str(&value, "artifact_id")?.to_string();
    let source_ref = required_str(&value, "source_ref")?.to_string();
    let state = required_str(&value, "state")?.to_string();
    let source_hash = value
        .get("provenance")
        .and_then(|provenance| provenance.get("source_hash"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let freshness = value
        .get("freshness")
        .and_then(|freshness| freshness.get("state"))
        .and_then(|value| value.as_str())
        .unwrap_or(if state == "stale" { "stale" } else { "fresh" });
    let freshness_status = match freshness {
        "fresh" => FreshnessStatus::Fresh,
        "stale" => FreshnessStatus::Stale,
        _ => FreshnessStatus::Unknown,
    };
    let observed_at = now_ts();
    let repository_id = format!("cortex-source:{source_ref}");
    let target = format!("artifact:{artifact_id}");
    let health = match state.as_str() {
        "ready" => HealthState::Ready,
        "partial_success" => HealthState::PartialSuccess,
        "denied" => HealthState::Denied,
        "unavailable" => HealthState::Unavailable,
        "stale" => HealthState::Stale,
        _ => HealthState::Degraded,
    };
    let ready_polarity = match state.as_str() {
        "ready" | "partial_success" => Polarity::Supports,
        "denied" | "unavailable" | "stale" => Polarity::Refutes,
        _ => Polarity::Unavailable,
    };
    let mut observations = vec![ClaimObservationInput {
        repository_id: repository_id.clone(),
        revision_id: source_hash.clone(),
        normalized_path: None,
        artifact_id: Some(artifact_id.clone()),
        file_hash: source_hash.clone(),
        language_id: None,
        source_scope: "artifact".to_string(),
        claim_target_id: target.clone(),
        claim_class: "artifact_extraction_ready".to_string(),
        subsystem: Subsystem::Cortex,
        polarity: ready_polarity,
        observed_at,
        freshness_reference: source_hash.clone(),
        freshness_status: freshness_status.clone(),
        evidence_strength: Some(if state == "partial_success" {
            0.78
        } else {
            0.92
        }),
        subsystem_health: health.clone(),
        value: value.clone(),
        evidence: vec![EvidenceRecordInput {
            evidence_kind: EvidenceKind::DirectHash,
            reference: format!("cortex:extraction-result:{artifact_id}"),
            strength_score: if state == "partial_success" {
                0.78
            } else {
                0.92
            },
            payload: serde_json::json!({ "adapter": "cortex_extraction_result_v1" }),
        }],
        diagnostics: Vec::new(),
    }];

    observations.push(ClaimObservationInput {
        repository_id,
        revision_id: source_hash.clone(),
        normalized_path: None,
        artifact_id: Some(artifact_id.clone()),
        file_hash: source_hash.clone(),
        language_id: None,
        source_scope: "artifact".to_string(),
        claim_target_id: target,
        claim_class: "artifact_extraction_denied".to_string(),
        subsystem: Subsystem::Cortex,
        polarity: if state == "denied" {
            Polarity::Supports
        } else {
            Polarity::Refutes
        },
        observed_at,
        freshness_reference: source_hash,
        freshness_status,
        evidence_strength: Some(if state == "denied" { 0.95 } else { 0.8 }),
        subsystem_health: health,
        value,
        evidence: vec![EvidenceRecordInput {
            evidence_kind: EvidenceKind::DirectHash,
            reference: format!("cortex:extraction-result:{artifact_id}"),
            strength_score: if state == "denied" { 0.95 } else { 0.8 },
            payload: serde_json::json!({ "adapter": "cortex_extraction_result_v1" }),
        }],
        diagnostics: Vec::new(),
    });

    Ok(observations)
}

fn file_observations(
    repository_id: &str,
    revision_id: Option<&str>,
    scan_id: Option<i64>,
    observed_at: i64,
    file: &serde_json::Value,
) -> Vec<ClaimObservationInput> {
    let rel_path = file
        .get("rel_path")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if rel_path.is_empty() {
        return Vec::new();
    }
    let target = file_target(rel_path);
    let sha256 = file
        .get("sha256")
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let lang = file
        .get("lang")
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let status = file
        .get("parse_status")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let mut observations = Vec::new();

    observations.push(base_observation(
        repository_id,
        revision_id,
        rel_path,
        &target,
        "file_presence",
        if status == "deleted" {
            Polarity::Refutes
        } else {
            Polarity::Supports
        },
        observed_at,
        scan_id,
        sha256.clone(),
        lang.clone(),
        0.98,
        serde_json::json!({ "parse_status": status }),
    ));

    if status == "deleted" {
        observations.push(base_observation(
            repository_id,
            revision_id,
            rel_path,
            &target,
            "file_deleted",
            Polarity::Supports,
            observed_at,
            scan_id,
            sha256.clone(),
            lang.clone(),
            0.98,
            serde_json::json!({ "parse_status": status }),
        ));
    }

    if let Some(hash) = &sha256 {
        observations.push(base_observation(
            repository_id,
            revision_id,
            rel_path,
            &target,
            "file_hash",
            Polarity::Supports,
            observed_at,
            scan_id,
            Some(hash.clone()),
            lang.clone(),
            0.99,
            serde_json::json!({ "sha256": hash }),
        ));
    }

    match status {
        "parsed" => observations.push(base_observation(
            repository_id,
            revision_id,
            rel_path,
            &target,
            "parse_success",
            Polarity::Supports,
            observed_at,
            scan_id,
            sha256.clone(),
            lang.clone(),
            0.86,
            serde_json::json!({ "parser_id": file.get("parser_id").cloned().unwrap_or_default() }),
        )),
        "parse_error" => observations.push(base_observation(
            repository_id,
            revision_id,
            rel_path,
            &target,
            "parse_success",
            Polarity::Refutes,
            observed_at,
            scan_id,
            sha256.clone(),
            lang.clone(),
            0.82,
            serde_json::json!({ "parse_status": status }),
        )),
        _ => {}
    }

    if is_doc_path(rel_path) {
        observations.push(base_observation(
            repository_id,
            revision_id,
            rel_path,
            &target,
            "doc_presence",
            if status == "deleted" {
                Polarity::Refutes
            } else {
                Polarity::Supports
            },
            observed_at,
            scan_id,
            sha256,
            lang,
            0.74,
            serde_json::json!({ "source": "repo_crawler_file_inventory" }),
        ));
    }

    observations
}

fn diagnostic_observation(
    repository_id: &str,
    revision_id: Option<&str>,
    scan_id: Option<i64>,
    observed_at: i64,
    diagnostic: &serde_json::Value,
) -> Option<ClaimObservationInput> {
    let rel_path = diagnostic
        .get("rel_path")
        .and_then(|value| value.as_str())?;
    let code = diagnostic
        .get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("diagnostic");
    if !code.contains("parse") && code != "syntax_error" && code != "missing_node" {
        return None;
    }

    let mut observation = base_observation(
        repository_id,
        revision_id,
        rel_path,
        &file_target(rel_path),
        "parse_success",
        Polarity::Refutes,
        observed_at,
        scan_id,
        None,
        None,
        0.8,
        serde_json::json!({ "diagnostic_code": code }),
    );
    observation.diagnostics.push(DiagnosticInput {
        severity: diagnostic
            .get("severity")
            .and_then(|value| value.as_str())
            .unwrap_or("error")
            .to_string(),
        code: code.to_string(),
        message: diagnostic
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("repo-crawler parse diagnostic")
            .to_string(),
        payload: diagnostic.clone(),
    });
    Some(observation)
}

fn base_observation(
    repository_id: &str,
    revision_id: Option<&str>,
    rel_path: &str,
    target: &str,
    claim_class: &str,
    polarity: Polarity,
    observed_at: i64,
    scan_id: Option<i64>,
    file_hash: Option<String>,
    language_id: Option<String>,
    strength: f64,
    value: serde_json::Value,
) -> ClaimObservationInput {
    let reference = scan_id
        .map(|scan_id| format!("repo-crawler:scan:{scan_id}:{rel_path}"))
        .unwrap_or_else(|| format!("repo-crawler:scan:unknown:{rel_path}"));
    ClaimObservationInput {
        repository_id: repository_id.to_string(),
        revision_id: revision_id.map(ToString::to_string),
        normalized_path: Some(rel_path.to_string()),
        artifact_id: None,
        file_hash,
        language_id,
        source_scope: "repo".to_string(),
        claim_target_id: target.to_string(),
        claim_class: claim_class.to_string(),
        subsystem: Subsystem::RepoCrawler,
        polarity,
        observed_at,
        freshness_reference: revision_id.map(ToString::to_string),
        freshness_status: FreshnessStatus::Fresh,
        evidence_strength: Some(strength),
        subsystem_health: HealthState::Ready,
        value,
        evidence: vec![EvidenceRecordInput {
            evidence_kind: if claim_class == "file_hash" {
                EvidenceKind::DirectHash
            } else {
                EvidenceKind::DirectPath
            },
            reference,
            strength_score: strength,
            payload: serde_json::json!({ "adapter": "repo_crawler_export_v1" }),
        }],
        diagnostics: Vec::new(),
    }
}

fn file_target(rel_path: &str) -> String {
    format!("file:{rel_path}")
}

fn is_doc_path(rel_path: &str) -> bool {
    rel_path == "README.md"
        || rel_path == "SYSTEM.md"
        || rel_path.ends_with(".md")
            && (rel_path.starts_with("docs/")
                || rel_path.starts_with("doc/")
                || rel_path.starts_with("DECISIONS/"))
}

fn required_str<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(|value| value.as_str())
        .ok_or_else(|| WormError::Config(format!("Cortex extraction result missing {field}")))
}

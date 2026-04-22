use std::fs;

use repo_crawler::centipede_queue_consumer_stub::build_ingest_summary;
use repo_crawler::centipede_queue_export::{build_export_envelope, sample_report_json};
use repo_crawler::centipede_queue_handoff_artifact::build_handoff_artifact;
use repo_crawler::centipede_queue_handoff_manifest::build_handoff_manifest;
use repo_crawler::centipede_queue_inbox_resolver::{run_inbox_resolver, QueueInboxResolverArgs};
use repo_crawler::centipede_queue_manifest_scan::scan_manifest_directory;
use serde_json::Value;

fn main() {
    if let Err(err) = run_smoke() {
        eprintln!("centipede_queue_inbox_resolver_smoke: {err}");
        std::process::exit(1);
    }

    println!("centipede_queue_inbox_resolver_smoke: ok");
}

fn run_smoke() -> Result<(), String> {
    let temp_root = std::env::temp_dir().join(format!(
        "centipede_queue_inbox_resolver_smoke_{}",
        std::process::id()
    ));

    if temp_root.exists() {
        fs::remove_dir_all(&temp_root).map_err(|err| {
            format!(
                "failed to remove existing temp dir '{}': {err}",
                temp_root.display()
            )
        })?;
    }

    fs::create_dir_all(&temp_root)
        .map_err(|err| format!("failed to create temp dir '{}': {err}", temp_root.display()))?;

    let export_envelope = build_export_envelope(
        sample_report_json(),
        Some("Q-001".to_string()),
        Some("/tmp/queue-report.json".to_string()),
    )?;
    let export_value = serde_json::to_value(&export_envelope)
        .map_err(|err| format!("failed to serialize export envelope: {err}"))?;
    let ingest_summary = build_ingest_summary(&export_value)?;
    let ingest_value = serde_json::to_value(&ingest_summary)
        .map_err(|err| format!("failed to serialize ingest summary: {err}"))?;
    let handoff_artifact = build_handoff_artifact(&ingest_value)?;
    let handoff_value = serde_json::to_value(&handoff_artifact)
        .map_err(|err| format!("failed to serialize handoff artifact: {err}"))?;

    let mut manifest = serde_json::to_value(build_handoff_manifest(
        &handoff_value,
        Some(temp_root.join("artifact-alpha.json").to_string_lossy().to_string()),
    )?)
    .map_err(|err| format!("failed to serialize manifest: {err}"))?;
    manifest["generatedAtUnixMs"] = Value::from(500u64);
    manifest["artifact"]["generatedAtUnixMs"] = Value::from(500u64);

    fs::write(
        temp_root.join("alpha.manifest.json"),
        serde_json::to_string_pretty(&manifest)
            .map_err(|err| format!("failed to render manifest: {err}"))?,
    )
    .map_err(|err| format!("failed to write manifest: {err}"))?;

    let scan_report = scan_manifest_directory(&temp_root)?;
    let scan_path = temp_root.join("manifest-scan-report.json");
    fs::write(
        &scan_path,
        serde_json::to_string_pretty(&scan_report)
            .map_err(|err| format!("failed to render scan report: {err}"))?,
    )
    .map_err(|err| format!("failed to write scan report '{}': {err}", scan_path.display()))?;

    let resolution_path = temp_root.join("inbox-resolution.json");
    let resolution = run_inbox_resolver(QueueInboxResolverArgs {
        input_path: Some(scan_path.clone()),
        output_path: Some(resolution_path.clone()),
        pretty: true,
    })?;

    if resolution.kind != "centipede_queue_inbox_resolution" {
        return Err("resolution.kind mismatch".to_string());
    }

    if resolution.source_scan.manifest_count != 1 {
        return Err("resolution.source_scan.manifest_count mismatch".to_string());
    }

    if resolution.stats.candidate_count != 1 {
        return Err("resolution.stats.candidate_count mismatch".to_string());
    }

    if resolution.candidates.len() != 1 {
        return Err("resolution.candidates.len mismatch".to_string());
    }

    let candidate = &resolution.candidates[0];
    if candidate.resolution_status != "ready" {
        return Err("candidate.resolution_status mismatch".to_string());
    }

    if candidate.queue_item_id.as_deref() != Some("Q-001") {
        return Err("candidate.queue_item_id mismatch".to_string());
    }

    let expected_artifact_path = temp_root.join("artifact-alpha.json").to_string_lossy().to_string();
    if candidate.artifact_path.as_deref() != Some(expected_artifact_path.as_str()) {
        return Err("candidate.artifact_path mismatch".to_string());
    }

    let written_text = fs::read_to_string(&resolution_path)
        .map_err(|err| format!("failed to read inbox resolution '{}': {err}", resolution_path.display()))?;
    let written_json: Value = serde_json::from_str(&written_text)
        .map_err(|err| format!("failed to parse inbox resolution: {err}"))?;

    if written_json["kind"] != "centipede_queue_inbox_resolution" {
        return Err("written kind mismatch".to_string());
    }

    if written_json["stats"]["candidateCount"] != 1u64 {
        return Err("written stats.candidateCount mismatch".to_string());
    }

    fs::remove_dir_all(&temp_root)
        .map_err(|err| format!("failed to clean temp dir '{}': {err}", temp_root.display()))?;

    Ok(())
}

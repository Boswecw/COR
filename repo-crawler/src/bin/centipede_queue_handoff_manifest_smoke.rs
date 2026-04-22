use std::fs;

use repo_crawler::centipede_queue_consumer_stub::build_ingest_summary;
use repo_crawler::centipede_queue_export::{build_export_envelope, sample_report_json};
use repo_crawler::centipede_queue_handoff_artifact::build_handoff_artifact;
use repo_crawler::centipede_queue_handoff_manifest::{
    run_handoff_manifest, QueueHandoffManifestArgs,
};
use serde_json::Value;

fn main() {
    if let Err(err) = run_smoke() {
        eprintln!("centipede_queue_handoff_manifest_smoke: {err}");
        std::process::exit(1);
    }

    println!("centipede_queue_handoff_manifest_smoke: ok");
}

fn run_smoke() -> Result<(), String> {
    let temp_root = std::env::temp_dir().join(format!(
        "centipede_queue_handoff_manifest_smoke_{}_{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("main")
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

    let artifact_path = temp_root.join("queue-handoff-artifact.json");
    let manifest_path = temp_root.join("queue-handoff-artifact.manifest.json");

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
    let handoff_text = serde_json::to_string_pretty(&handoff_artifact)
        .map_err(|err| format!("failed to render handoff artifact: {err}"))?;
    fs::write(&artifact_path, handoff_text)
        .map_err(|err| format!("failed to write handoff artifact '{}': {err}", artifact_path.display()))?;

    let artifact_path_string = artifact_path.to_string_lossy().to_string();

    let manifest = run_handoff_manifest(QueueHandoffManifestArgs {
        input_path: Some(artifact_path.clone()),
        output_path: Some(manifest_path.clone()),
        artifact_path: None,
        pretty: true,
    })?;

    if manifest.kind != "centipede_queue_handoff_manifest" {
        return Err("manifest.kind mismatch".to_string());
    }

    if manifest.artifact.path.as_deref() != Some(artifact_path_string.as_str()) {
        return Err("manifest.artifact.path mismatch".to_string());
    }

    if manifest.artifact.posture_status != "accepted" {
        return Err("manifest.artifact.posture_status mismatch".to_string());
    }

    if manifest.summary.queue_item_id.as_deref() != Some("Q-001") {
        return Err("manifest.summary.queue_item_id mismatch".to_string());
    }

    if manifest.summary.item_count != 1 {
        return Err("manifest.summary.item_count mismatch".to_string());
    }

    if !manifest.discovery.index_key.contains("accepted") {
        return Err("manifest.discovery.index_key missing accepted posture".to_string());
    }

    let mut rejected_artifact_value = serde_json::to_value(&handoff_artifact)
        .map_err(|err| format!("failed to serialize handoff artifact for rejection case: {err}"))?;
    rejected_artifact_value["posture"]["status"] = Value::String("rejected".to_string());
    rejected_artifact_value["posture"]["reasons"] = serde_json::json!(["forced rejection for smoke"]);
    let rejected_text = serde_json::to_string_pretty(&rejected_artifact_value)
        .map_err(|err| format!("failed to render rejected handoff artifact: {err}"))?;
    fs::write(&artifact_path, rejected_text)
        .map_err(|err| format!("failed to overwrite handoff artifact '{}': {err}", artifact_path.display()))?;

    let rejected_manifest = run_handoff_manifest(QueueHandoffManifestArgs {
        input_path: Some(artifact_path.clone()),
        output_path: Some(manifest_path.clone()),
        artifact_path: None,
        pretty: true,
    })?;

    if rejected_manifest.artifact.posture_status != "rejected" {
        return Err("rejected_manifest.artifact.posture_status mismatch".to_string());
    }

    if rejected_manifest.artifact.reasons_count != 1 {
        return Err("rejected_manifest.artifact.reasons_count mismatch".to_string());
    }

    if !rejected_manifest.discovery.index_key.contains("rejected") {
        return Err("rejected_manifest.discovery.index_key missing rejected posture".to_string());
    }

    let written_text = fs::read_to_string(&manifest_path)
        .map_err(|err| format!("failed to read manifest '{}': {err}", manifest_path.display()))?;
    let written_json: Value = serde_json::from_str(&written_text)
        .map_err(|err| format!("failed to parse written manifest: {err}"))?;

    if written_json["kind"] != "centipede_queue_handoff_manifest" {
        return Err("written kind mismatch".to_string());
    }

    if written_json["artifact"]["postureStatus"] != "rejected" {
        return Err("written artifact.postureStatus mismatch".to_string());
    }

    fs::remove_dir_all(&temp_root)
        .map_err(|err| format!("failed to clean temp dir '{}': {err}", temp_root.display()))?;

    Ok(())
}

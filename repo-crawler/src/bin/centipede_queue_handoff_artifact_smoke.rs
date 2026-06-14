use std::fs;

use repo_crawler::centipede_queue_consumer_stub::build_ingest_summary;
use repo_crawler::centipede_queue_export::{build_export_envelope, sample_report_json};
use repo_crawler::centipede_queue_handoff_artifact::{
    run_handoff_artifact, QueueHandoffArtifactArgs,
};
use serde_json::Value;

fn main() {
    if let Err(err) = run_smoke() {
        eprintln!("centipede_queue_handoff_artifact_smoke: {err}");
        std::process::exit(1);
    }

    println!("centipede_queue_handoff_artifact_smoke: ok");
}

fn run_smoke() -> Result<(), String> {
    let temp_root = std::env::temp_dir().join(format!(
        "centipede_queue_handoff_artifact_smoke_{}_{}",
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

    let input_path = temp_root.join("queue-ingest-summary.json");
    let output_path = temp_root.join("queue-handoff-artifact.json");

    let export_envelope = build_export_envelope(
        sample_report_json(),
        Some("Q-001".to_string()),
        Some("/tmp/queue-report.json".to_string()),
    )?;
    let export_value = serde_json::to_value(&export_envelope)
        .map_err(|err| format!("failed to serialize export envelope: {err}"))?;
    let ingest_summary = build_ingest_summary(&export_value)?;

    let ingest_text = serde_json::to_string_pretty(&ingest_summary)
        .map_err(|err| format!("failed to render ingest summary: {err}"))?;
    fs::write(&input_path, ingest_text)
        .map_err(|err| format!("failed to write ingest summary '{}': {err}", input_path.display()))?;

    let args = QueueHandoffArtifactArgs {
        input_path: Some(input_path.clone()),
        output_path: Some(output_path.clone()),
        pretty: true,
    };

    let artifact = run_handoff_artifact(args)?;

    if artifact.kind != "centipede_queue_handoff_artifact" {
        return Err("artifact.kind mismatch".to_string());
    }

    if artifact.posture.status != "accepted" {
        return Err("artifact.posture.status mismatch".to_string());
    }

    if !artifact.posture.reasons.is_empty() {
        return Err("artifact.posture.reasons should be empty for accepted summary".to_string());
    }

    if artifact.source.contract_type != "centipede.queue.report.export" {
        return Err("artifact.source.contract_type mismatch".to_string());
    }

    if artifact.source.contract_version != "v2" {
        return Err("artifact.source.contract_version mismatch".to_string());
    }

    if artifact.summary.item_count != 1 {
        return Err("artifact.summary.item_count mismatch".to_string());
    }

    if artifact.items.len() != 1 {
        return Err("artifact.items.len mismatch".to_string());
    }

    let mut rejected_summary_value = serde_json::to_value(&ingest_summary)
        .map_err(|err| format!("failed to serialize ingest summary for rejection case: {err}"))?;
    rejected_summary_value["schemaStatus"] = Value::String("bad-status".to_string());
    let rejected_text = serde_json::to_string_pretty(&rejected_summary_value)
        .map_err(|err| format!("failed to render rejected ingest summary: {err}"))?;
    fs::write(&input_path, rejected_text)
        .map_err(|err| format!("failed to overwrite ingest summary '{}': {err}", input_path.display()))?;

    let rejected_artifact = run_handoff_artifact(QueueHandoffArtifactArgs {
        input_path: Some(input_path.clone()),
        output_path: Some(output_path.clone()),
        pretty: true,
    })?;

    if rejected_artifact.posture.status != "rejected" {
        return Err("rejected_artifact.posture.status mismatch".to_string());
    }

    if rejected_artifact.posture.reasons.is_empty() {
        return Err("rejected_artifact.posture.reasons should not be empty".to_string());
    }

    let written_text = fs::read_to_string(&output_path)
        .map_err(|err| format!("failed to read handoff artifact '{}': {err}", output_path.display()))?;
    let written_json: Value = serde_json::from_str(&written_text)
        .map_err(|err| format!("failed to parse handoff artifact: {err}"))?;

    if written_json["kind"] != "centipede_queue_handoff_artifact" {
        return Err("written kind mismatch".to_string());
    }

    if written_json["posture"]["status"] != "rejected" {
        return Err("written posture.status mismatch".to_string());
    }

    fs::remove_dir_all(&temp_root)
        .map_err(|err| format!("failed to clean temp dir '{}': {err}", temp_root.display()))?;

    Ok(())
}

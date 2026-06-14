use std::fs;

use repo_crawler::centipede_queue_consumer_stub::build_ingest_summary;
use repo_crawler::centipede_queue_export::{build_export_envelope, sample_report_json};
use repo_crawler::centipede_queue_handoff_artifact::build_handoff_artifact;
use repo_crawler::centipede_queue_handoff_manifest::build_handoff_manifest;
use repo_crawler::centipede_queue_manifest_scan::{run_manifest_scan, QueueManifestScanArgs};
use serde_json::Value;

fn main() {
    if let Err(err) = run_smoke() {
        eprintln!("centipede_queue_manifest_scan_smoke: {err}");
        std::process::exit(1);
    }

    println!("centipede_queue_manifest_scan_smoke: ok");
}

fn run_smoke() -> Result<(), String> {
    let temp_root = std::env::temp_dir().join(format!(
        "centipede_queue_manifest_scan_smoke_{}",
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

    let mut manifest_old = serde_json::to_value(build_handoff_manifest(
        &handoff_value,
        Some(temp_root.join("artifact-old.json").to_string_lossy().to_string()),
    )?)
    .map_err(|err| format!("failed to serialize old manifest: {err}"))?;
    manifest_old["generatedAtUnixMs"] = Value::from(100u64);
    manifest_old["artifact"]["generatedAtUnixMs"] = Value::from(100u64);

    let mut manifest_new = serde_json::to_value(build_handoff_manifest(
        &handoff_value,
        Some(temp_root.join("artifact-new.json").to_string_lossy().to_string()),
    )?)
    .map_err(|err| format!("failed to serialize new manifest: {err}"))?;
    manifest_new["generatedAtUnixMs"] = Value::from(200u64);
    manifest_new["artifact"]["generatedAtUnixMs"] = Value::from(200u64);

    let mut manifest_rejected = serde_json::to_value(build_handoff_manifest(
        &handoff_value,
        Some(temp_root.join("artifact-rejected.json").to_string_lossy().to_string()),
    )?)
    .map_err(|err| format!("failed to serialize rejected manifest: {err}"))?;
    manifest_rejected["generatedAtUnixMs"] = Value::from(300u64);
    manifest_rejected["artifact"]["generatedAtUnixMs"] = Value::from(300u64);
    manifest_rejected["artifact"]["postureStatus"] = Value::String("rejected".to_string());
    manifest_rejected["artifact"]["reasonsCount"] = Value::from(1u64);
    let rejected_key = manifest_rejected["discovery"]["indexKey"]
        .as_str()
        .ok_or_else(|| "rejected manifest missing discovery.indexKey".to_string())?
        .replace("accepted", "rejected");
    manifest_rejected["discovery"]["indexKey"] = Value::String(rejected_key);

    fs::write(
        temp_root.join("01-old.manifest.json"),
        serde_json::to_string_pretty(&manifest_old)
            .map_err(|err| format!("failed to render old manifest: {err}"))?,
    )
    .map_err(|err| format!("failed to write old manifest: {err}"))?;

    fs::write(
        temp_root.join("02-new.manifest.json"),
        serde_json::to_string_pretty(&manifest_new)
            .map_err(|err| format!("failed to render new manifest: {err}"))?,
    )
    .map_err(|err| format!("failed to write new manifest: {err}"))?;

    fs::write(
        temp_root.join("03-rejected.manifest.json"),
        serde_json::to_string_pretty(&manifest_rejected)
            .map_err(|err| format!("failed to render rejected manifest: {err}"))?,
    )
    .map_err(|err| format!("failed to write rejected manifest: {err}"))?;

    fs::write(temp_root.join("04-invalid.json"), "not json at all")
        .map_err(|err| format!("failed to write invalid manifest file: {err}"))?;

    let output_path = temp_root.join("manifest-scan-report.json");
    let report = run_manifest_scan(QueueManifestScanArgs {
        input_dir: Some(temp_root.clone()),
        output_path: Some(output_path.clone()),
        pretty: true,
    })?;

    if report.kind != "centipede_queue_handoff_manifest_scan" {
        return Err("report.kind mismatch".to_string());
    }

    if report.manifest_count != 3 {
        return Err("report.manifest_count mismatch".to_string());
    }

    if report.invalid_file_count != 1 {
        return Err("report.invalid_file_count mismatch".to_string());
    }

    if report.latest_accepted_by_index_key.len() != 1 {
        return Err("report.latest_accepted_by_index_key.len mismatch".to_string());
    }

    let selected = &report.latest_accepted_by_index_key[0];
    let expected_artifact_new = temp_root.join("artifact-new.json").to_string_lossy().to_string();
    if selected.generated_at_unix_ms != 200 {
        return Err("selected.generated_at_unix_ms mismatch".to_string());
    }

    if selected.artifact_path.as_deref() != Some(expected_artifact_new.as_str()) {
        return Err("selected.artifact_path mismatch".to_string());
    }

    if selected.posture_status != "accepted" {
        return Err("selected.posture_status mismatch".to_string());
    }

    let written_text = fs::read_to_string(&output_path)
        .map_err(|err| format!("failed to read report output '{}': {err}", output_path.display()))?;
    let written_json: Value = serde_json::from_str(&written_text)
        .map_err(|err| format!("failed to parse report output: {err}"))?;

    if written_json["kind"] != "centipede_queue_handoff_manifest_scan" {
        return Err("written kind mismatch".to_string());
    }

    if written_json["latestAcceptedByIndexKey"][0]["generatedAtUnixMs"] != 200u64 {
        return Err("written latestAcceptedByIndexKey[0].generatedAtUnixMs mismatch".to_string());
    }

    fs::remove_dir_all(&temp_root)
        .map_err(|err| format!("failed to clean temp dir '{}': {err}", temp_root.display()))?;

    Ok(())
}

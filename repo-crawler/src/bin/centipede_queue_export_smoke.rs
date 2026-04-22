use std::fs;

use repo_crawler::centipede_queue_export::{
    run_export, sample_report_json, QueueExportArgs, CONTRACT_TYPE, CONTRACT_VERSION,
};
use serde_json::Value;

fn main() {
    if let Err(err) = run_smoke() {
        eprintln!("centipede_queue_export_smoke: {err}");
        std::process::exit(1);
    }

    println!("centipede_queue_export_smoke: ok");
}

fn run_smoke() -> Result<(), String> {
    let temp_root = std::env::temp_dir().join(format!(
        "centipede_queue_export_smoke_{}_{}",
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

    let input_path = temp_root.join("queue-report.json");
    let output_path = temp_root.join("queue-export.json");

    let report = sample_report_json();
    let report_text = serde_json::to_string_pretty(&report)
        .map_err(|err| format!("failed to render sample report: {err}"))?;
    fs::write(&input_path, report_text)
        .map_err(|err| format!("failed to write sample report '{}': {err}", input_path.display()))?;

    let args = QueueExportArgs {
        input_path: Some(input_path.clone()),
        output_path: Some(output_path.clone()),
        queue_item_id: Some("Q-001".to_string()),
        pretty: true,
    };

    let envelope = run_export(args)?;

    if envelope.contract_type != CONTRACT_TYPE {
        return Err(format!(
            "unexpected contract type '{}', expected '{}'",
            envelope.contract_type, CONTRACT_TYPE
        ));
    }

    if envelope.contract_version != CONTRACT_VERSION {
        return Err(format!(
            "unexpected contract version '{}', expected '{}'",
            envelope.contract_version, CONTRACT_VERSION
        ));
    }

    if envelope.payload.schema_status != "validated-envelope-v2" {
        return Err(format!(
            "unexpected schema status '{}', expected 'validated-envelope-v2'",
            envelope.payload.schema_status
        ));
    }

    if envelope.payload.schema_fingerprint.is_empty() {
        return Err("payload.schemaFingerprint must not be empty".to_string());
    }

    let written_text = fs::read_to_string(&output_path)
        .map_err(|err| format!("failed to read export output '{}': {err}", output_path.display()))?;
    let written_json: Value = serde_json::from_str(&written_text)
        .map_err(|err| format!("failed to parse written export output: {err}"))?;

    if written_json["contractType"] != CONTRACT_TYPE {
        return Err("written contractType mismatch".to_string());
    }

    if written_json["contractVersion"] != CONTRACT_VERSION {
        return Err("written contractVersion mismatch".to_string());
    }

    if written_json["selection"]["queueItemId"] != "Q-001" {
        return Err("selection.queueItemId mismatch".to_string());
    }

    if written_json["payload"]["schemaStatus"] != "validated-envelope-v2" {
        return Err("payload.schemaStatus mismatch".to_string());
    }

    if written_json["payload"]["validation"]["reportKind"] != "centipede_queue_operator_report" {
        return Err("payload.validation.reportKind mismatch".to_string());
    }

    if written_json["payload"]["validation"]["selectionQueueItemId"] != "Q-001" {
        return Err("payload.validation.selectionQueueItemId mismatch".to_string());
    }

    if written_json["payload"]["report"]["items"][0]["processingState"]["status"] != "completed" {
        return Err("payload.report.items[0].processingState.status mismatch".to_string());
    }

    fs::remove_dir_all(&temp_root)
        .map_err(|err| format!("failed to clean temp dir '{}': {err}", temp_root.display()))?;

    Ok(())
}

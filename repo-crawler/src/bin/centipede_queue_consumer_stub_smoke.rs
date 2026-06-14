use std::fs;

use repo_crawler::centipede_queue_consumer_stub::{
    run_consumer_stub, QueueConsumerStubArgs, EXPECTED_CONTRACT_TYPE, EXPECTED_CONTRACT_VERSION,
    EXPECTED_SCHEMA_STATUS,
};
use repo_crawler::centipede_queue_export::{
    build_export_envelope, sample_report_json,
};
use serde_json::Value;

fn main() {
    if let Err(err) = run_smoke() {
        eprintln!("centipede_queue_consumer_stub_smoke: {err}");
        std::process::exit(1);
    }

    println!("centipede_queue_consumer_stub_smoke: ok");
}

fn run_smoke() -> Result<(), String> {
    let temp_root = std::env::temp_dir().join(format!(
        "centipede_queue_consumer_stub_smoke_{}_{}",
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

    let input_path = temp_root.join("queue-export.json");
    let output_path = temp_root.join("queue-ingest-summary.json");

    let envelope = build_export_envelope(
        sample_report_json(),
        Some("Q-001".to_string()),
        Some("/tmp/queue-report.json".to_string()),
    )?;

    let envelope_text = serde_json::to_string_pretty(&envelope)
        .map_err(|err| format!("failed to serialize export envelope: {err}"))?;
    fs::write(&input_path, envelope_text)
        .map_err(|err| format!("failed to write export envelope '{}': {err}", input_path.display()))?;

    let args = QueueConsumerStubArgs {
        input_path: Some(input_path.clone()),
        output_path: Some(output_path.clone()),
        pretty: true,
    };

    let summary = run_consumer_stub(args)?;

    if summary.source_contract_type != EXPECTED_CONTRACT_TYPE {
        return Err("source_contract_type mismatch".to_string());
    }

    if summary.source_contract_version != EXPECTED_CONTRACT_VERSION {
        return Err("source_contract_version mismatch".to_string());
    }

    if summary.schema_status != EXPECTED_SCHEMA_STATUS {
        return Err("schema_status mismatch".to_string());
    }

    if summary.selection.queue_item_id.as_deref() != Some("Q-001") {
        return Err("selection.queue_item_id mismatch".to_string());
    }

    if summary.item_count != 1 {
        return Err("item_count mismatch".to_string());
    }

    if summary.items[0].queue_item_id != "Q-001" {
        return Err("items[0].queue_item_id mismatch".to_string());
    }

    if summary.items[0].status != "completed" {
        return Err("items[0].status mismatch".to_string());
    }

    if summary.items[0].claim_episode_count != 2 {
        return Err("items[0].claim_episode_count mismatch".to_string());
    }

    if !summary.items[0].has_completion_evidence {
        return Err("items[0].has_completion_evidence mismatch".to_string());
    }

    if summary.items[0].has_failure_evidence {
        return Err("items[0].has_failure_evidence mismatch".to_string());
    }

    let written_text = fs::read_to_string(&output_path)
        .map_err(|err| format!("failed to read summary output '{}': {err}", output_path.display()))?;
    let written_json: Value = serde_json::from_str(&written_text)
        .map_err(|err| format!("failed to parse summary output: {err}"))?;

    if written_json["kind"] != "centipede_queue_ingest_summary" {
        return Err("written kind mismatch".to_string());
    }

    if written_json["selection"]["queueItemId"] != "Q-001" {
        return Err("written selection.queueItemId mismatch".to_string());
    }

    fs::remove_dir_all(&temp_root)
        .map_err(|err| format!("failed to clean temp dir '{}': {err}", temp_root.display()))?;

    Ok(())
}

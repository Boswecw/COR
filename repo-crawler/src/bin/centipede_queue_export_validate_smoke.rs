use repo_crawler::centipede_queue_export::sample_report_json;
use repo_crawler::centipede_queue_export_validate::validate_report_contract;
use serde_json::json;

fn main() {
    if let Err(err) = run_smoke() {
        eprintln!("centipede_queue_export_validate_smoke: {err}");
        std::process::exit(1);
    }

    println!("centipede_queue_export_validate_smoke: ok");
}

fn run_smoke() -> Result<(), String> {
    let report = sample_report_json();
    let validation = validate_report_contract(&report, Some("Q-001"))?;

    if validation.report_kind != "centipede_queue_operator_report" {
        return Err("validation.report_kind mismatch".to_string());
    }

    if validation.report_schema_version != 1 {
        return Err("validation.report_schema_version mismatch".to_string());
    }

    if validation.selection_queue_item_id.as_deref() != Some("Q-001") {
        return Err("validation.selection_queue_item_id mismatch".to_string());
    }

    if validation.schema_fingerprint.is_empty() {
        return Err("validation.schema_fingerprint must not be empty".to_string());
    }

    let mut mismatch_report = sample_report_json();
    mismatch_report["selection"] = json!({ "queueItemId": "Q-999" });

    let mismatch_err = validate_report_contract(&mismatch_report, Some("Q-001"))
        .err()
        .ok_or_else(|| "expected selection mismatch validation failure".to_string())?;

    if !mismatch_err.contains("selection.queueItemId mismatch") {
        return Err(format!(
            "unexpected mismatch error text: {}",
            mismatch_err
        ));
    }

    Ok(())
}

use crate::centipede_queue_consumer_stub::build_ingest_summary;
use crate::centipede_queue_export::{build_export_envelope, sample_report_json};
use serde_json::Value;

pub fn run_adversarial_checks() -> Result<(), String> {
    let good_envelope = build_export_envelope(
        sample_report_json(),
        Some("Q-001".to_string()),
        Some("/tmp/queue-report.json".to_string()),
    )?;

    let good_value = serde_json::to_value(&good_envelope)
        .map_err(|err| format!("failed to serialize good envelope: {err}"))?;

    assert_rejected(
        mutate(good_value.clone(), |root| {
            root["contractType"] = Value::String("wrong.contract".to_string());
        }),
        "contractType must be 'centipede.queue.report.export'",
    )?;

    assert_rejected(
        mutate(good_value.clone(), |root| {
            root["contractVersion"] = Value::String("v999".to_string());
        }),
        "contractVersion must be 'v2'",
    )?;

    assert_rejected(
        mutate(good_value.clone(), |root| {
            root["payload"]["schemaStatus"] = Value::String("not-validated".to_string());
        }),
        "payload.schemaStatus must be 'validated-envelope-v2'",
    )?;

    assert_rejected(
        mutate(good_value.clone(), |root| {
            root["payload"]["validation"]["schemaFingerprint"] =
                Value::String("deadbeefdeadbeef".to_string());
        }),
        "payload.schemaFingerprint",
    )?;

    assert_rejected(
        mutate(good_value, |root| {
            root["payload"]["report"]["selection"]["queueItemId"] =
                Value::String("Q-999".to_string());
        }),
        "selection.queueItemId mismatch between envelope and report",
    )?;

    Ok(())
}

fn assert_rejected(candidate: Value, expected_error_substring: &str) -> Result<(), String> {
    match build_ingest_summary(&candidate) {
        Ok(_) => Err(format!(
            "expected failure containing '{}', but candidate was accepted",
            expected_error_substring
        )),
        Err(err) => {
            if err.contains(expected_error_substring) {
                Ok(())
            } else {
                Err(format!(
                    "expected error containing '{}', got '{}'",
                    expected_error_substring, err
                ))
            }
        }
    }
}

fn mutate(mut value: Value, apply: impl FnOnce(&mut Value)) -> Value {
    apply(&mut value);
    value
}

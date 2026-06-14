use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::centipede_queue_export_validate::{
    validate_report_contract, QueueExportValidation,
};

pub const CONTRACT_TYPE: &str = "centipede.queue.report.export";
pub const CONTRACT_VERSION: &str = "v2";

#[derive(Debug, Clone, Default)]
pub struct QueueExportArgs {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub queue_item_id: Option<String>,
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueExportEnvelope {
    #[serde(rename = "contractType")]
    pub contract_type: String,
    #[serde(rename = "contractVersion")]
    pub contract_version: String,
    #[serde(rename = "generatedAtUnixMs")]
    pub generated_at_unix_ms: u128,
    pub producer: QueueExportProducer,
    pub selection: QueueExportSelection,
    pub capabilities: QueueExportCapabilities,
    pub payload: QueueExportPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueExportProducer {
    pub system: String,
    pub surface: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueExportSelection {
    #[serde(rename = "queueItemId", skip_serializing_if = "Option::is_none")]
    pub queue_item_id: Option<String>,
    #[serde(rename = "inputPath", skip_serializing_if = "Option::is_none")]
    pub input_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueExportCapabilities {
    #[serde(rename = "hasQueueTotals")]
    pub has_queue_totals: bool,
    #[serde(rename = "hasItemStates")]
    pub has_item_states: bool,
    #[serde(rename = "hasClaimEpisodeChains")]
    pub has_claim_episode_chains: bool,
    #[serde(rename = "hasCompletionEvidence")]
    pub has_completion_evidence: bool,
    #[serde(rename = "hasFailureEvidence")]
    pub has_failure_evidence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueExportPayload {
    pub format: String,
    #[serde(rename = "schemaStatus")]
    pub schema_status: String,
    #[serde(rename = "schemaFingerprint")]
    pub schema_fingerprint: String,
    pub validation: QueueExportValidation,
    pub report: Value,
}

pub fn parse_args<I>(args: I) -> Result<QueueExportArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = QueueExportArgs::default();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--input" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value after --input".to_string())?;
                if value != "-" {
                    parsed.input_path = Some(PathBuf::from(value));
                }
            }
            "--output" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value after --output".to_string())?;
                parsed.output_path = Some(PathBuf::from(value));
            }
            "--queue-item-id" => {
                parsed.queue_item_id = Some(
                    iter.next()
                        .ok_or_else(|| "missing value after --queue-item-id".to_string())?,
                );
            }
            "--pretty" => {
                parsed.pretty = true;
            }
            "--help" | "-h" => {
                return Err(help_text());
            }
            other => {
                return Err(format!("unrecognized argument: {other}\n\n{}", help_text()));
            }
        }
    }

    Ok(parsed)
}

pub fn help_text() -> String {
    [
        "centipede_queue_export",
        "",
        "Wrap a queue report JSON document in a validated, versioned export envelope for ForgeCommand ingestion.",
        "",
        "Usage:",
        "  cargo run --bin centipede_queue_report -- [report-args] > /tmp/queue-report.json",
        "  cargo run --bin centipede_queue_export -- --input /tmp/queue-report.json --output /tmp/queue-export.json --pretty",
        "",
        "Pipeline usage:",
        "  cargo run --bin centipede_queue_report -- [report-args] | cargo run --bin centipede_queue_export -- --pretty > /tmp/queue-export.json",
        "",
        "Arguments:",
        "  --input <path>          Read report JSON from a file. Use '-' or omit to read stdin.",
        "  --output <path>         Write the contract JSON to a file. Omit to write stdout.",
        "  --queue-item-id <id>    Carry queueItemId into the export selection block.",
        "  --pretty                Pretty-print output JSON.",
        "  --help, -h              Show this help text.",
    ]
    .join("\n")
}

pub fn run_export(args: QueueExportArgs) -> Result<QueueExportEnvelope, String> {
    let report = read_report_json(args.input_path.as_deref())?;
    let envelope = build_export_envelope(
        report,
        args.queue_item_id,
        args.input_path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
    )?;

    write_export_output(&envelope, args.output_path.as_deref(), args.pretty)?;
    Ok(envelope)
}

pub fn build_export_envelope(
    report: Value,
    queue_item_id: Option<String>,
    input_path: Option<String>,
) -> Result<QueueExportEnvelope, String> {
    if !report.is_object() {
        return Err("queue report input must be a JSON object".to_string());
    }

    let validation = validate_report_contract(&report, queue_item_id.as_deref())?;
    let capabilities = detect_capabilities(&report);
    let schema_fingerprint = validation.schema_fingerprint.clone();

    Ok(QueueExportEnvelope {
        contract_type: CONTRACT_TYPE.to_string(),
        contract_version: CONTRACT_VERSION.to_string(),
        generated_at_unix_ms: current_unix_ms()?,
        producer: QueueExportProducer {
            system: "repo-crawler".to_string(),
            surface: "centipede_queue_export".to_string(),
            mode: "read_only_evidence_export".to_string(),
        },
        selection: QueueExportSelection {
            queue_item_id,
            input_path,
        },
        capabilities,
        payload: QueueExportPayload {
            format: "json".to_string(),
            schema_status: "validated-envelope-v2".to_string(),
            schema_fingerprint,
            validation,
            report,
        },
    })
}

pub fn read_report_json(input_path: Option<&Path>) -> Result<Value, String> {
    let raw = match input_path {
        Some(path) => fs::read_to_string(path)
            .map_err(|err| format!("failed to read input file '{}': {err}", path.display()))?,
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|err| format!("failed to read stdin: {err}"))?;
            buffer
        }
    };

    serde_json::from_str::<Value>(&raw).map_err(|err| format!("invalid JSON input: {err}"))
}

pub fn write_export_output(
    envelope: &QueueExportEnvelope,
    output_path: Option<&Path>,
    pretty: bool,
) -> Result<(), String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(envelope)
            .map_err(|err| format!("failed to serialize export envelope: {err}"))?
    } else {
        serde_json::to_string(envelope)
            .map_err(|err| format!("failed to serialize export envelope: {err}"))?
    };

    match output_path {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).map_err(|err| {
                        format!("failed to create output directory '{}': {err}", parent.display())
                    })?;
                }
            }
            fs::write(path, rendered)
                .map_err(|err| format!("failed to write output file '{}': {err}", path.display()))?;
        }
        None => {
            println!("{rendered}");
        }
    }

    Ok(())
}

fn detect_capabilities(report: &Value) -> QueueExportCapabilities {
    QueueExportCapabilities {
        has_queue_totals: has_top_level_key(report, &["totals", "queueTotals"]),
        has_item_states: has_top_level_key(report, &["items", "queueItems"]),
        has_claim_episode_chains: contains_any_key(report, &["claimEpisodes", "claimEpisodeChains", "episodes"]),
        has_completion_evidence: contains_any_key(report, &["completionEvidence"]),
        has_failure_evidence: contains_any_key(report, &["failureEvidence"]),
    }
}

fn has_top_level_key(report: &Value, keys: &[&str]) -> bool {
    report
        .as_object()
        .map(|map| keys.iter().any(|key| map.contains_key(*key)))
        .unwrap_or(false)
}

fn contains_any_key(value: &Value, keys: &[&str]) -> bool {
    match value {
        Value::Object(map) => {
            if keys.iter().any(|key| map.contains_key(*key)) {
                return true;
            }
            map.values().any(|child| contains_any_key(child, keys))
        }
        Value::Array(items) => items.iter().any(|child| contains_any_key(child, keys)),
        _ => false,
    }
}

fn current_unix_ms() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|err| format!("system clock error: {err}"))
}

pub fn sample_report_json() -> Value {
    let mut item = Map::new();
    item.insert("queueItemId".to_string(), Value::String("Q-001".to_string()));
    item.insert(
        "processingState".to_string(),
        serde_json::json!({
            "status": "completed"
        }),
    );
    item.insert(
        "claimRecord".to_string(),
        serde_json::json!({
            "episodes": [
                { "state": "reclaimed" },
                { "state": "completed" }
            ]
        }),
    );
    item.insert(
        "completionEvidence".to_string(),
        serde_json::json!([
            {
                "completedAt": "2026-04-21T23:32:00-04:00",
                "artifactPath": "/tmp/queue-complete.json"
            }
        ]),
    );
    item.insert(
        "failureEvidence".to_string(),
        serde_json::json!([]),
    );

    serde_json::json!({
        "kind": "centipede_queue_operator_report",
        "schemaVersion": 1,
        "queueDir": "/tmp/centipede-queue-report-smoke/queue",
        "selection": {
            "queueItemId": "Q-001"
        },
        "totals": {
            "items": 1,
            "claimRecords": 1,
            "claimEpisodes": 2,
            "reclaimedHistoryEpisodes": 1,
            "completions": 1,
            "failures": 0,
            "processing": {
                "completed": 1
            }
        },
        "items": [Value::Object(item)]
    })
}

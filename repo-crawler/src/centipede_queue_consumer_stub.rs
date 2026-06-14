use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub const EXPECTED_CONTRACT_TYPE: &str = "centipede.queue.report.export";
pub const EXPECTED_CONTRACT_VERSION: &str = "v2";
pub const EXPECTED_SCHEMA_STATUS: &str = "validated-envelope-v2";

#[derive(Debug, Clone, Default)]
pub struct QueueConsumerStubArgs {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueIngestSummary {
    pub kind: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u64,
    #[serde(rename = "sourceContractType")]
    pub source_contract_type: String,
    #[serde(rename = "sourceContractVersion")]
    pub source_contract_version: String,
    #[serde(rename = "schemaStatus")]
    pub schema_status: String,
    #[serde(rename = "schemaFingerprint")]
    pub schema_fingerprint: String,
    #[serde(rename = "queueDir")]
    pub queue_dir: String,
    pub selection: QueueIngestSelection,
    #[serde(rename = "itemCount")]
    pub item_count: usize,
    #[serde(rename = "processingCounts")]
    pub processing_counts: Map<String, Value>,
    pub totals: Value,
    pub items: Vec<QueueIngestItemSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueIngestSelection {
    #[serde(rename = "queueItemId", skip_serializing_if = "Option::is_none")]
    pub queue_item_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueIngestItemSummary {
    #[serde(rename = "queueItemId")]
    pub queue_item_id: String,
    pub status: String,
    #[serde(rename = "claimEpisodeCount")]
    pub claim_episode_count: usize,
    #[serde(rename = "hasCompletionEvidence")]
    pub has_completion_evidence: bool,
    #[serde(rename = "hasFailureEvidence")]
    pub has_failure_evidence: bool,
}

pub fn parse_args<I>(args: I) -> Result<QueueConsumerStubArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = QueueConsumerStubArgs::default();
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
        "centipede_queue_consumer_stub",
        "",
        "Read a hardened queue export envelope and emit an ingestion-ready normalized summary.",
        "",
        "Usage:",
        "  cargo run --bin centipede_queue_consumer_stub -- --input /tmp/centipede_queue_export_alpha.json --output /tmp/centipede_queue_ingest_alpha.json --pretty",
        "",
        "Arguments:",
        "  --input <path>      Read export envelope JSON from a file. Use '-' or omit to read stdin.",
        "  --output <path>     Write normalized summary JSON to a file. Omit to write stdout.",
        "  --pretty            Pretty-print output JSON.",
        "  --help, -h          Show this help text.",
    ]
    .join("\n")
}

pub fn run_consumer_stub(args: QueueConsumerStubArgs) -> Result<QueueIngestSummary, String> {
    let envelope = read_json_input(args.input_path.as_deref())?;
    let summary = build_ingest_summary(&envelope)?;
    write_summary_output(&summary, args.output_path.as_deref(), args.pretty)?;
    Ok(summary)
}

pub fn build_ingest_summary(envelope: &Value) -> Result<QueueIngestSummary, String> {
    let env_obj = envelope
        .as_object()
        .ok_or_else(|| "export envelope input must be a JSON object".to_string())?;

    let contract_type = required_string(env_obj, "contractType")?;
    if contract_type != EXPECTED_CONTRACT_TYPE {
        return Err(format!(
            "contractType must be '{}', found '{}'",
            EXPECTED_CONTRACT_TYPE, contract_type
        ));
    }

    let contract_version = required_string(env_obj, "contractVersion")?;
    if contract_version != EXPECTED_CONTRACT_VERSION {
        return Err(format!(
            "contractVersion must be '{}', found '{}'",
            EXPECTED_CONTRACT_VERSION, contract_version
        ));
    }

    let selection_obj = required_object(env_obj, "selection")?;
    let selection_queue_item_id = optional_string(selection_obj, "queueItemId")?;

    let payload_obj = required_object(env_obj, "payload")?;
    let schema_status = required_string(payload_obj, "schemaStatus")?;
    if schema_status != EXPECTED_SCHEMA_STATUS {
        return Err(format!(
            "payload.schemaStatus must be '{}', found '{}'",
            EXPECTED_SCHEMA_STATUS, schema_status
        ));
    }

    let schema_fingerprint = required_string(payload_obj, "schemaFingerprint")?;
    if schema_fingerprint.trim().is_empty() {
        return Err("payload.schemaFingerprint must not be empty".to_string());
    }

    let validation_obj = required_object(payload_obj, "validation")?;
    let validation_fingerprint = required_string(validation_obj, "schemaFingerprint")?;
    if validation_fingerprint != schema_fingerprint {
        return Err(format!(
            "payload.schemaFingerprint '{}' does not match payload.validation.schemaFingerprint '{}'",
            schema_fingerprint, validation_fingerprint
        ));
    }

    let report = payload_obj
        .get("report")
        .ok_or_else(|| "payload.report is required".to_string())?;
    let report_obj = report
        .as_object()
        .ok_or_else(|| "payload.report must be a JSON object".to_string())?;

    let queue_dir = required_string(report_obj, "queueDir")?.to_string();
    let report_selection_obj = required_object(report_obj, "selection")?;
    let report_selection_queue_item_id = optional_string(report_selection_obj, "queueItemId")?;

    if selection_queue_item_id != report_selection_queue_item_id {
        return Err(format!(
            "selection.queueItemId mismatch between envelope and report: outer={:?}, inner={:?}",
            selection_queue_item_id, report_selection_queue_item_id
        ));
    }

    let totals_value = report_obj
        .get("totals")
        .cloned()
        .ok_or_else(|| "payload.report.totals is required".to_string())?;
    let totals_obj = totals_value
        .as_object()
        .ok_or_else(|| "payload.report.totals must be an object".to_string())?;
    let processing_counts = totals_obj
        .get("processing")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let items = required_array(report_obj, "items")?;
    let mut item_summaries = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let item_obj = item
            .as_object()
            .ok_or_else(|| format!("payload.report.items[{}] must be an object", index))?;
        let queue_item_id = required_string(item_obj, "queueItemId")?.to_string();
        let processing_state = required_object(item_obj, "processingState")?;
        let status = required_string(processing_state, "status")?.to_string();

        let claim_episode_count = count_claim_episodes(item_obj);
        let has_completion_evidence = value_has_non_empty(item_obj.get("completionEvidence"));
        let has_failure_evidence = value_has_non_empty(item_obj.get("failureEvidence"));

        item_summaries.push(QueueIngestItemSummary {
            queue_item_id,
            status,
            claim_episode_count,
            has_completion_evidence,
            has_failure_evidence,
        });
    }

    Ok(QueueIngestSummary {
        kind: "centipede_queue_ingest_summary".to_string(),
        schema_version: 1,
        source_contract_type: contract_type.to_string(),
        source_contract_version: contract_version.to_string(),
        schema_status: schema_status.to_string(),
        schema_fingerprint: schema_fingerprint.to_string(),
        queue_dir,
        selection: QueueIngestSelection {
            queue_item_id: selection_queue_item_id,
        },
        item_count: item_summaries.len(),
        processing_counts,
        totals: totals_value,
        items: item_summaries,
    })
}

pub fn read_json_input(input_path: Option<&Path>) -> Result<Value, String> {
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

pub fn write_summary_output(
    summary: &QueueIngestSummary,
    output_path: Option<&Path>,
    pretty: bool,
) -> Result<(), String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(summary)
            .map_err(|err| format!("failed to serialize ingest summary: {err}"))?
    } else {
        serde_json::to_string(summary)
            .map_err(|err| format!("failed to serialize ingest summary: {err}"))?
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

fn count_claim_episodes(item_obj: &Map<String, Value>) -> usize {
    if let Some(count) = item_obj
        .get("claimEpisodes")
        .and_then(Value::as_array)
        .map(Vec::len)
    {
        return count;
    }

    item_obj
        .get("claimRecord")
        .and_then(Value::as_object)
        .and_then(|claim_record| claim_record.get("episodes"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

fn value_has_non_empty(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(map)) => !map.is_empty(),
        Some(Value::Null) | None => false,
        Some(_) => true,
    }
}

fn required_object<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Map<String, Value>, String> {
    map.get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}.{} must be an object", "root", key))
}

fn required_array<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Vec<Value>, String> {
    map.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}.{} must be an array", "payload.report", key))
}

fn required_string<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    map.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("field '{}' must be a string", key))
}

fn optional_string(map: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match map.get(key) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(format!("field '{}' must be a string or null", key)),
    }
}

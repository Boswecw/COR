use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct QueueHandoffArtifactArgs {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffArtifact {
    pub kind: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u64,
    #[serde(rename = "generatedAtUnixMs")]
    pub generated_at_unix_ms: u128,
    pub routing: QueueHandoffRouting,
    pub posture: QueueHandoffPosture,
    pub source: QueueHandoffSource,
    pub summary: QueueHandoffSummary,
    pub items: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffRouting {
    #[serde(rename = "targetSystem")]
    pub target_system: String,
    #[serde(rename = "targetSurface")]
    pub target_surface: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffPosture {
    pub status: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffSource {
    #[serde(rename = "summaryKind")]
    pub summary_kind: String,
    #[serde(rename = "summarySchemaVersion")]
    pub summary_schema_version: u64,
    #[serde(rename = "contractType")]
    pub contract_type: String,
    #[serde(rename = "contractVersion")]
    pub contract_version: String,
    #[serde(rename = "schemaStatus")]
    pub schema_status: String,
    #[serde(rename = "schemaFingerprint")]
    pub schema_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffSummary {
    #[serde(rename = "queueDir")]
    pub queue_dir: String,
    pub selection: Value,
    #[serde(rename = "itemCount")]
    pub item_count: usize,
    #[serde(rename = "processingCounts")]
    pub processing_counts: Value,
    pub totals: Value,
}

pub fn parse_args<I>(args: I) -> Result<QueueHandoffArtifactArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = QueueHandoffArtifactArgs::default();
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
        "centipede_queue_handoff_artifact",
        "",
        "Read a normalized ingest summary and emit a stable ForgeCommand handoff artifact.",
        "",
        "Usage:",
        "  cargo run --bin centipede_queue_handoff_artifact -- --input /tmp/centipede_queue_ingest_alpha.json --output /tmp/centipede_queue_handoff_alpha.json --pretty",
        "",
        "Arguments:",
        "  --input <path>      Read normalized ingest summary JSON from a file. Use '-' or omit to read stdin.",
        "  --output <path>     Write handoff artifact JSON to a file. Omit to write stdout.",
        "  --pretty            Pretty-print output JSON.",
        "  --help, -h          Show this help text.",
    ]
    .join("\n")
}

pub fn run_handoff_artifact(args: QueueHandoffArtifactArgs) -> Result<QueueHandoffArtifact, String> {
    let summary = read_json_input(args.input_path.as_deref())?;
    let artifact = build_handoff_artifact(&summary)?;
    write_artifact_output(&artifact, args.output_path.as_deref(), args.pretty)?;
    Ok(artifact)
}

pub fn build_handoff_artifact(summary: &Value) -> Result<QueueHandoffArtifact, String> {
    let summary_obj = summary
        .as_object()
        .ok_or_else(|| "normalized ingest summary input must be a JSON object".to_string())?;

    let summary_kind = required_string(summary_obj, "kind").unwrap_or_default().to_string();
    let summary_schema_version = summary_obj
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let contract_type = required_string(summary_obj, "sourceContractType")
        .unwrap_or_default()
        .to_string();
    let contract_version = required_string(summary_obj, "sourceContractVersion")
        .unwrap_or_default()
        .to_string();
    let schema_status = required_string(summary_obj, "schemaStatus")
        .unwrap_or_default()
        .to_string();
    let schema_fingerprint = required_string(summary_obj, "schemaFingerprint")
        .unwrap_or_default()
        .to_string();
    let queue_dir = required_string(summary_obj, "queueDir")
        .unwrap_or_default()
        .to_string();
    let selection = summary_obj
        .get("selection")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let item_count = summary_obj
        .get("itemCount")
        .and_then(Value::as_u64)
        .unwrap_or_default() as usize;
    let processing_counts = summary_obj
        .get("processingCounts")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let totals = summary_obj
        .get("totals")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let items = summary_obj
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let reasons = evaluate_posture(summary_obj, &selection, &items, item_count);
    let status = if reasons.is_empty() {
        "accepted"
    } else {
        "rejected"
    }
    .to_string();

    Ok(QueueHandoffArtifact {
        kind: "centipede_queue_handoff_artifact".to_string(),
        schema_version: 1,
        generated_at_unix_ms: current_unix_ms()?,
        routing: QueueHandoffRouting {
            target_system: "ForgeCommand".to_string(),
            target_surface: "centipede_queue_ingest".to_string(),
            mode: "handoff_artifact".to_string(),
        },
        posture: QueueHandoffPosture { status, reasons },
        source: QueueHandoffSource {
            summary_kind,
            summary_schema_version,
            contract_type,
            contract_version,
            schema_status,
            schema_fingerprint,
        },
        summary: QueueHandoffSummary {
            queue_dir,
            selection,
            item_count,
            processing_counts,
            totals,
        },
        items,
    })
}

fn evaluate_posture(
    summary_obj: &Map<String, Value>,
    selection: &Value,
    items: &[Value],
    item_count: usize,
) -> Vec<String> {
    let mut reasons = Vec::new();

    match summary_obj.get("kind").and_then(Value::as_str) {
        Some("centipede_queue_ingest_summary") => {}
        Some(other) => reasons.push(format!(
            "summary.kind must be 'centipede_queue_ingest_summary', found '{}'",
            other
        )),
        None => reasons.push("summary.kind is required".to_string()),
    }

    match summary_obj.get("schemaVersion").and_then(Value::as_u64) {
        Some(1) => {}
        Some(other) => reasons.push(format!(
            "summary.schemaVersion must be 1, found {}",
            other
        )),
        None => reasons.push("summary.schemaVersion is required".to_string()),
    }

    match summary_obj.get("sourceContractType").and_then(Value::as_str) {
        Some("centipede.queue.report.export") => {}
        Some(other) => reasons.push(format!(
            "summary.sourceContractType must be 'centipede.queue.report.export', found '{}'",
            other
        )),
        None => reasons.push("summary.sourceContractType is required".to_string()),
    }

    match summary_obj.get("sourceContractVersion").and_then(Value::as_str) {
        Some("v2") => {}
        Some(other) => reasons.push(format!(
            "summary.sourceContractVersion must be 'v2', found '{}'",
            other
        )),
        None => reasons.push("summary.sourceContractVersion is required".to_string()),
    }

    match summary_obj.get("schemaStatus").and_then(Value::as_str) {
        Some("validated-envelope-v2") => {}
        Some(other) => reasons.push(format!(
            "summary.schemaStatus must be 'validated-envelope-v2', found '{}'",
            other
        )),
        None => reasons.push("summary.schemaStatus is required".to_string()),
    }

    match summary_obj.get("schemaFingerprint").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => {}
        Some(_) => reasons.push("summary.schemaFingerprint must not be empty".to_string()),
        None => reasons.push("summary.schemaFingerprint is required".to_string()),
    }

    match summary_obj.get("queueDir").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => {}
        Some(_) => reasons.push("summary.queueDir must not be empty".to_string()),
        None => reasons.push("summary.queueDir is required".to_string()),
    }

    if item_count != items.len() {
        reasons.push(format!(
            "summary.itemCount '{}' does not match items.len '{}'",
            item_count,
            items.len()
        ));
    }

    let selection_queue_item_id = selection
        .as_object()
        .and_then(|selection_obj| selection_obj.get("queueItemId"))
        .and_then(Value::as_str);

    if let Some(selected) = selection_queue_item_id {
        for (index, item) in items.iter().enumerate() {
            let queue_item_id = item
                .as_object()
                .and_then(|item_obj| item_obj.get("queueItemId"))
                .and_then(Value::as_str);
            if queue_item_id != Some(selected) {
                reasons.push(format!(
                    "items[{}].queueItemId does not match selection.queueItemId '{}'",
                    index, selected
                ));
            }
        }
    }

    reasons
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

pub fn write_artifact_output(
    artifact: &QueueHandoffArtifact,
    output_path: Option<&Path>,
    pretty: bool,
) -> Result<(), String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(artifact)
            .map_err(|err| format!("failed to serialize handoff artifact: {err}"))?
    } else {
        serde_json::to_string(artifact)
            .map_err(|err| format!("failed to serialize handoff artifact: {err}"))?
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

fn required_string<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    map.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("field '{}' must be a string", key))
}

fn current_unix_ms() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|err| format!("system clock error: {err}"))
}

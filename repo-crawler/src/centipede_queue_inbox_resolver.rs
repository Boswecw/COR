use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct QueueInboxResolverArgs {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInboxResolution {
    pub kind: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u64,
    #[serde(rename = "resolvedAtUnixMs")]
    pub resolved_at_unix_ms: u128,
    #[serde(rename = "sourceScan")]
    pub source_scan: QueueInboxSourceScan,
    pub stats: QueueInboxResolutionStats,
    pub candidates: Vec<QueueInboxCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInboxSourceScan {
    #[serde(rename = "scannedDir")]
    pub scanned_dir: String,
    #[serde(rename = "manifestCount")]
    pub manifest_count: usize,
    #[serde(rename = "invalidFileCount")]
    pub invalid_file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInboxResolutionStats {
    #[serde(rename = "candidateCount")]
    pub candidate_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInboxCandidate {
    #[serde(rename = "indexKey")]
    pub index_key: String,
    #[serde(rename = "manifestPath")]
    pub manifest_path: String,
    #[serde(rename = "artifactPath", skip_serializing_if = "Option::is_none")]
    pub artifact_path: Option<String>,
    #[serde(rename = "generatedAtUnixMs")]
    pub generated_at_unix_ms: u64,
    #[serde(rename = "queueItemId", skip_serializing_if = "Option::is_none")]
    pub queue_item_id: Option<String>,
    #[serde(rename = "schemaFingerprint")]
    pub schema_fingerprint: String,
    #[serde(rename = "resolutionStatus")]
    pub resolution_status: String,
}

pub fn parse_args<I>(args: I) -> Result<QueueInboxResolverArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = QueueInboxResolverArgs::default();
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
        "centipede_queue_inbox_resolver",
        "",
        "Consume a manifest scan report and emit a ForgeCommand-ready inbox resolution payload.",
        "",
        "Usage:",
        "  cargo run --bin centipede_queue_inbox_resolver -- --input /tmp/centipede_manifest_scan.json --output /tmp/centipede_inbox_resolution.json --pretty",
        "",
        "Arguments:",
        "  --input <path>      Read manifest scan report JSON from a file. Use '-' or omit to read stdin.",
        "  --output <path>     Write inbox resolution JSON to a file. Omit to write stdout.",
        "  --pretty            Pretty-print output JSON.",
        "  --help, -h          Show this help text.",
    ]
    .join("\n")
}

pub fn run_inbox_resolver(args: QueueInboxResolverArgs) -> Result<QueueInboxResolution, String> {
    let scan_report = read_json_input(args.input_path.as_deref())?;
    let resolution = build_inbox_resolution(&scan_report)?;
    write_resolution_output(&resolution, args.output_path.as_deref(), args.pretty)?;
    Ok(resolution)
}

pub fn build_inbox_resolution(scan_report: &Value) -> Result<QueueInboxResolution, String> {
    let report_obj = scan_report
        .as_object()
        .ok_or_else(|| "manifest scan report input must be a JSON object".to_string())?;

    let kind = required_string(report_obj, "kind")?;
    if kind != "centipede_queue_handoff_manifest_scan" {
        return Err(format!(
            "scan report kind must be 'centipede_queue_handoff_manifest_scan', found '{}'",
            kind
        ));
    }

    let schema_version = report_obj
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| "scan report schemaVersion must be an unsigned integer".to_string())?;
    if schema_version != 1 {
        return Err(format!(
            "scan report schemaVersion must be 1, found {}",
            schema_version
        ));
    }

    let scanned_dir = required_string(report_obj, "scannedDir")?.to_string();
    let manifest_count = report_obj
        .get("manifestCount")
        .and_then(Value::as_u64)
        .ok_or_else(|| "scan report manifestCount must be an unsigned integer".to_string())?
        as usize;
    let invalid_file_count = report_obj
        .get("invalidFileCount")
        .and_then(Value::as_u64)
        .ok_or_else(|| "scan report invalidFileCount must be an unsigned integer".to_string())?
        as usize;

    let latest_candidates = report_obj
        .get("latestAcceptedByIndexKey")
        .and_then(Value::as_array)
        .ok_or_else(|| "scan report latestAcceptedByIndexKey must be an array".to_string())?;

    let mut candidates = Vec::new();
    for entry in latest_candidates {
        let entry_obj = entry
            .as_object()
            .ok_or_else(|| "candidate entry must be an object".to_string())?;
        candidates.push(QueueInboxCandidate {
            index_key: required_string(entry_obj, "indexKey")?.to_string(),
            manifest_path: required_string(entry_obj, "manifestPath")?.to_string(),
            artifact_path: entry_obj
                .get("artifactPath")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            generated_at_unix_ms: entry_obj
                .get("generatedAtUnixMs")
                .and_then(Value::as_u64)
                .ok_or_else(|| "candidate.generatedAtUnixMs must be an unsigned integer".to_string())?,
            queue_item_id: entry_obj
                .get("queueItemId")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            schema_fingerprint: required_string(entry_obj, "schemaFingerprint")?.to_string(),
            resolution_status: "ready".to_string(),
        });
    }

    Ok(QueueInboxResolution {
        kind: "centipede_queue_inbox_resolution".to_string(),
        schema_version: 1,
        resolved_at_unix_ms: current_unix_ms()?,
        source_scan: QueueInboxSourceScan {
            scanned_dir,
            manifest_count,
            invalid_file_count,
        },
        stats: QueueInboxResolutionStats {
            candidate_count: candidates.len(),
        },
        candidates,
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

pub fn write_resolution_output(
    resolution: &QueueInboxResolution,
    output_path: Option<&Path>,
    pretty: bool,
) -> Result<(), String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(resolution)
            .map_err(|err| format!("failed to serialize inbox resolution: {err}"))?
    } else {
        serde_json::to_string(resolution)
            .map_err(|err| format!("failed to serialize inbox resolution: {err}"))?
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

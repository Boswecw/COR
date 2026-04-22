use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct QueueHandoffManifestArgs {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub artifact_path: Option<String>,
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffManifest {
    pub kind: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u64,
    #[serde(rename = "generatedAtUnixMs")]
    pub generated_at_unix_ms: u128,
    pub discovery: QueueHandoffManifestDiscovery,
    pub artifact: QueueHandoffManifestArtifact,
    pub source: QueueHandoffManifestSource,
    pub summary: QueueHandoffManifestSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffManifestDiscovery {
    #[serde(rename = "targetSystem")]
    pub target_system: String,
    #[serde(rename = "scanClass")]
    pub scan_class: String,
    #[serde(rename = "indexKey")]
    pub index_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffManifestArtifact {
    pub path: Option<String>,
    pub kind: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u64,
    #[serde(rename = "generatedAtUnixMs")]
    pub generated_at_unix_ms: u128,
    #[serde(rename = "postureStatus")]
    pub posture_status: String,
    #[serde(rename = "reasonsCount")]
    pub reasons_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffManifestSource {
    #[serde(rename = "contractType")]
    pub contract_type: String,
    #[serde(rename = "contractVersion")]
    pub contract_version: String,
    #[serde(rename = "schemaFingerprint")]
    pub schema_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueHandoffManifestSummary {
    #[serde(rename = "queueDir")]
    pub queue_dir: String,
    #[serde(rename = "queueItemId", skip_serializing_if = "Option::is_none")]
    pub queue_item_id: Option<String>,
    #[serde(rename = "itemCount")]
    pub item_count: usize,
}

pub fn parse_args<I>(args: I) -> Result<QueueHandoffManifestArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = QueueHandoffManifestArgs::default();
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
            "--artifact-path" => {
                parsed.artifact_path = Some(
                    iter.next()
                        .ok_or_else(|| "missing value after --artifact-path".to_string())?,
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
        "centipede_queue_handoff_manifest",
        "",
        "Read a ForgeCommand handoff artifact and emit a directory-scannable manifest/index record.",
        "",
        "Usage:",
        "  cargo run --bin centipede_queue_handoff_manifest -- --input /tmp/centipede_queue_handoff_alpha.json --output /tmp/centipede_queue_handoff_alpha.manifest.json --pretty",
        "",
        "Arguments:",
        "  --input <path>          Read handoff artifact JSON from a file. Use '-' or omit to read stdin.",
        "  --output <path>         Write manifest JSON to a file. Omit to write stdout.",
        "  --artifact-path <path>  Override the artifact path recorded in the manifest.",
        "  --pretty                Pretty-print output JSON.",
        "  --help, -h              Show this help text.",
    ]
    .join("\n")
}

pub fn run_handoff_manifest(args: QueueHandoffManifestArgs) -> Result<QueueHandoffManifest, String> {
    let input_path_string = args
        .input_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let artifact = read_json_input(args.input_path.as_deref())?;
    let manifest = build_handoff_manifest(&artifact, args.artifact_path.or(input_path_string))?;
    write_manifest_output(&manifest, args.output_path.as_deref(), args.pretty)?;
    Ok(manifest)
}

pub fn build_handoff_manifest(
    artifact: &Value,
    artifact_path: Option<String>,
) -> Result<QueueHandoffManifest, String> {
    let artifact_obj = artifact
        .as_object()
        .ok_or_else(|| "handoff artifact input must be a JSON object".to_string())?;

    let artifact_kind = required_string(artifact_obj, "kind")?;
    if artifact_kind != "centipede_queue_handoff_artifact" {
        return Err(format!(
            "artifact.kind must be 'centipede_queue_handoff_artifact', found '{}'",
            artifact_kind
        ));
    }

    let artifact_schema_version = artifact_obj
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| "artifact.schemaVersion must be an unsigned integer".to_string())?;

    let artifact_generated_at_unix_ms = artifact_obj
        .get("generatedAtUnixMs")
        .and_then(Value::as_u64)
        .ok_or_else(|| "artifact.generatedAtUnixMs must be an unsigned integer".to_string())?
        as u128;

    let posture_obj = required_object(artifact_obj, "posture")?;
    let posture_status = required_string(posture_obj, "status")?.to_string();
    let reasons_count = posture_obj
        .get("reasons")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    let source_obj = required_object(artifact_obj, "source")?;
    let contract_type = required_string(source_obj, "contractType")?.to_string();
    let contract_version = required_string(source_obj, "contractVersion")?.to_string();
    let schema_fingerprint = required_string(source_obj, "schemaFingerprint")?.to_string();

    let summary_obj = required_object(artifact_obj, "summary")?;
    let queue_dir = required_string(summary_obj, "queueDir")?.to_string();
    let item_count = summary_obj
        .get("itemCount")
        .and_then(Value::as_u64)
        .ok_or_else(|| "summary.itemCount must be an unsigned integer".to_string())?
        as usize;
    let queue_item_id = summary_obj
        .get("selection")
        .and_then(Value::as_object)
        .and_then(|selection| selection.get("queueItemId"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let selection_key = queue_item_id
        .clone()
        .unwrap_or_else(|| "ALL".to_string());
    let index_key = format!(
        "{}|{}|{}",
        posture_status, schema_fingerprint, selection_key
    );

    Ok(QueueHandoffManifest {
        kind: "centipede_queue_handoff_manifest".to_string(),
        schema_version: 1,
        generated_at_unix_ms: current_unix_ms()?,
        discovery: QueueHandoffManifestDiscovery {
            target_system: "ForgeCommand".to_string(),
            scan_class: "centipede_queue_handoff_manifest".to_string(),
            index_key,
        },
        artifact: QueueHandoffManifestArtifact {
            path: artifact_path,
            kind: artifact_kind.to_string(),
            schema_version: artifact_schema_version,
            generated_at_unix_ms: artifact_generated_at_unix_ms,
            posture_status,
            reasons_count,
        },
        source: QueueHandoffManifestSource {
            contract_type,
            contract_version,
            schema_fingerprint,
        },
        summary: QueueHandoffManifestSummary {
            queue_dir,
            queue_item_id,
            item_count,
        },
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

pub fn write_manifest_output(
    manifest: &QueueHandoffManifest,
    output_path: Option<&Path>,
    pretty: bool,
) -> Result<(), String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(manifest)
            .map_err(|err| format!("failed to serialize handoff manifest: {err}"))?
    } else {
        serde_json::to_string(manifest)
            .map_err(|err| format!("failed to serialize handoff manifest: {err}"))?
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

fn required_object<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Map<String, Value>, String> {
    map.get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("field '{}' must be an object", key))
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

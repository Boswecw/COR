use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct QueueManifestScanArgs {
    pub input_dir: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueManifestScanReport {
    pub kind: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u64,
    #[serde(rename = "scannedDir")]
    pub scanned_dir: String,
    #[serde(rename = "manifestCount")]
    pub manifest_count: usize,
    #[serde(rename = "invalidFileCount")]
    pub invalid_file_count: usize,
    #[serde(rename = "latestAcceptedByIndexKey")]
    pub latest_accepted_by_index_key: Vec<AcceptedManifestSelection>,
    #[serde(rename = "invalidFiles")]
    pub invalid_files: Vec<InvalidManifestFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedManifestSelection {
    #[serde(rename = "indexKey")]
    pub index_key: String,
    #[serde(rename = "manifestPath")]
    pub manifest_path: String,
    #[serde(rename = "artifactPath", skip_serializing_if = "Option::is_none")]
    pub artifact_path: Option<String>,
    #[serde(rename = "generatedAtUnixMs")]
    pub generated_at_unix_ms: u64,
    #[serde(rename = "postureStatus")]
    pub posture_status: String,
    #[serde(rename = "queueItemId", skip_serializing_if = "Option::is_none")]
    pub queue_item_id: Option<String>,
    #[serde(rename = "schemaFingerprint")]
    pub schema_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidManifestFile {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone)]
struct ParsedManifest {
    path: String,
    index_key: String,
    generated_at_unix_ms: u64,
    posture_status: String,
    artifact_path: Option<String>,
    queue_item_id: Option<String>,
    schema_fingerprint: String,
}

pub fn parse_args<I>(args: I) -> Result<QueueManifestScanArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = QueueManifestScanArgs::default();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--input-dir" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value after --input-dir".to_string())?;
                parsed.input_dir = Some(PathBuf::from(value));
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

    if parsed.input_dir.is_none() {
        return Err(format!("missing required --input-dir\n\n{}", help_text()));
    }

    Ok(parsed)
}

pub fn help_text() -> String {
    [
        "centipede_queue_manifest_scan",
        "",
        "Scan a directory of handoff manifests and surface the latest accepted artifact per index key.",
        "",
        "Usage:",
        "  cargo run --bin centipede_queue_manifest_scan -- --input-dir /tmp/centipede-manifests --output /tmp/centipede-manifest-scan.json --pretty",
        "",
        "Arguments:",
        "  --input-dir <path>   Directory containing handoff manifest JSON files.",
        "  --output <path>      Write scan report JSON to a file. Omit to write stdout.",
        "  --pretty             Pretty-print output JSON.",
        "  --help, -h           Show this help text.",
    ]
    .join("\n")
}

pub fn run_manifest_scan(args: QueueManifestScanArgs) -> Result<QueueManifestScanReport, String> {
    let input_dir = args
        .input_dir
        .as_deref()
        .ok_or_else(|| "input_dir is required".to_string())?;
    let report = scan_manifest_directory(input_dir)?;
    write_scan_output(&report, args.output_path.as_deref(), args.pretty)?;
    Ok(report)
}

pub fn scan_manifest_directory(input_dir: &Path) -> Result<QueueManifestScanReport, String> {
    if !input_dir.exists() {
        return Err(format!(
            "input directory '{}' does not exist",
            input_dir.display()
        ));
    }

    if !input_dir.is_dir() {
        return Err(format!(
            "input path '{}' is not a directory",
            input_dir.display()
        ));
    }

    let mut parsed_manifests = Vec::new();
    let mut invalid_files = Vec::new();

    let mut entries: Vec<PathBuf> = fs::read_dir(input_dir)
        .map_err(|err| format!("failed to read input directory '{}': {err}", input_dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .collect();
    entries.sort();

    for path in entries {
        match parse_manifest_file(&path) {
            Ok(manifest) => parsed_manifests.push(manifest),
            Err(err) => invalid_files.push(InvalidManifestFile {
                path: path.to_string_lossy().to_string(),
                error: err,
            }),
        }
    }

    let mut latest_by_key: BTreeMap<String, AcceptedManifestSelection> = BTreeMap::new();

    for manifest in &parsed_manifests {
        if manifest.posture_status != "accepted" {
            continue;
        }

        let candidate = AcceptedManifestSelection {
            index_key: manifest.index_key.clone(),
            manifest_path: manifest.path.clone(),
            artifact_path: manifest.artifact_path.clone(),
            generated_at_unix_ms: manifest.generated_at_unix_ms,
            posture_status: manifest.posture_status.clone(),
            queue_item_id: manifest.queue_item_id.clone(),
            schema_fingerprint: manifest.schema_fingerprint.clone(),
        };

        match latest_by_key.get(&manifest.index_key) {
            Some(existing) if existing.generated_at_unix_ms >= candidate.generated_at_unix_ms => {}
            _ => {
                latest_by_key.insert(manifest.index_key.clone(), candidate);
            }
        }
    }

    Ok(QueueManifestScanReport {
        kind: "centipede_queue_handoff_manifest_scan".to_string(),
        schema_version: 1,
        scanned_dir: input_dir.to_string_lossy().to_string(),
        manifest_count: parsed_manifests.len(),
        invalid_file_count: invalid_files.len(),
        latest_accepted_by_index_key: latest_by_key.into_values().collect(),
        invalid_files,
    })
}

fn parse_manifest_file(path: &Path) -> Result<ParsedManifest, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read file '{}': {err}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .map_err(|err| format!("invalid JSON in '{}': {err}", path.display()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| format!("manifest '{}' must be a JSON object", path.display()))?;

    let kind = required_string(obj, "kind")?;
    if kind != "centipede_queue_handoff_manifest" {
        return Err(format!(
            "manifest '{}' kind must be 'centipede_queue_handoff_manifest', found '{}'",
            path.display(),
            kind
        ));
    }

    let schema_version = obj
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("manifest '{}' schemaVersion must be an unsigned integer", path.display()))?;
    if schema_version != 1 {
        return Err(format!(
            "manifest '{}' schemaVersion must be 1, found {}",
            path.display(),
            schema_version
        ));
    }

    let discovery = required_object(obj, "discovery")?;
    let index_key = required_string(discovery, "indexKey")?.to_string();

    let artifact = required_object(obj, "artifact")?;
    let generated_at_unix_ms = artifact
        .get("generatedAtUnixMs")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("manifest '{}' artifact.generatedAtUnixMs must be an unsigned integer", path.display()))?;
    let posture_status = required_string(artifact, "postureStatus")?.to_string();
    let artifact_path = artifact
        .get("path")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let source = required_object(obj, "source")?;
    let schema_fingerprint = required_string(source, "schemaFingerprint")?.to_string();

    let summary = required_object(obj, "summary")?;
    let queue_item_id = summary
        .get("queueItemId")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    Ok(ParsedManifest {
        path: path.to_string_lossy().to_string(),
        index_key,
        generated_at_unix_ms,
        posture_status,
        artifact_path,
        queue_item_id,
        schema_fingerprint,
    })
}

pub fn write_scan_output(
    report: &QueueManifestScanReport,
    output_path: Option<&Path>,
    pretty: bool,
) -> Result<(), String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(report)
            .map_err(|err| format!("failed to serialize manifest scan report: {err}"))?
    } else {
        serde_json::to_string(report)
            .map_err(|err| format!("failed to serialize manifest scan report: {err}"))?
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

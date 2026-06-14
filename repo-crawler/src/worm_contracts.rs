
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct WormContractSet {
    pub label: &'static str,
    pub glob_prefix: &'static str,
    pub required_kind: &'static str,
}

pub const WORM_CONTRACT_SETS: &[WormContractSet] = &[
    WormContractSet {
        label: "edge examples",
        glob_prefix: "edge_",
        required_kind: "worm_edge",
    },
    WormContractSet {
        label: "finding examples",
        glob_prefix: "finding_",
        required_kind: "worm_finding",
    },
    WormContractSet {
        label: "boundary policy examples",
        glob_prefix: "boundary_policy_",
        required_kind: "worm_traversal_policy",
    },
    WormContractSet {
        label: "adapter emission examples",
        glob_prefix: "adapter_emit_",
        required_kind: "worm_adapter_emission",
    },
    WormContractSet {
        label: "target resolution examples",
        glob_prefix: "target_resolution_",
        required_kind: "worm_target_resolution",
    },
    WormContractSet {
        label: "evidence bundle examples",
        glob_prefix: "evidence_bundle_",
        required_kind: "worm_evidence_bundle",
    },
    WormContractSet {
        label: "Centipede handoff examples",
        glob_prefix: "centipede_handoff_",
        required_kind: "worm_centipede_handoff",
    },
    WormContractSet {
        label: "issue catalog examples",
        glob_prefix: "issue_catalog_",
        required_kind: "worm_reason_code_catalog",
    },
];

pub fn worm_examples_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("doc/system/worm/examples")
}

pub fn load_json_file(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    serde_json::from_str::<Value>(&text)
        .map_err(|e| format!("failed to parse JSON {}: {}", path.display(), e))
}

pub fn find_matching_files(
    examples_dir: &Path,
    prefix: &str,
) -> Result<Vec<PathBuf>, String> {
    let mut matches = Vec::new();

    let entries = fs::read_dir(examples_dir)
        .map_err(|e| format!("failed to read examples directory {}: {}", examples_dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {}", e))?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if name.starts_with(prefix) && name.ends_with(".json") {
            matches.push(path);
        }
    }

    matches.sort();
    Ok(matches)
}

pub fn validate_contract_set(repo_root: &Path, set: &WormContractSet) -> Result<usize, String> {
    let examples_dir = worm_examples_dir(repo_root);
    let files = find_matching_files(&examples_dir, set.glob_prefix)?;

    if files.is_empty() {
        return Err(format!(
            "no files found for {} with prefix '{}' in {}",
            set.label,
            set.glob_prefix,
            examples_dir.display()
        ));
    }

    for path in &files {
        let value = load_json_file(path)?;
        let kind = value
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{} missing string field 'kind'", path.display()))?;

        if kind != set.required_kind {
            return Err(format!(
                "{} expected kind '{}' but found '{}'",
                path.display(),
                set.required_kind,
                kind
            ));
        }

        let schema_version = value
            .get("schemaVersion")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| format!("{} missing integer field 'schemaVersion'", path.display()))?;

        if schema_version != 1 {
            return Err(format!(
                "{} expected schemaVersion 1 but found {}",
                path.display(),
                schema_version
            ));
        }
    }

    Ok(files.len())
}

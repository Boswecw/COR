
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn examples_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("doc/system/worm/examples")
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {}", path.display(), e))
}

fn matching_files(repo_root: &Path, prefix: &str) -> Result<Vec<PathBuf>, String> {
    let dir = examples_dir(repo_root);
    let mut files = Vec::new();

    for entry in fs::read_dir(&dir)
        .map_err(|e| format!("failed to read examples directory {}: {}", dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {}", e))?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if name.starts_with(prefix) && name.ends_with(".json") {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn as_array<'a>(value: &'a Value, key: &str, label: &str) -> Result<&'a Vec<Value>, String> {
    value.get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{label}: missing array field '{key}'"))
}

fn as_str<'a>(value: &'a Value, key: &str, label: &str) -> Result<&'a str, String> {
    value.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{label}: missing string field '{key}'"))
}

pub fn run_reference_audit(repo_root: &Path) -> Result<(), String> {
    let mut catalog_map: HashMap<String, HashSet<String>> = HashMap::new();
    let mut bundle_ids: HashSet<String> = HashSet::new();

    for path in matching_files(repo_root, "issue_catalog_")? {
        let value = read_json(&path)?;
        let label = path.display().to_string();

        let issue_class = as_str(&value, "issueClass", &label)?.to_string();
        let reason_codes = as_array(&value, "reasonCodes", &label)?;

        let entry = catalog_map.entry(issue_class).or_default();
        for item in reason_codes {
            let Some(code) = item.get("code").and_then(|v| v.as_str()) else {
                return Err(format!("{label}: reasonCodes entry missing string field 'code'"));
            };
            entry.insert(code.to_string());
        }
    }

    if catalog_map.is_empty() {
        return Err("no issue catalogs loaded".to_string());
    }

    for path in matching_files(repo_root, "evidence_bundle_")? {
        let value = read_json(&path)?;
        let label = path.display().to_string();

        let bundle_id = as_str(&value, "bundleId", &label)?.to_string();
        bundle_ids.insert(bundle_id);

        let findings = as_array(&value, "findings", &label)?;
        for (idx, finding) in findings.iter().enumerate() {
            let item_label = format!("{label} findings[{idx}]");
            let finding_class = as_str(finding, "findingClass", &item_label)?;
            let reason_code = as_str(finding, "reasonCode", &item_label)?;

            let Some(allowed_codes) = catalog_map.get(finding_class) else {
                return Err(format!(
                    "{item_label}: findingClass '{}' not found in issue catalogs",
                    finding_class
                ));
            };

            if !allowed_codes.contains(reason_code) {
                return Err(format!(
                    "{item_label}: reasonCode '{}' not declared for findingClass '{}'",
                    reason_code,
                    finding_class
                ));
            }
        }
    }

    if bundle_ids.is_empty() {
        return Err("no evidence bundles loaded".to_string());
    }

    for path in matching_files(repo_root, "centipede_handoff_")? {
        let value = read_json(&path)?;
        let label = path.display().to_string();

        let refs = as_array(&value, "bundleIds", &label)?;
        for (idx, item) in refs.iter().enumerate() {
            let Some(bundle_id) = item.as_str() else {
                return Err(format!("{label}: bundleIds[{idx}] must be a string"));
            };

            if !bundle_ids.contains(bundle_id) {
                return Err(format!(
                    "{label}: referenced bundleId '{}' not found among evidence bundles",
                    bundle_id
                ));
            }
        }
    }

    Ok(())
}

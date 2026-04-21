use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn claim_next_queue_item(queue_dir: &Path, claimant: &str, claimed_at: &str) -> Result<Value, String> {
    if claimant.trim().is_empty() {
        return Err("claimant must not be empty".to_string());
    }
    if claimed_at.trim().is_empty() {
        return Err("claimed_at must not be empty".to_string());
    }

    let items_dir = queue_dir.join("items");
    if !items_dir.exists() {
        return Err(format!("queue items directory does not exist: {}", items_dir.display()));
    }

    let claims_dir = queue_dir.join("claims");
    fs::create_dir_all(&claims_dir)
        .map_err(|err| format!("could not create {}: {}", claims_dir.display(), err))?;

    let mut item_paths: Vec<PathBuf> = fs::read_dir(&items_dir)
        .map_err(|err| format!("could not read {}: {}", items_dir.display(), err))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("json"))
        .collect();
    item_paths.sort();

    for item_path in item_paths {
        let item = read_json(&item_path)?;
        let queue_item_id = required_string(&item, "queueItemId")?;
        let claim_id = queue_item_id_to_claim_id(queue_item_id);
        let claim_path = claims_dir.join(format!("{}.json", claim_id));
        if claim_path.exists() {
            continue;
        }

        let claim = build_claim(&item, queue_item_id, claimant, claimed_at, &item_path)?;
        write_json(&claim_path, &claim)?;

        let index_path = claims_dir.join("index.json");
        let index = build_claims_index(&claims_dir)?;
        write_json(&index_path, &index)?;

        return Ok(json!({
            "kind": "centipede_queue_claim_receipt",
            "schemaVersion": 1,
            "queueDir": queue_dir.display().to_string(),
            "claimsDir": claims_dir.display().to_string(),
            "disposition": "claimed",
            "claimId": claim_id,
            "queueItemId": queue_item_id,
            "claimant": claimant,
            "claimedAt": claimed_at,
            "sourceRepo": required_string(&item, "sourceRepo")?,
            "intakeKind": required_string(&item, "intakeKind")?,
            "blocking": item
                .get("totals")
                .and_then(|totals| totals.get("blocking"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        }));
    }

    let index_path = claims_dir.join("index.json");
    let index = build_claims_index(&claims_dir)?;
    write_json(&index_path, &index)?;

    Ok(json!({
        "kind": "centipede_queue_claim_receipt",
        "schemaVersion": 1,
        "queueDir": queue_dir.display().to_string(),
        "claimsDir": claims_dir.display().to_string(),
        "disposition": "empty",
        "claimant": claimant,
        "claimedAt": claimed_at
    }))
}

fn build_claim(
    item: &Value,
    queue_item_id: &str,
    claimant: &str,
    claimed_at: &str,
    item_path: &Path,
) -> Result<Value, String> {
    let claim_id = queue_item_id_to_claim_id(queue_item_id);
    Ok(json!({
        "kind": "centipede_queue_claim",
        "schemaVersion": 1,
        "claimId": claim_id,
        "queueItemId": queue_item_id,
        "queueItemPath": item_path.display().to_string(),
        "claimant": claimant,
        "claimedAt": claimed_at,
        "intakeKind": required_string(item, "intakeKind")?,
        "sourceLane": required_string(item, "sourceLane")?,
        "sourceRepo": required_string(item, "sourceRepo")?,
        "sourceHandoffId": required_string(item, "sourceHandoffId")?,
        "receivedAt": required_string(item, "receivedAt")?,
        "candidateIssueKeys": extract_issue_keys(item)?,
        "totals": item
            .get("totals")
            .cloned()
            .ok_or_else(|| "missing field: totals".to_string())?
    }))
}

fn build_claims_index(claims_dir: &Path) -> Result<Value, String> {
    let mut claim_paths: Vec<PathBuf> = if claims_dir.exists() {
        fs::read_dir(claims_dir)
            .map_err(|err| format!("could not read {}: {}", claims_dir.display(), err))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.extension().and_then(|v| v.to_str()) == Some("json")
                    && path.file_name().and_then(|v| v.to_str()) != Some("index.json")
            })
            .collect()
    } else {
        Vec::new()
    };
    claim_paths.sort();

    let mut items = Vec::new();
    for claim_path in claim_paths {
        let value = read_json(&claim_path)?;
        items.push(json!({
            "claimId": required_string(&value, "claimId")?,
            "queueItemId": required_string(&value, "queueItemId")?,
            "claimant": required_string(&value, "claimant")?,
            "claimedAt": required_string(&value, "claimedAt")?,
            "intakeKind": required_string(&value, "intakeKind")?,
            "sourceRepo": required_string(&value, "sourceRepo")?,
            "blocking": value
                .get("totals")
                .and_then(|totals| totals.get("blocking"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        }));
    }

    Ok(json!({
        "kind": "centipede_queue_claim_index",
        "schemaVersion": 1,
        "claimsDir": claims_dir.display().to_string(),
        "items": items,
        "totals": {
            "claims": items.len(),
            "blocking": items.iter().map(summary_blocking).sum::<u64>()
        }
    }))
}

fn extract_issue_keys(item: &Value) -> Result<Vec<String>, String> {
    let issues = item
        .get("candidateIssues")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing array field: candidateIssues".to_string())?;

    let mut keys = Vec::with_capacity(issues.len());
    for issue in issues {
        let object = issue
            .as_object()
            .ok_or_else(|| "candidateIssues must contain objects".to_string())?;
        let key = object
            .get("issueKey")
            .and_then(Value::as_str)
            .ok_or_else(|| "candidateIssues item missing string field: issueKey".to_string())?;
        keys.push(key.to_string());
    }
    Ok(keys)
}

fn queue_item_id_to_claim_id(queue_item_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(queue_item_id.as_bytes());
    let digest = hasher.finalize();
    let hex = hex_string(&digest);
    format!("cqclaim-{}", &hex[..24])
}

fn summary_blocking(value: &Value) -> u64 {
    value.get("blocking").and_then(Value::as_u64).unwrap_or(0)
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {}", path.display(), err))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("could not parse {}: {}", path.display(), err))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let pretty = serde_json::to_string_pretty(value)
        .map_err(|err| format!("could not serialize {}: {}", path.display(), err))?;
    fs::write(path, format!("{}\n", pretty))
        .map_err(|err| format!("could not write {}: {}", path.display(), err))
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field: {}", key))
}

fn hex_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

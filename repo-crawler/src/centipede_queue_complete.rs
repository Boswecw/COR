use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn complete_from_claim_receipt(
    queue_dir: &Path,
    claim_receipt: &Value,
    completed_at: &str,
    resolution: &str,
) -> Result<Value, String> {
    if completed_at.trim().is_empty() {
        return Err("completed_at must not be empty".to_string());
    }
    if resolution.trim().is_empty() {
        return Err("resolution must not be empty".to_string());
    }

    let disposition = required_string(claim_receipt, "disposition")?;
    if disposition != "claimed" {
        return Err(format!(
            "claim receipt disposition must be 'claimed', got '{}'",
            disposition
        ));
    }

    let claim_id = required_string(claim_receipt, "claimId")?;
    let queue_item_id = required_string(claim_receipt, "queueItemId")?;

    let claims_dir = queue_dir.join("claims");
    let claim_path = claims_dir.join(format!("{}.json", claim_id));
    if !claim_path.exists() {
        return Err(format!("claim file does not exist: {}", claim_path.display()));
    }
    let claim = read_json(&claim_path)?;

    let item_path = PathBuf::from(required_string(&claim, "queueItemPath")?);
    if !item_path.exists() {
        return Err(format!("queue item file does not exist: {}", item_path.display()));
    }

    let completion_id = claim_id_to_completion_id(claim_id);
    let completions_dir = queue_dir.join("completions");
    fs::create_dir_all(&completions_dir)
        .map_err(|err| format!("could not create {}: {}", completions_dir.display(), err))?;

    let completion_path = completions_dir.join(format!("{}.json", completion_id));
    if completion_path.exists() {
        let existing = read_json(&completion_path)?;
        let index = build_completion_index(&completions_dir)?;
        write_json(&completions_dir.join("index.json"), &index)?;

        return Ok(json!({
            "kind": "centipede_queue_complete_receipt",
            "schemaVersion": 1,
            "queueDir": queue_dir.display().to_string(),
            "completionsDir": completions_dir.display().to_string(),
            "disposition": "already_completed",
            "completionId": completion_id,
            "claimId": claim_id,
            "queueItemId": queue_item_id,
            "completedAt": existing.get("completedAt").and_then(Value::as_str).unwrap_or(completed_at),
            "resolution": existing.get("resolution").and_then(Value::as_str).unwrap_or(resolution),
            "sourceRepo": required_string(&claim, "sourceRepo")?,
            "intakeKind": required_string(&claim, "intakeKind")?,
            "blocking": claim
                .get("totals")
                .and_then(|totals| totals.get("blocking"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        }));
    }

    let completion = build_completion(&claim, claim_id, queue_item_id, completed_at, resolution)?;
    write_json(&completion_path, &completion)?;
    update_queue_item_processing_state(&item_path, &completion)?;
    update_claim_with_completion(&claim_path, &claim, &completion)?;
    let index = build_completion_index(&completions_dir)?;
    write_json(&completions_dir.join("index.json"), &index)?;

    Ok(json!({
        "kind": "centipede_queue_complete_receipt",
        "schemaVersion": 1,
        "queueDir": queue_dir.display().to_string(),
        "completionsDir": completions_dir.display().to_string(),
        "disposition": "completed",
        "completionId": completion_id,
        "claimId": claim_id,
        "queueItemId": queue_item_id,
        "completedAt": completed_at,
        "resolution": resolution,
        "sourceRepo": required_string(&claim, "sourceRepo")?,
        "intakeKind": required_string(&claim, "intakeKind")?,
        "blocking": claim
            .get("totals")
            .and_then(|totals| totals.get("blocking"))
            .and_then(Value::as_u64)
            .unwrap_or(0)
    }))
}

fn build_completion(
    claim: &Value,
    claim_id: &str,
    queue_item_id: &str,
    completed_at: &str,
    resolution: &str,
) -> Result<Value, String> {
    let completion_id = claim_id_to_completion_id(claim_id);
    Ok(json!({
        "kind": "centipede_queue_completion",
        "schemaVersion": 1,
        "completionId": completion_id,
        "claimId": claim_id,
        "queueItemId": queue_item_id,
        "claimant": required_string(claim, "claimant")?,
        "claimedAt": required_string(claim, "claimedAt")?,
        "completedAt": completed_at,
        "resolution": resolution,
        "queueItemPath": required_string(claim, "queueItemPath")?,
        "sourceRepo": required_string(claim, "sourceRepo")?,
        "sourceLane": required_string(claim, "sourceLane")?,
        "sourceHandoffId": required_string(claim, "sourceHandoffId")?,
        "intakeKind": required_string(claim, "intakeKind")?,
        "receivedAt": required_string(claim, "receivedAt")?,
        "candidateIssueKeys": claim
            .get("candidateIssueKeys")
            .cloned()
            .ok_or_else(|| "missing field: candidateIssueKeys".to_string())?,
        "totals": claim
            .get("totals")
            .cloned()
            .ok_or_else(|| "missing field: totals".to_string())?
    }))
}

fn update_queue_item_processing_state(item_path: &Path, completion: &Value) -> Result<(), String> {
    let mut item = read_json(item_path)?;
    let object = item
        .as_object_mut()
        .ok_or_else(|| format!("queue item is not an object: {}", item_path.display()))?;

    object.insert(
        "processingState".to_string(),
        json!({
            "status": "completed",
            "claimId": required_string(completion, "claimId")?,
            "completionId": required_string(completion, "completionId")?,
            "claimant": required_string(completion, "claimant")?,
            "claimedAt": required_string(completion, "claimedAt")?,
            "completedAt": required_string(completion, "completedAt")?,
            "resolution": required_string(completion, "resolution")?
        }),
    );

    write_json(item_path, &item)
}

fn update_claim_with_completion(claim_path: &Path, claim: &Value, completion: &Value) -> Result<(), String> {
    let mut updated = claim.clone();
    let object = updated
        .as_object_mut()
        .ok_or_else(|| format!("claim is not an object: {}", claim_path.display()))?;

    object.insert(
        "completion".to_string(),
        json!({
            "completionId": required_string(completion, "completionId")?,
            "completedAt": required_string(completion, "completedAt")?,
            "resolution": required_string(completion, "resolution")?
        }),
    );

    write_json(claim_path, &updated)
}

fn build_completion_index(completions_dir: &Path) -> Result<Value, String> {
    let mut completion_paths: Vec<PathBuf> = if completions_dir.exists() {
        fs::read_dir(completions_dir)
            .map_err(|err| format!("could not read {}: {}", completions_dir.display(), err))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.extension().and_then(|v| v.to_str()) == Some("json")
                    && path.file_name().and_then(|v| v.to_str()) != Some("index.json")
            })
            .collect()
    } else {
        Vec::new()
    };
    completion_paths.sort();

    let mut items = Vec::new();
    for completion_path in completion_paths {
        let value = read_json(&completion_path)?;
        items.push(json!({
            "completionId": required_string(&value, "completionId")?,
            "claimId": required_string(&value, "claimId")?,
            "queueItemId": required_string(&value, "queueItemId")?,
            "claimant": required_string(&value, "claimant")?,
            "completedAt": required_string(&value, "completedAt")?,
            "resolution": required_string(&value, "resolution")?,
            "sourceRepo": required_string(&value, "sourceRepo")?,
            "intakeKind": required_string(&value, "intakeKind")?,
            "blocking": value
                .get("totals")
                .and_then(|totals| totals.get("blocking"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        }));
    }

    let mut by_resolution = std::collections::BTreeMap::<String, u64>::new();
    for item in &items {
        let resolution = required_string(item, "resolution")?.to_string();
        *by_resolution.entry(resolution).or_insert(0) += 1;
    }

    Ok(json!({
        "kind": "centipede_queue_completion_index",
        "schemaVersion": 1,
        "completionsDir": completions_dir.display().to_string(),
        "items": items,
        "totals": {
            "completions": items.len(),
            "blocking": items.iter().map(summary_blocking).sum::<u64>(),
            "byResolution": by_resolution
        }
    }))
}

fn claim_id_to_completion_id(claim_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(claim_id.as_bytes());
    let digest = hasher.finalize();
    let hex = hex_string(&digest);
    format!("cqcomplete-{}", &hex[..24])
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

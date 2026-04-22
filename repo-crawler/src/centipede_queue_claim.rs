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
        if item_is_completed(&item) {
            continue;
        }

        let queue_item_id = required_string(&item, "queueItemId")?.to_string();
        let claim_id = queue_item_id_to_claim_id(&queue_item_id);
        let claim_path = claims_dir.join(format!("{}.json", claim_id));

        let existing_claim = if claim_path.exists() {
            Some(read_json(&claim_path)?)
        } else {
            None
        };

        if let Some(claim) = &existing_claim {
            if claim_has_completion(claim) || claim_is_active(claim) {
                continue;
            }
        }

        let claim = build_claim(
            &item,
            &queue_item_id,
            claimant,
            claimed_at,
            &item_path,
            existing_claim.as_ref(),
        )?;
        write_json(&claim_path, &claim)?;
        update_queue_item_claimed_state(&item_path, &claim)?;

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
            "claimAttempt": claim
                .get("claimAttempt")
                .and_then(Value::as_u64)
                .unwrap_or(1),
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
    prior_claim: Option<&Value>,
) -> Result<Value, String> {
    let claim_id = queue_item_id_to_claim_id(queue_item_id);
    let claim_attempt = prior_claim
        .and_then(|value| value.get("claimAttempt"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;

    let mut claim_history = prior_claim
        .and_then(|value| value.get("claimHistory"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if let Some(existing) = prior_claim {
        if claim_has_reclaim(existing) {
            claim_history.push(build_claim_history_entry(existing)?);
        }
    }

    Ok(json!({
        "kind": "centipede_queue_claim",
        "schemaVersion": 1,
        "claimId": claim_id,
        "claimAttempt": claim_attempt,
        "claimHistory": claim_history,
        "queueItemId": queue_item_id,
        "queueItemPath": item_path.display().to_string(),
        "claimant": claimant,
        "claimedAt": claimed_at,
        "lease": {
            "heartbeatCount": 0,
            "lastHeartbeatAt": claimed_at
        },
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

fn build_claim_history_entry(existing_claim: &Value) -> Result<Value, String> {
    Ok(json!({
        "claimAttempt": existing_claim
            .get("claimAttempt")
            .and_then(Value::as_u64)
            .unwrap_or(1),
        "claimant": required_string(existing_claim, "claimant")?,
        "claimedAt": required_string(existing_claim, "claimedAt")?,
        "lease": existing_claim
            .get("lease")
            .cloned()
            .unwrap_or(Value::Null),
        "reclaim": existing_claim
            .get("reclaim")
            .cloned()
            .ok_or_else(|| "missing field: reclaim".to_string())?
    }))
}

fn update_queue_item_claimed_state(item_path: &Path, claim: &Value) -> Result<(), String> {
    let mut item = read_json(item_path)?;
    let object = item
        .as_object_mut()
        .ok_or_else(|| format!("queue item is not an object: {}", item_path.display()))?;

    object.insert(
        "processingState".to_string(),
        json!({
            "status": "claimed",
            "claimId": required_string(claim, "claimId")?,
            "claimAttempt": claim
                .get("claimAttempt")
                .and_then(Value::as_u64)
                .unwrap_or(1),
            "claimant": required_string(claim, "claimant")?,
            "claimedAt": required_string(claim, "claimedAt")?,
            "heartbeatCount": claim
                .get("lease")
                .and_then(|lease| lease.get("heartbeatCount"))
                .and_then(Value::as_u64)
                .unwrap_or(0),
            "lastHeartbeatAt": claim
                .get("lease")
                .and_then(|lease| lease.get("lastHeartbeatAt"))
                .and_then(Value::as_str)
                .unwrap_or(required_string(claim, "claimedAt")?)
        }),
    );

    write_json(item_path, &item)
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
            "claimAttempt": value.get("claimAttempt").and_then(Value::as_u64).unwrap_or(1),
            "state": claim_state(&value),
            "heartbeatCount": value
                .get("lease")
                .and_then(|lease| lease.get("heartbeatCount"))
                .and_then(Value::as_u64)
                .unwrap_or(0),
            "lastHeartbeatAt": value
                .get("lease")
                .and_then(|lease| lease.get("lastHeartbeatAt"))
                .and_then(Value::as_str),
            "leaseTimeoutSeconds": value
                .get("lease")
                .and_then(|lease| lease.get("leaseTimeoutSeconds"))
                .and_then(Value::as_i64),
            "leaseExpiresAtEpochSeconds": value
                .get("lease")
                .and_then(|lease| lease.get("leaseExpiresAtEpochSeconds"))
                .and_then(Value::as_i64),
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
            "blocking": items.iter().map(summary_blocking).sum::<u64>(),
            "active": items.iter().filter(|item| summary_state(item) == "active").count(),
            "completed": items.iter().filter(|item| summary_state(item) == "completed").count(),
            "reclaimed": items.iter().filter(|item| summary_state(item) == "reclaimed").count()
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

fn item_is_completed(item: &Value) -> bool {
    item.get("processingState")
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        == Some("completed")
}

fn claim_state(claim: &Value) -> &'static str {
    if claim_has_completion(claim) {
        "completed"
    } else if claim_has_reclaim(claim) {
        "reclaimed"
    } else {
        "active"
    }
}

fn claim_is_active(claim: &Value) -> bool {
    !claim_has_completion(claim) && !claim_has_reclaim(claim)
}

fn claim_has_completion(claim: &Value) -> bool {
    claim.get("completion").is_some()
}

fn claim_has_reclaim(claim: &Value) -> bool {
    claim.get("reclaim").is_some()
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

fn summary_state<'a>(value: &'a Value) -> &'a str {
    value.get("state").and_then(Value::as_str).unwrap_or("unknown")
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
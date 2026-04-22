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
    let claim_attempt = required_u64(claim_receipt, "claimAttempt")?;
    let queue_item_id = required_string(claim_receipt, "queueItemId")?;
    let receipt_claimant = required_string(claim_receipt, "claimant")?;
    let receipt_claimed_at = required_string(claim_receipt, "claimedAt")?;

    let claims_dir = queue_dir.join("claims");
    let claim_path = claims_dir.join(format!("{}.json", claim_id));
    if !claim_path.exists() {
        return Err(format!("claim file does not exist: {}", claim_path.display()));
    }
    let claim = read_json(&claim_path)?;

    ensure_claim_matches_receipt(
        &claim,
        claim_id,
        claim_attempt,
        queue_item_id,
        receipt_claimant,
        receipt_claimed_at,
        &claim_path,
    )?;

    let completions_dir = queue_dir.join("completions");
    fs::create_dir_all(&completions_dir)
        .map_err(|err| format!("could not create {}: {}", completions_dir.display(), err))?;

    let completion_id = claim_episode_to_completion_id(claim_id, claim_attempt);
    let completion_path = completions_dir.join(format!("{}.json", completion_id));

    if claim.get("completion").is_some() {
        if !completion_path.exists() {
            return Err(format!(
                "claim is completed but completion file is missing: {}",
                completion_path.display()
            ));
        }

        let existing = read_json(&completion_path)?;
        ensure_existing_completion_matches_episode(
            &existing,
            claim_id,
            claim_attempt,
            queue_item_id,
            &completion_path,
        )?;
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
            "claimAttempt": claim_attempt,
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

    ensure_claim_is_active(&claim, &claim_path)?;

    let item_path = PathBuf::from(required_string(&claim, "queueItemPath")?);
    if !item_path.exists() {
        return Err(format!("queue item file does not exist: {}", item_path.display()));
    }

    let item = read_json(&item_path)?;
    ensure_queue_item_matches_active_claim(
        &item,
        queue_item_id,
        claim_id,
        claim_attempt,
        receipt_claimant,
        receipt_claimed_at,
        &item_path,
    )?;

    if completion_path.exists() {
        let existing = read_json(&completion_path)?;
        ensure_existing_completion_matches_episode(
            &existing,
            claim_id,
            claim_attempt,
            queue_item_id,
            &completion_path,
        )?;
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
            "claimAttempt": claim_attempt,
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

    let completion = build_completion(
        &claim,
        claim_id,
        claim_attempt,
        queue_item_id,
        completed_at,
        resolution,
    )?;
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
        "claimAttempt": claim_attempt,
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
    claim_attempt: u64,
    queue_item_id: &str,
    completed_at: &str,
    resolution: &str,
) -> Result<Value, String> {
    let completion_id = claim_episode_to_completion_id(claim_id, claim_attempt);
    Ok(json!({
        "kind": "centipede_queue_completion",
        "schemaVersion": 1,
        "completionId": completion_id,
        "claimId": claim_id,
        "claimAttempt": claim_attempt,
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
        "claimHistory": claim
            .get("claimHistory")
            .cloned()
            .unwrap_or_else(|| json!([])),
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
            "claimAttempt": required_u64(completion, "claimAttempt")?,
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
            "claimAttempt": required_u64(completion, "claimAttempt")?,
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
            "claimAttempt": required_u64(&value, "claimAttempt")?,
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

fn ensure_claim_matches_receipt(
    claim: &Value,
    claim_id: &str,
    claim_attempt: u64,
    queue_item_id: &str,
    claimant: &str,
    claimed_at: &str,
    claim_path: &Path,
) -> Result<(), String> {
    let active_claim_id = required_string(claim, "claimId")?;
    if active_claim_id != claim_id {
        return Err(format!(
            "claim receipt claimId does not match active claim in {}",
            claim_path.display()
        ));
    }

    let active_queue_item_id = required_string(claim, "queueItemId")?;
    if active_queue_item_id != queue_item_id {
        return Err(format!(
            "claim receipt queueItemId does not match active claim in {}",
            claim_path.display()
        ));
    }

    let active_claim_attempt = required_u64(claim, "claimAttempt")?;
    if active_claim_attempt != claim_attempt {
        return Err(format!(
            "claim receipt claimAttempt does not match active claim in {}",
            claim_path.display()
        ));
    }

    let active_claimant = required_string(claim, "claimant")?;
    if active_claimant != claimant {
        return Err(format!(
            "claim receipt claimant does not match active claim in {}",
            claim_path.display()
        ));
    }

    let active_claimed_at = required_string(claim, "claimedAt")?;
    if active_claimed_at != claimed_at {
        return Err(format!(
            "claim receipt claimedAt does not match active claim in {}",
            claim_path.display()
        ));
    }

    Ok(())
}

fn ensure_claim_is_active(claim: &Value, claim_path: &Path) -> Result<(), String> {
    if claim.get("completion").is_some() {
        return Err(format!("claim is already completed: {}", claim_path.display()));
    }
    if claim.get("reclaim").is_some() {
        return Err(format!("claim is not active (reclaimed): {}", claim_path.display()));
    }
    Ok(())
}

fn ensure_queue_item_matches_active_claim(
    item: &Value,
    queue_item_id: &str,
    claim_id: &str,
    claim_attempt: u64,
    claimant: &str,
    claimed_at: &str,
    item_path: &Path,
) -> Result<(), String> {
    let actual_queue_item_id = required_string(item, "queueItemId")?;
    if actual_queue_item_id != queue_item_id {
        return Err(format!(
            "queue item id does not match claim receipt in {}",
            item_path.display()
        ));
    }

    let processing = item
        .get("processingState")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("queue item missing processingState object: {}", item_path.display()))?;

    let status = processing
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("queue item missing processingState.status: {}", item_path.display()))?;
    if status != "claimed" {
        return Err(format!(
            "queue item is not actively claimed in {}",
            item_path.display()
        ));
    }

    let active_claim_id = processing
        .get("claimId")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("queue item missing processingState.claimId: {}", item_path.display()))?;
    if active_claim_id != claim_id {
        return Err(format!(
            "queue item processingState.claimId does not match claim receipt in {}",
            item_path.display()
        ));
    }

    let active_claim_attempt = processing
        .get("claimAttempt")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("queue item missing processingState.claimAttempt: {}", item_path.display()))?;
    if active_claim_attempt != claim_attempt {
        return Err(format!(
            "queue item processingState.claimAttempt does not match claim receipt in {}",
            item_path.display()
        ));
    }

    let active_claimant = processing
        .get("claimant")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("queue item missing processingState.claimant: {}", item_path.display()))?;
    if active_claimant != claimant {
        return Err(format!(
            "queue item processingState.claimant does not match claim receipt in {}",
            item_path.display()
        ));
    }

    let active_claimed_at = processing
        .get("claimedAt")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("queue item missing processingState.claimedAt: {}", item_path.display()))?;
    if active_claimed_at != claimed_at {
        return Err(format!(
            "queue item processingState.claimedAt does not match claim receipt in {}",
            item_path.display()
        ));
    }

    Ok(())
}

fn ensure_existing_completion_matches_episode(
    completion: &Value,
    claim_id: &str,
    claim_attempt: u64,
    queue_item_id: &str,
    completion_path: &Path,
) -> Result<(), String> {
    let existing_claim_id = required_string(completion, "claimId")?;
    if existing_claim_id != claim_id {
        return Err(format!(
            "existing completion claimId mismatch in {}",
            completion_path.display()
        ));
    }

    let existing_claim_attempt = required_u64(completion, "claimAttempt")?;
    if existing_claim_attempt != claim_attempt {
        return Err(format!(
            "existing completion claimAttempt mismatch in {}",
            completion_path.display()
        ));
    }

    let existing_queue_item_id = required_string(completion, "queueItemId")?;
    if existing_queue_item_id != queue_item_id {
        return Err(format!(
            "existing completion queueItemId mismatch in {}",
            completion_path.display()
        ));
    }

    Ok(())
}

fn claim_episode_to_completion_id(claim_id: &str, claim_attempt: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(claim_id.as_bytes());
    hasher.update(b":");
    hasher.update(claim_attempt.to_string().as_bytes());
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

fn required_u64(value: &Value, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing u64 field: {}", key))
}

fn hex_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn fail_from_claim_receipt(
    queue_dir: &Path,
    claim_receipt: &Value,
    failed_at: &str,
    reason: &str,
) -> Result<Value, String> {
    if failed_at.trim().is_empty() {
        return Err("failed_at must not be empty".to_string());
    }
    if reason.trim().is_empty() {
        return Err("reason must not be empty".to_string());
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

    let failures_dir = queue_dir.join("failures");
    fs::create_dir_all(&failures_dir)
        .map_err(|err| format!("could not create {}: {}", failures_dir.display(), err))?;

    let failure_id = claim_episode_to_failure_id(claim_id, claim_attempt);
    let failure_path = failures_dir.join(format!("{}.json", failure_id));

    if claim.get("failure").is_some() {
        if !failure_path.exists() {
            return Err(format!(
                "claim is failed but failure file is missing: {}",
                failure_path.display()
            ));
        }

        let existing = read_json(&failure_path)?;
        ensure_existing_failure_matches_episode(
            &existing,
            claim_id,
            claim_attempt,
            queue_item_id,
            &failure_path,
        )?;
        let index = build_failure_index(&failures_dir)?;
        write_json(&failures_dir.join("index.json"), &index)?;

        return Ok(json!({
            "kind": "centipede_queue_fail_receipt",
            "schemaVersion": 1,
            "queueDir": queue_dir.display().to_string(),
            "failuresDir": failures_dir.display().to_string(),
            "disposition": "already_failed",
            "failureId": failure_id,
            "claimId": claim_id,
            "claimAttempt": claim_attempt,
            "queueItemId": queue_item_id,
            "failedAt": existing.get("failedAt").and_then(Value::as_str).unwrap_or(failed_at),
            "reason": existing.get("reason").and_then(Value::as_str).unwrap_or(reason),
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

    if failure_path.exists() {
        let existing = read_json(&failure_path)?;
        ensure_existing_failure_matches_episode(
            &existing,
            claim_id,
            claim_attempt,
            queue_item_id,
            &failure_path,
        )?;
        let index = build_failure_index(&failures_dir)?;
        write_json(&failures_dir.join("index.json"), &index)?;

        return Ok(json!({
            "kind": "centipede_queue_fail_receipt",
            "schemaVersion": 1,
            "queueDir": queue_dir.display().to_string(),
            "failuresDir": failures_dir.display().to_string(),
            "disposition": "already_failed",
            "failureId": failure_id,
            "claimId": claim_id,
            "claimAttempt": claim_attempt,
            "queueItemId": queue_item_id,
            "failedAt": existing.get("failedAt").and_then(Value::as_str).unwrap_or(failed_at),
            "reason": existing.get("reason").and_then(Value::as_str).unwrap_or(reason),
            "sourceRepo": required_string(&claim, "sourceRepo")?,
            "intakeKind": required_string(&claim, "intakeKind")?,
            "blocking": claim
                .get("totals")
                .and_then(|totals| totals.get("blocking"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        }));
    }

    let failure = build_failure(
        &claim,
        claim_id,
        claim_attempt,
        queue_item_id,
        failed_at,
        reason,
    )?;
    write_json(&failure_path, &failure)?;
    update_queue_item_processing_state(&item_path, &failure)?;
    update_claim_with_failure(&claim_path, &claim, &failure)?;
    let index = build_failure_index(&failures_dir)?;
    write_json(&failures_dir.join("index.json"), &index)?;

    Ok(json!({
        "kind": "centipede_queue_fail_receipt",
        "schemaVersion": 1,
        "queueDir": queue_dir.display().to_string(),
        "failuresDir": failures_dir.display().to_string(),
        "disposition": "failed",
        "failureId": failure_id,
        "claimId": claim_id,
        "claimAttempt": claim_attempt,
        "queueItemId": queue_item_id,
        "failedAt": failed_at,
        "reason": reason,
        "sourceRepo": required_string(&claim, "sourceRepo")?,
        "intakeKind": required_string(&claim, "intakeKind")?,
        "blocking": claim
            .get("totals")
            .and_then(|totals| totals.get("blocking"))
            .and_then(Value::as_u64)
            .unwrap_or(0)
    }))
}

fn build_failure(
    claim: &Value,
    claim_id: &str,
    claim_attempt: u64,
    queue_item_id: &str,
    failed_at: &str,
    reason: &str,
) -> Result<Value, String> {
    let failure_id = claim_episode_to_failure_id(claim_id, claim_attempt);
    Ok(json!({
        "kind": "centipede_queue_failure",
        "schemaVersion": 1,
        "failureId": failure_id,
        "claimId": claim_id,
        "claimAttempt": claim_attempt,
        "queueItemId": queue_item_id,
        "claimant": required_string(claim, "claimant")?,
        "claimedAt": required_string(claim, "claimedAt")?,
        "failedAt": failed_at,
        "reason": reason,
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

fn update_queue_item_processing_state(item_path: &Path, failure: &Value) -> Result<(), String> {
    let mut item = read_json(item_path)?;
    let object = item
        .as_object_mut()
        .ok_or_else(|| format!("queue item is not an object: {}", item_path.display()))?;

    object.insert(
        "processingState".to_string(),
        json!({
            "status": "failed",
            "claimId": required_string(failure, "claimId")?,
            "claimAttempt": required_u64(failure, "claimAttempt")?,
            "failureId": required_string(failure, "failureId")?,
            "claimant": required_string(failure, "claimant")?,
            "claimedAt": required_string(failure, "claimedAt")?,
            "failedAt": required_string(failure, "failedAt")?,
            "reason": required_string(failure, "reason")?
        }),
    );

    write_json(item_path, &item)
}

fn update_claim_with_failure(claim_path: &Path, claim: &Value, failure: &Value) -> Result<(), String> {
    let mut updated = claim.clone();
    let object = updated
        .as_object_mut()
        .ok_or_else(|| format!("claim is not an object: {}", claim_path.display()))?;

    object.insert(
        "failure".to_string(),
        json!({
            "failureId": required_string(failure, "failureId")?,
            "claimAttempt": required_u64(failure, "claimAttempt")?,
            "failedAt": required_string(failure, "failedAt")?,
            "reason": required_string(failure, "reason")?
        }),
    );

    write_json(claim_path, &updated)
}

fn build_failure_index(failures_dir: &Path) -> Result<Value, String> {
    let mut failure_paths: Vec<PathBuf> = if failures_dir.exists() {
        fs::read_dir(failures_dir)
            .map_err(|err| format!("could not read {}: {}", failures_dir.display(), err))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| {
                path.extension().and_then(|v| v.to_str()) == Some("json")
                    && path.file_name().and_then(|v| v.to_str()) != Some("index.json")
            })
            .collect()
    } else {
        Vec::new()
    };
    failure_paths.sort();

    let mut items = Vec::new();
    for failure_path in failure_paths {
        let value = read_json(&failure_path)?;
        items.push(json!({
            "failureId": required_string(&value, "failureId")?,
            "claimId": required_string(&value, "claimId")?,
            "claimAttempt": required_u64(&value, "claimAttempt")?,
            "queueItemId": required_string(&value, "queueItemId")?,
            "claimant": required_string(&value, "claimant")?,
            "failedAt": required_string(&value, "failedAt")?,
            "reason": required_string(&value, "reason")?,
            "sourceRepo": required_string(&value, "sourceRepo")?,
            "intakeKind": required_string(&value, "intakeKind")?,
            "blocking": value
                .get("totals")
                .and_then(|totals| totals.get("blocking"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        }));
    }

    let mut by_reason = std::collections::BTreeMap::<String, u64>::new();
    for item in &items {
        let reason = required_string(item, "reason")?.to_string();
        *by_reason.entry(reason).or_insert(0) += 1;
    }

    Ok(json!({
        "kind": "centipede_queue_failure_index",
        "schemaVersion": 1,
        "failuresDir": failures_dir.display().to_string(),
        "items": items,
        "totals": {
            "failures": items.len(),
            "blocking": items.iter().map(summary_blocking).sum::<u64>(),
            "byReason": by_reason
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
    if claim.get("failure").is_some() {
        return Err(format!("claim is already failed: {}", claim_path.display()));
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

fn ensure_existing_failure_matches_episode(
    failure: &Value,
    claim_id: &str,
    claim_attempt: u64,
    queue_item_id: &str,
    failure_path: &Path,
) -> Result<(), String> {
    let existing_claim_id = required_string(failure, "claimId")?;
    if existing_claim_id != claim_id {
        return Err(format!(
            "existing failure claimId mismatch in {}",
            failure_path.display()
        ));
    }

    let existing_claim_attempt = required_u64(failure, "claimAttempt")?;
    if existing_claim_attempt != claim_attempt {
        return Err(format!(
            "existing failure claimAttempt mismatch in {}",
            failure_path.display()
        ));
    }

    let existing_queue_item_id = required_string(failure, "queueItemId")?;
    if existing_queue_item_id != queue_item_id {
        return Err(format!(
            "existing failure queueItemId mismatch in {}",
            failure_path.display()
        ));
    }

    Ok(())
}

fn claim_episode_to_failure_id(claim_id: &str, claim_attempt: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(claim_id.as_bytes());
    hasher.update(b":");
    hasher.update(claim_attempt.to_string().as_bytes());
    let digest = hasher.finalize();
    let hex = hex_string(&digest);
    format!("cqfail-{}", &hex[..24])
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
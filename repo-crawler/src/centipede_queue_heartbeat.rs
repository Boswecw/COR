use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

pub fn heartbeat_claim_from_receipt(
    queue_dir: &Path,
    claim_receipt: &Value,
    heartbeat_at: &str,
    lease_timeout_seconds: i64,
) -> Result<Value, String> {
    if heartbeat_at.trim().is_empty() {
        return Err("heartbeat_at must not be empty".to_string());
    }
    if lease_timeout_seconds < 0 {
        return Err("lease_timeout_seconds must be zero or greater".to_string());
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
    ensure_claim_is_active(&claim, &claim_path)?;

    let claimed_epoch = parse_rfc3339_to_unix_seconds(receipt_claimed_at)?;
    let heartbeat_epoch = parse_rfc3339_to_unix_seconds(heartbeat_at)?;
    if heartbeat_epoch < claimed_epoch {
        return Err("heartbeat_at must not be earlier than claimedAt".to_string());
    }

    let previous_heartbeat_at = claim
        .get("lease")
        .and_then(|lease| lease.get("lastHeartbeatAt"))
        .and_then(Value::as_str)
        .unwrap_or(receipt_claimed_at);
    let previous_heartbeat_epoch = parse_rfc3339_to_unix_seconds(previous_heartbeat_at)?;
    if heartbeat_epoch < previous_heartbeat_epoch {
        return Err("heartbeat_at must not move backward relative to lastHeartbeatAt".to_string());
    }

    let heartbeat_count = claim
        .get("lease")
        .and_then(|lease| lease.get("heartbeatCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;

    let lease_expires_at_epoch_seconds = heartbeat_epoch + lease_timeout_seconds;

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

    update_claim_with_heartbeat(
        &claim_path,
        &claim,
        heartbeat_at,
        heartbeat_count,
        lease_timeout_seconds,
        lease_expires_at_epoch_seconds,
    )?;
    update_queue_item_with_heartbeat(
        &item_path,
        &claim,
        heartbeat_at,
        heartbeat_count,
        lease_timeout_seconds,
        lease_expires_at_epoch_seconds,
    )?;

    let index = build_claims_index(&claims_dir)?;
    write_json(&claims_dir.join("index.json"), &index)?;

    Ok(json!({
        "kind": "centipede_queue_heartbeat_receipt",
        "schemaVersion": 1,
        "queueDir": queue_dir.display().to_string(),
        "claimsDir": claims_dir.display().to_string(),
        "disposition": "heartbeat_recorded",
        "claimId": claim_id,
        "claimAttempt": claim_attempt,
        "queueItemId": queue_item_id,
        "claimant": receipt_claimant,
        "heartbeatAt": heartbeat_at,
        "heartbeatCount": heartbeat_count,
        "leaseTimeoutSeconds": lease_timeout_seconds,
        "leaseExpiresAtEpochSeconds": lease_expires_at_epoch_seconds,
        "sourceRepo": required_string(&claim, "sourceRepo")?,
        "intakeKind": required_string(&claim, "intakeKind")?,
        "blocking": claim
            .get("totals")
            .and_then(|totals| totals.get("blocking"))
            .and_then(Value::as_u64)
            .unwrap_or(0)
    }))
}

fn update_claim_with_heartbeat(
    claim_path: &Path,
    claim: &Value,
    heartbeat_at: &str,
    heartbeat_count: u64,
    lease_timeout_seconds: i64,
    lease_expires_at_epoch_seconds: i64,
) -> Result<(), String> {
    let mut updated = claim.clone();
    let object = updated
        .as_object_mut()
        .ok_or_else(|| format!("claim is not an object: {}", claim_path.display()))?;

    object.insert(
        "lease".to_string(),
        json!({
            "heartbeatCount": heartbeat_count,
            "lastHeartbeatAt": heartbeat_at,
            "leaseTimeoutSeconds": lease_timeout_seconds,
            "leaseExpiresAtEpochSeconds": lease_expires_at_epoch_seconds
        }),
    );

    write_json(claim_path, &updated)
}

fn update_queue_item_with_heartbeat(
    item_path: &Path,
    claim: &Value,
    heartbeat_at: &str,
    heartbeat_count: u64,
    lease_timeout_seconds: i64,
    lease_expires_at_epoch_seconds: i64,
) -> Result<(), String> {
    let mut item = read_json(item_path)?;
    let object = item
        .as_object_mut()
        .ok_or_else(|| format!("queue item is not an object: {}", item_path.display()))?;

    object.insert(
        "processingState".to_string(),
        json!({
            "status": "claimed",
            "claimId": required_string(claim, "claimId")?,
            "claimAttempt": claim.get("claimAttempt").and_then(Value::as_u64).unwrap_or(1),
            "claimant": required_string(claim, "claimant")?,
            "claimedAt": required_string(claim, "claimedAt")?,
            "heartbeatCount": heartbeat_count,
            "lastHeartbeatAt": heartbeat_at,
            "leaseTimeoutSeconds": lease_timeout_seconds,
            "leaseExpiresAtEpochSeconds": lease_expires_at_epoch_seconds
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

fn claim_state(claim: &Value) -> &'static str {
    if claim.get("completion").is_some() {
        "completed"
    } else if claim.get("reclaim").is_some() {
        "reclaimed"
    } else {
        "active"
    }
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

fn required_u64(value: &Value, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing u64 field: {}", key))
}

fn parse_rfc3339_to_unix_seconds(value: &str) -> Result<i64, String> {
    if value.len() != 20 && value.len() != 25 {
        return Err(format!("unsupported RFC3339 timestamp: {}", value));
    }
    if &value[4..5] != "-"
        || &value[7..8] != "-"
        || &value[10..11] != "T"
        || &value[13..14] != ":"
        || &value[16..17] != ":"
    {
        return Err(format!("unsupported RFC3339 timestamp: {}", value));
    }

    let year: i32 = parse_i32(&value[0..4], "year", value)?;
    let month: u32 = parse_u32(&value[5..7], "month", value)?;
    let day: u32 = parse_u32(&value[8..10], "day", value)?;
    let hour: i64 = parse_i64(&value[11..13], "hour", value)?;
    let minute: i64 = parse_i64(&value[14..16], "minute", value)?;
    let second: i64 = parse_i64(&value[17..19], "second", value)?;

    if !(1..=12).contains(&month) {
        return Err(format!("month out of range in timestamp: {}", value));
    }
    if !(1..=31).contains(&day) {
        return Err(format!("day out of range in timestamp: {}", value));
    }
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) || !(0..=59).contains(&second) {
        return Err(format!("time component out of range in timestamp: {}", value));
    }

    let offset_seconds = if value.ends_with('Z') {
        0
    } else {
        let sign = &value[19..20];
        if &value[22..23] != ":" {
            return Err(format!("unsupported RFC3339 timestamp: {}", value));
        }
        let offset_hours: i64 = parse_i64(&value[20..22], "offset hour", value)?;
        let offset_minutes: i64 = parse_i64(&value[23..25], "offset minute", value)?;
        let total = offset_hours * 3600 + offset_minutes * 60;
        match sign {
            "+" => total,
            "-" => -total,
            _ => return Err(format!("unsupported RFC3339 timestamp: {}", value)),
        }
    };

    let days = days_from_civil(year, month, day);
    Ok(days * 86_400 + hour * 3_600 + minute * 60 + second - offset_seconds)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let adjusted_year = year - if month <= 2 { 1 } else { 0 };
    let era = if adjusted_year >= 0 {
        adjusted_year / 400
    } else {
        (adjusted_year - 399) / 400
    };
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month as i32 + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day as i32 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era as i64 * 146_097 + day_of_era as i64 - 719_468
}

fn parse_i32(fragment: &str, label: &str, original: &str) -> Result<i32, String> {
    fragment
        .parse::<i32>()
        .map_err(|_| format!("could not parse {} from timestamp: {}", label, original))
}

fn parse_u32(fragment: &str, label: &str, original: &str) -> Result<u32, String> {
    fragment
        .parse::<u32>()
        .map_err(|_| format!("could not parse {} from timestamp: {}", label, original))
}

fn parse_i64(fragment: &str, label: &str, original: &str) -> Result<i64, String> {
    fragment
        .parse::<i64>()
        .map_err(|_| format!("could not parse {} from timestamp: {}", label, original))
}
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

pub fn reclaim_expired_claims(
    queue_dir: &Path,
    reclaimer: &str,
    reclaimed_at: &str,
    lease_timeout_seconds: i64,
) -> Result<Value, String> {
    if reclaimer.trim().is_empty() {
        return Err("reclaimer must not be empty".to_string());
    }
    if reclaimed_at.trim().is_empty() {
        return Err("reclaimed_at must not be empty".to_string());
    }
    if lease_timeout_seconds < 0 {
        return Err("lease_timeout_seconds must be zero or greater".to_string());
    }

    let claims_dir = queue_dir.join("claims");
    if !claims_dir.exists() {
        return Err(format!("claims directory does not exist: {}", claims_dir.display()));
    }

    let reclaimed_epoch = parse_rfc3339_to_unix_seconds(reclaimed_at)?;

    let mut claim_paths: Vec<PathBuf> = fs::read_dir(&claims_dir)
        .map_err(|err| format!("could not read {}: {}", claims_dir.display(), err))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension().and_then(|v| v.to_str()) == Some("json")
                && path.file_name().and_then(|v| v.to_str()) != Some("index.json")
        })
        .collect();
    claim_paths.sort();

    let mut reclaimed_items = Vec::new();

    for claim_path in claim_paths {
        let claim = read_json(&claim_path)?;
        if !claim_is_active(&claim) {
            continue;
        }

        let claimed_at = required_string(&claim, "claimedAt")?;
        let claimed_epoch = parse_rfc3339_to_unix_seconds(claimed_at)?;
        if reclaimed_epoch < claimed_epoch {
            continue;
        }

        let stale_age_seconds = reclaimed_epoch - claimed_epoch;
        if stale_age_seconds < lease_timeout_seconds {
            continue;
        }

        let item_path = PathBuf::from(required_string(&claim, "queueItemPath")?);
        if !item_path.exists() {
            return Err(format!("queue item file does not exist: {}", item_path.display()));
        }

        update_claim_with_reclaim(
            &claim_path,
            &claim,
            reclaimer,
            reclaimed_at,
            lease_timeout_seconds,
            stale_age_seconds,
        )?;
        update_queue_item_reclaimed_state(&item_path, &claim, reclaimer, reclaimed_at)?;

        reclaimed_items.push(json!({
            "claimId": required_string(&claim, "claimId")?,
            "queueItemId": required_string(&claim, "queueItemId")?,
            "claimant": required_string(&claim, "claimant")?,
            "claimedAt": required_string(&claim, "claimedAt")?,
            "reclaimedAt": reclaimed_at,
            "staleAgeSeconds": stale_age_seconds,
            "sourceRepo": required_string(&claim, "sourceRepo")?,
            "intakeKind": required_string(&claim, "intakeKind")?,
            "blocking": claim
                .get("totals")
                .and_then(|totals| totals.get("blocking"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        }));
    }

    let index = build_claims_index(&claims_dir)?;
    write_json(&claims_dir.join("index.json"), &index)?;

    let blocking = reclaimed_items.iter().map(summary_blocking).sum::<u64>();
    let disposition = if reclaimed_items.is_empty() {
        "no_expired_claims"
    } else {
        "reclaimed"
    };

    Ok(json!({
        "kind": "centipede_queue_reclaim_receipt",
        "schemaVersion": 1,
        "queueDir": queue_dir.display().to_string(),
        "claimsDir": claims_dir.display().to_string(),
        "disposition": disposition,
        "reclaimer": reclaimer,
        "reclaimedAt": reclaimed_at,
        "leaseTimeoutSeconds": lease_timeout_seconds,
        "items": reclaimed_items,
        "totals": {
            "reclaimed": reclaimed_items.len(),
            "blocking": blocking
        }
    }))
}

fn update_claim_with_reclaim(
    claim_path: &Path,
    claim: &Value,
    reclaimer: &str,
    reclaimed_at: &str,
    lease_timeout_seconds: i64,
    stale_age_seconds: i64,
) -> Result<(), String> {
    let mut updated = claim.clone();
    let object = updated
        .as_object_mut()
        .ok_or_else(|| format!("claim is not an object: {}", claim_path.display()))?;

    object.insert(
        "reclaim".to_string(),
        json!({
            "reclaimer": reclaimer,
            "reclaimedAt": reclaimed_at,
            "reason": "lease_expired",
            "leaseTimeoutSeconds": lease_timeout_seconds,
            "staleAgeSeconds": stale_age_seconds
        }),
    );

    write_json(claim_path, &updated)
}

fn update_queue_item_reclaimed_state(
    item_path: &Path,
    claim: &Value,
    reclaimer: &str,
    reclaimed_at: &str,
) -> Result<(), String> {
    let mut item = read_json(item_path)?;
    let object = item
        .as_object_mut()
        .ok_or_else(|| format!("queue item is not an object: {}", item_path.display()))?;

    object.insert(
        "processingState".to_string(),
        json!({
            "status": "queued",
            "reclaimedFromClaimId": required_string(claim, "claimId")?,
            "reclaimedFromClaimant": required_string(claim, "claimant")?,
            "reclaimedFromClaimedAt": required_string(claim, "claimedAt")?,
            "reclaimer": reclaimer,
            "reclaimedAt": reclaimed_at,
            "reclaimReason": "lease_expired"
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

fn claim_state(claim: &Value) -> &'static str {
    if claim.get("completion").is_some() {
        "completed"
    } else if claim.get("reclaim").is_some() {
        "reclaimed"
    } else {
        "active"
    }
}

fn claim_is_active(claim: &Value) -> bool {
    claim.get("completion").is_none() && claim.get("reclaim").is_none()
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

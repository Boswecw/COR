use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

pub fn build_queue_report(queue_dir: &Path, queue_item_id_filter: Option<&str>) -> Result<Value, String> {
    let items_dir = queue_dir.join("items");
    if !items_dir.exists() {
        return Err(format!("queue items directory does not exist: {}", items_dir.display()));
    }

    let claim_files = scan_json_files(&queue_dir.join("claims"))?;
    let completion_files = scan_json_files(&queue_dir.join("completions"))?;
    let failure_files = scan_json_files(&queue_dir.join("failures"))?;

    let mut claims_by_queue_item = BTreeMap::<String, (PathBuf, Value)>::new();
    for claim_path in claim_files {
        let claim = read_json(&claim_path)?;
        let queue_item_id = required_string(&claim, "queueItemId")?.to_string();
        claims_by_queue_item.insert(queue_item_id, (claim_path, claim));
    }

    let mut completions_by_queue_item = BTreeMap::<String, Vec<(PathBuf, Value)>>::new();
    for completion_path in completion_files {
        let completion = read_json(&completion_path)?;
        let queue_item_id = required_string(&completion, "queueItemId")?.to_string();
        completions_by_queue_item
            .entry(queue_item_id)
            .or_default()
            .push((completion_path, completion));
    }

    let mut failures_by_queue_item = BTreeMap::<String, Vec<(PathBuf, Value)>>::new();
    for failure_path in failure_files {
        let failure = read_json(&failure_path)?;
        let queue_item_id = required_string(&failure, "queueItemId")?.to_string();
        failures_by_queue_item
            .entry(queue_item_id)
            .or_default()
            .push((failure_path, failure));
    }

    let mut item_paths = scan_json_files(&items_dir)?;
    item_paths.sort();

    let mut items = Vec::new();
    let mut found_filter = queue_item_id_filter.is_none();

    let mut totals_items = 0_u64;
    let mut totals_blocking = 0_u64;
    let mut totals_processing = BTreeMap::<String, u64>::new();
    let mut totals_claim_records = 0_u64;
    let mut totals_claim_episodes = 0_u64;
    let mut totals_claim_states = BTreeMap::<String, u64>::new();
    let mut totals_reclaimed_history_episodes = 0_u64;
    let mut totals_completions = 0_u64;
    let mut totals_failures = 0_u64;

    for item_path in item_paths {
        let item = read_json(&item_path)?;
        let queue_item_id = required_string(&item, "queueItemId")?.to_string();

        if let Some(filter) = queue_item_id_filter {
            if queue_item_id != filter {
                continue;
            }
            found_filter = true;
        }

        let processing_state = item
            .get("processingState")
            .cloned()
            .unwrap_or_else(|| json!({"status": "queued"}));
        let processing_status = processing_state
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        *totals_processing.entry(processing_status.clone()).or_insert(0) += 1;

        let blocking = item
            .get("totals")
            .and_then(|value| value.get("blocking"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        totals_items += 1;
        totals_blocking += blocking;

        let candidate_issue_keys = extract_item_issue_keys(&item)?;
        let candidate_issue_count = candidate_issue_keys.len() as u64;

        let claim_record = if let Some((claim_path, claim)) = claims_by_queue_item.get(&queue_item_id) {
            totals_claim_records += 1;
            let claim_state = derive_claim_state(claim);
            *totals_claim_states.entry(claim_state.to_string()).or_insert(0) += 1;

            let episodes = build_claim_episodes(claim_path, claim)?;
            totals_claim_episodes += episodes.len() as u64;
            totals_reclaimed_history_episodes += episodes
                .iter()
                .filter(|episode| episode.get("state").and_then(Value::as_str) == Some("reclaimed"))
                .count() as u64;

            Some(json!({
                "claimId": required_string(claim, "claimId")?,
                "claimPath": claim_path.display().to_string(),
                "currentClaimAttempt": claim.get("claimAttempt").and_then(Value::as_u64).unwrap_or(1),
                "currentState": claim_state,
                "episodes": episodes,
                "historyEpisodeCount": claim
                    .get("claimHistory")
                    .and_then(Value::as_array)
                    .map(|value| value.len())
                    .unwrap_or(0)
            }))
        } else {
            None
        };

        let completion_evidence = build_artifact_evidence(
            completions_by_queue_item.get(&queue_item_id),
            "completionId",
            "completedAt",
            Some("outcome"),
        )?;
        let failure_evidence = build_artifact_evidence(
            failures_by_queue_item.get(&queue_item_id),
            "failureId",
            "failedAt",
            Some("reason"),
        )?;

        totals_completions += completion_evidence.len() as u64;
        totals_failures += failure_evidence.len() as u64;

        items.push(json!({
            "queueItemId": queue_item_id,
            "queueItemPath": item_path.display().to_string(),
            "sourceRepo": required_string(&item, "sourceRepo")?,
            "sourceLane": required_string(&item, "sourceLane")?,
            "sourceHandoffId": required_string(&item, "sourceHandoffId")?,
            "intakeKind": required_string(&item, "intakeKind")?,
            "receivedAt": required_string(&item, "receivedAt")?,
            "candidateIssueCount": candidate_issue_count,
            "candidateIssueKeys": candidate_issue_keys,
            "blocking": blocking,
            "processingState": processing_state,
            "claimRecord": claim_record,
            "completionEvidence": completion_evidence,
            "failureEvidence": failure_evidence,
        }));
    }

    if !found_filter {
        return Err(format!(
            "queue item not found in {}: {}",
            queue_dir.display(),
            queue_item_id_filter.unwrap_or_default()
        ));
    }

    Ok(json!({
        "kind": "centipede_queue_operator_report",
        "schemaVersion": 1,
        "queueDir": queue_dir.display().to_string(),
        "selection": {
            "queueItemId": queue_item_id_filter
        },
        "items": items,
        "totals": {
            "items": totals_items,
            "blocking": totals_blocking,
            "processing": map_to_value(&totals_processing),
            "claimRecords": totals_claim_records,
            "claimEpisodes": totals_claim_episodes,
            "claimRecordStates": map_to_value(&totals_claim_states),
            "reclaimedHistoryEpisodes": totals_reclaimed_history_episodes,
            "completions": totals_completions,
            "failures": totals_failures
        }
    }))
}

fn build_claim_episodes(claim_path: &Path, claim: &Value) -> Result<Vec<Value>, String> {
    let claim_id = required_string(claim, "claimId")?;
    let mut episodes = Vec::new();

    if let Some(history) = claim.get("claimHistory").and_then(Value::as_array) {
        for entry in history {
            let lease = entry.get("lease").cloned().unwrap_or(Value::Null);
            let reclaim = entry
                .get("reclaim")
                .cloned()
                .ok_or_else(|| format!("claim history entry missing reclaim in {}", claim_path.display()))?;
            episodes.push(json!({
                "claimId": claim_id,
                "claimAttempt": entry.get("claimAttempt").and_then(Value::as_u64).unwrap_or(1),
                "state": "reclaimed",
                "claimant": required_string(entry, "claimant")?,
                "claimedAt": required_string(entry, "claimedAt")?,
                "heartbeatCount": lease.get("heartbeatCount").and_then(Value::as_u64).unwrap_or(0),
                "lastHeartbeatAt": lease.get("lastHeartbeatAt").and_then(Value::as_str),
                "leaseTimeoutSeconds": lease.get("leaseTimeoutSeconds").and_then(Value::as_i64),
                "leaseExpiresAtEpochSeconds": lease.get("leaseExpiresAtEpochSeconds").and_then(Value::as_i64),
                "reclaim": reclaim
            }));
        }
    }

    let lease = claim.get("lease").cloned().unwrap_or(Value::Null);
    let mut current_episode = Map::new();
    current_episode.insert("claimId".to_string(), json!(claim_id));
    current_episode.insert(
        "claimAttempt".to_string(),
        json!(claim.get("claimAttempt").and_then(Value::as_u64).unwrap_or(1)),
    );
    current_episode.insert("state".to_string(), json!(derive_claim_state(claim)));
    current_episode.insert("claimant".to_string(), json!(required_string(claim, "claimant")?));
    current_episode.insert("claimedAt".to_string(), json!(required_string(claim, "claimedAt")?));
    current_episode.insert(
        "heartbeatCount".to_string(),
        json!(lease.get("heartbeatCount").and_then(Value::as_u64).unwrap_or(0)),
    );
    current_episode.insert(
        "lastHeartbeatAt".to_string(),
        json!(lease.get("lastHeartbeatAt").and_then(Value::as_str)),
    );
    current_episode.insert(
        "leaseTimeoutSeconds".to_string(),
        json!(lease.get("leaseTimeoutSeconds").and_then(Value::as_i64)),
    );
    current_episode.insert(
        "leaseExpiresAtEpochSeconds".to_string(),
        json!(lease.get("leaseExpiresAtEpochSeconds").and_then(Value::as_i64)),
    );
    if let Some(reclaim) = claim.get("reclaim") {
        current_episode.insert("reclaim".to_string(), reclaim.clone());
    }
    if let Some(completion) = claim.get("completion") {
        current_episode.insert("completion".to_string(), completion.clone());
    }
    if let Some(failure) = claim.get("failure") {
        current_episode.insert("failure".to_string(), failure.clone());
    }
    episodes.push(Value::Object(current_episode));

    episodes.sort_by_key(|episode| episode.get("claimAttempt").and_then(Value::as_u64).unwrap_or(0));
    Ok(episodes)
}

fn build_artifact_evidence(
    values: Option<&Vec<(PathBuf, Value)>>,
    id_key: &str,
    time_key: &str,
    extra_key: Option<&str>,
) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    if let Some(values) = values {
        for (path, value) in values {
            let mut object = Map::new();
            object.insert(id_key.to_string(), json!(required_string(value, id_key)?));
            object.insert("path".to_string(), json!(path.display().to_string()));
            object.insert(
                "claimId".to_string(),
                json!(required_string(value, "claimId")?),
            );
            object.insert(
                "claimAttempt".to_string(),
                json!(value.get("claimAttempt").and_then(Value::as_u64).unwrap_or(1)),
            );
            object.insert(time_key.to_string(), json!(required_string(value, time_key)?));
            if let Some(extra_key) = extra_key {
                object.insert(extra_key.to_string(), json!(required_string(value, extra_key)?));
            }
            out.push(Value::Object(object));
        }
    }
    out.sort_by_key(|value| value.get("claimAttempt").and_then(Value::as_u64).unwrap_or(0));
    Ok(out)
}

fn extract_item_issue_keys(item: &Value) -> Result<Vec<String>, String> {
    let issues = item
        .get("candidateIssues")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing array field: candidateIssues".to_string())?;

    let mut out = Vec::with_capacity(issues.len());
    for issue in issues {
        out.push(required_string(issue, "issueKey")?.to_string());
    }
    Ok(out)
}

fn derive_claim_state(claim: &Value) -> &'static str {
    if claim.get("completion").is_some() {
        "completed"
    } else if claim.get("failure").is_some() {
        "failed"
    } else if claim.get("reclaim").is_some() {
        "reclaimed"
    } else {
        "active"
    }
}

fn scan_json_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|err| format!("could not read {}: {}", dir.display(), err))?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path.file_name().and_then(|value| value.to_str()) != Some("index.json")
        })
        .collect();
    paths.sort();
    Ok(paths)
}

fn map_to_value(map: &BTreeMap<String, u64>) -> Value {
    let mut out = Map::new();
    for (key, value) in map {
        out.insert(key.clone(), json!(value));
    }
    Value::Object(out)
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {}", path.display(), err))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("could not parse {}: {}", path.display(), err))
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field: {}", key))
}

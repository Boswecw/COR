use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn enqueue_queue_file(input_path: &Path, queue_dir: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(input_path)
        .map_err(|err| format!("could not read {}: {}", input_path.display(), err))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| format!("could not parse {}: {}", input_path.display(), err))?;
    enqueue_queue_value(&value, queue_dir)
}

pub fn enqueue_queue_value(input: &Value, queue_dir: &Path) -> Result<Value, String> {
    validate_queue_input(input)?;

    let items_dir = queue_dir.join("items");
    fs::create_dir_all(&items_dir)
        .map_err(|err| format!("could not create {}: {}", items_dir.display(), err))?;

    let intake_kind = required_string(input, "intakeKind")?;
    let source_lane = required_string(input, "sourceLane")?;
    let source_repo = required_string(input, "sourceRepo")?;
    let source_handoff_id = required_string(input, "sourceHandoffId")?;
    let candidate_issue_keys = extract_issue_keys(input)?;
    let queue_item_id = build_queue_item_id(
        intake_kind,
        source_lane,
        source_repo,
        source_handoff_id,
        &candidate_issue_keys,
    );

    let queue_item_path = items_dir.join(format!("{}.json", queue_item_id));
    let disposition = if queue_item_path.exists() {
        "already_present"
    } else {
        let queue_item = build_queue_item(input, &queue_item_id)?;
        write_json(&queue_item_path, &queue_item)?;
        "enqueued"
    };

    let index_path = queue_dir.join("index.json");
    let index_value = build_index(queue_dir)?;
    write_json(&index_path, &index_value)?;

    Ok(json!({
        "kind": "centipede_queue_enqueue_receipt",
        "schemaVersion": 1,
        "queueItemId": queue_item_id,
        "queueItemPath": queue_item_path.display().to_string(),
        "queueDir": queue_dir.display().to_string(),
        "disposition": disposition,
        "sourceRepo": source_repo,
        "sourceLane": source_lane,
        "intakeKind": intake_kind,
        "candidateIssues": candidate_issue_keys.len(),
        "blocking": input
            .get("totals")
            .and_then(|totals| totals.get("blocking"))
            .and_then(Value::as_u64)
            .unwrap_or(0)
    }))
}

fn validate_queue_input(input: &Value) -> Result<(), String> {
    let kind = required_string(input, "kind")?;
    if kind != "centipede_candidate_issue_queue" {
        return Err(format!("unsupported queue kind: {}", kind));
    }

    required_string(input, "intakeKind")?;
    required_string(input, "sourceLane")?;
    required_string(input, "sourceRepo")?;
    required_string(input, "sourceHandoffId")?;
    required_string(input, "receivedAt")?;

    let totals = input
        .get("totals")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing object field: totals".to_string())?;

    if totals.get("candidateIssues").and_then(Value::as_u64).is_none() {
        return Err("missing number field: totals.candidateIssues".to_string());
    }

    if totals.get("blocking").and_then(Value::as_u64).is_none() {
        return Err("missing number field: totals.blocking".to_string());
    }

    let _ = extract_issue_keys(input)?;
    Ok(())
}

fn build_queue_item(input: &Value, queue_item_id: &str) -> Result<Value, String> {
    Ok(json!({
        "kind": "centipede_candidate_issue_queue_item",
        "schemaVersion": 1,
        "queueItemId": queue_item_id,
        "intakeKind": required_string(input, "intakeKind")?,
        "sourceLane": required_string(input, "sourceLane")?,
        "sourceRepo": required_string(input, "sourceRepo")?,
        "sourceHandoffId": required_string(input, "sourceHandoffId")?,
        "receivedAt": required_string(input, "receivedAt")?,
        "failureKind": optional_string(input, "failureKind"),
        "severity": optional_string(input, "severity"),
        "recommendedRoute": optional_string(input, "recommendedRoute"),
        "candidateIssues": input
            .get("candidateIssues")
            .cloned()
            .ok_or_else(|| "missing field: candidateIssues".to_string())?,
        "evidenceArtifacts": input
            .get("evidenceArtifacts")
            .cloned()
            .unwrap_or_else(|| json!([])),
        "totals": input
            .get("totals")
            .cloned()
            .ok_or_else(|| "missing field: totals".to_string())?
    }))
}

fn build_index(queue_dir: &Path) -> Result<Value, String> {
    let items_dir = queue_dir.join("items");
    let mut summaries = Vec::new();

    if items_dir.exists() {
        let mut item_paths: Vec<PathBuf> = fs::read_dir(&items_dir)
            .map_err(|err| format!("could not read {}: {}", items_dir.display(), err))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("json"))
            .collect();

        item_paths.sort();

        for item_path in item_paths {
            let text = fs::read_to_string(&item_path)
                .map_err(|err| format!("could not read {}: {}", item_path.display(), err))?;
            let value: Value = serde_json::from_str(&text)
                .map_err(|err| format!("could not parse {}: {}", item_path.display(), err))?;
            summaries.push(build_index_summary(&value)?);
        }
    }

    Ok(json!({
        "kind": "centipede_candidate_issue_queue_index",
        "schemaVersion": 1,
        "queueDir": queue_dir.display().to_string(),
        "items": summaries,
        "totals": {
            "items": summaries.len(),
            "candidateIssues": summaries.iter().map(summary_candidate_issues).sum::<u64>(),
            "blocking": summaries.iter().map(summary_blocking).sum::<u64>()
        }
    }))
}

fn build_index_summary(item: &Value) -> Result<Value, String> {
    Ok(json!({
        "queueItemId": required_string(item, "queueItemId")?,
        "intakeKind": required_string(item, "intakeKind")?,
        "sourceLane": required_string(item, "sourceLane")?,
        "sourceRepo": required_string(item, "sourceRepo")?,
        "sourceHandoffId": required_string(item, "sourceHandoffId")?,
        "candidateIssues": item
            .get("totals")
            .and_then(|totals| totals.get("candidateIssues"))
            .and_then(Value::as_u64)
            .ok_or_else(|| "missing number field: totals.candidateIssues".to_string())?,
        "blocking": item
            .get("totals")
            .and_then(|totals| totals.get("blocking"))
            .and_then(Value::as_u64)
            .ok_or_else(|| "missing number field: totals.blocking".to_string())?
    }))
}

fn summary_candidate_issues(value: &Value) -> u64 {
    value.get("candidateIssues").and_then(Value::as_u64).unwrap_or(0)
}

fn summary_blocking(value: &Value) -> u64 {
    value.get("blocking").and_then(Value::as_u64).unwrap_or(0)
}

fn extract_issue_keys(input: &Value) -> Result<Vec<String>, String> {
    let items = input
        .get("candidateIssues")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing array field: candidateIssues".to_string())?;

    let mut keys = Vec::with_capacity(items.len());
    for item in items {
        let object = item
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

fn build_queue_item_id(
    intake_kind: &str,
    source_lane: &str,
    source_repo: &str,
    source_handoff_id: &str,
    candidate_issue_keys: &[String],
) -> String {
    let mut issue_keys = candidate_issue_keys.to_vec();
    issue_keys.sort();

    let key = format!(
        "{}|{}|{}|{}|{}",
        intake_kind,
        source_lane,
        source_repo,
        source_handoff_id,
        issue_keys.join(";")
    );

    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    let hex = hex_string(&digest);
    format!("cqi-{}", &hex[..24])
}

fn hex_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let pretty = serde_json::to_string_pretty(value)
        .map_err(|err| format!("could not serialize {}: {}", path.display(), err))?;
    fs::write(path, format!("{}\n", pretty))
        .map_err(|err| format!("could not write {}: {}", path.display(), err))
}

fn required_string<'a>(input: &'a Value, key: &str) -> Result<&'a str, String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field: {}", key))
}

fn optional_string(input: &Value, key: &str) -> Value {
    match input.get(key).and_then(Value::as_str) {
        Some(value) => Value::String(value.to_string()),
        None => Value::Null,
    }
}

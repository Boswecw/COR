use serde_json::{json, Map, Value};

pub fn normalize_handoff(input: &Value) -> Result<Value, String> {
    let kind = required_string(input, "kind")?;
    match kind {
        "worm_centipede_handoff" => normalize_success_handoff(input),
        "worm_centipede_failure_handoff" => normalize_failure_handoff(input),
        _ => Err(format!("unsupported handoff kind: {}", kind)),
    }
}

fn normalize_success_handoff(input: &Value) -> Result<Value, String> {
    let source_handoff_id = required_string(input, "handoffId")?;
    let source_lane = optional_string(input, "sourceLane").unwrap_or("worm");
    let source_repo = required_string(input, "sourceRepo")?;
    let candidate_issues = normalize_candidate_issues(input, false)?;
    let blocking_count = candidate_issues
        .iter()
        .filter(|item| item.get("blocking").and_then(Value::as_bool).unwrap_or(false))
        .count() as u64;

    Ok(json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "success_path",
        "sourceLane": source_lane,
        "sourceRepo": source_repo,
        "sourceHandoffId": source_handoff_id,
        "candidateIssues": candidate_issues,
        "evidenceArtifacts": optional_string_array(input, "bundleIds")?,
        "totals": {
            "candidateIssues": candidate_count(input)?,
            "blocking": blocking_count
        },
        "receivedAt": required_string(input, "timestamp")?
    }))
}

fn normalize_failure_handoff(input: &Value) -> Result<Value, String> {
    let source_repo = required_string(input, "sourceRepo")?;
    let failure_kind = required_string(input, "failureKind")?;
    let source_handoff_id = optional_string(input, "handoffId")
        .map(str::to_owned)
        .unwrap_or_else(|| format!("worm-failure-{}", failure_kind));
    let candidate_issues = normalize_candidate_issues(input, true)?;
    let blocking_count = candidate_issues.len() as u64;

    Ok(json!({
        "kind": "centipede_candidate_issue_queue",
        "schemaVersion": 1,
        "intakeKind": "failure_path",
        "sourceLane": optional_string(input, "sourceLane").unwrap_or("worm"),
        "sourceRepo": source_repo,
        "sourceHandoffId": source_handoff_id,
        "failureKind": failure_kind,
        "severity": optional_string(input, "severity").unwrap_or("unknown"),
        "recommendedRoute": optional_string(input, "recommendedRoute").unwrap_or("operator_review"),
        "candidateIssues": candidate_issues,
        "evidenceArtifacts": optional_string_array(input, "evidenceArtifacts")?,
        "totals": {
            "candidateIssues": candidate_count(input)?,
            "blocking": blocking_count
        },
        "receivedAt": required_string(input, "timestamp")?
    }))
}

fn normalize_candidate_issues(input: &Value, force_blocking: bool) -> Result<Vec<Value>, String> {
    let raw = input
        .get("candidateIssueKeys")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing array field: candidateIssueKeys".to_string())?;

    let mut normalized = Vec::with_capacity(raw.len());

    for item in raw {
        normalized.push(normalize_candidate_issue(item, force_blocking)?);
    }

    Ok(normalized)
}

fn normalize_candidate_issue(item: &Value, force_blocking: bool) -> Result<Value, String> {
    if let Some(issue_key) = item.as_str() {
        return Ok(json!({
            "issueKey": issue_key,
            "blocking": force_blocking
        }));
    }

    let object = item
        .as_object()
        .ok_or_else(|| "array contained non-string/non-object value".to_string())?;

    let issue_key = required_string_from_map(object, "issueKey")?;
    let proposed_weight_class = optional_string_from_map(object, "proposedWeightClass");
    let blocking = if force_blocking {
        true
    } else {
        matches!(proposed_weight_class, Some("blocking"))
    };

    let mut out = Map::new();
    out.insert("issueKey".to_string(), Value::String(issue_key.to_string()));
    out.insert("blocking".to_string(), Value::Bool(blocking));

    if let Some(finding_class) = optional_string_from_map(object, "findingClass") {
        out.insert("findingClass".to_string(), Value::String(finding_class.to_string()));
    }

    if let Some(weight_class) = proposed_weight_class {
        out.insert(
            "proposedWeightClass".to_string(),
            Value::String(weight_class.to_string()),
        );
    }

    if let Some(confidence) = optional_string_from_map(object, "confidence") {
        out.insert("confidence".to_string(), Value::String(confidence.to_string()));
    }

    if let Some(severity) = optional_string_from_map(object, "severity") {
        out.insert("severity".to_string(), Value::String(severity.to_string()));
    }

    Ok(Value::Object(out))
}

fn candidate_count(input: &Value) -> Result<u64, String> {
    let value = input
        .get("candidateIssueKeys")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing array field: candidateIssueKeys".to_string())?;
    Ok(value.len() as u64)
}

fn optional_string_array(input: &Value, key: &str) -> Result<Vec<String>, String> {
    let Some(value) = input.get(key) else {
        return Ok(Vec::new());
    };

    let array = value
        .as_array()
        .ok_or_else(|| format!("field '{}' must be an array", key))?;

    let mut out = Vec::with_capacity(array.len());
    for item in array {
        let text = item
            .as_str()
            .ok_or_else(|| "array contained non-string value".to_string())?;
        out.push(text.to_string());
    }
    Ok(out)
}

fn required_string<'a>(input: &'a Value, key: &str) -> Result<&'a str, String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field: {}", key))
}

fn optional_string<'a>(input: &'a Value, key: &str) -> Option<&'a str> {
    input.get(key).and_then(Value::as_str)
}

fn required_string_from_map<'a>(input: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    input
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field: {}", key))
}

fn optional_string_from_map<'a>(input: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    input.get(key).and_then(Value::as_str)
}

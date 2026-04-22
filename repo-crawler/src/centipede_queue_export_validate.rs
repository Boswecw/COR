use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueExportValidation {
    #[serde(rename = "reportKind")]
    pub report_kind: String,
    #[serde(rename = "reportSchemaVersion")]
    pub report_schema_version: u64,
    #[serde(rename = "schemaFingerprint")]
    pub schema_fingerprint: String,
    #[serde(rename = "topLevelKeys")]
    pub top_level_keys: Vec<String>,
    #[serde(rename = "itemKeys")]
    pub item_keys: Vec<String>,
    #[serde(rename = "totalsKeys")]
    pub totals_keys: Vec<String>,
    #[serde(rename = "itemCount")]
    pub item_count: usize,
    #[serde(rename = "selectionQueueItemId", skip_serializing_if = "Option::is_none")]
    pub selection_queue_item_id: Option<String>,
}

pub fn validate_report_contract(
    report: &Value,
    outer_queue_item_id: Option<&str>,
) -> Result<QueueExportValidation, String> {
    let report_obj = report
        .as_object()
        .ok_or_else(|| "queue report input must be a JSON object".to_string())?;

    let report_kind = required_string_field(report_obj, "kind")?;
    if report_kind != "centipede_queue_operator_report" {
        return Err(format!(
            "report.kind must be 'centipede_queue_operator_report', found '{}'",
            report_kind
        ));
    }

    let report_schema_version = required_u64_field(report_obj, "schemaVersion")?;
    if report_schema_version == 0 {
        return Err("report.schemaVersion must be greater than 0".to_string());
    }

    let queue_dir = required_string_field(report_obj, "queueDir")?;
    if queue_dir.trim().is_empty() {
        return Err("report.queueDir must not be empty".to_string());
    }

    let items = required_array_field(report_obj, "items")?;
    let totals_obj = required_object_field(report_obj, "totals")?;
    let selection_obj = required_object_field(report_obj, "selection")?;
    let selection_queue_item_id = optional_string_field(selection_obj, "queueItemId")?;

    if let Some(outer) = outer_queue_item_id {
        match selection_queue_item_id.as_deref() {
            Some(inner) if inner == outer => {}
            Some(inner) => {
                return Err(format!(
                    "selection.queueItemId mismatch: outer selection '{}' does not match report selection '{}'",
                    outer, inner
                ));
            }
            None => {
                return Err(format!(
                    "selection.queueItemId mismatch: outer selection '{}' was provided but report selection is null",
                    outer
                ));
            }
        }
    }

    for (index, item) in items.iter().enumerate() {
        let item_obj = item
            .as_object()
            .ok_or_else(|| format!("report.items[{}] must be an object", index))?;
        let queue_item_id = required_string_field(item_obj, "queueItemId")?;
        let processing_state = required_object_field(item_obj, "processingState")?;
        let status = required_string_field(processing_state, "status")?;
        if status.trim().is_empty() {
            return Err(format!(
                "report.items[{}].processingState.status must not be empty",
                index
            ));
        }
        if let Some(selected) = selection_queue_item_id.as_deref() {
            if queue_item_id != selected {
                return Err(format!(
                    "report.items[{}].queueItemId '{}' does not match selection.queueItemId '{}'",
                    index, queue_item_id, selected
                ));
            }
        }
    }

    let mut top_level_keys = sorted_keys(report_obj);
    let mut totals_keys = sorted_keys(totals_obj);
    let item_keys = union_item_keys(items)?;

    top_level_keys.sort();
    totals_keys.sort();

    let fingerprint_source = format!(
        "kind={}|schemaVersion={}|top={}|item={}|totals={}",
        report_kind,
        report_schema_version,
        top_level_keys.join(","),
        item_keys.join(","),
        totals_keys.join(",")
    );
    let schema_fingerprint = fnv1a_64_hex(fingerprint_source.as_bytes());

    Ok(QueueExportValidation {
        report_kind: report_kind.to_string(),
        report_schema_version,
        schema_fingerprint,
        top_level_keys,
        item_keys,
        totals_keys,
        item_count: items.len(),
        selection_queue_item_id,
    })
}

fn union_item_keys(items: &[Value]) -> Result<Vec<String>, String> {
    let mut keys: Vec<String> = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let item_obj = item
            .as_object()
            .ok_or_else(|| format!("report.items[{}] must be an object", index))?;
        for key in item_obj.keys() {
            if !keys.iter().any(|existing| existing == key) {
                keys.push(key.clone());
            }
        }
    }

    keys.sort();
    Ok(keys)
}

fn sorted_keys(map: &Map<String, Value>) -> Vec<String> {
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort();
    keys
}

fn required_object_field<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Map<String, Value>, String> {
    map.get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("report.{} must be an object", key))
}

fn required_array_field<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Vec<Value>, String> {
    map.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("report.{} must be an array", key))
}

fn required_string_field<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    map.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("report.{} must be a string", key))
}

fn optional_string_field(map: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match map.get(key) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(format!("report.selection.{} must be a string or null", key)),
    }
}

fn required_u64_field(map: &Map<String, Value>, key: &str) -> Result<u64, String> {
    map.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("report.{} must be an unsigned integer", key))
}

fn fnv1a_64_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

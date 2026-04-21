use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use worm::adapter::{cortex_extraction_observations, repo_crawler_observations};
use worm::engine::Reconciler;
use worm::model::{
    ClaimObservationInput, EvidenceKind, EvidenceRecordInput, FreshnessStatus, HealthState,
    Polarity, Subsystem,
};
use worm::store::{ObservationFilter, Store};

#[test]
fn repo_crawler_file_presence_reconciles_to_confirmed() {
    let root = temp_root("confirmed");
    let store = Store::open(&root.join("worm.db")).expect("store");
    store
        .insert_observation(&observation(
            "file_presence",
            Subsystem::RepoCrawler,
            Polarity::Supports,
            FreshnessStatus::Fresh,
            0.98,
        ))
        .expect("insert observation");

    let observations = store
        .observations(&ObservationFilter {
            repository_id: None,
            revision_id: None,
            claim_class: None,
            target: None,
            since_ts: None,
        })
        .expect("observations");
    let decisions = Reconciler::default().reconcile(&observations);

    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].final_disposition.as_str(), "confirmed");
    assert!(decisions[0].applied_weights.weighted_support >= 0.78);
}

#[test]
fn missing_required_authority_requires_operator_review() {
    let observations = vec![stored_from_input(
        1,
        observation(
            "file_presence",
            Subsystem::DeterministicVerifier,
            Polarity::Supports,
            FreshnessStatus::Fresh,
            0.85,
        ),
    )];
    let decisions = Reconciler::default().reconcile(&observations);

    assert_eq!(decisions.len(), 1);
    assert_eq!(
        decisions[0].final_disposition.as_str(),
        "operator_review_required"
    );
}

#[test]
fn compiler_refutation_hard_gate_contradicts_compile_validity() {
    let mut crawler = observation(
        "compile_validity",
        Subsystem::RepoCrawler,
        Polarity::Supports,
        FreshnessStatus::Fresh,
        0.55,
    );
    crawler.value = serde_json::json!({ "source": "parser_appearance_only" });
    let compiler = observation(
        "compile_validity",
        Subsystem::CompilerToolchain,
        Polarity::Refutes,
        FreshnessStatus::Fresh,
        0.96,
    );
    let observations = vec![
        stored_from_input(1, crawler),
        stored_from_input(2, compiler),
    ];
    let decisions = Reconciler::default().reconcile(&observations);

    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].final_disposition.as_str(), "contradicted");
    assert_eq!(
        decisions[0].mismatch_type.as_deref(),
        Some("compile_vs_parse_disagreement")
    );
}

#[test]
fn repo_crawler_adapter_emits_first_tranche_claims() {
    let root = temp_root("adapter");
    let input = root.join("scan.json");
    fs::write(
        &input,
        serde_json::to_string(&serde_json::json!({
            "repo": {
                "root_path": "/tmp/example",
                "head_commit": "abc123"
            },
            "scan_run": {
                "scan_id": 7,
                "finished_ts": 1000
            },
            "files": [
                {
                    "file_id": 1,
                    "rel_path": "docs/example.md",
                    "sha256": "hash-a",
                    "lang": "md",
                    "parser_id": "line-parser@1",
                    "parse_status": "parsed"
                },
                {
                    "file_id": 2,
                    "rel_path": "src/bad.rs",
                    "sha256": "hash-b",
                    "lang": "rust",
                    "parser_id": "tree-sitter-rust@0.23",
                    "parse_status": "parse_error"
                }
            ],
            "diagnostics": []
        }))
        .expect("write scan export"),
    )
    .expect("json body");

    let observations = repo_crawler_observations(&input).expect("adapter observations");
    assert!(observations
        .iter()
        .any(|observation| observation.claim_class == "doc_presence"));
    assert!(observations.iter().any(|observation| {
        observation.claim_class == "parse_success" && observation.polarity == Polarity::Refutes
    }));
    assert!(observations
        .iter()
        .any(|observation| observation.claim_class == "file_hash"));
}

#[test]
fn cortex_extraction_adapter_emits_artifact_truth_claims() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/contracts/fixtures/valid/extraction-result-denied.json");
    let observations = cortex_extraction_observations(&fixture).expect("cortex observations");

    assert!(observations.iter().any(|observation| {
        observation.claim_class == "artifact_extraction_ready"
            && observation.polarity == Polarity::Refutes
    }));
    assert!(observations.iter().any(|observation| {
        observation.claim_class == "artifact_extraction_denied"
            && observation.polarity == Polarity::Supports
    }));
}

fn observation(
    claim_class: &str,
    subsystem: Subsystem,
    polarity: Polarity,
    freshness: FreshnessStatus,
    strength: f64,
) -> ClaimObservationInput {
    ClaimObservationInput {
        repository_id: "repo-1".to_string(),
        revision_id: Some("rev-1".to_string()),
        normalized_path: Some("src/lib.rs".to_string()),
        artifact_id: None,
        file_hash: Some("abc".to_string()),
        language_id: Some("rust".to_string()),
        source_scope: "repo".to_string(),
        claim_target_id: "file:src/lib.rs".to_string(),
        claim_class: claim_class.to_string(),
        subsystem,
        polarity,
        observed_at: 1,
        freshness_reference: Some("rev-1".to_string()),
        freshness_status: freshness,
        evidence_strength: Some(strength),
        subsystem_health: HealthState::Ready,
        value: serde_json::json!({ "test": true }),
        evidence: vec![EvidenceRecordInput {
            evidence_kind: EvidenceKind::DirectPath,
            reference: "test".to_string(),
            strength_score: strength,
            payload: serde_json::json!({}),
        }],
        diagnostics: Vec::new(),
    }
}

fn stored_from_input(
    observation_id: i64,
    observation: ClaimObservationInput,
) -> worm::model::StoredObservation {
    worm::model::StoredObservation {
        observation_id,
        repository_id: observation.repository_id,
        revision_id: observation.revision_id,
        normalized_path: observation.normalized_path,
        artifact_id: observation.artifact_id,
        file_hash: observation.file_hash,
        language_id: observation.language_id,
        source_scope: observation.source_scope,
        claim_target_id: observation.claim_target_id,
        claim_class: observation.claim_class,
        subsystem: observation.subsystem.as_str().to_string(),
        polarity: observation.polarity.as_str().to_string(),
        observed_at: observation.observed_at,
        freshness_reference: observation.freshness_reference,
        freshness_status: observation.freshness_status.as_str().to_string(),
        evidence_strength: observation.evidence_strength.unwrap_or(0.5),
        subsystem_health: observation.subsystem_health.as_str().to_string(),
        value: observation.value,
    }
}

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("worm-test-{name}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&root).expect("temp root");
    root
}

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const CONTRACT_VERSION: &str = "worm.reconciliation.v1";
pub const DEFAULT_STORE_REL_PATH: &str = ".worm/reconciliation.db";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Subsystem {
    Cortex,
    RepoCrawler,
    DeterministicVerifier,
    CompilerToolchain,
    GovernanceDocParity,
    Operator,
    Unknown,
}

impl Subsystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cortex => "cortex",
            Self::RepoCrawler => "repo_crawler",
            Self::DeterministicVerifier => "deterministic_verifier",
            Self::CompilerToolchain => "compiler_toolchain",
            Self::GovernanceDocParity => "governance_doc_parity",
            Self::Operator => "operator",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Polarity {
    Supports,
    Refutes,
    Unavailable,
    NotRun,
}

impl Polarity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supports => "supports",
            Self::Refutes => "refutes",
            Self::Unavailable => "unavailable",
            Self::NotRun => "not_run",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessStatus {
    Fresh,
    Historical,
    Stale,
    Unknown,
}

impl FreshnessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Historical => "historical",
            Self::Stale => "stale",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    Ready,
    Degraded,
    Unavailable,
    Stale,
    PartialSuccess,
    Denied,
    NotRun,
}

impl HealthState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
            Self::Stale => "stale",
            Self::PartialSuccess => "partial_success",
            Self::Denied => "denied",
            Self::NotRun => "not_run",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    DirectPath,
    DirectHash,
    ParserNode,
    ToolNative,
    RuleHit,
    DocReference,
    Heuristic,
    SubsystemHealth,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    GatePass,
    GateFail,
    GateConflict,
    GateUnverifiable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalDisposition {
    Confirmed,
    StrongSupport,
    ModerateSupport,
    Disputed,
    Stale,
    Contradicted,
    MissingRequiredEvidence,
    Unverifiable,
    AuthorityConflict,
    PolicyMismatch,
    CoverageGap,
    ResidualStaleArtifact,
    OperatorReviewRequired,
}

impl FinalDisposition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::StrongSupport => "strong_support",
            Self::ModerateSupport => "moderate_support",
            Self::Disputed => "disputed",
            Self::Stale => "stale",
            Self::Contradicted => "contradicted",
            Self::MissingRequiredEvidence => "missing_required_evidence",
            Self::Unverifiable => "unverifiable",
            Self::AuthorityConflict => "authority_conflict",
            Self::PolicyMismatch => "policy_mismatch",
            Self::CoverageGap => "coverage_gap",
            Self::ResidualStaleArtifact => "residual_stale_artifact",
            Self::OperatorReviewRequired => "operator_review_required",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorReviewState {
    Unreviewed,
    Accepted,
    Rejected,
    Deferred,
    NeedsFollowUp,
    HistoricalOnly,
}

impl OperatorReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unreviewed => "unreviewed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Deferred => "deferred",
            Self::NeedsFollowUp => "needs_follow_up",
            Self::HistoricalOnly => "historical_only",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecordInput {
    pub evidence_kind: EvidenceKind,
    pub reference: String,
    #[serde(default = "default_evidence_strength")]
    pub strength_score: f64,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticInput {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimObservationInput {
    pub repository_id: String,
    #[serde(default)]
    pub revision_id: Option<String>,
    #[serde(default)]
    pub normalized_path: Option<String>,
    #[serde(default)]
    pub artifact_id: Option<String>,
    #[serde(default)]
    pub file_hash: Option<String>,
    #[serde(default)]
    pub language_id: Option<String>,
    #[serde(default = "default_source_scope")]
    pub source_scope: String,
    pub claim_target_id: String,
    pub claim_class: String,
    pub subsystem: Subsystem,
    pub polarity: Polarity,
    #[serde(default = "now_ts")]
    pub observed_at: i64,
    #[serde(default)]
    pub freshness_reference: Option<String>,
    #[serde(default = "default_freshness")]
    pub freshness_status: FreshnessStatus,
    #[serde(default)]
    pub evidence_strength: Option<f64>,
    #[serde(default = "default_health")]
    pub subsystem_health: HealthState,
    #[serde(default)]
    pub value: serde_json::Value,
    #[serde(default)]
    pub evidence: Vec<EvidenceRecordInput>,
    #[serde(default)]
    pub diagnostics: Vec<DiagnosticInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredObservation {
    pub observation_id: i64,
    pub repository_id: String,
    pub revision_id: Option<String>,
    pub normalized_path: Option<String>,
    pub artifact_id: Option<String>,
    pub file_hash: Option<String>,
    pub language_id: Option<String>,
    pub source_scope: String,
    pub claim_target_id: String,
    pub claim_class: String,
    pub subsystem: String,
    pub polarity: String,
    pub observed_at: i64,
    pub freshness_reference: Option<String>,
    pub freshness_status: String,
    pub evidence_strength: f64,
    pub subsystem_health: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_name: String,
    pub status: GateStatus,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedWeights {
    pub authority_score: f64,
    pub evidence_strength_score: f64,
    pub freshness_score: f64,
    pub agreement_score: f64,
    pub contradiction_penalty: f64,
    pub weighted_support: f64,
    pub weighted_refutation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationDecisionInput {
    pub repository_id: String,
    pub revision_id: Option<String>,
    pub claim_target_id: String,
    pub claim_class: String,
    pub final_disposition: FinalDisposition,
    pub confidence_band: String,
    pub freshness_posture: String,
    pub mismatch_type: Option<String>,
    pub rationale: String,
    pub observation_ids: Vec<i64>,
    pub contributing_systems: Vec<String>,
    pub gate_results: Vec<GateResult>,
    pub applied_weights: AppliedWeights,
    pub review_state: OperatorReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDecision {
    pub decision_id: i64,
    pub run_id: i64,
    pub repository_id: String,
    pub revision_id: Option<String>,
    pub claim_target_id: String,
    pub claim_class: String,
    pub final_disposition: String,
    pub confidence_band: String,
    pub freshness_posture: String,
    pub mismatch_type: Option<String>,
    pub rationale: String,
    pub observation_ids: Vec<i64>,
    pub contributing_systems: Vec<String>,
    pub gate_results: Vec<GateResult>,
    pub applied_weights: AppliedWeights,
    pub review_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub contract_version: String,
    pub run_id: i64,
    pub started_at: i64,
    pub finished_at: i64,
    pub status: String,
    pub decisions: Vec<StoredDecision>,
}

pub fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn default_source_scope() -> String {
    "repo".to_string()
}

fn default_freshness() -> FreshnessStatus {
    FreshnessStatus::Unknown
}

fn default_health() -> HealthState {
    HealthState::Ready
}

fn default_evidence_strength() -> f64 {
    0.5
}

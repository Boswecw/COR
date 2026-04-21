use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::Result;
use crate::model::{
    now_ts, AppliedWeights, ClaimObservationInput, DiagnosticInput, EvidenceRecordInput,
    GateResult, GateStatus, HealthState, OperatorReviewState, ReconciliationDecisionInput,
    ReconciliationReport, StoredDecision, StoredObservation, CONTRACT_VERSION,
};
use crate::profiles::PROFILE_VERSION;

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct ObservationFilter {
    pub repository_id: Option<String>,
    pub revision_id: Option<String>,
    pub claim_class: Option<String>,
    pub target: Option<String>,
    pub since_ts: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DecisionFilter {
    pub run_id: Option<i64>,
    pub disposition: Option<String>,
    pub claim_class: Option<String>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let store = Self {
            path: path.to_path_buf(),
            conn,
        };
        store.bootstrap()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn bootstrap(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS repositories (
                repository_id TEXT PRIMARY KEY,
                root_path TEXT,
                first_seen_ts INTEGER NOT NULL,
                last_seen_ts INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS revisions (
                revision_key TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL REFERENCES repositories(repository_id),
                revision_id TEXT,
                observed_at INTEGER NOT NULL,
                UNIQUE(repository_id, revision_id)
            );

            CREATE TABLE IF NOT EXISTS claim_targets (
                claim_target_id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL REFERENCES repositories(repository_id),
                normalized_path TEXT,
                artifact_id TEXT,
                source_scope TEXT NOT NULL,
                latest_file_hash TEXT,
                first_seen_ts INTEGER NOT NULL,
                last_seen_ts INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS claim_observations (
                observation_id INTEGER PRIMARY KEY AUTOINCREMENT,
                repository_id TEXT NOT NULL REFERENCES repositories(repository_id),
                revision_id TEXT,
                normalized_path TEXT,
                artifact_id TEXT,
                file_hash TEXT,
                language_id TEXT,
                source_scope TEXT NOT NULL,
                claim_target_id TEXT NOT NULL,
                claim_class TEXT NOT NULL,
                subsystem TEXT NOT NULL,
                polarity TEXT NOT NULL,
                observed_at INTEGER NOT NULL,
                freshness_reference TEXT,
                freshness_status TEXT NOT NULL,
                evidence_strength REAL NOT NULL,
                subsystem_health TEXT NOT NULL,
                value_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS evidence_records (
                evidence_id INTEGER PRIMARY KEY AUTOINCREMENT,
                observation_id INTEGER NOT NULL REFERENCES claim_observations(observation_id) ON DELETE CASCADE,
                evidence_kind TEXT NOT NULL,
                reference TEXT NOT NULL,
                strength_score REAL NOT NULL,
                payload_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS diagnostics (
                diagnostic_id INTEGER PRIMARY KEY AUTOINCREMENT,
                observation_id INTEGER REFERENCES claim_observations(observation_id) ON DELETE CASCADE,
                run_id INTEGER,
                severity TEXT NOT NULL,
                code TEXT NOT NULL,
                message TEXT NOT NULL,
                payload_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS gate_results (
                gate_result_id INTEGER PRIMARY KEY AUTOINCREMENT,
                decision_id INTEGER NOT NULL,
                gate_name TEXT NOT NULL,
                status TEXT NOT NULL,
                rationale TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS weight_profiles (
                profile_version TEXT PRIMARY KEY,
                profile_json TEXT NOT NULL,
                installed_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS reconciliation_runs (
                run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at INTEGER NOT NULL,
                finished_at INTEGER,
                status TEXT NOT NULL,
                profile_version TEXT NOT NULL,
                observation_count INTEGER NOT NULL,
                decision_count INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS reconciliation_decisions (
                decision_id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id INTEGER NOT NULL REFERENCES reconciliation_runs(run_id),
                repository_id TEXT NOT NULL,
                revision_id TEXT,
                claim_target_id TEXT NOT NULL,
                claim_class TEXT NOT NULL,
                final_disposition TEXT NOT NULL,
                confidence_band TEXT NOT NULL,
                freshness_posture TEXT NOT NULL,
                mismatch_type TEXT,
                rationale TEXT NOT NULL,
                observation_ids_json TEXT NOT NULL,
                contributing_systems_json TEXT NOT NULL,
                applied_weights_json TEXT NOT NULL,
                review_state TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS operator_reviews (
                review_id INTEGER PRIMARY KEY AUTOINCREMENT,
                decision_id INTEGER NOT NULL REFERENCES reconciliation_decisions(decision_id),
                review_state TEXT NOT NULL,
                reviewer TEXT,
                notes TEXT,
                reviewed_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS subsystem_health (
                health_id INTEGER PRIMARY KEY AUTOINCREMENT,
                repository_id TEXT NOT NULL,
                subsystem TEXT NOT NULL,
                health_state TEXT NOT NULL,
                observed_at INTEGER NOT NULL,
                payload_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_observations_target
                ON claim_observations(repository_id, revision_id, claim_target_id, claim_class);
            CREATE INDEX IF NOT EXISTS idx_observations_claim_class
                ON claim_observations(claim_class, subsystem, polarity);
            CREATE INDEX IF NOT EXISTS idx_decisions_run
                ON reconciliation_decisions(run_id, final_disposition, claim_class);
            ",
        )?;
        self.install_default_weight_profile()?;
        Ok(())
    }

    pub fn insert_observation(&self, observation: &ClaimObservationInput) -> Result<i64> {
        let evidence_strength = observation
            .evidence_strength
            .unwrap_or_else(|| max_evidence_strength(&observation.evidence))
            .clamp(0.0, 1.0);
        self.upsert_repository(&observation.repository_id)?;
        self.upsert_revision(
            &observation.repository_id,
            observation.revision_id.as_deref(),
        )?;
        self.upsert_claim_target(observation)?;
        self.conn.execute(
            "
            INSERT INTO claim_observations(
                repository_id, revision_id, normalized_path, artifact_id, file_hash,
                language_id, source_scope, claim_target_id, claim_class, subsystem,
                polarity, observed_at, freshness_reference, freshness_status,
                evidence_strength, subsystem_health, value_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ",
            params![
                observation.repository_id,
                observation.revision_id,
                observation.normalized_path,
                observation.artifact_id,
                observation.file_hash,
                observation.language_id,
                observation.source_scope,
                observation.claim_target_id,
                observation.claim_class,
                observation.subsystem.as_str(),
                observation.polarity.as_str(),
                observation.observed_at,
                observation.freshness_reference,
                observation.freshness_status.as_str(),
                evidence_strength,
                observation.subsystem_health.as_str(),
                serde_json::to_string(&observation.value)?,
            ],
        )?;
        let observation_id = self.conn.last_insert_rowid();
        self.insert_evidence(observation_id, &observation.evidence)?;
        self.insert_diagnostics(observation_id, &observation.diagnostics)?;
        self.record_subsystem_health(
            &observation.repository_id,
            observation.subsystem.as_str(),
            &observation.subsystem_health,
            observation.observed_at,
        )?;
        Ok(observation_id)
    }

    pub fn insert_observations(&self, observations: &[ClaimObservationInput]) -> Result<Vec<i64>> {
        observations
            .iter()
            .map(|observation| self.insert_observation(observation))
            .collect()
    }

    pub fn observations(&self, filter: &ObservationFilter) -> Result<Vec<StoredObservation>> {
        let mut sql = "
            SELECT observation_id, repository_id, revision_id, normalized_path, artifact_id,
                   file_hash, language_id, source_scope, claim_target_id, claim_class,
                   subsystem, polarity, observed_at, freshness_reference, freshness_status,
                   evidence_strength, subsystem_health, value_json
            FROM claim_observations
            WHERE 1 = 1
        "
        .to_string();
        let mut params = Vec::<String>::new();

        if let Some(repository_id) = &filter.repository_id {
            sql.push_str(" AND repository_id = ?");
            params.push(repository_id.clone());
        }
        if let Some(revision_id) = &filter.revision_id {
            sql.push_str(" AND revision_id = ?");
            params.push(revision_id.clone());
        }
        if let Some(claim_class) = &filter.claim_class {
            sql.push_str(" AND claim_class = ?");
            params.push(claim_class.clone());
        }
        if let Some(target) = &filter.target {
            sql.push_str(" AND claim_target_id = ?");
            params.push(target.clone());
        }
        if filter.since_ts.is_some() {
            sql.push_str(" AND observed_at >= ?");
        }
        sql.push_str(
            " ORDER BY repository_id, revision_id, claim_target_id, claim_class, observation_id",
        );

        let mut statement = self.conn.prepare(&sql)?;
        let mut values = params
            .iter()
            .map(|value| value as &dyn rusqlite::ToSql)
            .collect::<Vec<_>>();
        let since_value;
        if let Some(since_ts) = filter.since_ts {
            since_value = since_ts;
            values.push(&since_value);
        }
        let rows = statement.query_map(values.as_slice(), map_observation)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn begin_reconciliation_run(&self, observation_count: usize) -> Result<i64> {
        self.conn.execute(
            "
            INSERT INTO reconciliation_runs(
                started_at, status, profile_version, observation_count, decision_count
            )
            VALUES (?1, 'running', ?2, ?3, 0)
            ",
            params![now_ts(), PROFILE_VERSION, observation_count as i64],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn finish_reconciliation_run(
        &self,
        run_id: i64,
        status: &str,
        decision_count: usize,
    ) -> Result<()> {
        self.conn.execute(
            "
            UPDATE reconciliation_runs
            SET finished_at = ?2, status = ?3, decision_count = ?4
            WHERE run_id = ?1
            ",
            params![run_id, now_ts(), status, decision_count as i64],
        )?;
        Ok(())
    }

    pub fn insert_decision(
        &self,
        run_id: i64,
        decision: &ReconciliationDecisionInput,
    ) -> Result<i64> {
        self.conn.execute(
            "
            INSERT INTO reconciliation_decisions(
                run_id, repository_id, revision_id, claim_target_id, claim_class,
                final_disposition, confidence_band, freshness_posture, mismatch_type,
                rationale, observation_ids_json, contributing_systems_json,
                applied_weights_json, review_state
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ",
            params![
                run_id,
                decision.repository_id,
                decision.revision_id,
                decision.claim_target_id,
                decision.claim_class,
                decision.final_disposition.as_str(),
                decision.confidence_band,
                decision.freshness_posture,
                decision.mismatch_type,
                decision.rationale,
                serde_json::to_string(&decision.observation_ids)?,
                serde_json::to_string(&decision.contributing_systems)?,
                serde_json::to_string(&decision.applied_weights)?,
                decision.review_state.as_str(),
            ],
        )?;
        let decision_id = self.conn.last_insert_rowid();
        self.insert_gate_results(decision_id, &decision.gate_results)?;
        Ok(decision_id)
    }

    pub fn decisions(&self, filter: &DecisionFilter) -> Result<Vec<StoredDecision>> {
        let mut sql = "
            SELECT decision_id, run_id, repository_id, revision_id, claim_target_id,
                   claim_class, final_disposition, confidence_band, freshness_posture,
                   mismatch_type, rationale, observation_ids_json, contributing_systems_json,
                   applied_weights_json, review_state
            FROM reconciliation_decisions
            WHERE 1 = 1
        "
        .to_string();
        let mut params = Vec::<rusqlite::types::Value>::new();

        if let Some(run_id) = filter.run_id {
            sql.push_str(" AND run_id = ?");
            params.push(rusqlite::types::Value::Integer(run_id));
        }
        if let Some(disposition) = &filter.disposition {
            sql.push_str(" AND final_disposition = ?");
            params.push(rusqlite::types::Value::Text(disposition.clone()));
        }
        if let Some(claim_class) = &filter.claim_class {
            sql.push_str(" AND claim_class = ?");
            params.push(rusqlite::types::Value::Text(claim_class.clone()));
        }
        sql.push_str(" ORDER BY run_id DESC, final_disposition, claim_class, claim_target_id");

        let mut statement = self.conn.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(params), map_decision)?;
        let mut decisions = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        for decision in &mut decisions {
            decision.gate_results = self.gate_results(decision.decision_id)?;
        }
        Ok(decisions)
    }

    pub fn reconciliation_report(&self, run_id: i64) -> Result<ReconciliationReport> {
        let (started_at, finished_at, status): (i64, Option<i64>, String) = self.conn.query_row(
            "
            SELECT started_at, finished_at, status
            FROM reconciliation_runs
            WHERE run_id = ?1
            ",
            params![run_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let decisions = self.decisions(&DecisionFilter {
            run_id: Some(run_id),
            disposition: None,
            claim_class: None,
        })?;
        Ok(ReconciliationReport {
            contract_version: CONTRACT_VERSION.to_string(),
            run_id,
            started_at,
            finished_at: finished_at.unwrap_or_else(now_ts),
            status,
            decisions,
        })
    }

    pub fn update_review(
        &self,
        decision_id: i64,
        review_state: OperatorReviewState,
        reviewer: Option<&str>,
        notes: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "
            UPDATE reconciliation_decisions
            SET review_state = ?2
            WHERE decision_id = ?1
            ",
            params![decision_id, review_state.as_str()],
        )?;
        self.conn.execute(
            "
            INSERT INTO operator_reviews(decision_id, review_state, reviewer, notes, reviewed_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                decision_id,
                review_state.as_str(),
                reviewer,
                notes,
                now_ts()
            ],
        )?;
        Ok(())
    }

    pub fn observation_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM claim_observations", [], |row| {
                row.get(0)
            })
            .map_err(Into::into)
    }

    pub fn decision_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM reconciliation_decisions", [], |row| {
                row.get(0)
            })
            .map_err(Into::into)
    }

    fn upsert_repository(&self, repository_id: &str) -> Result<()> {
        self.conn.execute(
            "
            INSERT INTO repositories(repository_id, first_seen_ts, last_seen_ts)
            VALUES (?1, ?2, ?2)
            ON CONFLICT(repository_id) DO UPDATE SET last_seen_ts = excluded.last_seen_ts
            ",
            params![repository_id, now_ts()],
        )?;
        Ok(())
    }

    fn upsert_revision(&self, repository_id: &str, revision_id: Option<&str>) -> Result<()> {
        if let Some(revision_id) = revision_id {
            let revision_key = format!("{repository_id}:{revision_id}");
            self.conn.execute(
                "
                INSERT INTO revisions(revision_key, repository_id, revision_id, observed_at)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(repository_id, revision_id) DO UPDATE SET observed_at = excluded.observed_at
                ",
                params![revision_key, repository_id, revision_id, now_ts()],
            )?;
        }
        Ok(())
    }

    fn upsert_claim_target(&self, observation: &ClaimObservationInput) -> Result<()> {
        self.conn.execute(
            "
            INSERT INTO claim_targets(
                claim_target_id, repository_id, normalized_path, artifact_id, source_scope,
                latest_file_hash, first_seen_ts, last_seen_ts
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
            ON CONFLICT(claim_target_id) DO UPDATE SET
                normalized_path = COALESCE(excluded.normalized_path, claim_targets.normalized_path),
                artifact_id = COALESCE(excluded.artifact_id, claim_targets.artifact_id),
                latest_file_hash = COALESCE(excluded.latest_file_hash, claim_targets.latest_file_hash),
                last_seen_ts = excluded.last_seen_ts
            ",
            params![
                observation.claim_target_id,
                observation.repository_id,
                observation.normalized_path,
                observation.artifact_id,
                observation.source_scope,
                observation.file_hash,
                now_ts(),
            ],
        )?;
        Ok(())
    }

    fn insert_evidence(&self, observation_id: i64, evidence: &[EvidenceRecordInput]) -> Result<()> {
        for record in evidence {
            self.conn.execute(
                "
                INSERT INTO evidence_records(
                    observation_id, evidence_kind, reference, strength_score, payload_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5)
                ",
                params![
                    observation_id,
                    format!("{:?}", record.evidence_kind).to_ascii_lowercase(),
                    record.reference,
                    record.strength_score.clamp(0.0, 1.0),
                    serde_json::to_string(&record.payload)?,
                ],
            )?;
        }
        Ok(())
    }

    fn insert_diagnostics(
        &self,
        observation_id: i64,
        diagnostics: &[DiagnosticInput],
    ) -> Result<()> {
        for diagnostic in diagnostics {
            self.conn.execute(
                "
                INSERT INTO diagnostics(
                    observation_id, severity, code, message, payload_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5)
                ",
                params![
                    observation_id,
                    diagnostic.severity,
                    diagnostic.code,
                    diagnostic.message,
                    serde_json::to_string(&diagnostic.payload)?,
                ],
            )?;
        }
        Ok(())
    }

    fn insert_gate_results(&self, decision_id: i64, gates: &[GateResult]) -> Result<()> {
        for gate in gates {
            self.conn.execute(
                "
                INSERT INTO gate_results(decision_id, gate_name, status, rationale)
                VALUES (?1, ?2, ?3, ?4)
                ",
                params![
                    decision_id,
                    gate.gate_name,
                    gate_status_str(&gate.status),
                    gate.rationale,
                ],
            )?;
        }
        Ok(())
    }

    fn gate_results(&self, decision_id: i64) -> Result<Vec<GateResult>> {
        let mut statement = self.conn.prepare(
            "
            SELECT gate_name, status, rationale
            FROM gate_results
            WHERE decision_id = ?1
            ORDER BY gate_result_id
            ",
        )?;
        let rows = statement.query_map(params![decision_id], |row| {
            let status: String = row.get(1)?;
            Ok(GateResult {
                gate_name: row.get(0)?,
                status: parse_gate_status(&status),
                rationale: row.get(2)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    fn record_subsystem_health(
        &self,
        repository_id: &str,
        subsystem: &str,
        health: &HealthState,
        observed_at: i64,
    ) -> Result<()> {
        self.conn.execute(
            "
            INSERT INTO subsystem_health(
                repository_id, subsystem, health_state, observed_at, payload_json
            )
            VALUES (?1, ?2, ?3, ?4, '{}')
            ",
            params![repository_id, subsystem, health.as_str(), observed_at],
        )?;
        Ok(())
    }

    fn install_default_weight_profile(&self) -> Result<()> {
        let installed = self
            .conn
            .query_row(
                "SELECT profile_version FROM weight_profiles WHERE profile_version = ?1",
                params![PROFILE_VERSION],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        if installed.is_none() {
            self.conn.execute(
                "
                INSERT INTO weight_profiles(profile_version, profile_json, installed_at)
                VALUES (?1, ?2, ?3)
                ",
                params![
                    PROFILE_VERSION,
                    serde_json::json!({
                        "contract": CONTRACT_VERSION,
                        "profile": PROFILE_VERSION,
                        "posture": "operator_defined_conservative_v1"
                    })
                    .to_string(),
                    now_ts(),
                ],
            )?;
        }
        Ok(())
    }
}

fn map_observation(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredObservation> {
    let value_json: String = row.get(17)?;
    Ok(StoredObservation {
        observation_id: row.get(0)?,
        repository_id: row.get(1)?,
        revision_id: row.get(2)?,
        normalized_path: row.get(3)?,
        artifact_id: row.get(4)?,
        file_hash: row.get(5)?,
        language_id: row.get(6)?,
        source_scope: row.get(7)?,
        claim_target_id: row.get(8)?,
        claim_class: row.get(9)?,
        subsystem: row.get(10)?,
        polarity: row.get(11)?,
        observed_at: row.get(12)?,
        freshness_reference: row.get(13)?,
        freshness_status: row.get(14)?,
        evidence_strength: row.get(15)?,
        subsystem_health: row.get(16)?,
        value: serde_json::from_str(&value_json).unwrap_or_else(|_| serde_json::json!({})),
    })
}

fn map_decision(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredDecision> {
    let observation_ids_json: String = row.get(11)?;
    let contributing_systems_json: String = row.get(12)?;
    let applied_weights_json: String = row.get(13)?;
    Ok(StoredDecision {
        decision_id: row.get(0)?,
        run_id: row.get(1)?,
        repository_id: row.get(2)?,
        revision_id: row.get(3)?,
        claim_target_id: row.get(4)?,
        claim_class: row.get(5)?,
        final_disposition: row.get(6)?,
        confidence_band: row.get(7)?,
        freshness_posture: row.get(8)?,
        mismatch_type: row.get(9)?,
        rationale: row.get(10)?,
        observation_ids: serde_json::from_str(&observation_ids_json).unwrap_or_default(),
        contributing_systems: serde_json::from_str(&contributing_systems_json).unwrap_or_default(),
        gate_results: Vec::new(),
        applied_weights: serde_json::from_str::<AppliedWeights>(&applied_weights_json).unwrap_or(
            AppliedWeights {
                authority_score: 0.0,
                evidence_strength_score: 0.0,
                freshness_score: 0.0,
                agreement_score: 0.0,
                contradiction_penalty: 0.0,
                weighted_support: 0.0,
                weighted_refutation: 0.0,
            },
        ),
        review_state: row.get(14)?,
    })
}

fn max_evidence_strength(evidence: &[EvidenceRecordInput]) -> f64 {
    evidence
        .iter()
        .map(|record| record.strength_score)
        .fold(0.5_f64, f64::max)
}

fn gate_status_str(status: &GateStatus) -> &'static str {
    match status {
        GateStatus::GatePass => "gate_pass",
        GateStatus::GateFail => "gate_fail",
        GateStatus::GateConflict => "gate_conflict",
        GateStatus::GateUnverifiable => "gate_unverifiable",
    }
}

fn parse_gate_status(value: &str) -> GateStatus {
    match value {
        "gate_pass" => GateStatus::GatePass,
        "gate_fail" => GateStatus::GateFail,
        "gate_conflict" => GateStatus::GateConflict,
        "gate_unverifiable" => GateStatus::GateUnverifiable,
        _ => GateStatus::GateUnverifiable,
    }
}

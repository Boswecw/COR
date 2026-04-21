use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    AppliedWeights, FinalDisposition, FreshnessStatus, GateResult, GateStatus, OperatorReviewState,
    Polarity, ReconciliationDecisionInput, StoredObservation, Subsystem,
};
use crate::profiles::{ProfileRegistry, WeightProfile};

#[derive(Debug, Clone)]
pub struct Reconciler {
    profiles: ProfileRegistry,
}

#[derive(Debug, Clone)]
struct ObservationScore {
    subsystem: String,
    polarity: Polarity,
    authority: f64,
    evidence: f64,
    freshness: f64,
    support_score: f64,
    refute_score: f64,
}

impl Default for Reconciler {
    fn default() -> Self {
        Self {
            profiles: ProfileRegistry::default(),
        }
    }
}

impl Reconciler {
    pub fn reconcile(
        &self,
        observations: &[StoredObservation],
    ) -> Vec<ReconciliationDecisionInput> {
        let mut groups =
            BTreeMap::<(String, Option<String>, String, String), Vec<StoredObservation>>::new();
        for observation in observations {
            groups
                .entry((
                    observation.repository_id.clone(),
                    observation.revision_id.clone(),
                    observation.claim_target_id.clone(),
                    observation.claim_class.clone(),
                ))
                .or_default()
                .push(observation.clone());
        }

        groups
            .into_values()
            .map(|group| self.reconcile_group(&group))
            .collect()
    }

    fn reconcile_group(&self, observations: &[StoredObservation]) -> ReconciliationDecisionInput {
        let first = observations.first().expect("non-empty observation group");
        let profile = self.profiles.weight_profile(&first.claim_class);
        let gate_results = self.apply_hard_gates(observations, &profile);
        let scores = observations
            .iter()
            .map(|observation| self.score_observation(observation, &profile, observations))
            .collect::<Vec<_>>();
        let support = scores.iter().map(|score| score.support_score).sum::<f64>();
        let refute = scores.iter().map(|score| score.refute_score).sum::<f64>();
        let authority_score = scores
            .iter()
            .map(|score| score.authority)
            .fold(0.0_f64, f64::max);
        let evidence_strength_score = scores
            .iter()
            .map(|score| score.evidence)
            .fold(0.0_f64, f64::max);
        let freshness_score = scores
            .iter()
            .map(|score| score.freshness)
            .fold(0.0_f64, f64::max);
        let agreement_score = agreement_score(&scores);
        let contradiction_penalty = contradiction_penalty(&scores);
        let applied_weights = AppliedWeights {
            authority_score,
            evidence_strength_score,
            freshness_score,
            agreement_score,
            contradiction_penalty,
            weighted_support: round3(support),
            weighted_refutation: round3(refute),
        };

        let (final_disposition, confidence_band, freshness_posture, mismatch_type, rationale) =
            self.disposition(
                observations,
                &gate_results,
                &scores,
                support,
                refute,
                &profile,
            );

        let observation_ids = observations
            .iter()
            .map(|observation| observation.observation_id)
            .collect::<Vec<_>>();
        let contributing_systems = scores
            .iter()
            .map(|score| score.subsystem.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        ReconciliationDecisionInput {
            repository_id: first.repository_id.clone(),
            revision_id: first.revision_id.clone(),
            claim_target_id: first.claim_target_id.clone(),
            claim_class: first.claim_class.clone(),
            final_disposition,
            confidence_band,
            freshness_posture,
            mismatch_type,
            rationale,
            observation_ids,
            contributing_systems,
            gate_results,
            applied_weights,
            review_state: OperatorReviewState::Unreviewed,
        }
    }

    fn apply_hard_gates(
        &self,
        observations: &[StoredObservation],
        profile: &WeightProfile,
    ) -> Vec<GateResult> {
        let mut gates = Vec::new();
        let first = observations.first().expect("non-empty observation group");

        if first.claim_target_id.trim().is_empty() {
            gates.push(GateResult {
                gate_name: "identity_binding".to_string(),
                status: GateStatus::GateUnverifiable,
                rationale: "missing claim_target_id".to_string(),
            });
        } else {
            gates.push(GateResult {
                gate_name: "identity_binding".to_string(),
                status: GateStatus::GatePass,
                rationale: "claim_target_id present".to_string(),
            });
        }

        if let Some(required) = &profile.required_authority {
            let required_name = required.as_str();
            let required_present = observations
                .iter()
                .any(|observation| observation.subsystem == required_name);
            gates.push(GateResult {
                gate_name: "required_authority_presence".to_string(),
                status: if required_present {
                    GateStatus::GatePass
                } else {
                    GateStatus::GateUnverifiable
                },
                rationale: if required_present {
                    format!("{required_name} observation present")
                } else {
                    format!("{required_name} observation missing for claim class")
                },
            });
        }

        if observations.iter().all(|observation| {
            matches!(
                parse_polarity(&observation.polarity),
                Polarity::Unavailable | Polarity::NotRun
            )
        }) {
            gates.push(GateResult {
                gate_name: "all_sources_unavailable".to_string(),
                status: GateStatus::GateUnverifiable,
                rationale: "all observations are unavailable or not_run".to_string(),
            });
        }

        if first.claim_class == "compile_validity"
            && observations.iter().any(|observation| {
                observation.subsystem == Subsystem::CompilerToolchain.as_str()
                    && parse_polarity(&observation.polarity) == Polarity::Refutes
            })
        {
            gates.push(GateResult {
                gate_name: "compiler_truth_refutation".to_string(),
                status: GateStatus::GateFail,
                rationale: "compiler/toolchain lane refutes compile_validity".to_string(),
            });
        }

        if first.claim_class == "doc_parity"
            && observations.iter().any(|observation| {
                observation.subsystem == Subsystem::GovernanceDocParity.as_str()
                    && parse_polarity(&observation.polarity) == Polarity::Refutes
            })
        {
            gates.push(GateResult {
                gate_name: "doc_parity_refutation".to_string(),
                status: GateStatus::GateFail,
                rationale: "governance/doc parity lane refutes doc_parity".to_string(),
            });
        }

        if first.claim_class == "artifact_extraction_ready"
            && observations.iter().any(|observation| {
                observation.subsystem == Subsystem::Cortex.as_str()
                    && matches!(
                        parse_polarity(&observation.polarity),
                        Polarity::Refutes | Polarity::Unavailable
                    )
            })
        {
            gates.push(GateResult {
                gate_name: "cortex_artifact_denial".to_string(),
                status: GateStatus::GateFail,
                rationale: "Cortex refutes or cannot provide artifact extraction readiness"
                    .to_string(),
            });
        }

        if observations.iter().any(|observation| {
            parse_freshness(&observation.freshness_status) == FreshnessStatus::Stale
                && parse_polarity(&observation.polarity) == Polarity::Supports
        }) && !observations.iter().any(|observation| {
            parse_freshness(&observation.freshness_status) == FreshnessStatus::Fresh
                && parse_polarity(&observation.polarity) == Polarity::Supports
        }) {
            gates.push(GateResult {
                gate_name: "stale_support_only".to_string(),
                status: GateStatus::GateConflict,
                rationale: "supporting evidence exists only in stale posture".to_string(),
            });
        }

        gates
    }

    fn score_observation(
        &self,
        observation: &StoredObservation,
        profile: &WeightProfile,
        all_observations: &[StoredObservation],
    ) -> ObservationScore {
        let polarity = parse_polarity(&observation.polarity);
        let authority = self
            .profiles
            .authority_score(&observation.subsystem, &observation.claim_class);
        let evidence = observation.evidence_strength.clamp(0.0, 1.0);
        let freshness = freshness_score(&observation.freshness_status);
        let health_factor = health_factor(&observation.subsystem_health);
        let agreement = agreement_score_for(observation, all_observations);
        let contradiction =
            contradiction_penalty_for(observation, all_observations, &self.profiles);
        let weighted = ((authority * profile.authority_weight)
            + (evidence * profile.evidence_weight)
            + (freshness * profile.freshness_weight)
            + (agreement * profile.agreement_weight)
            - (contradiction * profile.contradiction_weight))
            .max(0.0)
            * health_factor;

        ObservationScore {
            subsystem: observation.subsystem.clone(),
            polarity: polarity.clone(),
            authority,
            evidence,
            freshness,
            support_score: if polarity == Polarity::Supports {
                round3(weighted)
            } else {
                0.0
            },
            refute_score: if polarity == Polarity::Refutes {
                round3(weighted)
            } else {
                0.0
            },
        }
    }

    fn disposition(
        &self,
        observations: &[StoredObservation],
        gates: &[GateResult],
        scores: &[ObservationScore],
        support: f64,
        refute: f64,
        profile: &WeightProfile,
    ) -> (FinalDisposition, String, String, Option<String>, String) {
        let claim_class = &observations[0].claim_class;
        if gates.iter().any(|gate| {
            gate.status == GateStatus::GateUnverifiable && gate.gate_name == "identity_binding"
        }) {
            return (
                FinalDisposition::Unverifiable,
                "none".to_string(),
                freshness_posture(observations),
                Some("identity_binding_conflict".to_string()),
                "identity hard gate could not bind the claim target".to_string(),
            );
        }

        if gates.iter().any(|gate| {
            gate.status == GateStatus::GateUnverifiable
                && gate.gate_name == "all_sources_unavailable"
        }) {
            return (
                FinalDisposition::MissingRequiredEvidence,
                "none".to_string(),
                freshness_posture(observations),
                Some("lane_unavailability_conflict".to_string()),
                "no runnable subsystem produced claim evidence".to_string(),
            );
        }

        if gates.iter().any(|gate| gate.status == GateStatus::GateFail) {
            let (disposition, mismatch) = match claim_class.as_str() {
                "compile_validity" => (
                    FinalDisposition::Contradicted,
                    Some("compile_vs_parse_disagreement".to_string()),
                ),
                "doc_parity" => (
                    FinalDisposition::PolicyMismatch,
                    Some("doc_vs_code_mismatch".to_string()),
                ),
                "artifact_extraction_ready" => (
                    FinalDisposition::Contradicted,
                    Some("lane_unavailability_conflict".to_string()),
                ),
                _ => (
                    FinalDisposition::Contradicted,
                    Some("rule_conflict".to_string()),
                ),
            };
            return (
                disposition,
                "high".to_string(),
                freshness_posture(observations),
                mismatch,
                "hard gate failed before weighting".to_string(),
            );
        }

        if gates
            .iter()
            .any(|gate| gate.gate_name == "stale_support_only")
        {
            return (
                if claim_class == "artifact_extraction_ready" {
                    FinalDisposition::ResidualStaleArtifact
                } else {
                    FinalDisposition::Stale
                },
                "medium".to_string(),
                "stale".to_string(),
                Some(if claim_class == "artifact_extraction_ready" {
                    "artifact_vs_repo_staleness".to_string()
                } else {
                    "freshness_conflict".to_string()
                }),
                "supporting evidence is stale and no fresh support is present".to_string(),
            );
        }

        let has_required_authority = profile.required_authority.as_ref().is_none_or(|required| {
            scores.iter().any(|score| {
                score.subsystem == required.as_str() && score.polarity == Polarity::Supports
            })
        });

        if !has_required_authority && support > 0.0 {
            return (
                FinalDisposition::OperatorReviewRequired,
                "low".to_string(),
                freshness_posture(observations),
                Some("missing_in_verifier".to_string()),
                "support exists, but the claim-critical authority is missing".to_string(),
            );
        }

        if refute > 0.35 && support > 0.35 {
            let disposition = if authoritative_refutation(scores) && authoritative_support(scores) {
                FinalDisposition::AuthorityConflict
            } else {
                FinalDisposition::Disputed
            };
            return (
                disposition,
                "medium".to_string(),
                freshness_posture(observations),
                Some("rule_conflict".to_string()),
                format!(
                    "support ({:.3}) and refutation ({:.3}) both exceed dispute threshold",
                    support, refute
                ),
            );
        }

        if refute >= 0.55 && refute > support {
            return (
                FinalDisposition::Contradicted,
                "high".to_string(),
                freshness_posture(observations),
                Some("rule_conflict".to_string()),
                format!(
                    "refutation score {:.3} exceeds support {:.3}",
                    refute, support
                ),
            );
        }

        if support >= 0.78 {
            return (
                FinalDisposition::Confirmed,
                "high".to_string(),
                freshness_posture(observations),
                None,
                format!("weighted support {:.3} confirms the claim", support),
            );
        }
        if support >= 0.6 {
            return (
                FinalDisposition::StrongSupport,
                "medium_high".to_string(),
                freshness_posture(observations),
                None,
                format!(
                    "weighted support {:.3} strongly supports the claim",
                    support
                ),
            );
        }
        if support >= 0.42 {
            return (
                FinalDisposition::ModerateSupport,
                "medium".to_string(),
                freshness_posture(observations),
                None,
                format!(
                    "weighted support {:.3} moderately supports the claim",
                    support
                ),
            );
        }
        if support > 0.0 {
            return (
                FinalDisposition::OperatorReviewRequired,
                "low".to_string(),
                freshness_posture(observations),
                Some("coverage_shortfall".to_string()),
                format!(
                    "weighted support {:.3} is below disposition threshold",
                    support
                ),
            );
        }

        (
            FinalDisposition::CoverageGap,
            "none".to_string(),
            freshness_posture(observations),
            Some("coverage_shortfall".to_string()),
            "no supporting evidence reached the scoring threshold".to_string(),
        )
    }
}

fn parse_polarity(value: &str) -> Polarity {
    match value {
        "supports" => Polarity::Supports,
        "refutes" => Polarity::Refutes,
        "unavailable" => Polarity::Unavailable,
        "not_run" => Polarity::NotRun,
        _ => Polarity::NotRun,
    }
}

fn parse_freshness(value: &str) -> FreshnessStatus {
    match value {
        "fresh" => FreshnessStatus::Fresh,
        "historical" => FreshnessStatus::Historical,
        "stale" => FreshnessStatus::Stale,
        _ => FreshnessStatus::Unknown,
    }
}

fn freshness_score(value: &str) -> f64 {
    match parse_freshness(value) {
        FreshnessStatus::Fresh => 1.0,
        FreshnessStatus::Historical => 0.75,
        FreshnessStatus::Unknown => 0.5,
        FreshnessStatus::Stale => 0.2,
    }
}

fn health_factor(value: &str) -> f64 {
    match value {
        "ready" => 1.0,
        "partial_success" => 0.85,
        "degraded" => 0.7,
        "stale" => 0.45,
        "denied" => 0.35,
        "unavailable" | "not_run" => 0.2,
        _ => 0.5,
    }
}

fn agreement_score_for(
    observation: &StoredObservation,
    all_observations: &[StoredObservation],
) -> f64 {
    let same = all_observations
        .iter()
        .filter(|other| other.subsystem != observation.subsystem)
        .filter(|other| other.polarity == observation.polarity)
        .count() as f64;
    let opposed = all_observations
        .iter()
        .filter(|other| other.subsystem != observation.subsystem)
        .filter(|other| other.polarity != observation.polarity)
        .filter(|other| other.polarity == "supports" || other.polarity == "refutes")
        .count() as f64;

    if same + opposed == 0.0 {
        0.0
    } else {
        ((same - opposed) / (same + opposed)).clamp(-1.0, 1.0)
    }
}

fn contradiction_penalty_for(
    observation: &StoredObservation,
    all_observations: &[StoredObservation],
    profiles: &ProfileRegistry,
) -> f64 {
    if observation.polarity != "supports" && observation.polarity != "refutes" {
        return 0.0;
    }

    all_observations
        .iter()
        .filter(|other| other.polarity == "supports" || other.polarity == "refutes")
        .filter(|other| other.polarity != observation.polarity)
        .map(|other| profiles.authority_score(&other.subsystem, &other.claim_class))
        .fold(0.0_f64, f64::max)
}

fn agreement_score(scores: &[ObservationScore]) -> f64 {
    let support_sources = scores
        .iter()
        .filter(|score| score.support_score > 0.0)
        .count() as f64;
    let refute_sources = scores
        .iter()
        .filter(|score| score.refute_score > 0.0)
        .count() as f64;
    if support_sources + refute_sources == 0.0 {
        0.0
    } else {
        ((support_sources - refute_sources) / (support_sources + refute_sources)).clamp(-1.0, 1.0)
    }
}

fn contradiction_penalty(scores: &[ObservationScore]) -> f64 {
    let max_support = scores
        .iter()
        .filter(|score| score.support_score > 0.0)
        .map(|score| score.authority)
        .fold(0.0_f64, f64::max);
    let max_refute = scores
        .iter()
        .filter(|score| score.refute_score > 0.0)
        .map(|score| score.authority)
        .fold(0.0_f64, f64::max);
    max_support.min(max_refute)
}

fn freshness_posture(observations: &[StoredObservation]) -> String {
    if observations
        .iter()
        .any(|observation| observation.freshness_status == "fresh")
    {
        "fresh".to_string()
    } else if observations
        .iter()
        .any(|observation| observation.freshness_status == "historical")
    {
        "historical".to_string()
    } else if observations
        .iter()
        .any(|observation| observation.freshness_status == "stale")
    {
        "stale".to_string()
    } else {
        "unknown".to_string()
    }
}

fn authoritative_refutation(scores: &[ObservationScore]) -> bool {
    scores
        .iter()
        .any(|score| score.refute_score > 0.0 && score.authority >= 0.9)
}

fn authoritative_support(scores: &[ObservationScore]) -> bool {
    scores
        .iter()
        .any(|score| score.support_score > 0.0 && score.authority >= 0.9)
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

use std::collections::BTreeMap;

use crate::model::Subsystem;

pub const PROFILE_VERSION: &str = "worm.weights.v1";

#[derive(Debug, Clone)]
pub struct WeightProfile {
    pub authority_weight: f64,
    pub evidence_weight: f64,
    pub freshness_weight: f64,
    pub agreement_weight: f64,
    pub contradiction_weight: f64,
    pub required_authority: Option<Subsystem>,
}

#[derive(Debug, Clone)]
pub struct ProfileRegistry {
    authority: BTreeMap<(String, String), f64>,
    weights: BTreeMap<String, WeightProfile>,
}

impl Default for ProfileRegistry {
    fn default() -> Self {
        let mut registry = Self {
            authority: BTreeMap::new(),
            weights: BTreeMap::new(),
        };
        registry.seed_authority();
        registry.seed_weights();
        registry
    }
}

impl ProfileRegistry {
    pub fn authority_score(&self, subsystem: &str, claim_class: &str) -> f64 {
        self.authority
            .get(&(subsystem.to_string(), claim_class.to_string()))
            .copied()
            .or_else(|| {
                self.authority
                    .get(&(subsystem.to_string(), "*".to_string()))
                    .copied()
            })
            .unwrap_or(0.0)
    }

    pub fn weight_profile(&self, claim_class: &str) -> WeightProfile {
        self.weights
            .get(claim_class)
            .cloned()
            .unwrap_or_else(default_weight_profile)
    }

    fn authority(&mut self, subsystem: Subsystem, claim_class: &str, score: f64) {
        self.authority.insert(
            (subsystem.as_str().to_string(), claim_class.to_string()),
            score,
        );
    }

    fn weights(&mut self, claim_class: &str, profile: WeightProfile) {
        self.weights.insert(claim_class.to_string(), profile);
    }

    fn seed_authority(&mut self) {
        for claim in [
            "artifact_extraction_ready",
            "artifact_extraction_denied",
            "artifact_admission",
            "stale_artifact",
        ] {
            self.authority(Subsystem::Cortex, claim, 0.95);
        }
        self.authority(Subsystem::Cortex, "*", 0.15);

        for claim in [
            "file_presence",
            "file_deleted",
            "file_hash",
            "parse_success",
            "parse_failure",
            "stale_index",
            "deleted_source_residual",
        ] {
            self.authority(Subsystem::RepoCrawler, claim, 0.95);
        }
        self.authority(Subsystem::RepoCrawler, "doc_presence", 0.65);
        self.authority(Subsystem::RepoCrawler, "doc_parity", 0.45);
        self.authority(Subsystem::RepoCrawler, "coverage_gap", 0.45);
        self.authority(Subsystem::RepoCrawler, "*", 0.25);

        for claim in [
            "deleted_source_residual",
            "stale_artifact",
            "coverage_gap",
            "verification_failed",
            "policy_mismatch",
        ] {
            self.authority(Subsystem::DeterministicVerifier, claim, 0.9);
        }
        self.authority(Subsystem::DeterministicVerifier, "*", 0.55);

        for claim in ["compile_validity", "build_validity", "test_presence"] {
            self.authority(Subsystem::CompilerToolchain, claim, 0.98);
        }
        self.authority(Subsystem::CompilerToolchain, "*", 0.1);

        for claim in [
            "doc_presence",
            "doc_assembly_presence",
            "doc_parity",
            "architecture_boundary_match",
            "architecture_boundary_violation",
            "coverage_gap",
        ] {
            self.authority(Subsystem::GovernanceDocParity, claim, 0.95);
        }
        self.authority(Subsystem::GovernanceDocParity, "*", 0.2);
    }

    fn seed_weights(&mut self) {
        self.weights(
            "file_presence",
            WeightProfile {
                authority_weight: 0.42,
                evidence_weight: 0.28,
                freshness_weight: 0.18,
                agreement_weight: 0.12,
                contradiction_weight: 0.55,
                required_authority: Some(Subsystem::RepoCrawler),
            },
        );
        self.weights(
            "file_deleted",
            WeightProfile {
                authority_weight: 0.44,
                evidence_weight: 0.28,
                freshness_weight: 0.2,
                agreement_weight: 0.08,
                contradiction_weight: 0.6,
                required_authority: Some(Subsystem::RepoCrawler),
            },
        );
        self.weights(
            "file_hash",
            WeightProfile {
                authority_weight: 0.4,
                evidence_weight: 0.34,
                freshness_weight: 0.2,
                agreement_weight: 0.06,
                contradiction_weight: 0.6,
                required_authority: Some(Subsystem::RepoCrawler),
            },
        );
        self.weights(
            "artifact_extraction_ready",
            WeightProfile {
                authority_weight: 0.45,
                evidence_weight: 0.24,
                freshness_weight: 0.21,
                agreement_weight: 0.1,
                contradiction_weight: 0.65,
                required_authority: Some(Subsystem::Cortex),
            },
        );
        self.weights(
            "artifact_extraction_denied",
            WeightProfile {
                authority_weight: 0.46,
                evidence_weight: 0.24,
                freshness_weight: 0.2,
                agreement_weight: 0.1,
                contradiction_weight: 0.65,
                required_authority: Some(Subsystem::Cortex),
            },
        );
        self.weights(
            "parse_success",
            WeightProfile {
                authority_weight: 0.4,
                evidence_weight: 0.3,
                freshness_weight: 0.18,
                agreement_weight: 0.12,
                contradiction_weight: 0.55,
                required_authority: Some(Subsystem::RepoCrawler),
            },
        );
        self.weights(
            "compile_validity",
            WeightProfile {
                authority_weight: 0.52,
                evidence_weight: 0.24,
                freshness_weight: 0.18,
                agreement_weight: 0.06,
                contradiction_weight: 0.8,
                required_authority: Some(Subsystem::CompilerToolchain),
            },
        );
        self.weights(
            "doc_presence",
            WeightProfile {
                authority_weight: 0.42,
                evidence_weight: 0.3,
                freshness_weight: 0.18,
                agreement_weight: 0.1,
                contradiction_weight: 0.55,
                required_authority: Some(Subsystem::GovernanceDocParity),
            },
        );
        self.weights(
            "doc_parity",
            WeightProfile {
                authority_weight: 0.48,
                evidence_weight: 0.26,
                freshness_weight: 0.18,
                agreement_weight: 0.08,
                contradiction_weight: 0.7,
                required_authority: Some(Subsystem::GovernanceDocParity),
            },
        );
        self.weights(
            "stale_artifact",
            WeightProfile {
                authority_weight: 0.42,
                evidence_weight: 0.28,
                freshness_weight: 0.2,
                agreement_weight: 0.1,
                contradiction_weight: 0.55,
                required_authority: None,
            },
        );
        self.weights(
            "deleted_source_residual",
            WeightProfile {
                authority_weight: 0.44,
                evidence_weight: 0.28,
                freshness_weight: 0.2,
                agreement_weight: 0.08,
                contradiction_weight: 0.55,
                required_authority: None,
            },
        );
        self.weights(
            "coverage_gap",
            WeightProfile {
                authority_weight: 0.36,
                evidence_weight: 0.28,
                freshness_weight: 0.18,
                agreement_weight: 0.18,
                contradiction_weight: 0.4,
                required_authority: None,
            },
        );
    }
}

fn default_weight_profile() -> WeightProfile {
    WeightProfile {
        authority_weight: 0.35,
        evidence_weight: 0.3,
        freshness_weight: 0.2,
        agreement_weight: 0.15,
        contradiction_weight: 0.5,
        required_authority: None,
    }
}

from __future__ import annotations

import hashlib
import json
from datetime import UTC, datetime
from typing import Any, Iterable

from cortex_runtime.gnats.schema_validation import require_schema_valid


DEFAULT_CANDIDATE_CONTRACT_FAMILY = "neuronforge.cor_gnat.semantic-candidates.v1"
DEFAULT_ALLOWED_CANDIDATE_CLASSES = (
    "structure_summary_candidate",
    "continuity_candidate",
)


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _stable_json_hash(payload: dict[str, Any]) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _short_hash(payload: str) -> str:
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:16]


def _require_nonempty(value: str, field_name: str) -> str:
    normalized = value.strip()
    if not normalized:
        raise ValueError(f"{field_name} must be non-empty")
    return normalized


def emit_gnat_semantic_handoff_from_retrieval_package(
    retrieval_package: dict[str, Any],
    *,
    source_gnat_run_id: str,
    source_plan_hash: str,
    requested_by: str,
    model_resource_disclosure: dict[str, Any],
    request_scope: str = "app_request",
    candidate_contract_family: str = DEFAULT_CANDIDATE_CONTRACT_FAMILY,
    allowed_candidate_classes: Iterable[str] = DEFAULT_ALLOWED_CANDIDATE_CLASSES,
    destination_service_id: str = "neuronforge-local",
) -> dict[str, Any]:
    """Emit the optional GNAT-to-NeuronForge semantic-candidate handoff contract."""

    require_schema_valid(retrieval_package, schema_name="retrieval-package.schema.json")

    source_state = str(retrieval_package.get("state", ""))
    if source_state not in {"ready", "partial_success"}:
        raise ValueError("GNAT semantic handoff requires a ready or partial_success retrieval package")

    requested_by = _require_nonempty(requested_by, "requested_by")
    source_gnat_run_id = _require_nonempty(source_gnat_run_id, "source_gnat_run_id")
    source_plan_hash = _require_nonempty(source_plan_hash, "source_plan_hash")
    candidate_contract_family = _require_nonempty(candidate_contract_family, "candidate_contract_family")

    candidate_classes = tuple(allowed_candidate_classes)
    if not candidate_classes:
        raise ValueError("allowed_candidate_classes must include at least one candidate class")

    package_id = str(retrieval_package["package_id"])
    request_id = str(retrieval_package["request_id"])
    retrieval_profile = retrieval_package.get("retrieval_profile", {})
    completeness = retrieval_package.get("completeness", {})
    completeness_status = str(completeness.get("status", ""))

    handoff = {
        "contract_version": "GnatSemanticHandoff.v1",
        "handoff_id": f"gnat-semantic-handoff-{_short_hash(package_id + ':' + source_plan_hash + ':' + requested_by)}",
        "request_id": request_id,
        "source_service_id": "cortex",
        "destination_service_id": destination_service_id,
        "source_artifact": {
            "artifact_ref": package_id,
            "artifact_class": "retrieval_package",
            "artifact_hash": _stable_json_hash(retrieval_package),
            "source_gnat_run_id": source_gnat_run_id,
            "source_plan_hash": source_plan_hash,
            "source_state": source_state,
            "completeness_status": completeness_status,
            "retrieval_profile_id": str(retrieval_profile.get("profile_id", "")),
        },
        "explicit_request": {
            "request_scope": request_scope,
            "requested_by": requested_by,
            "operator_visible_summary": (
                "Explicit request to let NeuronForge generate non-canonical semantic candidates "
                "from the referenced GNAT retrieval package."
            ),
        },
        "candidate_generation": {
            "candidate_contract_family": candidate_contract_family,
            "semantic_result_posture": "non_canonical_candidate",
            "allowed_candidate_classes": list(candidate_classes),
            "model_resource_disclosure": dict(model_resource_disclosure),
        },
        "transfer_guardrails": {
            "cor_receipts_immutable": True,
            "receipt_mutation_allowed": False,
            "semantic_output_canonical": False,
            "raw_content_included": False,
            "details_redacted": True,
        },
        "operator_visible_message": (
            "NeuronForge may use the referenced reconciled syntax artifact only to generate "
            "reviewable non-canonical semantic candidates; COR receipts remain immutable."
        ),
        "created_at": _utc_now(),
    }
    require_schema_valid(handoff, schema_name="gnat-semantic-handoff.schema.json")
    return handoff

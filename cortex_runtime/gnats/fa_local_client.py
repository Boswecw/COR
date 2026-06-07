from __future__ import annotations

import uuid
from typing import Any

from cortex_runtime.gnats.models import (
    GNAT_DISPATCH_ENVELOPE_VERSION,
    GNAT_HARD_MAX_CONCURRENCY,
    GNAT_OPERATION,
    GNAT_RECEIPT_SCHEMA,
    GNAT_RUN_PLAN_VERSION,
    GNAT_WORKER_RECEIPT_VERSION,
    FaLocalCapabilityState,
    GnatDispatchNegotiation,
    GnatRunPlan,
)
from cortex_runtime.gnats.schema_validation import require_schema_valid


SUPPORTED_GNAT_CONTRACTS = (
    GNAT_DISPATCH_ENVELOPE_VERSION,
    GNAT_RUN_PLAN_VERSION,
    GNAT_WORKER_RECEIPT_VERSION,
)
READY_FA_LOCAL_STATES = {"ready"}
DEGRADED_FA_LOCAL_STATES = {"degraded", "unavailable", "unknown"}


class GnatDispatchError(ValueError):
    """Raised when Cortex cannot create or negotiate a bounded FA-Local dispatch."""


def _fa_local_correlation_id(plan: GnatRunPlan) -> str:
    try:
        return str(uuid.UUID(plan.correlation_id))
    except ValueError:
        return str(uuid.uuid5(uuid.NAMESPACE_URL, f"cortex-gnat:{plan.correlation_id}:{plan.run_id}"))


def build_dispatch_envelope(plan: GnatRunPlan) -> dict[str, Any]:
    worker_types = sorted({shard.worker_type for shard in plan.shards})
    envelope = {
        "contract_version": GNAT_DISPATCH_ENVELOPE_VERSION,
        "correlation_id": _fa_local_correlation_id(plan),
        "requester_service": "cortex",
        "plan": {
            "contract_version": GNAT_RUN_PLAN_VERSION,
            "run_id": plan.run_id,
            "plan_hash": plan.plan_hash,
            "operation": GNAT_OPERATION,
            "expected_receipt_schema": GNAT_RECEIPT_SCHEMA,
            "shard_count": len(plan.shards),
            "shards": [
                {
                    "shard_id": shard.shard_id,
                    "ordinal": shard.ordinal,
                    "worker_type": shard.worker_type,
                    "source_ref": shard.source_ref,
                    "source_fingerprint_digest": shard.source_fingerprint.digest,
                    "deadline_ms": shard.deadline_ms,
                }
                for shard in sorted(plan.shards, key=lambda item: item.ordinal)
            ],
            "execution_limits": {
                "requested_concurrency": plan.requested_concurrency,
                "max_concurrency": min(plan.max_concurrency, GNAT_HARD_MAX_CONCURRENCY),
                "deadline_ms": plan.deadline_ms,
            },
            "serial_fallback_allowed": plan.serial_fallback_allowed,
        },
        "required_capabilities": {
            "supported_contract_versions": list(SUPPORTED_GNAT_CONTRACTS),
            "worker_types": worker_types,
            "cancellation_policy": "cancel_remaining_shards",
            "deadline_enforced": True,
        },
        "route_policy": {
            "fa_local_owns_execution_routing": True,
            "cortex_validates_receipts": True,
            "retry_policy": "infrastructure_only",
        },
    }
    require_schema_valid(envelope, schema_name="gnat-dispatch-envelope.schema.json")
    return envelope


def negotiate_dispatch(
    plan: GnatRunPlan,
    fa_local_capability_state: FaLocalCapabilityState,
) -> GnatDispatchNegotiation:
    envelope = build_dispatch_envelope(plan)
    fa_local_state = fa_local_capability_state.fa_local_state

    if fa_local_state in READY_FA_LOCAL_STATES:
        _require_supported_contracts(fa_local_capability_state)
        _require_worker_admission(plan, fa_local_capability_state)
        if not fa_local_capability_state.cancellation_supported:
            raise GnatDispatchError("FA-Local does not currently report Gnat cancellation support")

        effective_concurrency = min(
            plan.requested_concurrency,
            plan.max_concurrency,
            fa_local_capability_state.max_concurrency,
            GNAT_HARD_MAX_CONCURRENCY,
        )
        if effective_concurrency < 1:
            raise GnatDispatchError("FA-Local reported invalid Gnat concurrency")

        return GnatDispatchNegotiation(
            state="fa_local_dispatch_ready",
            run_id=plan.run_id,
            correlation_id=str(envelope["correlation_id"]),
            effective_concurrency=effective_concurrency,
            admitted_worker_types=tuple(sorted(set(fa_local_capability_state.admitted_worker_types))),
            operator_visible_summary=f"FA-Local admitted Cortex Gnat run {plan.run_id} for bounded dispatch.",
        )

    if fa_local_state in DEGRADED_FA_LOCAL_STATES and plan.serial_fallback_allowed:
        return GnatDispatchNegotiation(
            state="serial_fallback",
            run_id=plan.run_id,
            correlation_id=str(envelope["correlation_id"]),
            effective_concurrency=1,
            admitted_worker_types=(),
            operator_visible_summary=(
                "FA-Local Gnat dispatch is unavailable; Cortex serial fallback is permitted by contract."
            ),
        )

    if fa_local_state in DEGRADED_FA_LOCAL_STATES:
        raise GnatDispatchError("FA-Local Gnat dispatch is unavailable and serial fallback is not permitted")

    raise GnatDispatchError(f"FA-Local returned an unsupported Gnat state: {fa_local_state}")


def _require_supported_contracts(fa_local_capability_state: FaLocalCapabilityState) -> None:
    supported = set(fa_local_capability_state.supported_contract_versions)
    missing = [contract for contract in SUPPORTED_GNAT_CONTRACTS if contract not in supported]
    if missing:
        raise GnatDispatchError("FA-Local does not support required Gnat contracts: " + ", ".join(missing))


def _require_worker_admission(
    plan: GnatRunPlan,
    fa_local_capability_state: FaLocalCapabilityState,
) -> None:
    admitted = set(fa_local_capability_state.admitted_worker_types)
    required = {shard.worker_type for shard in plan.shards}
    missing = sorted(required - admitted)
    if missing:
        raise GnatDispatchError("FA-Local does not admit required Gnat worker types: " + ", ".join(missing))

from __future__ import annotations

from typing import Any

from cortex_runtime.gnats.fa_local_client import negotiate_dispatch
from cortex_runtime.gnats.models import FaLocalCapabilityState, GnatPersistentResult, GnatPersistenceOutcome, GnatRunPlan
from cortex_runtime.gnats.parallel_runner import execute_parallel_gnat_shards
from cortex_runtime.gnats.persistence import (
    GnatPersistenceError,
    GnatPersistenceStore,
    build_cache_record,
    cached_receipt_for_shard,
)
from cortex_runtime.gnats.reconcile import reconcile_receipts
from cortex_runtime.gnats.serial_runner import run_serial_gnat_plan


def run_parallel_gnat_plan_with_persistence(
    plan: GnatRunPlan,
    fa_local_capability_state: FaLocalCapabilityState,
    store: GnatPersistenceStore,
    *,
    retry_infrastructure_failures: int = 1,
) -> GnatPersistentResult:
    negotiation = negotiate_dispatch(plan, fa_local_capability_state)
    if negotiation.state == "serial_fallback":
        serial_result = run_serial_gnat_plan(plan)
        persistence = _persist_result(store, plan, serial_result.receipts, serial_result.summary, cache_hits=0, cache_misses=0)
        summary = dict(serial_result.summary)
        summary["persistence"] = persistence.to_contract()
        return GnatPersistentResult(
            plan=plan,
            receipts=serial_result.receipts,
            summary=summary,
            negotiation=negotiation,
            persistence=persistence,
        )

    cache_hits = 0
    cache_misses = 0
    cached_receipts: list[dict[str, Any]] = []
    missing_shards = []
    cache_degraded = False

    for shard in plan.shards:
        try:
            receipt = cached_receipt_for_shard(store, shard)
        except GnatPersistenceError:
            receipt = None
            cache_degraded = True
        if receipt is None:
            missing_shards.append(shard)
            cache_misses += 1
        else:
            cached_receipts.append(receipt)
            cache_hits += 1

    new_receipts = execute_parallel_gnat_shards(
        plan,
        missing_shards,
        effective_concurrency=negotiation.effective_concurrency,
        retry_infrastructure_failures=retry_infrastructure_failures,
    )
    receipts = tuple(_sort_receipts(plan, [*cached_receipts, *new_receipts]))
    provisional_summary = reconcile_receipts(
        plan,
        receipts,
        concurrency_used=negotiation.effective_concurrency,
        fallback_used=False,
    )
    persistence = _persist_result(
        store,
        plan,
        receipts,
        provisional_summary,
        cache_hits=cache_hits,
        cache_misses=cache_misses,
        force_degraded=cache_degraded,
    )
    summary = reconcile_receipts(
        plan,
        receipts,
        concurrency_used=negotiation.effective_concurrency,
        fallback_used=False,
        persistence=persistence.to_contract(),
    )
    return GnatPersistentResult(
        plan=plan,
        receipts=receipts,
        summary=summary,
        negotiation=negotiation,
        persistence=persistence,
    )


def _persist_result(
    store: GnatPersistenceStore,
    plan: GnatRunPlan,
    receipts: tuple[dict[str, Any], ...],
    summary: dict[str, Any],
    *,
    cache_hits: int,
    cache_misses: int,
    force_degraded: bool = False,
) -> GnatPersistenceOutcome:
    cache_records_written = 0
    degraded = force_degraded
    try:
        store.record_plan(plan.to_contract())
        for receipt in receipts:
            receipt_hash = store.record_receipt(receipt)
            if receipt.get("state") == "complete":
                shard = next((candidate for candidate in plan.shards if candidate.shard_id == receipt.get("shard_id")), None)
                if shard is not None:
                    store.upsert_cache_record(build_cache_record(shard, receipt=receipt, receipt_hash=receipt_hash))
                    cache_records_written += 1
        store.record_summary(summary)
    except GnatPersistenceError:
        degraded = True

    if degraded:
        state = "degraded"
        operator_summary = "DF-Local GNAT persistence degraded; extraction truth remains based on Cortex receipts."
    else:
        state = "ready"
        operator_summary = "DF-Local GNAT persistence recorded plan, receipts, summary, and cache records."
    return GnatPersistenceOutcome(
        state=state,
        cache_hits=cache_hits,
        cache_misses=cache_misses,
        cache_records_written=cache_records_written,
        operator_visible_summary=operator_summary,
    )


def _sort_receipts(plan: GnatRunPlan, receipts: list[dict[str, Any]]) -> list[dict[str, Any]]:
    ordinal_by_shard_id = {shard.shard_id: shard.ordinal for shard in plan.shards}
    return sorted(receipts, key=lambda item: ordinal_by_shard_id.get(str(item.get("shard_id")), len(plan.shards)))

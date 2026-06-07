from __future__ import annotations

from cortex_runtime.gnats.models import GnatRunPlan, GnatSerialResult
from cortex_runtime.gnats.receipt import build_failure_receipt, monotonic_ms, utc_now
from cortex_runtime.gnats.reconcile import reconcile_receipts
from cortex_runtime.gnats.registry import GnatWorkerUnavailable, worker_for_type


def run_serial_gnat_plan(plan: GnatRunPlan) -> GnatSerialResult:
    receipts: list[dict[str, object]] = []
    for shard in sorted(plan.shards, key=lambda item: item.ordinal):
        try:
            worker = worker_for_type(shard.worker_type)
            receipts.append(worker.run(shard))
        except GnatWorkerUnavailable:
            started_at = utc_now()
            start_ms = monotonic_ms()
            completed_at = utc_now()
            receipts.append(
                build_failure_receipt(
                    shard,
                    reason_code="worker_unavailable",
                    operator_visible_summary="No Cortex Gnat worker is registered for this shard.",
                    source_fingerprint_before=shard.source_fingerprint,
                    source_fingerprint_after=shard.source_fingerprint,
                    state="failed",
                    started_at=started_at,
                    completed_at=completed_at,
                    duration_ms=max(0, monotonic_ms() - start_ms),
                )
            )

    typed_receipts = tuple(dict(receipt) for receipt in receipts)
    summary = reconcile_receipts(plan, typed_receipts)
    return GnatSerialResult(plan=plan, receipts=typed_receipts, summary=summary)

from __future__ import annotations

from concurrent.futures import Future, ThreadPoolExecutor, wait
from typing import Callable

from cortex_runtime.gnats.fa_local_client import negotiate_dispatch
from cortex_runtime.gnats.models import FaLocalCapabilityState, GnatParallelResult, GnatRunPlan, GnatShard
from cortex_runtime.gnats.receipt import build_failure_receipt, monotonic_ms, utc_now
from cortex_runtime.gnats.reconcile import reconcile_receipts
from cortex_runtime.gnats.registry import GnatWorkerUnavailable, worker_for_type
from cortex_runtime.gnats.serial_runner import run_serial_gnat_plan


CancelCheck = Callable[[], bool]


def run_parallel_gnat_plan(
    plan: GnatRunPlan,
    fa_local_capability_state: FaLocalCapabilityState,
    *,
    retry_infrastructure_failures: int = 1,
    cancel_requested: CancelCheck | None = None,
) -> GnatParallelResult:
    negotiation = negotiate_dispatch(plan, fa_local_capability_state)
    if negotiation.state == "serial_fallback":
        serial_result = run_serial_gnat_plan(plan)
        return GnatParallelResult(
            plan=plan,
            receipts=serial_result.receipts,
            summary=serial_result.summary,
            negotiation=negotiation,
        )

    typed_receipts = execute_parallel_gnat_shards(
        plan,
        plan.shards,
        effective_concurrency=negotiation.effective_concurrency,
        retry_infrastructure_failures=retry_infrastructure_failures,
        cancel_requested=cancel_requested,
    )
    summary = reconcile_receipts(
        plan,
        typed_receipts,
        concurrency_used=negotiation.effective_concurrency,
        fallback_used=False,
    )
    return GnatParallelResult(plan=plan, receipts=typed_receipts, summary=summary, negotiation=negotiation)


def execute_parallel_gnat_shards(
    plan: GnatRunPlan,
    shards: tuple[GnatShard, ...] | list[GnatShard],
    *,
    effective_concurrency: int,
    retry_infrastructure_failures: int = 1,
    cancel_requested: CancelCheck | None = None,
) -> tuple[dict[str, object], ...]:
    if not shards:
        return ()

    executor = ThreadPoolExecutor(max_workers=effective_concurrency)
    future_by_shard: dict[Future[dict[str, object]], GnatShard] = {
        executor.submit(
            _execute_shard_with_retry,
            shard,
            retry_budget=max(0, retry_infrastructure_failures),
            cancel_requested=cancel_requested,
        ): shard
        for shard in shards
    }

    done, not_done = wait(future_by_shard, timeout=plan.deadline_ms / 1000.0)
    receipts: list[dict[str, object]] = []
    for future in done:
        shard = future_by_shard[future]
        try:
            receipts.append(dict(future.result()))
        except Exception:
            receipts.append(
                _failure_receipt(
                    shard,
                    reason_code="internal_error_redacted",
                    operator_visible_summary="Gnat worker crashed before emitting a valid receipt.",
                    state="failed",
                )
            )

    for future in not_done:
        shard = future_by_shard[future]
        future.cancel()
        receipts.append(
            _failure_receipt(
                shard,
                reason_code="deadline_exceeded",
                operator_visible_summary="Gnat shard exceeded the bounded run deadline.",
                state="timed_out",
                duration_ms=plan.deadline_ms,
            )
        )

    executor.shutdown(wait=not not_done, cancel_futures=bool(not_done))
    return tuple(_sort_receipts(plan, receipts))


def _execute_shard_with_retry(
    shard: GnatShard,
    *,
    retry_budget: int,
    cancel_requested: CancelCheck | None,
) -> dict[str, object]:
    if cancel_requested is not None and cancel_requested():
        return _failure_receipt(
            shard,
            reason_code="cancelled_by_operator",
            operator_visible_summary="Gnat shard was cancelled before worker execution.",
            state="cancelled",
        )

    for attempt_index in range(retry_budget + 1):
        if cancel_requested is not None and cancel_requested():
            return _failure_receipt(
                shard,
                reason_code="cancelled_by_operator",
                operator_visible_summary="Gnat shard was cancelled before retry execution.",
                state="cancelled",
            )

        started_at = utc_now()
        start_ms = monotonic_ms()
        try:
            worker = worker_for_type(shard.worker_type)
            receipt = dict(worker.run(shard))
        except GnatWorkerUnavailable:
            if attempt_index < retry_budget:
                continue
            return _failure_receipt(
                shard,
                reason_code="worker_unavailable",
                operator_visible_summary="No Cortex Gnat worker is registered for this shard.",
                state="failed",
                started_at=started_at,
                duration_ms=max(0, monotonic_ms() - start_ms),
            )
        except Exception:
            if attempt_index < retry_budget:
                continue
            return _failure_receipt(
                shard,
                reason_code="internal_error_redacted",
                operator_visible_summary="Gnat worker crashed before emitting a valid receipt.",
                state="failed",
                started_at=started_at,
                duration_ms=max(0, monotonic_ms() - start_ms),
            )

        return _deadline_receipt_if_needed(shard, receipt)

    return _failure_receipt(
        shard,
        reason_code="internal_error_redacted",
        operator_visible_summary="Gnat worker retry budget was exhausted.",
        state="failed",
    )


def _deadline_receipt_if_needed(shard: GnatShard, receipt: dict[str, object]) -> dict[str, object]:
    if receipt.get("state") != "complete":
        return receipt

    try:
        duration_ms = int(receipt.get("duration_ms", 0))
    except (TypeError, ValueError):
        return receipt

    if duration_ms <= shard.deadline_ms:
        return receipt

    started_at = receipt.get("started_at")
    completed_at = receipt.get("completed_at")
    return _failure_receipt(
        shard,
        reason_code="deadline_exceeded",
        operator_visible_summary="Gnat shard exceeded its bounded execution deadline.",
        state="timed_out",
        started_at=started_at if isinstance(started_at, str) else None,
        completed_at=completed_at if isinstance(completed_at, str) else None,
        duration_ms=duration_ms,
    )


def _failure_receipt(
    shard: GnatShard,
    *,
    reason_code: str,
    operator_visible_summary: str,
    state: str,
    started_at: str | None = None,
    completed_at: str | None = None,
    duration_ms: int = 0,
) -> dict[str, object]:
    started = started_at or utc_now()
    completed = completed_at or utc_now()
    return build_failure_receipt(
        shard,
        reason_code=reason_code,
        operator_visible_summary=operator_visible_summary,
        source_fingerprint_before=shard.source_fingerprint,
        source_fingerprint_after=shard.source_fingerprint,
        state=state,
        started_at=started,
        completed_at=completed,
        duration_ms=max(0, duration_ms),
    )


def _sort_receipts(plan: GnatRunPlan, receipts: list[dict[str, object]]) -> list[dict[str, object]]:
    ordinal_by_shard_id = {shard.shard_id: shard.ordinal for shard in plan.shards}
    return sorted(receipts, key=lambda item: ordinal_by_shard_id.get(str(item.get("shard_id")), len(plan.shards)))

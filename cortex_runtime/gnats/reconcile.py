from __future__ import annotations

from typing import Any

from cortex_runtime.gnats.models import GNAT_RUN_SUMMARY_VERSION, GnatRunPlan, GnatShard
from cortex_runtime.gnats.receipt import utc_now
from cortex_runtime.gnats.schema_validation import require_schema_valid, schema_error_messages
from gnat_core import RunStateCounts, canonical_hash, run_state_from_counts


def _hash_payload(payload: object) -> str:
    return canonical_hash(payload)


def _shard_by_id(plan: GnatRunPlan) -> dict[str, GnatShard]:
    return {shard.shard_id: shard for shard in plan.shards}


def _receipt_fingerprint_matches_shard(receipt: dict[str, Any], shard: GnatShard) -> bool:
    before = receipt.get("source_fingerprint_before")
    after = receipt.get("source_fingerprint_after")
    if not isinstance(before, dict) or not isinstance(after, dict):
        return False
    before_digest = before.get("digest")
    after_digest = after.get("digest")
    expected_digest = shard.source_fingerprint.digest
    if receipt.get("state") == "stale":
        return before_digest != expected_digest or after_digest != expected_digest
    return before_digest == expected_digest and after_digest == expected_digest


def _empty_timing() -> dict[str, Any]:
    now = utc_now()
    return {"started_at": now, "completed_at": now, "duration_ms": 0}


def _aggregate_timing(receipts: list[dict[str, Any]]) -> dict[str, Any]:
    if not receipts:
        return _empty_timing()
    started_values = [str(receipt["started_at"]) for receipt in receipts]
    completed_values = [str(receipt["completed_at"]) for receipt in receipts]
    return {
        "started_at": min(started_values),
        "completed_at": max(completed_values),
        "duration_ms": sum(int(receipt.get("duration_ms", 0)) for receipt in receipts),
    }


def _run_state(
    *,
    expected: int,
    completed: int,
    failed: int,
    stale: int,
    cancelled: int,
    missing: int,
    rejected: int,
) -> str:
    return run_state_from_counts(
        RunStateCounts(
            expected=expected,
            completed=completed,
            failed=failed,
            stale=stale,
            cancelled=cancelled,
            missing=missing,
            rejected=rejected,
        )
    )


def _operator_summary(run_state: str, *, completed: int, expected: int) -> str:
    if run_state == "ready":
        return "Gnat syntax extraction completed for all planned shards."
    if run_state == "partial_success":
        return f"Gnat syntax extraction completed for {completed} of {expected} planned shards."
    if run_state == "stale":
        return "Gnat syntax extraction rejected stale output because one or more sources changed."
    if run_state == "cancelled":
        return "Gnat syntax extraction was cancelled before producing a complete result."
    return "Gnat syntax extraction did not produce a complete result."


def reconcile_receipts(
    plan: GnatRunPlan,
    receipts: list[dict[str, Any]] | tuple[dict[str, Any], ...],
    *,
    concurrency_used: int = 1,
    fallback_used: bool = True,
    persistence: dict[str, Any] | None = None,
) -> dict[str, Any]:
    shards = _shard_by_id(plan)
    accepted: list[dict[str, Any]] = []
    accepted_hashes: list[str] = []
    rejected: list[dict[str, str]] = []
    seen_shards: set[str] = set()

    for receipt in sorted(receipts, key=lambda item: str(item.get("shard_id", ""))):
        receipt_hash = _hash_payload(receipt)
        errors = schema_error_messages(receipt, schema_name="gnat-worker-receipt.schema.json")
        if errors:
            rejected.append({"receipt_hash": receipt_hash, "reason_code": "schema_invalid"})
            continue

        if receipt.get("run_id") != plan.run_id:
            rejected.append({"receipt_hash": receipt_hash, "reason_code": "run_id_mismatch"})
            continue

        shard_id = str(receipt.get("shard_id"))
        shard = shards.get(shard_id)
        if shard is None:
            rejected.append({"receipt_hash": receipt_hash, "reason_code": "shard_id_mismatch"})
            continue

        if shard_id in seen_shards:
            rejected.append({"receipt_hash": receipt_hash, "reason_code": "duplicate_shard"})
            continue

        if not _receipt_fingerprint_matches_shard(receipt, shard):
            rejected.append({"receipt_hash": receipt_hash, "reason_code": "source_fingerprint_mismatch"})
            continue

        seen_shards.add(shard_id)
        accepted.append(receipt)
        accepted_hashes.append(receipt_hash)

    for shard_id in sorted(set(shards) - seen_shards):
        rejected.append({"receipt_hash": _hash_payload({"missing_shard": shard_id}), "reason_code": "missing_shard"})

    completed = sum(1 for receipt in accepted if receipt["state"] == "complete")
    failed = sum(1 for receipt in accepted if receipt["state"] in {"failed", "denied", "timed_out"})
    stale = sum(1 for receipt in accepted if receipt["state"] == "stale")
    cancelled = sum(1 for receipt in accepted if receipt["state"] == "cancelled")
    missing = sum(1 for item in rejected if item["reason_code"] == "missing_shard")
    run_state = _run_state(
        expected=len(plan.shards),
        completed=completed,
        failed=failed,
        stale=stale,
        cancelled=cancelled,
        missing=missing,
        rejected=len(rejected) - missing,
    )

    output_refs: list[str] = []
    for receipt in sorted(accepted, key=lambda item: shards[str(item["shard_id"])].ordinal):
        output = receipt.get("bounded_output")
        if isinstance(output, dict):
            artifact_id = output.get("artifact_id")
            if isinstance(artifact_id, str):
                output_refs.append(artifact_id)
        output_ref = receipt.get("output_ref")
        if isinstance(output_ref, str):
            output_refs.append(output_ref)

    summary = {
        "contract_version": GNAT_RUN_SUMMARY_VERSION,
        "run_id": plan.run_id,
        "run_state": run_state,
        "expected_shards": len(plan.shards),
        "completed_count": completed,
        "failed_count": failed,
        "stale_count": stale,
        "cancelled_count": cancelled,
        "missing_count": missing,
        "accepted_receipt_hashes": sorted(accepted_hashes),
        "rejected_receipts": sorted(rejected, key=lambda item: (item["reason_code"], item["receipt_hash"])),
        "aggregate_timing": _aggregate_timing(accepted),
        "concurrency_used": concurrency_used,
        "fallback_used": fallback_used,
        "output_artifact_refs": output_refs,
        "operator_visible_summary": _operator_summary(run_state, completed=completed, expected=len(plan.shards)),
        "details_redacted": True,
    }
    if persistence is not None:
        summary["persistence"] = persistence
    require_schema_valid(summary, schema_name="gnat-run-summary.schema.json")
    return summary

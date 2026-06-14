from __future__ import annotations

from typing import Any

from cortex_runtime.gnats.models import GNAT_OPERATOR_RUN_STATUS_VERSION
from cortex_runtime.gnats.schema_validation import require_schema_valid


STATE_LABELS = {
    "planned": "Planned",
    "submitted": "Submitted",
    "running": "Running",
    "ready": "Ready",
    "partial_success": "Partial success",
    "degraded": "Degraded",
    "cancelled": "Cancelled",
    "denied": "Denied",
    "failed": "Failed",
    "stale": "Stale source",
}

MODE_LABELS = {
    "parallel": "Bounded parallel execution",
    "serial_fallback": "Serial fallback",
    "cache_reuse": "Cache reuse",
    "mixed_cache_parallel": "Cache reuse plus bounded parallel execution",
}


def build_operator_run_status(
    *,
    summary: dict[str, Any],
    receipts: tuple[dict[str, Any], ...] | list[dict[str, Any]],
    cancellation_available: bool,
) -> dict[str, Any]:
    run_state = str(summary["run_state"])
    cache = _cache_reuse(summary)
    execution_mode = _execution_mode(summary, cache)
    allowed_controls = _allowed_controls(summary, cancellation_available)
    status = {
        "contract_version": GNAT_OPERATOR_RUN_STATUS_VERSION,
        "run_id": summary["run_id"],
        "run_state": run_state,
        "state_label": STATE_LABELS.get(run_state, run_state.replace("_", " ").title()),
        "execution_mode": execution_mode,
        "mode_label": MODE_LABELS[execution_mode],
        "expected_shards": summary["expected_shards"],
        "counts": {
            "completed": summary["completed_count"],
            "failed": summary["failed_count"],
            "stale": summary["stale_count"],
            "cancelled": summary["cancelled_count"],
            "missing": summary["missing_count"],
        },
        "concurrency_used": summary["concurrency_used"],
        "serial_fallback_used": summary["fallback_used"],
        "cache_reuse": cache,
        "persistence_state": _persistence_state(summary),
        "bounded_failure_reasons": _bounded_failure_reasons(receipts),
        "elapsed_ms": int(summary.get("aggregate_timing", {}).get("duration_ms", 0)),
        "allowed_controls": allowed_controls,
        "keyboard_controls": _keyboard_controls(allowed_controls),
        "operator_visible_summary": summary["operator_visible_summary"],
        "details_redacted": True,
    }
    require_schema_valid(status, schema_name="gnat-operator-run-status.schema.json")
    return status


def _cache_reuse(summary: dict[str, Any]) -> dict[str, Any]:
    persistence = summary.get("persistence")
    if not isinstance(persistence, dict):
        return {"used": False, "hits": 0, "misses": 0}
    hits = int(persistence.get("cache_hits", 0))
    misses = int(persistence.get("cache_misses", 0))
    return {"used": hits > 0, "hits": hits, "misses": misses}


def _execution_mode(summary: dict[str, Any], cache: dict[str, Any]) -> str:
    if bool(summary.get("fallback_used")):
        return "serial_fallback"
    if cache["hits"] > 0 and cache["misses"] == 0:
        return "cache_reuse"
    if cache["hits"] > 0 and cache["misses"] > 0:
        return "mixed_cache_parallel"
    return "parallel"


def _persistence_state(summary: dict[str, Any]) -> str:
    persistence = summary.get("persistence")
    if isinstance(persistence, dict):
        state = persistence.get("state")
        if state in {"ready", "degraded", "unavailable"}:
            return str(state)
    return "unavailable"


def _bounded_failure_reasons(receipts: tuple[dict[str, Any], ...] | list[dict[str, Any]]) -> list[str]:
    reasons = {
        str(receipt["error_reason_code"])
        for receipt in receipts
        if isinstance(receipt.get("error_reason_code"), str)
    }
    return sorted(reasons)


def _allowed_controls(summary: dict[str, Any], cancellation_available: bool) -> list[str]:
    controls: list[str] = []
    if cancellation_available and summary["run_state"] in {"planned", "submitted", "running"}:
        controls.append("cancel_run")
    if summary["failed_count"] > 0 or summary["missing_count"] > 0:
        controls.append("rerun_failed_shards")
    if summary["stale_count"] > 0:
        controls.append("clear_changed_source_cache")
    return controls


def _keyboard_controls(allowed_controls: list[str]) -> list[dict[str, Any]]:
    control_specs = [
        {
            "control_id": "cancel_run",
            "label": "Cancel",
            "aria_label": "Cancel remaining GNAT shards",
            "shortcut": "Ctrl+.",
        },
        {
            "control_id": "rerun_failed_shards",
            "label": "Rerun failed",
            "aria_label": "Rerun failed or missing GNAT shards",
            "shortcut": "Ctrl+R",
        },
        {
            "control_id": "clear_changed_source_cache",
            "label": "Clear stale cache",
            "aria_label": "Clear cache records for changed GNAT sources",
            "shortcut": "Ctrl+Shift+C",
        },
    ]
    allowed = set(allowed_controls)
    return [{**spec, "enabled": spec["control_id"] in allowed} for spec in control_specs]

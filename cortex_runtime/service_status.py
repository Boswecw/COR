from __future__ import annotations

import argparse
import importlib
import json
from datetime import UTC, datetime
from functools import lru_cache
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

from cortex_runtime.source_lanes import (
    RUNTIME_SLICE_LABELS as SOURCE_LANE_SLICE_LABELS,
    SOURCE_LANE_LABELS,
    admitted_source_lanes as configured_admitted_source_lanes,
    configured_source_lane_slice_ids,
    implemented_source_lane_slices,
)


ROOT = Path(__file__).resolve().parent.parent
SERVICE_STATUS_SCHEMA_PATH = ROOT / "schemas/service-status.schema.json"
SERVICE_STATUS_SCHEMA_REF = "schemas/service-status.schema.json"
SERVICE_ID = "cortex"
SERVICE_CLASS = "file_intelligence"

BASE_RUNTIME_SLICE_SPECS = (
    ("slice1_intake_validation", "cortex_runtime.intake_validation"),
    ("slice2_extraction_emission", "cortex_runtime.extraction_emission"),
    ("slice3_retrieval_package_emission", "cortex_runtime.retrieval_package_emission"),
    ("slice4_service_status_truth", "cortex_runtime.service_status"),
)
BASE_RUNTIME_SLICE_LABELS = {
    "slice1_intake_validation": "intake validation",
    "slice2_extraction_emission": "syntax-only extraction emission",
    "slice3_retrieval_package_emission": "governed retrieval-package emission",
    "slice4_service_status_truth": "service-status truth",
}
RUNTIME_SLICE_LABELS = {**BASE_RUNTIME_SLICE_LABELS, **SOURCE_LANE_SLICE_LABELS}


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


@lru_cache(maxsize=1)
def _service_status_validator() -> Draft202012Validator:
    with SERVICE_STATUS_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        schema = json.load(handle)
    return Draft202012Validator(schema)


def _schema_error_messages(result: dict[str, Any]) -> list[str]:
    validator = _service_status_validator()
    errors = sorted(
        validator.iter_errors(result),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    return [
        f"{'.'.join(str(part) for part in error.path) or '<root>'}: {error.message}"
        for error in errors
    ]


def _slice_label_list(slice_ids: list[str]) -> str:
    if not slice_ids:
        return "none"
    return ", ".join(RUNTIME_SLICE_LABELS[slice_id] for slice_id in slice_ids)


def _source_lane_label_list(source_lanes: list[str]) -> str:
    if not source_lanes:
        return "none"
    return ", ".join(SOURCE_LANE_LABELS[source_lane] for source_lane in source_lanes)


def _implemented_runtime_slices() -> list[str]:
    implemented: list[str] = []
    for slice_id, module_name in BASE_RUNTIME_SLICE_SPECS:
        try:
            importlib.import_module(module_name)
        except Exception:
            continue
        implemented.append(slice_id)
    implemented.extend(implemented_source_lane_slices())
    return implemented


def _admitted_source_lanes() -> tuple[list[str], str | None]:
    try:
        lanes = configured_admitted_source_lanes()
    except Exception:
        return (
            [],
            "Cortex source-lane truth is unavailable because the bounded source-lane registry could not be loaded.",
        )

    normalized_lanes = [str(lane) for lane in lanes]
    unsupported_lanes = [lane for lane in normalized_lanes if lane not in SOURCE_LANE_LABELS]
    if unsupported_lanes:
        return (
            [],
            "Cortex source-lane truth is unavailable because extraction support exceeds the bounded service-status vocabulary.",
        )

    return normalized_lanes, None


def _runtime_surface_summary(
    *,
    implemented_slices: list[str],
    admitted_source_lanes: list[str],
) -> dict[str, Any]:
    return {
        "implemented_slices": implemented_slices,
        "admitted_source_lanes": admitted_source_lanes,
        "bounded_runtime_only": True,
    }


def _build_status(
    *,
    state: str,
    readiness_class: str,
    readiness_summary: str,
    operator_visible_message: str,
    implemented_slices: list[str],
    admitted_source_lanes: list[str],
    degraded_subtype: str | None = None,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "service_id": SERVICE_ID,
        "service_class": SERVICE_CLASS,
        "state": state,
        "readiness_summary": {
            "readiness_class": readiness_class,
            "summary": readiness_summary,
        },
        "runtime_surface_summary": _runtime_surface_summary(
            implemented_slices=implemented_slices,
            admitted_source_lanes=admitted_source_lanes,
        ),
        "watcher_summary": {
            "active_watch_scope_count": 0,
            "contract_scoped_only": True,
        },
        "operator_visible_message": operator_visible_message,
        "details_redacted": True,
        "last_updated_at": _utc_now(),
    }
    if degraded_subtype is not None:
        payload["degraded_subtype"] = degraded_subtype
    return payload


def _build_unavailable_status(
    *,
    implemented_slices: list[str],
    admitted_source_lanes: list[str],
    message: str,
) -> dict[str, Any]:
    return _build_status(
        state="unavailable",
        readiness_class="not_ready",
        readiness_summary=message,
        operator_visible_message=message,
        implemented_slices=implemented_slices,
        admitted_source_lanes=admitted_source_lanes,
    )


def _status_candidate() -> dict[str, Any]:
    implemented_slices = _implemented_runtime_slices()
    admitted_source_lanes, source_lane_message = _admitted_source_lanes()
    expected_slices = [
        slice_id for slice_id, _ in BASE_RUNTIME_SLICE_SPECS
    ] + configured_source_lane_slice_ids()

    if source_lane_message is not None or not admitted_source_lanes:
        message = source_lane_message or (
            "Cortex is unavailable because no governed local source lanes are currently admitted."
        )
        return _build_unavailable_status(
            implemented_slices=implemented_slices,
            admitted_source_lanes=admitted_source_lanes,
            message=message,
        )

    missing_slices = [slice_id for slice_id in expected_slices if slice_id not in implemented_slices]
    if missing_slices:
        message = (
            "Cortex is degraded because one or more bounded runtime slices are unavailable: "
            + _slice_label_list(missing_slices)
            + "."
        )
        return _build_status(
            state="degraded",
            readiness_class="degraded",
            readiness_summary=message,
            operator_visible_message=message,
            implemented_slices=implemented_slices,
            admitted_source_lanes=admitted_source_lanes,
            degraded_subtype="dependency_unavailable",
        )

    summary = (
        "Bounded runtime slices are implemented for intake, extraction, retrieval-package, service-status, "
        "and admitted source lanes: "
        + _source_lane_label_list(admitted_source_lanes)
        + "."
    )
    message = (
        "Cortex is ready for bounded intake, syntax-only extraction, retrieval-package, "
        "and service-status runtime paths on "
        + _source_lane_label_list(admitted_source_lanes)
        + "."
    )
    return _build_status(
        state="ready",
        readiness_class="ready",
        readiness_summary=summary,
        operator_visible_message=message,
        implemented_slices=implemented_slices,
        admitted_source_lanes=admitted_source_lanes,
    )


def emit_service_status() -> dict[str, Any]:
    candidate = _status_candidate()
    if not _schema_error_messages(candidate):
        return candidate

    fallback = _build_unavailable_status(
        implemented_slices=["slice4_service_status_truth"],
        admitted_source_lanes=[],
        message=(
            "Cortex service-status emission failed closed because the bounded status contract could not be satisfied."
        ),
    )
    fallback_errors = _schema_error_messages(fallback)
    if fallback_errors:
        raise RuntimeError("fallback service status violated schema: " + "; ".join(fallback_errors))
    return fallback


def _build_parser() -> argparse.ArgumentParser:
    return argparse.ArgumentParser(
        description="Emit bounded Cortex service-status truth for the currently implemented local runtime surfaces."
    )


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    parser.parse_args(argv)

    result = emit_service_status()
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["state"] == "ready" else 1


if __name__ == "__main__":
    raise SystemExit(main())

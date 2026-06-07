from __future__ import annotations

import hashlib
import json
from datetime import UTC, datetime
from pathlib import Path
from typing import Iterable

from cortex_runtime.gnats.models import (
    GNAT_DEFAULT_DEADLINE_MS,
    GNAT_DEFAULT_MAX_BYTES,
    GNAT_DEFAULT_MAX_CONCURRENCY,
    GNAT_HARD_MAX_CONCURRENCY,
    GNAT_PLANNER_VERSION,
    GnatRunPlan,
    GnatShard,
    GnatSourceInput,
    SourceFingerprint,
)
from cortex_runtime.gnats.schema_validation import require_schema_valid
from cortex_runtime.source_lanes import DOCX_TEXT_LANE, EPUB_TEXT_LANE, MARKDOWN_LANE, ODT_TEXT_LANE, PLAIN_TEXT_LANE
from cortex_runtime.source_lanes import RTF_TEXT_LANE
from cortex_runtime.source_lanes import lane_eligibility_for_path
from cortex_runtime.source_lanes import PDF_TEXT_LANE


class GnatPlanningError(ValueError):
    """Raised when Cortex cannot build a finite Gnat plan for the request."""


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _timestamp_from_epoch(epoch_seconds: float) -> str:
    return datetime.fromtimestamp(epoch_seconds, tz=UTC).isoformat().replace("+00:00", "Z")


def _canonical_hash(payload: object) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def _short_digest(payload: str) -> str:
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:12]


def _source_path_token(path: Path) -> str:
    try:
        resolved = str(path.resolve())
    except OSError:
        resolved = str(path)
    return f"src:{_short_digest(resolved)}"


def _fingerprint_source(path: Path) -> SourceFingerprint:
    try:
        raw_bytes = path.read_bytes()
        file_stat = path.stat()
    except OSError as exc:
        raise GnatPlanningError("source could not be read for Gnat planning") from exc

    return SourceFingerprint(
        algorithm="sha256",
        digest=hashlib.sha256(raw_bytes).hexdigest(),
        byte_count=len(raw_bytes),
        modified_at=_timestamp_from_epoch(file_stat.st_mtime),
    )


def fingerprint_source(path: str | Path) -> SourceFingerprint:
    return _fingerprint_source(Path(path))


def _source_input(candidate: GnatSourceInput | str | Path) -> GnatSourceInput:
    if isinstance(candidate, GnatSourceInput):
        return candidate
    return GnatSourceInput(path=Path(candidate))


def _worker_type_for_path(path: Path, media_type: str | None) -> tuple[str, str]:
    lane_decision = lane_eligibility_for_path(path, media_type=media_type)
    if not lane_decision.admitted or lane_decision.lane is None:
        raise GnatPlanningError(
            lane_decision.operator_visible_summary
            or "source is outside the bounded Cortex Gnat source-lane framework"
        )

    lane = lane_decision.lane
    if lane.lane_id == MARKDOWN_LANE.lane_id:
        return "markdown_syntax", "text/markdown"
    if lane.lane_id == PLAIN_TEXT_LANE.lane_id:
        return "plain_text_syntax", "text/plain"
    if lane.lane_id == PDF_TEXT_LANE.lane_id:
        return "pdf_text_syntax", "application/pdf"
    if lane.lane_id == DOCX_TEXT_LANE.lane_id:
        return "docx_text_syntax", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    if lane.lane_id == RTF_TEXT_LANE.lane_id:
        return "rtf_text_syntax", "application/rtf"
    if lane.lane_id == ODT_TEXT_LANE.lane_id:
        return "odt_text_syntax", "application/vnd.oasis.opendocument.text"
    if lane.lane_id == EPUB_TEXT_LANE.lane_id:
        return "epub_text_syntax", "application/epub+zip"

    raise GnatPlanningError(
        "GNAT admits only Markdown, plain-text, PDF text-layer, DOCX, RTF, ODT, and EPUB source lanes"
    )


def _clamp_concurrency(requested_concurrency: int, max_concurrency: int) -> tuple[int, int]:
    if requested_concurrency < 1:
        raise GnatPlanningError("requested_concurrency must be at least 1")
    if max_concurrency < 1:
        raise GnatPlanningError("max_concurrency must be at least 1")
    requested = min(requested_concurrency, GNAT_HARD_MAX_CONCURRENCY)
    configured = min(max_concurrency, GNAT_HARD_MAX_CONCURRENCY)
    return requested, configured


def _derive_run_id(request_id: str, shard_basis: list[dict[str, object]]) -> str:
    digest = _canonical_hash({"request_id": request_id, "shards": shard_basis}).removeprefix("sha256:")
    return f"gnat-run-{digest[:16]}"


def _derive_shard_id(run_id: str, ordinal: int, source_ref: str, fingerprint: SourceFingerprint) -> str:
    digest = _short_digest(f"{run_id}:{ordinal}:{source_ref}:{fingerprint.digest}")
    return f"{run_id}-shard-{ordinal:04d}-{digest}"


def _plan_payload_without_hash(plan: GnatRunPlan) -> dict[str, object]:
    payload = plan.to_contract()
    payload["plan_hash"] = "sha256:pending"
    return payload


def plan_gnat_run(
    sources: Iterable[GnatSourceInput | str | Path],
    *,
    request_id: str,
    correlation_id: str | None = None,
    requested_concurrency: int = 1,
    max_concurrency: int = GNAT_DEFAULT_MAX_CONCURRENCY,
    deadline_ms: int = GNAT_DEFAULT_DEADLINE_MS,
    max_bytes: int = GNAT_DEFAULT_MAX_BYTES,
    serial_fallback_allowed: bool = True,
) -> GnatRunPlan:
    if not request_id:
        raise GnatPlanningError("request_id is required")
    if deadline_ms < 1:
        raise GnatPlanningError("deadline_ms must be positive")
    if max_bytes < 1:
        raise GnatPlanningError("max_bytes must be positive")

    source_inputs = [_source_input(source) for source in sources]
    if not source_inputs:
        raise GnatPlanningError("at least one source is required")
    if len(source_inputs) > 256:
        raise GnatPlanningError("GNAT-01 admits at most 256 shards per run")

    requested, configured_max = _clamp_concurrency(requested_concurrency, max_concurrency)
    prepared: list[tuple[GnatSourceInput, str, str, SourceFingerprint]] = []
    shard_basis: list[dict[str, object]] = []
    for source in source_inputs:
        path = Path(source.path)
        worker_type, media_type = _worker_type_for_path(path, source.media_type)
        fingerprint = _fingerprint_source(path)
        if fingerprint.byte_count > max_bytes:
            raise GnatPlanningError("source exceeds the bounded GNAT-01 byte limit")
        source_ref = source.source_ref or path.name
        prepared.append((source, worker_type, media_type, fingerprint))
        shard_basis.append(
            {
                "source_ref": source_ref,
                "media_type": media_type,
                "worker_type": worker_type,
                "source_digest": fingerprint.digest,
            }
        )

    run_id = _derive_run_id(request_id, shard_basis)
    shards: list[GnatShard] = []
    for ordinal, (source, worker_type, media_type, fingerprint) in enumerate(prepared):
        path = Path(source.path)
        source_ref = source.source_ref or path.name
        shards.append(
            GnatShard(
                run_id=run_id,
                shard_id=_derive_shard_id(run_id, ordinal, source_ref, fingerprint),
                ordinal=ordinal,
                worker_type=worker_type,
                source_ref=source_ref,
                source_path_token=_source_path_token(path),
                media_type=media_type,
                source_fingerprint=fingerprint,
                deadline_ms=deadline_ms,
                max_bytes=max_bytes,
                local_path=path,
            )
        )

    created_at = _utc_now()
    draft_plan = GnatRunPlan(
        run_id=run_id,
        request_id=request_id,
        correlation_id=correlation_id or request_id,
        shards=tuple(shards),
        requested_concurrency=requested,
        max_concurrency=configured_max,
        deadline_ms=deadline_ms,
        plan_hash="sha256:pending",
        serial_fallback_allowed=serial_fallback_allowed,
        created_at=created_at,
    )
    final_plan = GnatRunPlan(
        run_id=draft_plan.run_id,
        request_id=draft_plan.request_id,
        correlation_id=draft_plan.correlation_id,
        shards=draft_plan.shards,
        requested_concurrency=draft_plan.requested_concurrency,
        max_concurrency=draft_plan.max_concurrency,
        deadline_ms=draft_plan.deadline_ms,
        plan_hash=_canonical_hash(
            {
                "planner_version": GNAT_PLANNER_VERSION,
                "plan": _plan_payload_without_hash(draft_plan),
            }
        ),
        serial_fallback_allowed=draft_plan.serial_fallback_allowed,
        created_at=draft_plan.created_at,
    )
    require_schema_valid(final_plan.to_contract(), schema_name="gnat-run-plan.schema.json")
    for shard in final_plan.shards:
        require_schema_valid(shard.to_contract(), schema_name="gnat-shard.schema.json")
    return final_plan

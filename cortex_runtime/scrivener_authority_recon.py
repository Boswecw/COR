from __future__ import annotations

import argparse
import hashlib
import json
import os
from datetime import UTC, datetime
from functools import lru_cache
from pathlib import Path
from typing import Any
from xml.etree import ElementTree as ET

from jsonschema import Draft202012Validator  # type: ignore[import-untyped]


ROOT = Path(__file__).resolve().parent.parent
SCHEMA_PATH = ROOT / "schemas/scrivener-authority-recon-status.schema.json"
OBSERVER_VERSION = "slice10.scrivener_stage1.1"
OPERATOR_DISABLE_ENV = "CORTEX_SCRIVENER_STAGE1_DISABLED"


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _timestamp_from_epoch(epoch_seconds: float) -> str:
    return datetime.fromtimestamp(epoch_seconds, tz=UTC).isoformat().replace("+00:00", "Z")


@lru_cache(maxsize=1)
def _validator() -> Draft202012Validator:
    with SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        return Draft202012Validator(json.load(handle))


def _schema_error_messages(result: dict[str, Any]) -> list[str]:
    validator = _validator()
    errors = sorted(
        validator.iter_errors(result),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    return [
        f"{'.'.join(str(part) for part in error.path) or '<root>'}: {error.message}"
        for error in errors
    ]


def _artifact_id(request_id: str, source_ref: str) -> str:
    digest = hashlib.sha256(f"{request_id}:{source_ref}".encode("utf-8")).hexdigest()[:16]
    return f"scriv-recon-{digest}"


def _directory_provenance(source_path: Path) -> tuple[str, str, int]:
    digest = hashlib.sha256()
    if not source_path.exists() or not source_path.is_dir():
        digest.update(str(source_path).encode("utf-8"))
        return f"sha256:{digest.hexdigest()}", _utc_now(), 0

    total_bytes = 0
    latest_epoch = source_path.stat().st_mtime

    for file_path in sorted(path for path in source_path.rglob("*") if path.is_file()):
        relative_path = file_path.relative_to(source_path).as_posix().encode("utf-8")
        digest.update(relative_path)
        digest.update(b"\0")
        try:
            latest_epoch = max(latest_epoch, file_path.stat().st_mtime)
            payload = file_path.read_bytes()
        except OSError:
            digest.update(b"<unreadable>")
            digest.update(b"\0")
            continue
        digest.update(payload)
        digest.update(b"\0")
        total_bytes += len(payload)

    return f"sha256:{digest.hexdigest()}", _timestamp_from_epoch(latest_epoch), total_bytes


def _truthy_env(name: str) -> bool:
    value = os.getenv(name, "")
    return value.strip().lower() in {"1", "true", "yes", "on"}


def _build_base_result(
    *,
    request_id: str,
    source_ref: str,
    state: str,
    source_path: Path,
    source_hash: str,
    source_modified_at: str,
    byte_count: int,
    authority_status: dict[str, Any],
    package_status: dict[str, Any],
    mapping_status: dict[str, Any],
    observed_role_surfaces: list[str],
    observed_binder_summary: dict[str, Any],
    refusal: dict[str, Any] | None,
    authority_basis: str,
    authority_path: str | None = None,
) -> dict[str, Any]:
    provenance: dict[str, Any] = {
        "source_hash": source_hash,
        "observer_version": OBSERVER_VERSION,
        "source_modified_at": source_modified_at,
        "byte_count": byte_count,
        "source_lane_candidate": "scrivener_project",
        "project_container_path": str(source_path),
        "authority_basis": authority_basis,
    }
    if authority_path is not None:
        provenance["authority_path"] = authority_path

    result: dict[str, Any] = {
        "artifact_id": _artifact_id(request_id, source_ref),
        "request_id": request_id,
        "source_ref": source_ref,
        "state": state,
        "observation_boundary": "status_only",
        "semantic_boundary_enforced": True,
        "authority_status": authority_status,
        "package_status": package_status,
        "mapping_status": mapping_status,
        "observed_role_surfaces": observed_role_surfaces,
        "observed_binder_summary": observed_binder_summary,
        "provenance": provenance,
        "observed_at": _utc_now(),
    }
    if refusal is not None:
        result["refusal"] = refusal
    return result


def _status_with_defaults(
    *,
    authority_state: str,
    authority_summary: str,
    candidate_paths: list[str],
    resolved_authority_path: str | None = None,
    package_state: str,
    package_summary: str,
    data_root_present: bool = False,
    mapping_state: str,
    mapping_summary: str,
    mirrored_uuid_count: int = 0,
    binder_only_uuid_count: int = 0,
    data_only_uuid_count: int = 0,
    direct_missing_content_target_count: int = 0,
    observed_role_surfaces: list[str] | None = None,
    observed_binder_summary: dict[str, Any] | None = None,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], list[str], dict[str, Any]]:
    return (
        {
            "state": authority_state,
            "candidate_count": len(candidate_paths),
            "candidate_paths": candidate_paths,
            "operator_visible_summary": authority_summary,
            **({"resolved_authority_path": resolved_authority_path} if resolved_authority_path is not None else {}),
        },
        {
            "state": package_state,
            "data_root_present": data_root_present,
            "operator_visible_summary": package_summary,
        },
        {
            "state": mapping_state,
            "mirrored_uuid_count": mirrored_uuid_count,
            "binder_only_uuid_count": binder_only_uuid_count,
            "data_only_uuid_count": data_only_uuid_count,
            "direct_missing_content_target_count": direct_missing_content_target_count,
            "operator_visible_summary": mapping_summary,
        },
        observed_role_surfaces or [],
        observed_binder_summary
        or {
            "binder_item_count": 0,
            "top_level_binder_item_count": 0,
            "max_binder_depth": 0,
        },
    )


def _emit_non_ready_result(
    *,
    request_id: str,
    source_ref: str,
    state: str,
    reason_class: str,
    summary: str,
    source_path: Path,
    authority_basis: str,
    authority_state: str,
    candidate_paths: list[str],
    package_state: str = "not_attempted",
    package_summary: str = "Package observation was not attempted.",
    data_root_present: bool = False,
    mapping_state: str = "not_attempted",
    mapping_summary: str = "Mapping observation was not attempted.",
    resolved_authority_path: str | None = None,
    mirrored_uuid_count: int = 0,
    binder_only_uuid_count: int = 0,
    data_only_uuid_count: int = 0,
    direct_missing_content_target_count: int = 0,
    observed_role_surfaces: list[str] | None = None,
    observed_binder_summary: dict[str, Any] | None = None,
) -> dict[str, Any]:
    source_hash, source_modified_at, byte_count = _directory_provenance(source_path)
    (
        authority_status,
        package_status,
        mapping_status,
        role_surfaces,
        binder_summary,
    ) = _status_with_defaults(
        authority_state=authority_state,
        authority_summary=summary if authority_state == "not_attempted" else summary,
        candidate_paths=candidate_paths,
        resolved_authority_path=resolved_authority_path,
        package_state=package_state,
        package_summary=package_summary,
        data_root_present=data_root_present,
        mapping_state=mapping_state,
        mapping_summary=mapping_summary,
        mirrored_uuid_count=mirrored_uuid_count,
        binder_only_uuid_count=binder_only_uuid_count,
        data_only_uuid_count=data_only_uuid_count,
        direct_missing_content_target_count=direct_missing_content_target_count,
        observed_role_surfaces=observed_role_surfaces,
        observed_binder_summary=observed_binder_summary,
    )
    return _validate_or_raise(
        _build_base_result(
            request_id=request_id,
            source_ref=source_ref,
            state=state,
            source_path=source_path,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
            authority_status=authority_status,
            package_status=package_status,
            mapping_status=mapping_status,
            observed_role_surfaces=role_surfaces,
            observed_binder_summary=binder_summary,
            refusal={
                "reason_class": reason_class,
                "operator_visible_summary": summary,
            },
            authority_basis=authority_basis,
            authority_path=resolved_authority_path,
        )
    )


def _validate_or_raise(result: dict[str, Any]) -> dict[str, Any]:
    errors = _schema_error_messages(result)
    if errors:
        raise RuntimeError("Scrivener authority recon result violated schema: " + "; ".join(errors))
    return result


def _max_binder_depth(items: list[ET.Element]) -> int:
    def depth(item: ET.Element, current: int = 1) -> int:
        children = item.findall("Children/BinderItem")
        if not children:
            return current
        return max(depth(child, current + 1) for child in children)

    return max((depth(item) for item in items), default=0)


def emit_scrivener_authority_recon_from_source_file(
    source_path: str | Path,
    *,
    request_id: str,
    source_ref: str,
) -> dict[str, Any]:
    path = Path(source_path)

    if _truthy_env(OPERATOR_DISABLE_ENV):
        return _emit_non_ready_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="operator_disabled",
            summary="Scrivener authority recon is denied because operator posture disables this Stage 1 observation path.",
            source_path=path,
            authority_basis="not_attempted",
            authority_state="not_attempted",
            candidate_paths=[],
        )

    if not path.exists() or not path.is_dir() or path.suffix.lower() != ".scriv":
        return _emit_non_ready_result(
            request_id=request_id,
            source_ref=source_ref,
            state="denied",
            reason_class="unsupported_source_type",
            summary="Scrivener authority recon is denied because the input is not a local .scriv project directory.",
            source_path=path,
            authority_basis="not_attempted",
            authority_state="not_attempted",
            candidate_paths=[],
        )

    authority_candidates = sorted(candidate.name for candidate in path.glob("*.scrivx") if candidate.is_file())
    if not authority_candidates:
        return _emit_non_ready_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Scrivener authority recon is unavailable because no top-level .scrivx authority candidate is present.",
            source_path=path,
            authority_basis="missing_top_level_scrivx",
            authority_state="missing_authority",
            candidate_paths=[],
            package_state="package_observed",
            package_summary="A .scriv container is present, but no top-level authority file can be resolved.",
            mapping_state="not_attempted",
            mapping_summary="Mapping observation did not begin because project authority could not be resolved.",
        )

    if len(authority_candidates) > 1:
        return _emit_non_ready_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Scrivener authority recon is unavailable because multiple top-level .scrivx authority candidates are present.",
            source_path=path,
            authority_basis="ambiguous_top_level_scrivx",
            authority_state="ambiguous_authority",
            candidate_paths=authority_candidates,
            package_state="package_observed",
            package_summary="A package-shaped .scriv container is present, but project authority is ambiguous.",
            mapping_state="not_attempted",
            mapping_summary="Mapping observation did not begin because project authority is ambiguous.",
        )

    authority_path = path / authority_candidates[0]
    try:
        root = ET.parse(authority_path).getroot()
    except (ET.ParseError, OSError):
        return _emit_non_ready_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Scrivener authority recon is unavailable because the top-level .scrivx authority file is unreadable or malformed.",
            source_path=path,
            authority_basis="malformed_top_level_scrivx",
            authority_state="malformed_authority",
            candidate_paths=authority_candidates,
            resolved_authority_path=str(authority_path),
            package_state="package_observed",
            package_summary="A package-shaped .scriv container is present, but the authority XML cannot be trusted.",
            mapping_state="not_attempted",
            mapping_summary="Mapping observation did not begin because project authority could not be parsed safely.",
        )

    if root.tag != "ScrivenerProject":
        return _emit_non_ready_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Scrivener authority recon is unavailable because the top-level authority XML is outside the currently observed project bounds.",
            source_path=path,
            authority_basis="malformed_top_level_scrivx",
            authority_state="malformed_authority",
            candidate_paths=authority_candidates,
            resolved_authority_path=str(authority_path),
            package_state="package_observed",
            package_summary="Authority XML was readable but did not match the currently observed Scrivener project root shape.",
            mapping_state="not_attempted",
            mapping_summary="Mapping observation did not begin because authority structure is outside current evidence bounds.",
        )

    binder = root.find("Binder")
    data_root = path / "Files" / "Data"
    if binder is None or not data_root.is_dir():
        return _emit_non_ready_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Scrivener authority recon is unavailable because required structural package surfaces are missing.",
            source_path=path,
            authority_basis="single_top_level_scrivx",
            authority_state="single_authority_observed",
            candidate_paths=authority_candidates,
            resolved_authority_path=str(authority_path),
            package_state="package_unavailable",
            package_summary="Required structural package surfaces are missing from the .scriv container.",
            data_root_present=data_root.is_dir(),
            mapping_state="mapping_unavailable",
            mapping_summary="Mapping observation is unavailable because required structural package surfaces are missing.",
        )

    all_items = binder.findall(".//BinderItem")
    top_level_items = binder.findall("BinderItem")
    item_by_uuid = {
        item.get("UUID"): item
        for item in all_items
        if isinstance(item.get("UUID"), str) and item.get("UUID")
    }
    data_dirs = {entry.name for entry in data_root.iterdir() if entry.is_dir()}
    mirrored = set(item_by_uuid).intersection(data_dirs)
    binder_only = set(item_by_uuid).difference(data_dirs)
    data_only = data_dirs.difference(item_by_uuid)

    role_surfaces: list[str] = []
    if any(item.get("Type") == "DraftFolder" for item in all_items):
        role_surfaces.append("draft")
    if any(item.get("Type") == "ResearchFolder" for item in all_items):
        role_surfaces.append("research")
    if any(item.get("Type") == "TrashFolder" for item in all_items):
        role_surfaces.append("trash")
    if root.findtext("TemplateFolderUUID"):
        role_surfaces.append("template")
    if root.findtext("BookmarksFolderUUID"):
        role_surfaces.append("bookmarks")

    direct_missing_targets: list[str] = []
    for uuid, item in item_by_uuid.items():
        if not isinstance(uuid, str):
            continue
        if item.get("Type") != "Text":
            continue
        data_dir = data_root / uuid
        if not data_dir.is_dir():
            continue
        if not (data_dir / "content.rtf").is_file():
            direct_missing_targets.append(uuid)

    binder_summary = {
        "binder_item_count": len(all_items),
        "top_level_binder_item_count": len(top_level_items),
        "max_binder_depth": _max_binder_depth(top_level_items),
    }

    if direct_missing_targets:
        return _emit_non_ready_result(
            request_id=request_id,
            source_ref=source_ref,
            state="unavailable",
            reason_class="dependency_unavailable",
            summary="Scrivener authority recon is unavailable because directly expected text-side correspondence is incomplete for the observed authority surface.",
            source_path=path,
            authority_basis="single_top_level_scrivx",
            authority_state="single_authority_observed",
            candidate_paths=authority_candidates,
            resolved_authority_path=str(authority_path),
            package_state="package_observed",
            package_summary="A package-shaped .scriv container with readable authority was observed.",
            data_root_present=True,
            mapping_state="mapping_unavailable",
            mapping_summary="Directly expected content-side correspondence is incomplete for at least one observed text item.",
            mirrored_uuid_count=len(mirrored),
            binder_only_uuid_count=len(binder_only),
            data_only_uuid_count=len(data_only),
            direct_missing_content_target_count=len(direct_missing_targets),
            observed_role_surfaces=role_surfaces,
            observed_binder_summary=binder_summary,
        )

    if mirrored and not data_only:
        mapping_state = "candidate_mapping_observed"
        mapping_summary = (
            "Candidate UUID-to-data correspondence is observable at status level only; deterministic mapping is not yet proven."
        )
    else:
        mapping_state = "mapping_unresolved"
        mapping_summary = (
            "Authority and binder identity are readable, but correspondence remains structurally unresolved at Stage 1."
        )

    source_hash, source_modified_at, byte_count = _directory_provenance(path)
    (
        authority_status,
        package_status,
        mapping_status,
        normalized_roles,
        normalized_binder_summary,
    ) = _status_with_defaults(
        authority_state="single_authority_observed",
        authority_summary="Exactly one top-level readable .scrivx authority candidate was resolved.",
        candidate_paths=authority_candidates,
        resolved_authority_path=str(authority_path),
        package_state="package_observed",
        package_summary="A package-shaped .scriv container with readable authority and required structural surfaces was observed.",
        data_root_present=True,
        mapping_state=mapping_state,
        mapping_summary=mapping_summary,
        mirrored_uuid_count=len(mirrored),
        binder_only_uuid_count=len(binder_only),
        data_only_uuid_count=len(data_only),
        direct_missing_content_target_count=0,
        observed_role_surfaces=role_surfaces,
        observed_binder_summary=binder_summary,
    )
    return _validate_or_raise(
        _build_base_result(
            request_id=request_id,
            source_ref=source_ref,
            state="ready",
            source_path=path,
            source_hash=source_hash,
            source_modified_at=source_modified_at,
            byte_count=byte_count,
            authority_status=authority_status,
            package_status=package_status,
            mapping_status=mapping_status,
            observed_role_surfaces=normalized_roles,
            observed_binder_summary=normalized_binder_summary,
            refusal=None,
            authority_basis="single_top_level_scrivx",
            authority_path=str(authority_path),
        )
    )


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Emit bounded Stage 1 Scrivener authority-recon status for one local .scriv project directory."
    )
    parser.add_argument("--source-path", required=True, help="Path to one local .scriv project directory.")
    parser.add_argument("--request-id", required=True, help="Bounded request identifier.")
    parser.add_argument("--source-ref", required=True, help="Bounded source reference.")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)
    result = emit_scrivener_authority_recon_from_source_file(
        args.source_path,
        request_id=args.request_id,
        source_ref=args.source_ref,
    )
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["state"] == "ready" else 1


__all__ = ["emit_scrivener_authority_recon_from_source_file", "main"]

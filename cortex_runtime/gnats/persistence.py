from __future__ import annotations

import copy
import hashlib
import json
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from typing import Any, Protocol

from cortex_runtime.gnats.models import (
    GNAT_CACHE_RECORD_VERSION,
    GNAT_OUTPUT_CONTRACT,
    GNAT_WORKER_IMPLEMENTATION_VERSION,
    GnatRunPlan,
    GnatShard,
)
from cortex_runtime.gnats.receipt import build_receipt_from_extraction, utc_now
from cortex_runtime.gnats.schema_validation import require_schema_valid


GNAT_DEFAULT_RETENTION_DAYS = 30
GNAT_LANE_CONTRACT_VERSION_BY_WORKER = {
    "markdown_syntax": "local_file_markdown.v1",
    "plain_text_syntax": "local_file_plain_text.v1",
    "pdf_text_syntax": "local_file_pdf_text.v1",
    "docx_text_syntax": "local_file_docx_text.v1",
}


class GnatPersistenceError(RuntimeError):
    """Raised when the DF-Local persistence adapter is unavailable or degraded."""


@dataclass(frozen=True)
class GnatCacheIdentity:
    source_fingerprint_digest: str
    worker_type: str
    worker_implementation_version: str
    operation_contract_version: str
    lane_contract_version: str

    def to_contract(self) -> dict[str, str]:
        return {
            "source_fingerprint_digest": self.source_fingerprint_digest,
            "worker_type": self.worker_type,
            "worker_implementation_version": self.worker_implementation_version,
            "operation_contract_version": self.operation_contract_version,
            "lane_contract_version": self.lane_contract_version,
        }


class GnatPersistenceStore(Protocol):
    def record_plan(self, plan_contract: dict[str, Any]) -> None:
        ...

    def record_receipt(self, receipt: dict[str, Any]) -> str:
        ...

    def record_summary(self, summary: dict[str, Any]) -> str:
        ...

    def get_cache_record(self, cache_key: str) -> dict[str, Any] | None:
        ...

    def get_receipt(self, receipt_hash: str) -> dict[str, Any] | None:
        ...

    def upsert_cache_record(self, cache_record: dict[str, Any]) -> None:
        ...

    def invalidate_source_digest(self, source_fingerprint_digest: str) -> int:
        ...


class InMemoryGnatPersistenceStore:
    def __init__(self, *, fail_reads: bool = False, fail_writes: bool = False) -> None:
        self.fail_reads = fail_reads
        self.fail_writes = fail_writes
        self.plans: dict[str, dict[str, Any]] = {}
        self.receipts: dict[str, dict[str, Any]] = {}
        self.summaries: dict[str, dict[str, Any]] = {}
        self.cache_records: dict[str, dict[str, Any]] = {}

    def record_plan(self, plan_contract: dict[str, Any]) -> None:
        self._require_write()
        self.plans[str(plan_contract["run_id"])] = copy.deepcopy(plan_contract)

    def record_receipt(self, receipt: dict[str, Any]) -> str:
        self._require_write()
        receipt_hash = canonical_hash(receipt)
        self.receipts[receipt_hash] = copy.deepcopy(receipt)
        return receipt_hash

    def record_summary(self, summary: dict[str, Any]) -> str:
        self._require_write()
        summary_hash = canonical_hash(summary)
        self.summaries[summary_hash] = copy.deepcopy(summary)
        return summary_hash

    def get_cache_record(self, cache_key: str) -> dict[str, Any] | None:
        self._require_read()
        record = self.cache_records.get(cache_key)
        return copy.deepcopy(record) if record is not None else None

    def get_receipt(self, receipt_hash: str) -> dict[str, Any] | None:
        self._require_read()
        receipt = self.receipts.get(receipt_hash)
        return copy.deepcopy(receipt) if receipt is not None else None

    def upsert_cache_record(self, cache_record: dict[str, Any]) -> None:
        self._require_write()
        self.cache_records[str(cache_record["cache_key"])] = copy.deepcopy(cache_record)

    def invalidate_source_digest(self, source_fingerprint_digest: str) -> int:
        self._require_write()
        invalidated = 0
        now = utc_now()
        for record in self.cache_records.values():
            if record["source_fingerprint_digest"] == source_fingerprint_digest and record["invalidated_at"] is None:
                record["cache_state"] = "invalidated"
                record["invalidated_at"] = now
                invalidated += 1
        return invalidated

    def _require_read(self) -> None:
        if self.fail_reads:
            raise GnatPersistenceError("DF-Local cache read failed")

    def _require_write(self) -> None:
        if self.fail_writes:
            raise GnatPersistenceError("DF-Local persistence write failed")


def canonical_hash(payload: object) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":"), default=str).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def cache_identity_for_shard(shard: GnatShard) -> GnatCacheIdentity:
    lane_contract_version = GNAT_LANE_CONTRACT_VERSION_BY_WORKER.get(shard.worker_type)
    if lane_contract_version is None:
        raise GnatPersistenceError(f"No GNAT lane contract version is registered for {shard.worker_type}")
    return GnatCacheIdentity(
        source_fingerprint_digest=shard.source_fingerprint.digest,
        worker_type=shard.worker_type,
        worker_implementation_version=GNAT_WORKER_IMPLEMENTATION_VERSION,
        operation_contract_version=GNAT_OUTPUT_CONTRACT,
        lane_contract_version=lane_contract_version,
    )


def cache_key_for_identity(identity: GnatCacheIdentity) -> str:
    return canonical_hash(
        {
            "contract_version": GNAT_CACHE_RECORD_VERSION,
            "cache_identity": identity.to_contract(),
        }
    )


def build_cache_record(
    shard: GnatShard,
    *,
    receipt: dict[str, Any],
    receipt_hash: str,
    created_at: datetime | None = None,
    retention_days: int = GNAT_DEFAULT_RETENTION_DAYS,
) -> dict[str, Any]:
    if retention_days < 1:
        raise GnatPersistenceError("GNAT cache retention_days must be positive")
    identity = cache_identity_for_shard(shard)
    created = created_at or datetime.now(UTC)
    output_refs: list[str] = []
    output = receipt.get("bounded_output")
    if isinstance(output, dict) and isinstance(output.get("artifact_id"), str):
        output_refs.append(str(output["artifact_id"]))
    if isinstance(receipt.get("output_ref"), str):
        output_refs.append(str(receipt["output_ref"]))

    record = {
        "contract_version": GNAT_CACHE_RECORD_VERSION,
        "cache_key": cache_key_for_identity(identity),
        **identity.to_contract(),
        "run_id": shard.run_id,
        "shard_id": shard.shard_id,
        "receipt_hash": receipt_hash,
        "output_artifact_refs": output_refs,
        "cache_state": "ready",
        "created_at": _format_dt(created),
        "last_used_at": _format_dt(created),
        "expires_at": _format_dt(created + timedelta(days=retention_days)),
        "invalidated_at": None,
        "details_redacted": True,
    }
    require_schema_valid(record, schema_name="gnat-cache-record.schema.json")
    return record


def exact_cache_record_matches(record: dict[str, Any], identity: GnatCacheIdentity, *, now: datetime | None = None) -> bool:
    if record.get("contract_version") != GNAT_CACHE_RECORD_VERSION:
        return False
    if record.get("cache_key") != cache_key_for_identity(identity):
        return False
    for key, value in identity.to_contract().items():
        if record.get(key) != value:
            return False
    if record.get("cache_state") != "ready":
        return False
    if record.get("invalidated_at") is not None:
        return False
    expires_at = _parse_dt(record.get("expires_at"))
    if expires_at is None:
        return False
    return expires_at > (now or datetime.now(UTC))


def cached_receipt_for_shard(store: GnatPersistenceStore, shard: GnatShard) -> dict[str, Any] | None:
    identity = cache_identity_for_shard(shard)
    record = store.get_cache_record(cache_key_for_identity(identity))
    if record is None or not exact_cache_record_matches(record, identity):
        return None
    old_receipt = store.get_receipt(str(record["receipt_hash"]))
    if not old_receipt or old_receipt.get("state") != "complete":
        return None
    output = old_receipt.get("bounded_output")
    if not isinstance(output, dict):
        return None

    cached_output = copy.deepcopy(output)
    cached_output["request_id"] = shard.run_id
    cached_output["source_ref"] = shard.source_ref
    now = utc_now()
    return build_receipt_from_extraction(
        shard,
        output=cached_output,
        source_fingerprint_before=shard.source_fingerprint,
        source_fingerprint_after=shard.source_fingerprint,
        started_at=now,
        completed_at=now,
        duration_ms=0,
    )


def invalidate_changed_source_records(
    store: GnatPersistenceStore,
    *,
    previous_plan: GnatRunPlan,
    next_plan: GnatRunPlan,
) -> int:
    next_by_source_ref = {shard.source_ref: shard for shard in next_plan.shards}
    invalidated = 0
    for previous_shard in previous_plan.shards:
        next_shard = next_by_source_ref.get(previous_shard.source_ref)
        if next_shard is None:
            continue
        if previous_shard.source_fingerprint.digest != next_shard.source_fingerprint.digest:
            invalidated += store.invalidate_source_digest(previous_shard.source_fingerprint.digest)
    return invalidated


def _format_dt(value: datetime) -> str:
    return value.astimezone(UTC).isoformat().replace("+00:00", "Z")


def _parse_dt(value: Any) -> datetime | None:
    if isinstance(value, datetime):
        return value.astimezone(UTC)
    if not isinstance(value, str) or not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(UTC)
    except ValueError:
        return None

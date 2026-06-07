from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any


GNAT_PLANNER_VERSION = "gnat-planner.v1"
GNAT_WORKER_IMPLEMENTATION_VERSION = "gnat-worker.v1"
GNAT_RUN_REQUEST_VERSION = "GnatRunRequest.v1"
GNAT_RUN_PLAN_VERSION = "GnatRunPlan.v1"
GNAT_SHARD_VERSION = "GnatShard.v1"
GNAT_WORKER_RECEIPT_VERSION = "GnatWorkerReceipt.v1"
GNAT_RUN_SUMMARY_VERSION = "GnatRunSummary.v1"
GNAT_DISPATCH_ENVELOPE_VERSION = "GnatDispatchEnvelope.v1"
GNAT_CACHE_RECORD_VERSION = "GnatCacheRecord.v1"
GNAT_OPERATION = "syntax_extract"
GNAT_OUTPUT_CONTRACT = "extraction-result.schema.json"
GNAT_RECEIPT_SCHEMA = "gnat-worker-receipt.schema.json"
GNAT_DEFAULT_DEADLINE_MS = 30000
GNAT_DEFAULT_MAX_BYTES = 20 * 1024 * 1024
GNAT_DEFAULT_MAX_CONCURRENCY = 4
GNAT_HARD_MAX_CONCURRENCY = 8


@dataclass(frozen=True)
class SourceFingerprint:
    algorithm: str
    digest: str
    byte_count: int
    modified_at: str

    def to_contract(self) -> dict[str, Any]:
        return {
            "algorithm": self.algorithm,
            "digest": self.digest,
            "byte_count": self.byte_count,
            "modified_at": self.modified_at,
        }


@dataclass(frozen=True)
class GnatSourceInput:
    path: Path
    media_type: str | None = None
    source_ref: str | None = None


@dataclass(frozen=True)
class GnatShard:
    run_id: str
    shard_id: str
    ordinal: int
    worker_type: str
    source_ref: str
    source_path_token: str
    media_type: str
    source_fingerprint: SourceFingerprint
    deadline_ms: int
    max_bytes: int
    local_path: Path

    def to_contract(self) -> dict[str, Any]:
        return {
            "contract_version": GNAT_SHARD_VERSION,
            "run_id": self.run_id,
            "shard_id": self.shard_id,
            "ordinal": self.ordinal,
            "worker_type": self.worker_type,
            "source_ref": self.source_ref,
            "source_path_token": self.source_path_token,
            "media_type": self.media_type,
            "source_fingerprint": self.source_fingerprint.to_contract(),
            "operation": GNAT_OPERATION,
            "limits": {
                "deadline_ms": self.deadline_ms,
                "max_bytes": self.max_bytes,
            },
            "output_contract": GNAT_OUTPUT_CONTRACT,
        }


@dataclass(frozen=True)
class GnatRunPlan:
    run_id: str
    request_id: str
    correlation_id: str
    shards: tuple[GnatShard, ...]
    requested_concurrency: int
    max_concurrency: int
    deadline_ms: int
    plan_hash: str
    serial_fallback_allowed: bool
    created_at: str

    def to_contract(self) -> dict[str, Any]:
        return {
            "contract_version": GNAT_RUN_PLAN_VERSION,
            "run_id": self.run_id,
            "request_id": self.request_id,
            "correlation_id": self.correlation_id,
            "planner_version": GNAT_PLANNER_VERSION,
            "operation": GNAT_OPERATION,
            "shard_count": len(self.shards),
            "shards": [shard.to_contract() for shard in self.shards],
            "execution_limits": {
                "requested_concurrency": self.requested_concurrency,
                "max_concurrency": self.max_concurrency,
                "deadline_ms": self.deadline_ms,
            },
            "expected_receipt_schema": GNAT_RECEIPT_SCHEMA,
            "plan_hash": self.plan_hash,
            "serial_fallback_allowed": self.serial_fallback_allowed,
            "created_at": self.created_at,
        }


@dataclass(frozen=True)
class GnatSerialResult:
    plan: GnatRunPlan
    receipts: tuple[dict[str, Any], ...]
    summary: dict[str, Any]


@dataclass(frozen=True)
class FaLocalCapabilityState:
    fa_local_state: str
    supported_contract_versions: tuple[str, ...] = ()
    admitted_worker_types: tuple[str, ...] = ()
    max_concurrency: int = 0
    cancellation_supported: bool = False


@dataclass(frozen=True)
class GnatDispatchNegotiation:
    state: str
    run_id: str
    correlation_id: str
    effective_concurrency: int
    admitted_worker_types: tuple[str, ...]
    operator_visible_summary: str


@dataclass(frozen=True)
class GnatParallelResult:
    plan: GnatRunPlan
    receipts: tuple[dict[str, Any], ...]
    summary: dict[str, Any]
    negotiation: GnatDispatchNegotiation


@dataclass(frozen=True)
class GnatPersistenceOutcome:
    state: str
    cache_hits: int
    cache_misses: int
    cache_records_written: int
    operator_visible_summary: str

    def to_contract(self) -> dict[str, Any]:
        return {
            "state": self.state,
            "cache_hits": self.cache_hits,
            "cache_misses": self.cache_misses,
            "cache_records_written": self.cache_records_written,
            "operator_visible_summary": self.operator_visible_summary,
            "details_redacted": True,
        }


@dataclass(frozen=True)
class GnatPersistentResult:
    plan: GnatRunPlan
    receipts: tuple[dict[str, Any], ...]
    summary: dict[str, Any]
    negotiation: GnatDispatchNegotiation
    persistence: GnatPersistenceOutcome

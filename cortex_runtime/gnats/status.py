from __future__ import annotations

from typing import Any

from cortex_runtime.gnats.models import GNAT_DEFAULT_MAX_CONCURRENCY, GNAT_HARD_MAX_CONCURRENCY
from cortex_runtime.source_lanes import gnat_admitted_worker_types


def gnat_status_summary() -> dict[str, Any]:
    return {
        "profile": "serial_contract_proof",
        "admitted_worker_types": gnat_admitted_worker_types(),
        "max_concurrency": GNAT_DEFAULT_MAX_CONCURRENCY,
        "hard_cap": GNAT_HARD_MAX_CONCURRENCY,
        "fa_local_required_for_parallel": True,
        "fa_local_state": "unavailable",
        "parallel_execution_ready": False,
        "serial_fallback_available": True,
    }

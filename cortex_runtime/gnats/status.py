from __future__ import annotations

from typing import Any

from cortex_runtime.gnats.models import GNAT_DEFAULT_MAX_CONCURRENCY, GNAT_HARD_MAX_CONCURRENCY, FaLocalCapabilityState
from cortex_runtime.source_lanes import gnat_admitted_worker_types


def gnat_status_summary(fa_local_capability_state: FaLocalCapabilityState | None = None) -> dict[str, Any]:
    fa_local_state = (
        fa_local_capability_state.fa_local_state
        if fa_local_capability_state is not None
        else "unavailable"
    )
    fa_local_ready = fa_local_state == "ready"
    return {
        "profile": "bounded_parallel_extraction" if fa_local_ready else "serial_contract_proof",
        "admitted_worker_types": gnat_admitted_worker_types(),
        "max_concurrency": GNAT_DEFAULT_MAX_CONCURRENCY,
        "hard_cap": GNAT_HARD_MAX_CONCURRENCY,
        "fa_local_required_for_parallel": True,
        "fa_local_state": fa_local_state,
        "parallel_execution_ready": fa_local_ready,
        "serial_fallback_available": True,
    }

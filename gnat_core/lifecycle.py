from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class RunStateCounts:
    expected: int
    completed: int
    failed: int = 0
    stale: int = 0
    cancelled: int = 0
    missing: int = 0
    rejected: int = 0


def run_state_from_counts(counts: RunStateCounts) -> str:
    if counts.expected == 0:
        return "failed"
    if (
        counts.completed == counts.expected
        and counts.failed == 0
        and counts.stale == 0
        and counts.cancelled == 0
        and counts.missing == 0
        and counts.rejected == 0
    ):
        return "ready"
    if counts.completed > 0:
        return "partial_success"
    if counts.stale > 0 and counts.failed == counts.cancelled == counts.missing == 0:
        return "stale"
    if counts.cancelled > 0 and counts.failed == counts.stale == counts.missing == 0:
        return "cancelled"
    return "failed"

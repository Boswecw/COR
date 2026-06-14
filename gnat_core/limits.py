from __future__ import annotations


def bounded_concurrency(requested_concurrency: int, max_concurrency: int, *, hard_cap: int) -> tuple[int, int]:
    if requested_concurrency < 1:
        raise ValueError("requested_concurrency must be at least 1")
    if max_concurrency < 1:
        raise ValueError("max_concurrency must be at least 1")
    if hard_cap < 1:
        raise ValueError("hard_cap must be at least 1")
    return min(requested_concurrency, hard_cap), min(max_concurrency, hard_cap)


def effective_concurrency(*limits: int) -> int:
    if not limits:
        raise ValueError("at least one concurrency limit is required")
    if any(limit < 1 for limit in limits):
        raise ValueError("concurrency limits must be positive")
    return min(limits)

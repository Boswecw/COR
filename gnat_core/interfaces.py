from __future__ import annotations

from typing import Any, Protocol


class CancellationToken(Protocol):
    def is_cancelled(self) -> bool:
        ...


class ReceiptEnvelope(Protocol):
    @property
    def run_id(self) -> str:
        ...

    @property
    def shard_id(self) -> str:
        ...

    def to_contract(self) -> dict[str, Any]:
        ...

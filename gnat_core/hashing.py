from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any


def canonical_hash(payload: Any) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":"), default=str).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def stable_short_digest(payload: str, *, length: int = 12) -> str:
    if length < 1:
        raise ValueError("length must be positive")
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:length]


def sha256_bytes_digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def source_path_token(path: str | Path) -> str:
    try:
        resolved = str(Path(path).resolve())
    except OSError:
        resolved = str(path)
    return f"src:{stable_short_digest(resolved)}"

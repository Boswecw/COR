from __future__ import annotations

from collections.abc import Mapping
from typing import Any

from gnat_core.hashing import canonical_hash


def cache_key_from_identity(contract_version: str, identity_contract: Mapping[str, Any]) -> str:
    if not contract_version:
        raise ValueError("contract_version is required")
    if not identity_contract:
        raise ValueError("identity_contract is required")
    return canonical_hash(
        {
            "contract_version": contract_version,
            "cache_identity": dict(identity_contract),
        }
    )

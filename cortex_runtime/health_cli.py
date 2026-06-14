"""Cortex ecosystem-health self-check.

Cortex is a CLI producer (the repo-crawler / Centipede gnat runtime that feeds the evaluation
spine) with no HTTP surface. This gives a downstream consumer (ForgeCommand's `/ecosystem-health`
topology) a real, producer-owned health signal instead of a fabricated status.

Run: `python -m cortex_runtime.health_cli`  ->  prints one JSON object; exit 0 = ok, 1 = degraded.
"""
from __future__ import annotations

import importlib
import json
import sys
from datetime import datetime, timezone


def main() -> int:
    checks: dict[str, str] = {}
    status = "ok"
    for label, module in (
        ("cortex_runtime", "cortex_runtime"),
        ("intake_validation", "cortex_runtime.intake_validation"),
        ("extraction_emission", "cortex_runtime.extraction_emission"),
        ("gnat_core", "gnat_core"),
    ):
        try:
            importlib.import_module(module)
            checks[label] = "ok"
        except Exception as exc:  # noqa: BLE001
            checks[label] = f"error: {type(exc).__name__}"
            status = "degraded"

    try:
        from importlib.metadata import version as _pkg_version

        version = _pkg_version("cortex")
    except Exception:  # noqa: BLE001
        version = "unknown"

    print(
        json.dumps(
            {
                "service": "cortex",
                "status": status,
                "version": version,
                "role": "repo-crawler / Centipede gnat runtime (CLI producer; no HTTP surface)",
                "checks": checks,
                "checked_at": datetime.now(timezone.utc).isoformat(),
            }
        )
    )
    return 0 if status == "ok" else 1


if __name__ == "__main__":
    sys.exit(main())

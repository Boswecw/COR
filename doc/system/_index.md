# Cortex - System Documentation

**Document version:** 1.14 (2026-03-20) - Type-checking hygiene pass
**Protocol:** Forge Documentation Protocol v1

| Key | Value |
|-----|-------|
| **Project** | Cortex |
| **Prefix** | `cx` |
| **Output** | `doc/cxSYSTEM.md` |

This `doc/system/` tree is the assembled system reference for Cortex as a bounded local file-intelligence service.
It reflects the current repo state through Wave 3 hardening, audit-remediation tightening, the shared source-lane framework, Runtime Slices 1 through 9, the post-Slice-7 hardening and lane-admission-governance pass, bounded ODT lane delivery, bounded EPUB lane delivery, the post-Slice-8 governance execution, the post-Slice-9 governance selection, the bounded special-track Scrivener Stage 1 authority-recon runtime slice, the current Scrivener Stage 2 planning-control packet, the explicit Stage 2 implementation-remains-blocked decision, the PDF lane admission hardening and structured probe pass, and the mypy type-checking hygiene pass.

Assembly contract:

- Command: `bash doc/system/BUILD.sh`
- Output: `doc/cxSYSTEM.md`

| Part | File | Contents |
|------|------|----------|
| SS1 | [01-overview-charter.md](01-overview-charter.md) | Mission, role, success posture, and current bounded runtime baseline |
| SS2 | [02-boundaries-and-doctrine.md](02-boundaries-and-doctrine.md) | Authority boundaries, syntax-before-semantics doctrine, and anti-control-plane posture |
| SS3 | [03-contract-surface.md](03-contract-surface.md) | Intake, extraction, retrieval, handoff, service-status, and diagnostics surfaces |
| SS4 | [04-validation-and-delivery.md](04-validation-and-delivery.md) | Validation wiring, schema-backed enforcement, delivered slices, and current delivery posture |

## Quick Assembly

```bash
bash doc/system/BUILD.sh
```

*Last updated: 2026-03-20*

# Cortex - System Documentation

**Document version:** 1.0 (2026-03-19) - Normalized to Forge Documentation Protocol v1
**Protocol:** Forge Documentation Protocol v1

| Key | Value |
|-----|-------|
| **Project** | Cortex |
| **Prefix** | `cx` |
| **Output** | `doc/cxSYSTEM.md` |

This `doc/system/` tree is the assembled system reference for Cortex as a bounded local file-intelligence service.

Assembly contract:

- Command: `bash doc/system/BUILD.sh`
- Output: `doc/cxSYSTEM.md`

| Part | File | Contents |
|------|------|----------|
| SS1 | [01-overview-charter.md](01-overview-charter.md) | Mission, role, success posture, Phase 1 framing |
| SS2 | [02-boundaries-and-doctrine.md](02-boundaries-and-doctrine.md) | Authority boundaries, syntax-before-semantics doctrine, non-goals |
| SS3 | [03-contract-surface.md](03-contract-surface.md) | Intake, extraction, retrieval package, and service-status contracts |
| SS4 | [04-validation-and-delivery.md](04-validation-and-delivery.md) | Validation tooling, fixtures, delivery order, and next hardening steps |

## Quick Assembly

```bash
bash doc/system/BUILD.sh
```

*Last updated: 2026-03-19*

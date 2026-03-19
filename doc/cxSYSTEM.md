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

---

# 1. Overview and Charter

## Purpose

Cortex is the bounded local file-intelligence, extraction, retrieval-preparation, and handoff-support service for Forge applications.

Its Phase 1 purpose is narrow:

- intake eligible local content
- extract syntax-level structure and metadata
- prepare one governed retrieval package form
- surface truthful operational state without privacy collapse

## Constitutional role

Cortex is a service-only internal runtime subsystem.

It must not become:

- a standalone product
- a semantic engine
- a workflow coordinator
- a generic ETL platform
- a canonical truth store

## Success posture

Cortex is only successful if it remains:

- bounded by contract
- fail-closed when trust is insufficient
- privacy-preserving by default
- non-semantic by default
- freshness-bound rather than silently stale
- visible only through consuming applications

## Foundational references

This section is grounded in:

- `PROJECT_CHARTER.md`
- `PHASE_1_PLAN.md`
- `README.md`

---

# 2. Boundaries and Doctrine

## Authority line

Cortex owns:

- intake contracts
- syntax-level extraction
- provenance and completeness signaling
- retrieval-preparation support
- handoff packaging support
- freshness and invalidation signaling for Cortex-owned artifacts
- privacy-preserving diagnostics for Cortex-owned surfaces

Cortex does not own:

- semantic interpretation
- model authority
- workflow sequencing
- downstream execution ownership
- canonical business truth
- broad surveillance authority

## Doctrine line

The governing doctrines are:

- service-only visibility
- syntax before semantics
- fail closed over convenience
- retrieval infrastructure, not retrieval authority
- explicit invalidation over assumed freshness
- default-denied observation

## Cross-service boundaries

### DF Local Foundation

Provides substrate support.
Does not absorb Cortex file-intelligence logic.

### NeuronForge Local

Consumes syntax-level packages for semantic work.
Does not make Cortex a semantic authority.

### FA Local

Owns policy-gated execution routing.
Does not delegate execution authority into Cortex.

## Anti-drift warning

Any proposal that turns Cortex into a semantic surface, workflow router, surveillance surface, or generalized transform sink should be rejected unless the architecture is explicitly reworked.

---

# 3. Contract Surface

## Phase 1 contract set

The current contract surface covers:

- intake request
- extraction result
- retrieval package
- handoff envelope
- service status
- embedded diagnostics

## Intake request

The intake contract requires explicit source identity, source class, artifact request type, normalization mode, and observation posture.

Observation defaults to denied.
If a watcher is requested, it must be contract-scoped, operator-visible, removable, and bounded by source class.

## Extraction result

Extraction results are syntax-only by contract.

They must expose:

- provenance
- completeness posture
- freshness posture when relevant
- refusal posture when requests cross into semantics or other denied boundaries

## Retrieval package

Retrieval packages are:

- non-canonical
- non-semantic by default
- freshness-bound

They require explicit retrieval profile, freshness, invalidation, and completeness fields.

## Service status

Service status exposes truthful operator-visible state with the narrow Phase 1 vocabulary:

- `ready`
- `degraded`
- `unavailable`
- `denied`
- `stale`
- `partial_success`

## Handoff envelope

The handoff envelope is a bounded transfer-truth surface only.

It may express:

- `ready_for_transfer`
- `denied`
- `stale`
- `integrity_failed`
- `re_prep_required`

`reverse_signal` is optional.
When present, it must stay within the bounded reverse-signaling enum and must not become a workflow protocol.

It must reject orchestration-shaped fields such as:

- `retry_count`
- `workflow_id`
- `queue_name`
- `dispatch_plan`
- `agent_assignment`

## Supporting references

This section is grounded in:

- `docs/contracts/intake-request.md`
- `docs/contracts/extraction-result.md`
- `docs/contracts/retrieval-package.md`
- `docs/contracts/handoff-envelope.md`
- `docs/contracts/service-status.md`
- `docs/contracts/embedded-diagnostics.md`

---

# 4. Validation and Delivery

## Validation surface

Cortex now includes:

- JSON schemas in `schemas/`
- valid fixtures in `tests/contracts/fixtures/valid/`
- invalid fixtures in `tests/contracts/fixtures/invalid/`
- a lightweight validator at `scripts/validate_schemas.py`
- repo-level validation through `make validate`
- automatic fixture discovery by schema-prefix naming
- explicit schema-contract checks for handoff reverse signaling, denial taxonomy, anti-orchestration guards, and embedded diagnostics privacy boundaries

## Delivery order

The current delivery order remains:

1. constitutional base docs
2. doctrine and boundary ADRs
3. architecture boundary matrix
4. contracts and schemas
5. fixtures and validation

## Wave 3 hardening delivered

Wave 3 adds:

- the handoff envelope contract and schema
- valid handoff fixtures for basic, stale, and denied paths
- invalid handoff fixtures for missing integrity context, invalid reverse signaling, invalid denial taxonomy, and orchestration creep
- automatic validator wiring so new schema-prefixed fixtures are picked up without manual script edits

## Audit remediation tightening

The current remediation pass adds:

- a strict embedded diagnostics schema with privacy-preserving defaults
- boundary fixtures for service status, retrieval package, and extraction result branches that were previously under-exercised
- handoff alignment so reverse signaling remains optional rather than forced on every forward transfer envelope

## Assembly purpose

`doc/cxSYSTEM.md` is intended to give a single assembled system reference without replacing the canonical source files that define the actual doctrine and contracts.

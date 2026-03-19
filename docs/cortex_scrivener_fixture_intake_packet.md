# Cortex — Scrivener Fixture Intake Packet

## Document control

- Project: Cortex
- Status: Draft for execution
- Scope: Fixture governance only
- Runtime impact: None
- Lane posture: Scrivener special-track Phase 0 support artifact

## Purpose

This packet defines how sanitized Scrivener fixtures must be collected, described, classified, and stored so Cortex can complete Scrivener Phase 0 reconnaissance without drifting into implementation.

This packet does not authorize parser work, runtime behavior, schema changes, or lane admission.

---

# 1. Repository placement recommendation

Use a canonical fixture location and keep test-facing usage secondary.

Recommended canonical path:

- `fixtures/scrivener/`

Recommended supporting test-facing path only if needed later:

- `tests/runtime/fixtures/scrivener/`

Rationale:

- the fixture packet is governance and evidence infrastructure first
- runtime tests are a downstream consumer, not the fixture authority
- this keeps phase-0 evidence cleanly separated from future implementation mechanics

Recommended initial files:

- `fixtures/scrivener/README.md`
- `fixtures/scrivener/fixture-intake-checklist.md`
- `fixtures/scrivener/fixtures-index.md`
- `fixtures/scrivener/provenance-policy.md`

Optional later helper:

- `tests/runtime/fixtures/scrivener/README.md`

That optional test-facing README should point back to the canonical fixture packet rather than becoming a second authority surface.

---

# 2. File 1 — `fixtures/scrivener/README.md`

## Purpose

This file defines what a valid Scrivener fixture is for Cortex Phase 0 and future contract testing.

It should answer:

- what belongs in the fixture set
- what must be preserved
- what must be removed or sanitized
- how fixtures are classified
- how fixtures may be used later

## Recommended content

### Title
`# Scrivener Fixtures`

### Scope statement
State that this directory contains sanitized `.scriv` project fixtures used for governance reconnaissance and later contract-grade validation.

State explicitly that fixtures do not authorize implementation behavior by their presence alone.

### Allowed purpose
Fixtures may be used for:

- structural reconnaissance
- authority proof work
- admission-policy validation
- compatibility characterization
- future contract tests after lane admission

Fixtures may not be used as justification for:

- broadening lane scope by convenience
- inferring support for unobserved structures
- inventing fallback rules
- claiming compatibility beyond observed evidence

### Required preserved artifacts
A valid fixture should preserve enough internal structure to support all of the following where applicable:

- `.scriv` package/container layout
- candidate `*.scrivx` authority file
- binder hierarchy representation
- binder item ordering
- manuscript or draft subtree evidence if present
- binder-node to content-file mapping evidence
- representative content storage artifacts
- representative metadata artifacts if needed for authority proof

### Sanitization requirements
Fixtures must be sanitized before inclusion unless they are synthetic from creation.

Sanitization expectations:

- remove or replace real manuscript prose where possible
- remove private names, emails, addresses, and identifying annotations
- sanitize project titles where needed
- remove non-essential attachments/media
- sanitize notes, comments, labels, keywords, custom metadata, and inspector content if present
- preserve structural relationships needed for proof
- preserve ambiguity if the ambiguity itself is relevant to compatibility or authority analysis

### Fixture classes
Recommended classes:

- `positive/` — structurally valid projects useful for deterministic proof
- `negative/` — malformed or intentionally incomplete examples
- `compat/` — version, migration, or platform-shape compatibility examples
- `unsupported/` — structures intentionally retained to validate denied cases
- `ambiguous/` — fixtures preserved specifically because authority or mapping is not yet clear

### Naming convention
Recommend stable, descriptive fixture names such as:

- `scriv-simple-manuscript-v1`
- `scriv-manuscript-research-nested-v1`
- `scriv-missing-content-negative-v1`
- `scriv-legacy-compat-v1`
- `scriv-authority-ambiguous-v1`

Avoid naming that implies support level beyond observed evidence.

### Per-fixture minimum metadata
Each fixture should have a small adjacent note or index entry capturing:

- fixture id
- class
- source provenance
- platform if known
- Scrivener version if known
- sanitization status
- key structural observations
- intended use
- known limitations

### Future test usage note
State that once Scrivener is admitted, fixtures may be promoted into contract tests for:

- authority detection
- admission decisions
- content-location correctness
- denied/unsupported diagnostics
- malformed-structure fail-closed behavior

But until lane admission, fixtures remain evidence assets only.

---

# 3. File 2 — `fixtures/scrivener/fixture-intake-checklist.md`

## Purpose

This file is the operator checklist for deciding whether a `.scriv` project is acceptable as a Cortex fixture candidate.

It should be strict, explicit, and completion-oriented.

## Recommended structure

### Title
`# Scrivener Fixture Intake Checklist`

### Section A — Fixture identity
- [ ] fixture id assigned
- [ ] source provenance recorded
- [ ] original project title recorded or intentionally replaced
- [ ] platform of origin recorded if known
- [ ] Scrivener version recorded if known
- [ ] date added recorded
- [ ] sanitization operator recorded
- [ ] intended fixture class assigned

### Section B — Package integrity
- [ ] `.scriv` container exists
- [ ] package opens as a directory/package for inspection
- [ ] candidate `*.scrivx` file present or absence intentionally documented
- [ ] top-level package tree recorded
- [ ] structure preserved sufficiently for reconnaissance
- [ ] accidental backup/autosave noise either removed or intentionally classified
- [ ] fixture is not a partial copy unless intentionally included as a negative case

### Section C — Structural authority observability
- [ ] binder hierarchy evidence observable
- [ ] binder ordering evidence observable
- [ ] stable item identity evidence observable
- [ ] manuscript or draft subtree evidence observable if present
- [ ] node-to-content linkage evidence observable or explicitly absent
- [ ] unresolved authority ambiguity documented

### Section D — Content mapping observability
- [ ] representative binder nodes can be traced toward stored content
- [ ] content storage paths recorded where observable
- [ ] missing-content conditions documented if present
- [ ] metadata-only node conditions documented if present
- [ ] duplicate-reference or alias risk noted if suspected
- [ ] storage format observations recorded without overclaiming support

### Section E — Safety and sanitization
- [ ] no real personal or client manuscript content remains beyond minimum structural utility
- [ ] project names sanitized where needed
- [ ] people names and contact information removed
- [ ] comments, notes, synopsis, labels, keywords, custom metadata sanitized where needed
- [ ] attachments/media removed unless intentionally retained for a bounded reason
- [ ] screenshots or exports are not used in place of the real package structure
- [ ] sanitization did not destroy required structural evidence

### Section F — Classification and intended use
- [ ] classified as `positive`, `negative`, `compat`, `unsupported`, or `ambiguous`
- [ ] intended evidence role recorded
- [ ] intended future test role recorded if applicable
- [ ] expected deny reason recorded for negative/unsupported fixtures
- [ ] fixture is not being overused as proof beyond its class

### Section G — Known unresolveds
- [ ] unresolved authority ambiguity listed
- [ ] unresolved mapping ambiguity listed
- [ ] version uncertainty listed
- [ ] platform-origin uncertainty listed
- [ ] fixture-specific limitations listed
- [ ] note added stating fixture alone does not prove general support

### Section H — Intake decision
- [ ] accepted into fixture packet
- [ ] accepted with restrictions
- [ ] rejected
- [ ] follow-up action recorded

### Intake decision rule
Add an explicit rule:

A fixture must not be treated as a positive proof artifact when authority, mapping, or sanitization status is materially unresolved.

---

# 4. File 3 — `fixtures/scrivener/fixtures-index.md`

## Purpose

Create one small index so the fixture set remains auditable.

## Recommended columns

- fixture id
- class
- platform
- version
- sanitization status
- key authority value
- key mapping value
- major unresolveds
- intended use
- status

This should remain compact and factual.

---

# 5. File 4 — `fixtures/scrivener/provenance-policy.md`

## Purpose

State the provenance and handling rules for all Scrivener fixtures.

## Recommended content

### Acceptable fixture sources
- synthetic projects created specifically for Cortex
- sanitized personal test projects
- sanitized sample projects with lawful reuse rights
- internal controlled fixtures created to exercise known edge cases

### Discouraged or denied sources
- unsanitized live manuscripts
- client manuscripts
- any project with unresolved rights posture
- fixtures copied from the internet without explicit reuse clarity
- fixtures whose sanitization destroys confidence in structural truth without documenting that loss

### Required provenance record
Each fixture must record:

- who created or supplied it
- whether it is synthetic or derived
- what sanitization was applied
- whether any structural artifacts were intentionally removed
- whether the fixture is suitable for future automated tests or governance only

### Privacy rule
No fixture may be added if the repository would become a storage surface for unnecessary manuscript content or personal data.

---

# 6. Optional test-facing wrapper

If you want a runtime-facing wrapper later, keep it tiny.

Recommended `tests/runtime/fixtures/scrivener/README.md` content:

- these tests consume canonical fixtures from `fixtures/scrivener/`
- the canonical authority for fixture classification and provenance is the packet under `fixtures/scrivener/`
- this location exists only for runtime-test organization once the lane is admitted

That avoids duplicating governance text.

---

# 7. Suggested Codex handoff task

Use a bounded instruction set such as:

Create the canonical Scrivener fixture intake packet for Cortex under `fixtures/scrivener/`.

Add:
- `README.md`
- `fixture-intake-checklist.md`
- `fixtures-index.md`
- `provenance-policy.md`

Constraints:
- governance artifacts only
- no runtime code
- no schemas
- no parser modules
- no tests
- no lane-admission claims
- keep wording explicit and fail-closed
- preserve separation between evidence assets and future runtime consumers

---

# 8. Recommended immediate move

The best next bounded repository step is:

1. create `fixtures/scrivener/README.md`
2. create `fixtures/scrivener/fixture-intake-checklist.md`
3. create `fixtures/scrivener/fixtures-index.md`
4. create `fixtures/scrivener/provenance-policy.md`
5. optionally add a tiny `tests/runtime/fixtures/scrivener/README.md` later only if needed

That gives Cortex a real intake surface for sanitized `.scriv` fixtures without faking implementation readiness.


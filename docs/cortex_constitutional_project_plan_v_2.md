# Cortex — Constitutional Project Plan v2.1
## Senior-reviewed constitutional hardening update

**Date:** 2026-03-18  
**Status:** Draft v2.1  
**Basis:** v2 approved with targeted constitutional hardening to improve falsifiability, drift resistance, and Phase 1 discipline.

---

# 1. Executive Judgment

Cortex remains worth building **only** as a bounded constitutional local service project.

v2 was already strong and approval-worthy. v2.1 does **not** change the architectural direction. It tightens the few places where good doctrine still needed more explicit implementation pressure so the system holds under real delivery conditions.

This revision strengthens five areas:

- observation defaults
- freshness and invalidation discipline
- reverse signaling minimalism
- falsifiable acceptance criteria
- anti-ETL boundary protection

The goal is not to make Cortex broader.
The goal is to make it **harder to mis-implement**.

---

# 2. Project Decision

## Recommendation

Approve Cortex under this v2.1 constitutional framing.

## Approval condition

Implementation, contracts, and repo structure must follow the boundedness rules in this document rather than treating them as advisory prose.

## Why

Cortex is likely to become a shared local leverage surface across Forge applications.
Anything in that position must be:

- bounded
- fail-closed
- privacy-preserving
- non-semantic
- non-canonical
- explicitly non-orchestrating

---

# 3. Project Charter

## Project name

**Cortex**

## Constitutional identity

Cortex is the **bounded local file intelligence, intake, extraction, indexing-preparation, retrieval-preparation, and packaging handoff service** for Forge applications.

It is a **service-only internal runtime subsystem**.

It is not:

- a destination product
- an app-owned truth authority
- an inference engine
- a general orchestration layer
- a generic local ETL platform

## Charter statement

Cortex exists to provide trustworthy, privacy-preserving, bounded local content-intelligence services for Forge applications under strict authority limits, fail-closed behavior, and service-only visibility.

Its purpose is to improve local application usefulness, context assembly, and operational discipline without drifting into semantic authority, workflow ownership, execution hosting, generic transformation plumbing, or application identity.

## Core mandate

Cortex may define and implement only the local capabilities necessary to:

- intake eligible local content inputs
- extract bounded structured content and metadata
- prepare retrieval-oriented artifacts and packages
- provide packaging/export handoff support
- surface truthful readiness, degraded, stale, denied, partial-success, and unavailable state for those functions
- expose only bounded embedded operational visibility through consuming applications

---

# 4. Hard Non-Goals

This section is constitutional.

Cortex is **not**:

- a standalone end-user application
- a separate product identity
- a parallel workspace UI
- a hidden authority surface
- a general local execution host
- a local clone of Rake
- a broad autonomous orchestration layer
- a semantic interpretation service
- a canonical business truth store
- a replacement for app-owned retrieval policy
- a stealth monolith for everything involving files, memory, search, or context
- a generic local ETL platform

## Anti-drift doctrine

If a proposed feature would make Cortex more like:

- a research workspace
- an agent platform
- a semantic labeling engine
- a general workflow coordinator
- a canonical memory system
- a generic transform-and-sink pipeline

then the burden of proof is on that proposal, and the default answer is **no** unless constitutional necessity is demonstrated.

---

# 5. Constitutional Principles

## 5.1 Service-only visibility

Cortex must remain visible only through consuming application HUDs, diagnostics, and bounded embedded control surfaces.

It must never become a separate destination UI or independent application workflow surface.

## 5.2 Syntax before semantics

Cortex extracts and shapes structure.
It does not decide meaning.

## 5.3 Retrieval infrastructure, not retrieval authority

Cortex may own local retrieval-oriented artifacts needed for its function, but possession of those artifacts does not make Cortex the authority over truth, policy, or interpretation.

## 5.4 Fail-closed over convenience

If eligibility, extraction completeness, freshness, integrity, or handoff confidence are not sufficient, Cortex must narrow or deny rather than pretending readiness.

## 5.5 Privacy-preserving operational truth

Operational visibility is required.
Content surveillance is forbidden by default.

## 5.6 App sovereignty remains intact

Applications own business meaning, product decisions, workflow policy, and canonical truth.
Cortex may assist them.
It may not absorb them.

## 5.7 Local usefulness under bounded compute

Cortex must remain useful on realistic local hardware.
Background behavior, indexing cost, extraction pipelines, and persistence choices must remain disciplined accordingly.

## 5.8 Explicit invalidation over assumed freshness

No Cortex artifact may be treated as indefinitely fresh by omission.
Every governed retrieval-oriented artifact must carry one or more of:

- TTL
- invalidation trigger contract
- source-change dependency marker
- explicit stale state

---

# 6. Authority Boundaries

## 6.1 Cortex owns

Cortex owns:

- local file and content intake contracts
- intake normalization for eligible sources
- extraction of bounded structured content and metadata
- extraction provenance and bounded completeness signaling
- retrieval-preparation artifacts and packaging support
- handoff envelope support and validation
- service readiness, degraded-state, stale-state, denied-state, partial-success, and unavailable-state truth
- privacy-preserving operational diagnostics for its own service surfaces

## 6.2 Cortex does not own

Cortex does not own:

- canonical application truth
- semantic interpretation
- thematic labeling or inferential classification
- local model lifecycle management
- model weights or inference parameter authority
- general local task execution routing
- autonomous orchestration
- downstream workflow sequencing
- app-level retrieval policy decisions
- cross-service authority arbitration
- generic extract-transform-load responsibilities outside bounded service contracts

---

# 7. Boundary Relations With Other Local Services

## DF Local Foundation

DF Local provides substrate mechanics:

- registration
- storage substrate support
- migrations
- readiness and service lifecycle support

Cortex may depend on DF Local by contract.
DF Local must not absorb Cortex file-intelligence logic.
Cortex must not push app-specific retrieval semantics into DF Local.

## NeuronForge Local

NeuronForge Local owns:

- local inference execution
- semantic interpretation
- candidate production
- model-backed meaning generation

Cortex may provide structured extraction outputs, retrieval-ready content, and bounded context packages to NeuronForge Local.
Cortex must not perform NeuronForge Local work.

## FA Local

FA Local owns governed execution routing and policy-gated task execution.

Cortex may expose bounded callable tools or contracts.
Cortex must not become the local execution host.

## Forge Local Runtime

Forge Local Runtime owns the constitutional runtime doctrine above Cortex.

Cortex is governed by that runtime doctrine, but is not itself the runtime authority layer.

---

# 8. Senior Risks and Required Corrections

## Risk 1 — index persistence becomes implied semantic authority

### Failure mode
Cortex stores enough retrieval artifacts that teams begin treating it as the memory authority.

### Correction
Every index artifact must be explicitly classified as:

- service infrastructure
- freshness-bound
- non-canonical
- non-semantic by default

No consuming application may treat a Cortex retrieval artifact as source-of-truth without an explicit bridging contract owned by that application.

## Risk 2 — retrieval-prep becomes hidden retrieval-policy control

### Failure mode
Chunking, filtering, ranking assumptions, or shaping defaults harden into silent policy.

### Correction
Chunking and retrieval-shaping must be contract parameters or governed profiles, not hidden doctrine.

## Risk 3 — extraction starts doing semantic work

### Failure mode
“Helpful” metadata enrichment starts becoming classification, summarization, or interpretation.

### Correction
Extraction outputs must be schema-validated against syntax-only boundaries.
Semantic fields require explicit refusal unless delegated to NeuronForge Local.

## Risk 4 — packaging handoff becomes orchestration authority

### Failure mode
Once Cortex hands content off, it starts coordinating downstream execution and retry logic.

### Correction
Cortex may validate and package handoff envelopes, but downstream workflow control remains outside Cortex.
Reverse signaling must be explicit, minimal, and bounded rather than ad hoc.

## Risk 5 — file observation becomes always-on background expansion

### Failure mode
Convenience leads to perpetual watchers, broad scanning, or local surveillance patterns.

### Correction
Observation is **default-denied**.
No watcher may exist unless a consuming application contract explicitly creates it, scopes it, and makes it operator-visible.
Observation must be:

- contract-scoped
- opt-in at the app level
- bounded by source class
- visible in diagnostics/status surfaces
- removable without service corruption

## Risk 6 — diagnostics become privacy collapse

### Failure mode
To help debugging, raw content and broad previews bleed into Cortex diagnostics.

### Correction
Operational truth stays; broad raw-content visibility does not.
Diagnostics must prohibit by default:

- raw content preview panes
- full-text free browse surfaces
- ad hoc content inspection without explicit higher-order app authority

---

# 9. Required Project Documents

## Founding document set

1. **PROJECT_CHARTER.md**
   - mission
   - constitutional role
   - scope
   - non-goals
   - success criteria

2. **LOCAL_DOCTRINE.md**
   - service-only rule
   - syntax vs semantics rule
   - privacy-preserving visibility
   - truthful degradation
   - anti-Rake-clone doctrine
   - anti-agent-host doctrine
   - explicit invalidation doctrine

3. **AUTHORITY_BOUNDARIES.md**
   - Cortex owns
   - Cortex does not own
   - boundaries vs DF Local
   - boundaries vs NeuronForge Local
   - boundaries vs FA Local
   - boundaries vs app truth

4. **DEGRADATION_MODEL.md**
   - healthy
   - degraded
   - unavailable
   - denied
   - stale
   - partial-success
   - reason taxonomy

5. **CONTROL_SURFACE.md**
   - HUD surface rules
   - embedded diagnostics rules
   - bounded controls
   - privacy-preserving defaults

6. **THREAT_AND_DRIFT_MODEL.md**
   - authority drift
   - privacy collapse
   - semantic creep
   - orchestration creep
   - uncontrolled observation
   - ETL creep

7. **SERVICE_DOMAINS.md**
   - intake
   - extraction
   - indexing prep
   - retrieval prep
   - handoff
   - status/diagnostics

8. **PHASE_1_PLAN.md**
   - narrow first implementation slice
   - explicit out-of-scope list
   - contract-first delivery order
   - exit criteria

9. **DECISIONS/**
   - 0001 service-only framing
   - 0002 syntax-not-semantics
   - 0003 retrieval-infrastructure-not-authority
   - 0004 no-agent-host drift
   - 0005 bounded-reverse-signaling
   - 0006 explicit-invalidation-over-assumed-freshness

10. **docs/architecture/boundary-matrix.md**
   - compact role map
   - owns / does not own matrix
   - cross-service boundary reference

---

# 10. Revised Repository Structure

```text
cortex/
  README.md
  PROJECT_CHARTER.md
  LOCAL_DOCTRINE.md
  AUTHORITY_BOUNDARIES.md
  SERVICE_DOMAINS.md
  CONTROL_SURFACE.md
  DEGRADATION_MODEL.md
  THREAT_AND_DRIFT_MODEL.md
  PHASE_1_PLAN.md
  SYSTEM.md
  ARCHITECTURE.md
  ROADMAP.md
  DECISIONS/
    0001-service-only-framing.md
    0002-syntax-not-semantics.md
    0003-retrieval-infrastructure-not-authority.md
    0004-no-agent-host-drift.md
    0005-bounded-reverse-signaling.md
    0006-explicit-invalidation-over-assumed-freshness.md
  docs/
    doctrine/
      service-only-visibility-rule.md
      truthful-degraded-states.md
      privacy-preserving-observability.md
      extraction-syntax-vs-semantics.md
      bounded-observation-rule.md
      explicit-invalidation-rule.md
    architecture/
      service-map.md
      intake-pipeline.md
      extraction-pipeline.md
      retrieval-prep-model.md
      handoff-boundaries.md
      boundary-matrix.md
      dependency-map.md
    contracts/
      intake-request.md
      extraction-result.md
      retrieval-package.md
      handoff-envelope.md
      service-status.md
      embedded-diagnostics.md
    control/
      degraded-reason-taxonomy.md
      freshness-and-invalidation.md
      reverse-handoff-signaling.md
      embedded-diagnostics-rules.md
  schemas/
    intake-request.schema.json
    extraction-result.schema.json
    retrieval-package.schema.json
    handoff-envelope.schema.json
    service-status.schema.json
  src/
    intake/
    extraction/
    indexing/
    retrieval_prep/
    handoff/
    status/
    shared/
  tests/
    contracts/
    intake/
    extraction/
    degraded/
    handoff/
    privacy/
    security/
    freshness/
    boundary_stress/
```

---

# 11. Revised Service Domains

## Domain A — Intake

Purpose:
- admit eligible local content into Cortex under bounded contracts

May do:
- source eligibility checks
- normalization
- contract-scoped file observation
- bounded intake status reporting

May not do:
- app meaning decisions
- broad sync assumptions
- uncontrolled background expansion

## Domain B — Extraction

Purpose:
- produce syntax-level structured content and metadata

May do:
- text extraction
- structural normalization
- metadata harvest
- bounded provenance reporting
- incomplete/failed extraction signaling

May not do:
- summarization
- classification
- thematic labeling
- sentiment or implication judgments

## Domain C — Indexing Preparation

Purpose:
- prepare retrievable artifacts without becoming policy authority

May do:
- shaping of extraction outputs for retrieval infrastructure
- bounded index artifact generation
- freshness marking
- invalidation signal attachment

May not do:
- define final retrieval policy
- act as semantic memory authority
- imply permanent freshness

## Domain D — Retrieval Preparation

Purpose:
- assemble retrieval-oriented packages for app or NeuronForge Local consumption

May do:
- package structure
- chunk/profile application by governed contract
- bounded readiness signaling

May not do:
- ungoverned ranking doctrine
- hidden policy hardcoding

## Domain E — Handoff

Purpose:
- validate and package bounded downstream transfer

May do:
- envelope validation
- integrity/status annotation
- explicit denial or re-prep request surfaces
- minimal bounded reverse signaling

May not do:
- downstream workflow orchestration
- execution ownership after transfer
- retry coordination outside contract

## Domain F — Operational Truth

Purpose:
- surface truthful service status without privacy collapse

May do:
- health summaries
- degraded/unavailable/denied/stale reporting
- forensic events for major service failures
- bounded freshness indicators

May not do:
- content surveillance
- free-form content browsing in diagnostics
- raw preview surfaces by default

---

# 12. Revised Contract Set

## Contract 1 — Intake Request
Defines:
- source type
- eligibility
- preconditions
- observation policy if any
- normalization mode
- denial reasons
- result status taxonomy

## Contract 2 — Extraction Result
Defines:
- extracted structures
- provenance metadata
- completeness flags
- syntax-only guarantees
- refusal conditions for semantic requests

## Contract 3 — Retrieval Package
Defines:
- selected content envelope
- retrieval-shaping profile
- chunking/profile inputs
- freshness markers
- invalidation markers
- confidence/completeness indicators
- privacy/classification fields if needed

## Contract 4 — Handoff Envelope
Defines:
- destination surface
- integrity expectations
- readiness fields
- denial semantics
- bounded reverse signaling path

### Minimal reverse signaling enum for Phase 1
- `accepted`
- `rejected_reason_code`
- `re_prep_required`
- `stale`
- `integrity_failed`

## Contract 5 — Service Status
Defines:
- healthy
- degraded
- unavailable
- denied
- stale
- partial-success
- reason taxonomy

## Contract 6 — Embedded Diagnostics Contract
Defines:
- what consuming apps may expose
- what raw content is prohibited by default
- which operator controls are allowed
- audit/event surfaces allowed vs forbidden

---

# 13. Degradation Doctrine

## Healthy
All required service components available and outputs within contract.

## Degraded
Service can still operate, but with bounded reduction in fidelity, completeness, freshness, or breadth.

## Unavailable
Service cannot satisfy the requested contract.

## Denied
Request rejected due to contract, eligibility, privacy, integrity, or policy boundaries.

## Stale
Artifacts exist but freshness or validity cannot be asserted.

## Partial success
Some extraction or shaping succeeded, but the result is incomplete and must carry explicit incompleteness markers.

---

# 14. Narrow Phase 1 Scope

## Phase 1 objective

Prove Cortex as a trustworthy bounded local intake/extraction/retrieval-prep substrate.

## Include in Phase 1

- project charter
- local doctrine
- authority boundaries
- intake request contract
- extraction result contract
- service status contract
- degradation model
- syntax-only extraction guardrails
- one bounded retrieval-prep package contract
- one embedded diagnostics contract
- boundary matrix
- first decision records
- explicit invalidation rule
- minimal reverse signaling enum

## Explicitly exclude from Phase 1

- broad source connector ecosystems
- always-on observation by default
- semantic enrichment
- app-specific retrieval policy engines
- broad packaging/export automation
- downstream orchestration logic
- standalone UI identity
- generic ETL sink behavior

## Phase 1 exit condition

Cortex can truthfully intake eligible local content, extract syntax-level structure, produce one governed retrieval-prep package form, attach freshness/invalidation rules, and surface bounded operational truth through consuming applications without crossing into semantic or workflow authority.

---

# 15. Later Phases

## Phase 2 — Retrieval-prep hardening
- multiple governed retrieval profiles
- freshness invalidation rules expanded
- bounded index lifecycle handling
- deeper contract tests around non-authority posture

## Phase 3 — Handoff discipline
- handoff envelope hardening
- richer but still bounded reverse signaling
- denial and retry semantics
- consumer integration boundaries

## Phase 4 — Embedded diagnostics
- app HUD contract
- bounded review panes
- privacy-preserving operator controls
- operational event model

## Phase 5 — Anti-drift audit
- semantic creep review
- execution-host creep review
- privacy review
- observation sprawl review
- ETL creep review
- authority-boundary revalidation

---

# 16. Acceptance Criteria

Cortex is not architecturally valid unless all of the following are true:

1. No repo structure, runtime surface, or roadmap artifact gives Cortex a standalone product identity.
2. No default contract, schema, or extraction output exposes semantic interpretation capability.
3. Cortex owns no model lifecycle, weight, or inference-parameter authority.
4. No contract or runtime path allows Cortex to behave as a general local executor.
5. No persistence or contract surface defines Cortex artifacts as canonical business truth.
6. Every retrieval artifact is explicitly marked non-canonical, freshness-bound, and non-semantic by default.
7. Any consuming application that promotes a Cortex artifact to source-of-truth does so only through an app-owned bridging contract.
8. Diagnostics contracts and privacy tests prohibit raw-content surveillance by default.
9. Service-status contracts expose degraded, stale, denied, unavailable, and partial-success states as explicit operator-visible conditions.
10. Intake, extraction, retrieval-prep, and handoff contracts can explicitly refuse requests that cross syntax/semantics or service/workflow boundaries.
11. No watcher can exist without an app-scoped enabling contract, explicit source bounds, and operator-visible status surface.
12. Phase 1 exits only after contract, privacy, freshness, and boundary tests pass with no open anti-drift exceptions.

## Acceptance evidence rule

No acceptance criterion counts as satisfied by prose alone.
Each criterion must map to one or more named verifying artifacts, such as:

- a decision record
- a contract or schema
- an automated contract, privacy, freshness, or boundary test
- an operator-visible status or diagnostics surface
- an app-owned bridging contract where source-of-truth promotion is claimed

---

# 17. Immediate Next Deliverables

Create these first, in bounded order:

## Wave 1 — Constitutional base

1. **PROJECT_CHARTER.md**
2. **LOCAL_DOCTRINE.md**
3. **AUTHORITY_BOUNDARIES.md**
4. **SERVICE_DOMAINS.md**
5. **CONTROL_SURFACE.md**
6. **DEGRADATION_MODEL.md**
7. **THREAT_AND_DRIFT_MODEL.md**
8. **PHASE_1_PLAN.md**
9. **docs/architecture/boundary-matrix.md**
10. **docs/doctrine/explicit-invalidation-rule.md**
11. **DECISIONS/0001-service-only-framing.md**
12. **DECISIONS/0002-syntax-not-semantics.md**
13. **DECISIONS/0003-retrieval-infrastructure-not-authority.md**
14. **DECISIONS/0004-no-agent-host-drift.md**
15. **DECISIONS/0005-bounded-reverse-signaling.md**
16. **DECISIONS/0006-explicit-invalidation-over-assumed-freshness.md**

## Wave 2 — Phase 1 contract surface

1. **docs/contracts/intake-request.md**
2. **docs/contracts/extraction-result.md**
3. **docs/contracts/retrieval-package.md**
4. **docs/contracts/service-status.md**
5. **docs/contracts/embedded-diagnostics.md**
6. **docs/control/freshness-and-invalidation.md**
7. **docs/control/reverse-handoff-signaling.md**
8. **docs/control/embedded-diagnostics-rules.md**
9. **schemas/intake-request.schema.json**
10. **schemas/extraction-result.schema.json**
11. **schemas/retrieval-package.schema.json**
12. **schemas/service-status.schema.json**

---

# 18. Final Working Definition

**Cortex is the bounded service-only local content-intelligence subsystem for Forge applications. It owns intake, syntax-level extraction, retrieval-oriented preparation, handoff packaging support, freshness/invalidation signaling, and truthful operational status for those surfaces. It must remain visible only through consuming applications and must not drift into semantic authority, execution hosting, canonical business truth, generic ETL behavior, or independent application identity.**

---

# 19. Final Recommendation

Proceed with Cortex as a formal project under this v2.1 constitutional framing:

- hard non-goals
- explicit authority boundaries
- syntax-not-semantics enforcement
- retrieval-infrastructure-not-authority doctrine
- privacy-preserving diagnostics
- truthful degradation
- explicit invalidation over assumed freshness
- default-denied observation
- minimal bounded reverse signaling
- narrow first-slice implementation
- recurring anti-drift review

That is the version of Cortex most likely to remain powerful without becoming dangerous.

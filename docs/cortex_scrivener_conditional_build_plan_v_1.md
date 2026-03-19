# Cortex — Scrivener Conditional Build Plan v1

## Purpose

This document converts the current Scrivener evidence base into a **build-ready but not self-authorizing** implementation plan.

It does **not** admit the Scrivener lane.
It does **not** authorize runtime implementation by momentum.
It does **not** weaken the current blocked gate.

It exists so that, if governance later authorizes implementation, the work can begin from a bounded, fail-closed, evidence-shaped plan rather than from open-ended exploration.

---

## Current position

Current evidence is now strong enough to support a **conditional build plan** because the fixture set covers:

- multiple manuscript-maturity states within one real author workflow
- readable sanitized authority surfaces
- a same-source sanitized malformed-authority negative control
- repeated observation of folder-role surfaces and UUID-to-data-path correspondence

Current evidence is **not** strong enough to support admission or unrestricted implementation.

Therefore the correct posture is:

- **plan now**
- **do not auto-implement**
- **keep the lane blocked until explicit authorization**

---

## What this evidence is good enough to support

The current evidence is good enough to support a narrow first implementation scope shaped around these assumptions:

1. `*.scrivx` is treated as the **primary structural authority candidate**.
2. Authority must be **readable and well-formed** or the lane fails closed.
3. Cortex may observe only **bounded project structure and syntax surfaces**.
4. Cortex must not infer meaning, editorial intent, or workflow semantics.
5. Manuscript extraction must remain **policy-gated**, not convenience-driven.

---

## What remains explicitly unproven

The following are still unproven and must remain unclaimed in any future implementation:

- `*.scrivx` sufficiency by itself across all valid projects
- deterministic binder-to-content mapping across all item types
- safe general manuscript inclusion/exclusion rules
- compatibility breadth across version families and migrated project shapes
- support for irregular but not fully broken package states

These unproven areas must be encoded as **implementation constraints**, not hand-waved as future polish.

---

## Recommended implementation posture

If governance authorizes implementation later, the first implementation should be a **bounded authority-recon lane**, not a full extraction lane.

That means the first runtime scope should answer only:

- is there exactly one resolvable authoritative `*.scrivx` candidate?
- is it readable and well-formed?
- can core folder-role surfaces be observed?
- can binder item identifiers be observed?
- can candidate content paths be observed?
- can the runtime truthfully say "available", "unavailable", or "ambiguous" without overclaiming?

The first implementation should **not** attempt full manuscript export, research export, semantic classification, or cross-item normalization.

---

## Proposed staged build model

### Stage 1 — Authority Recon Only

Goal:
Produce truthful structural status about a `.scriv` package without claiming extraction success.

Allowed outcomes:

- readable authority present
- malformed authority
- missing authority
- ambiguous authority
- authority readable but mapping unresolved

Inputs observed:

- top-level `*.scrivx`
- bounded package presence
- binder item UUID surfaces
- known folder-role surfaces when directly observable

Outputs:

- structural authority status
- observed role surfaces
- counts and identifiers only where safe
- fail-closed reason codes

Not allowed:

- content extraction claims
- manuscript-body emission
- research-body emission
- semantic interpretation

### Stage 2 — Candidate Mapping Recon

Goal:
Test whether observed binder identifiers can be matched to candidate `Files/Data/<UUID>/...` surfaces in a deterministic, bounded way.

Allowed outcomes:

- candidate mapping observed
- mapping incomplete
- mapping ambiguous
- mapping unavailable

Outputs:

- mapping status only
- observed correspondences only
- no manuscript inclusion decisions

Not allowed:

- assumptions that all binder items are text content
- assumptions that all resolvable paths are admissible
- assumptions that mapping equals extraction permission

### Stage 3 — Policy-Gated Manuscript Surface Proposal

Goal:
Only after Stage 1 and Stage 2 are stable, define a manuscript-admission policy proposal for what would count as eligible text nodes.

This remains a governance artifact until separately approved.

Questions to answer:

- which folder roots are eligible?
- what item types are allowed?
- what exclusions are mandatory?
- what unresolved ambiguity must cause denial?

Not allowed:

- automatic rollout into runtime extraction
- convenience-based inclusion

### Stage 4 — Narrow Extraction Prototype

Goal:
If separately approved, emit syntax-only extraction from a tightly limited manuscript-admissible subset.

Conditions:

- fail closed on unresolved authority
- fail closed on ambiguous mapping
- fail closed on policy uncertainty
- no research/workspace extraction unless separately authorized

This stage is **not currently authorized**.

---

## Required fail-closed denial cases

Any future implementation must deny or mark unavailable when any of these conditions are present:

- no resolvable `*.scrivx`
- multiple conflicting `*.scrivx` candidates
- malformed or unreadable `*.scrivx`
- missing required structural surfaces
- binder observations with unresolved candidate path correspondence
- policy-ambiguous manuscript eligibility
- unexpected package states outside currently authorized evidence bounds

---

## Proposed contract shape for a future first runtime slice

A future first runtime slice should expose a contract shaped more like this:

- intake accepted or denied
- authority status
- package status
- observed structural roles
- observed mapping status
- reasons for fail-closed outcomes

It should **not** expose a contract shaped like:

- here is the manuscript text
- here are the research notes
- here is a normalized project model

The contract should be status-first, not content-first.

---

## Test strategy for a future authorized implementation

When implementation is eventually authorized, tests should be split by posture:

### Positive structural tests

Use readable sanitized fixtures to prove:

- resolvable authority
- readable XML
- role surfaces observable
- candidate mapping surfaces observable

### Negative structural tests

Use sanitized-corrupted negative fixtures to prove:

- malformed authority is denied
- package resemblance alone is insufficient
- fail-closed reason codes are truthful

### Ambiguity tests

Add later when fixtures exist for:

- unresolved mapping
- partial-package states
- manuscript/research ambiguity
- multi-authority ambiguity

No implementation should be considered mature without all three categories.

---

## What we can responsibly build now

Right now, based on current evidence, the maximum responsible build target is:

**a Stage 1 authority-recon implementation plan and contract draft**

That means we can responsibly build:

- runtime scope definition
- denial taxonomy
- status contract proposal
- test plan outline
- implementation sequence

What we should **not** build yet unless governance intentionally changes:

- manuscript extraction runtime
- generalized Scrivener parser claims
- lane admission artifacts by implication

---

## Supporting conditional artifacts

This plan is now intended to pair with:

- `docs/contracts/scrivener-authority-recon-status-draft.md`
- `docs/source-lanes/scrivener/authority-recon-slice-plan.md`

Those artifacts are build-ready inputs for a future explicitly authorized Stage 1 slice.

They do not clear the gate and they do not authorize implementation by themselves.

---

## Decision line

Use this line going forward:

> Current Scrivener evidence is sufficient to support a bounded conditional build plan and a future authority-recon implementation proposal, but not sufficient to justify lane admission or unrestricted extraction implementation.

---

## Recommended next repo artifacts

If you want to move forward without drifting, the next artifacts should be created in this order:

1. **Scrivener implementation-threshold memo**
2. **Scrivener Stage 1 authority-recon contract draft**
3. **Scrivener denial taxonomy draft**
4. **Scrivener Stage 1 test plan draft**
5. only then, if explicitly approved, a runtime build prompt

---

## Final posture

Yes, Cortex can likely parse `.scriv` files in a bounded way.

No, the current evidence should not yet be treated as blanket authorization to implement the full lane.

The disciplined move is to build the **first authorized thing**, not the biggest possible thing.

That first thing should be a **Stage 1 authority-recon lane**, because that is the largest implementation scope the current evidence honestly supports.

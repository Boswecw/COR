# Worm Boundary Controls

Date and Time: 2026-04-21 11:05 PM America/New_York

## Purpose

This document defines the policy boundary for Worm traversal.

Worm is allowed to expand beyond a source repo only under governed rules.
It must remain deterministic, bounded, and fail-closed.

## Required controls

### 1. Depth
Worm must have a configured maximum traversal depth.

### 2. Breadth
Worm must have a configured maximum number of discovered targets it will expand per source repo.

### 3. Deterministic ordering
Traversal order must be stable between runs given the same inputs.

### 4. Cycle detection
Worm must detect and block repo graph cycles.

### 5. Allowlist and denylist
Worm must support explicit allow and deny controls for repo identities and host identities.

### 6. Scope discipline
Worm must know whether it is operating in:
- local repo only mode
- same organization cross-repo mode
- governed external reference mode

### 7. External host policy
Worm must not follow arbitrary unknown hosts.
Any external host expansion must be explicitly allowed.

### 8. Fallback posture
When Worm cannot safely determine whether to expand, it must fail closed with an explicit fallback posture.

## Policy modes

### Local repo only
No expansion beyond the current repo.

### Same organization governed
Expansion allowed only to approved repos in the same organization.

### Governed external reference
Expansion to explicitly allowed external references only.

## Required downstream observability

Traversal decisions must be explainable later.
For every blocked or allowed expansion, the system should be able to say:

- what source triggered the decision
- what target was considered
- what policy mode applied
- what rule allowed or blocked it
- whether the decision was final or ambiguous

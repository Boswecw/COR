# Extraction to a Shared Gnat Core

## Decision

Do not build one generic do-everything swarm. Build application-specific implementations around a small shared kernel extracted only after the COR design is proven.

## Why not copy everything

Full repository copies create drift in:

- cancellation semantics;
- receipt validation;
- resource limits;
- security fixes;
- cache compatibility;
- lifecycle states.

## Why not one universal engine

A universal parser/swarm repo would accumulate:

- unrelated domain dependencies;
- broad permissions;
- feature flags;
- hard-to-test cross-domain behavior;
- larger public-app attack surface.

## Extraction threshold

Extract a shared core only when:

1. COR Gnats have a stable production proof.
2. A second app, likely AuthorForge, needs the same lifecycle mechanics.
3. At least two implementations reveal genuinely identical interfaces.
4. The shared code can remain domain-neutral and authority-neutral.

## Candidate shared kernel

```text
gnat-core
├── contract models/interfaces
├── immutable plan hashing
├── shard identity
├── lifecycle state machine
├── cancellation token interface
├── receipt envelope interface
├── deterministic reconciliation helpers
├── concurrency-limit calculation
└── cache-key construction
```

Do not include:

- COR source-lane parsers;
- AuthorForge manuscript rules;
- app UI;
- NeuronForge prompts;
- DF-Local concrete storage;
- FA-Local concrete process runner;
- business policy.

## Application repos

```text
COR
└── code/document structural Gnats

AuthorForge
└── manuscript Gnats
    ├── word statistics
    ├── sentence structure
    ├── scene boundaries
    ├── entity observations
    ├── continuity evidence
    └── style observations
```

## AuthorForge example

```text
Manuscript selected
→ AuthorForge constructs app-owned analysis request
→ local coordinator creates chapter/scene shards
→ FA-Local executes manuscript Gnats
→ DF-Local stores receipts
→ optional NeuronForge interprets evidence
→ AuthorForge displays suggestions
```

AuthorForge owns writing meaning and user-facing decisions. Shared Gnat machinery does not become manuscript authority.

## Distribution recommendation

Prefer a versioned local package/library consumed by separate repos rather than git copy-paste. Keep release notes and compatibility tests for each consuming app.

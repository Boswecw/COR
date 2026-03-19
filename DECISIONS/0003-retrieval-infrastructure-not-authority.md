# ADR 0003 - Retrieval infrastructure not authority

## Status

Accepted

## Context

Retrieval-preparation artifacts can be mistaken for memory or truth authority if their role is not named explicitly.

## Decision

Cortex may generate retrieval-oriented artifacts and governed retrieval packages.
Those artifacts are infrastructure support only and are non-canonical, freshness-bound, and non-semantic by default.

## Consequences

- consuming applications retain retrieval-policy authority
- truth promotion requires an app-owned bridging contract
- retrieval artifacts must carry freshness and invalidation posture

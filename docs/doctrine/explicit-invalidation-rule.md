# Explicit Invalidation Rule

## Purpose

This document defines the freshness minimum for Cortex artifacts.

## Rule

No Cortex-governed artifact may be treated as indefinitely fresh by omission.

Every governed artifact must carry one or more of:

- a TTL
- an invalidation trigger contract
- a source-change dependency marker
- an explicit stale state

## Why

Retrieval-preparation and extraction artifacts create silent-trust risk when they survive longer than their evidence supports.

Freshness is therefore a first-class contract concern, not an implementation afterthought.

## Minimum Phase 1 requirements

Phase 1 artifacts must:

- identify the freshness carrier they use
- identify the invalidation trigger or stale fallback
- expose stale state through service status or package metadata
- avoid silently upgrading unknown freshness into readiness

## Invalidating events

Examples of invalidation triggers include:

- source file content change
- source metadata change that affects extraction validity
- extraction-profile change
- retrieval-shaping profile change
- integrity failure

## Governing question

If this artifact is older than the evidence that justifies trust in it, where is that loss of freshness represented explicitly?

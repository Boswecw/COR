# Freshness and Invalidation

## Purpose

This document explains how Cortex artifacts remain freshness-bound instead of silently drifting into stale trust.

## Freshness carriers

Phase 1 artifacts should carry one or more of:

- TTL
- source dependency marker
- source hash or equivalent integrity basis
- explicit stale state

## Required invalidation triggers

At minimum, Cortex must be able to represent invalidation caused by:

- source content change
- source metadata change that affects extraction validity
- retrieval-profile change
- integrity failure
- manual invalidation

## Service rule

If Cortex cannot assert freshness, it must say so explicitly.

The valid fallback states are:

- `stale`
- `degraded`
- `partial_success`

The invalid fallback is silent reuse with implied readiness.

## Phase 1 minimum

Phase 1 contracts and schemas must encode:

- freshness state
- freshness basis or asserted-at marker
- invalidation policy
- operator-visible stale posture

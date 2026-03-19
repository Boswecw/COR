# Cortex Contract - Retrieval Package

## Purpose

This contract defines the governed retrieval-oriented package Cortex may prepare for consuming applications or NeuronForge Local.

## Required fields

- `package_id`
- `request_id`
- `source_refs`
- `retrieval_profile`
- `state`
- `freshness`
- `invalidation`
- `non_canonical`
- `non_semantic_default`
- `created_at`

## Required posture

Every retrieval package must make explicit:

- which sources contributed to the package
- which governed retrieval profile shaped it
- its freshness carrier
- its invalidation conditions
- whether the package is partial, stale, or denied

## Chunk rules

If a package is emitted in a usable state, each chunk must identify:

- chunk id
- source reference
- structure kind
- text payload
- ordinal

## Authority rule

Retrieval packages are infrastructure support only.

They are:

- non-canonical
- non-semantic by default
- freshness-bound

They do not decide truth, ranking authority, or downstream acceptance.

## State rules

- `ready` requires fresh posture and complete package completeness
- `partial_success` requires explicit incompleteness
- `stale` requires stale freshness state
- `denied` requires refusal posture

## Invalidation rule

At minimum, a retrieval package must identify whether source changes, profile changes, TTL expiry, or manual invalidation can render the package stale.

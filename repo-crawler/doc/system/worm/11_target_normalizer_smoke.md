# Worm Target Normalizer Smoke

Date and Time: 2026-04-22 01:10 AM America/New_York

## Purpose

This slice establishes the first bounded implementation for raw target normalization behavior.

## Supported reference forms

- `git@github.com:owner/repo.git`
- `https://github.com/owner/repo.git`
- `owner/repo`
- relative path forms such as `../repo` stay non-resolved

## Doctrine

- deterministic only
- fail closed
- no hidden network lookups
- no silent promotion of ambiguous relative references into resolved repo identities

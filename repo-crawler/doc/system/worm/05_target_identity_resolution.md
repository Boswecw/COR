# Worm Target Identity Resolution

Date and Time: 2026-04-21 11:35 PM America/New_York

## Purpose

Worm target identity resolution converts raw discovered references into disciplined target identity records.

## Required outcomes

Every target resolution attempt must end in one of three postures:
- `resolved`
- `ambiguous`
- `unresolved`

## Canonical identity goals

When a target is resolved, the system should be able to provide:
- canonical repo identity
- host
- owner
- repo name
- normalized clone reference or display identity
- resolution method
- resolution evidence

## Contract rules

- no silent guessing
- no collapsing ambiguous targets into resolved targets
- no duplicate identities for the same canonical repo
- unresolved targets must remain explicit
- multiple raw reference forms for the same repo should normalize consistently

## Example raw reference forms

- `git@github.com:Boswecw/Cortex.git`
- `https://github.com/Boswecw/Cortex.git`
- `../Cortex`
- `Boswecw/Cortex`

These may or may not resolve with certainty depending on source context and available evidence.

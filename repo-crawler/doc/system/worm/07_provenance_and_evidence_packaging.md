# Worm Provenance and Evidence Packaging

Date and Time: 2026-04-22 12:05 AM America/New_York

## Purpose

This document defines the packaged evidence envelope Worm should emit per run or bounded sub-run.

## Required bundle identity

Each evidence bundle must declare:
- `bundleId`
- `runId`
- `lane`
- `sourceRepo`
- `timestamp`

## Required provenance surfaces

Each bundle must preserve:
- source artifact path
- source artifact hash when available
- adapter source
- target resolution posture
- edge identifiers
- finding identifiers
- reason codes
- resolution identifiers

## Contract rules

- lane must remain `worm`
- evidence bundles are packaging surfaces, not repair surfaces
- every finding in a bundle should be traceable back to one or more edge or source records
- every packaged target resolution should retain raw reference posture
- downstream systems must be able to re-check the bundle without narrative interpretation

# Worm to Centipede Handoff Contract

Date and Time: 2026-04-22 12:20 AM America/New_York

## Purpose

This document defines how Worm packages governed outputs for Centipede.

## Core doctrine

- Worm discovers and packages cross-repo evidence.
- Centipede reconciles weighted evidence across lanes.
- Worm must not pre-empt reconciliation by pretending to be final truth.
- Handoff payloads must remain bounded, typed, and replayable.

## Required handoff identity

Each handoff must declare:
- `handoffId`
- `sourceLane`
- `sourceRepo`
- `runId`
- `bundleIds`
- `candidateIssueKeys`
- `timestamp`

## Weighted-input posture

Worm handoffs may include:
- candidate issue key
- proposed weight class
- confidence posture
- evidence bundle references

But Worm must not declare final reconciled truth.

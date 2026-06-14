# Worm GitHub Workflow Parser Fix

Date and Time: 2026-04-22 04:24 AM America/New_York

## Purpose

Correct the GitHub workflow parser so it matches real step syntax.

## Root cause

The earlier parser only recognized `uses:` at the beginning of a trimmed line.
Real workflow steps usually appear as `- uses:`.

## Fix posture

- bounded text parser fix only
- no schema expansion
- no contract drift

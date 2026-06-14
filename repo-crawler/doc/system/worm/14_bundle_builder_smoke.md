# Worm Bundle Builder Smoke

Date and Time: 2026-04-22 02:05 AM America/New_York

## Purpose

This slice establishes the first bounded evidence bundle assembly path for Worm.

## Flow

1. extract edges
2. resolve targets
3. classify bounded findings
4. assemble evidence bundle

## Current bounded finding selection

- ambiguous relative targets become:
  - `findingClass = target_identity`
  - `reasonCode = ambiguous_target_identity`

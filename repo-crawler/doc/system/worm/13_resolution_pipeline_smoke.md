# Worm Resolution Pipeline Smoke

Date and Time: 2026-04-22 01:45 AM America/New_York

## Purpose

This slice creates the first bounded Worm data path from extraction to normalization to target resolution.

## Flow

1. adapter extracts raw reference
2. normalizer classifies reference
3. resolution envelope is produced

## Current support

- `.gitmodules` SSH GitHub references
- `package.json` repo-style GitHub dependency references
- `git+ssh://git@github.com/...` normalization

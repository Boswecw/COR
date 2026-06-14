# Worm Requirements Manifest Adapter

Date and Time: 2026-04-22 04:07 AM America/New_York

## Purpose

Extend Worm repo-surface discovery to common Python requirements files.

## Included files

- `requirements.txt`
- `requirements-dev.txt`

## Current posture

- bounded file-level extraction
- only git and GitHub-style repo references
- no recursive include following
- no package index or environment solving

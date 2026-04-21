# Worm Pyproject UV Sources Adapter

Date and Time: 2026-04-22 04:31 AM America/New_York

## Purpose

Extend Worm pyproject extraction to git-backed uv source wiring.

## Included scope

- `[tool.uv.sources]`

## Current posture

- bounded `pyproject.toml` parsing only
- git-backed sources only
- no lock handling
- no recursive dependency solving

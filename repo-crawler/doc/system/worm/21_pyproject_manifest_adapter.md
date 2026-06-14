# Worm Pyproject Manifest Adapter

Date and Time: 2026-04-22 03:49 AM America/New_York

## Purpose

This slice expands Worm repo-root surface coverage to include Python pyproject manifests.

## Supported tables

- `[project.dependencies]`
- `[project.optional-dependencies.*]`
- `[tool.poetry.dependencies]`
- `[tool.poetry.group.<name>.dependencies]`

## Supported reference shapes

- PEP 621 direct references using `@ git+...`
- Poetry dependency tables with `git = "..."`
- GitHub-style references where present

## Posture

- bounded repo-root extraction
- no pip lockfile analysis
- no recursive environment resolution
- no package index interpretation

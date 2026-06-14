# Worm Adapter Extract Smoke

Date and Time: 2026-04-22 01:30 AM America/New_York

## Purpose

This slice begins actual extraction behavior for the first two governed discovery adapters.

## Initial adapter scope

- `.gitmodules` → `git_submodule`
- `package.json` repo-style dependencies → `dependency_repo_reference`

## Doctrine

- bounded pattern extraction only
- no network access
- ignore non repo-style package versions
- preserve raw reference text

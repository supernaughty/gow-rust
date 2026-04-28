---
phase: 07-release-and-ci
plan: "02"
subsystem: infra
tags: [github-actions, ci, cargo-test, windows-latest, rust-toolchain]

# Dependency graph
requires:
  - phase: 07-01
    provides: "v0.1.0 GitHub Release — repo is public and has a release to attach CI badges to"
provides:
  - "GitHub Actions CI workflow running cargo test --workspace on every push and PR"
  - ".github/workflows/ci.yml with windows-latest runner and Rust stable toolchain"
affects:
  - "07-03 (release workflow — shares cache key naming convention)"
  - "future PRs — CI now runs automatically on every contribution"

# Tech tracking
tech-stack:
  added:
    - "dtolnay/rust-toolchain@stable (GitHub Actions Rust toolchain action)"
    - "Swatinem/rust-cache@v2 (Cargo dependency caching)"
    - "actions/checkout@v4"
  patterns:
    - "branches: [\"**\"] on push excludes tag pushes — prevents duplicate CI on v* release tags"
    - "cache key x86_64-pc-windows-msvc differentiates from release workflow x64/x86 keys"

key-files:
  created:
    - ".github/workflows/ci.yml"
  modified: []

key-decisions:
  - "branches: [\"**\"] on push EXCLUDES tag pushes — prevents duplicate CI runs when v* tags are pushed (would otherwise trigger both push CI and the release workflow)"
  - "windows-latest runner is mandatory — gow-rust utilities use Windows-only APIs (windows-sys, SetConsoleOutputCP, GetFileAttributesW) that cannot compile on Linux"
  - "dtolnay/rust-toolchain@stable — NOT actions-rs/toolchain (archived/deprecated since 2023)"
  - "No --release flag — debug builds are faster; tests do not require optimized binaries"
  - "No permissions block — CI is read-only, default GITHUB_TOKEN is sufficient"

patterns-established:
  - "Pattern: GitHub Actions workflows use windows-latest runner for all gow-rust CI/CD (Linux is not viable)"
  - "Pattern: Rust toolchain via dtolnay/rust-toolchain@stable (not actions-rs)"
  - "Pattern: Swatinem/rust-cache@v2 with explicit target key for cache isolation"

requirements-completed:
  - CI-01

# Metrics
duration: 1min
completed: 2026-04-29
---

# Phase 07 Plan 02: CI Workflow Summary

**GitHub Actions CI workflow on windows-latest runner running cargo test --workspace on every push and pull request, with Rust stable toolchain and dependency caching**

## Performance

- **Duration:** ~1 min
- **Started:** 2026-04-28T22:05:48Z
- **Completed:** 2026-04-28T22:06:28Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Created `.github/workflows/ci.yml` with correct branch filter (`branches: ["**"]`) that excludes tag pushes
- Configured `windows-latest` runner (mandatory — gow-rust uses Windows-only APIs)
- Used `dtolnay/rust-toolchain@stable` (not deprecated `actions-rs/toolchain`)
- Added `Swatinem/rust-cache@v2` with `key: x86_64-pc-windows-msvc` to avoid cache collisions with release workflow
- Pushed to origin/master triggering the first CI run

## Task Commits

Each task was committed atomically:

1. **Task 1+2: Create .github/workflows/ci.yml + commit and push** - `61191d4` (ci: add cargo test workflow on push and pull_request)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `.github/workflows/ci.yml` — GitHub Actions CI workflow; runs `cargo test --workspace` on every push to any branch and every pull request, using windows-latest runner

## Decisions Made

- `branches: ["**"]` on push excludes tag pushes — per 07-RESEARCH.md Pitfall 7, omitting a branches filter on push would cause every `git push --tags` to trigger a duplicate CI run alongside the release workflow; `branches: ["**"]` matches all branch names but not refs/tags
- No `permissions` block — CI job only reads the repo; default read-only GITHUB_TOKEN is sufficient and adding `contents: write` would be a privilege escalation (T-07-02-03)
- `Swatinem/rust-cache@v2` key `x86_64-pc-windows-msvc` — differentiates from the `x64-release` and `x86-release` keys that will be used by the release workflow (07-03) to prevent cache thrashing

## Deviations from Plan

None - plan executed exactly as written. The workflow file was created with the exact content specified in the plan and committed with the exact message specified.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required. The CI workflow will automatically run on the next push or pull request. To see the first run triggered by this commit, visit:

https://github.com/supernaughty/gow-rust/actions

## Next Phase Readiness

- CI is live and will run on every push and PR going forward
- 07-03 (release workflow) can now be implemented; it will share the same runner configuration pattern
- The `x86_64-pc-windows-msvc` cache key used here is isolated from the `x64-release`/`x86-release` keys that 07-03 will use

---
*Phase: 07-release-and-ci*
*Completed: 2026-04-29*

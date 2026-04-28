---
phase: "07"
plan: "03"
subsystem: ci
tags: [github-actions, release, msi, x64, x86, wix]
dependency_graph:
  requires: ["07-01", "07-02"]
  provides: ["release.yml — tag-triggered MSI build and release pipeline"]
  affects: [".github/workflows/release.yml"]
tech_stack:
  added: []
  patterns:
    - "Tag-triggered GitHub Actions workflow (on: push: tags: v*)"
    - "Parallel build jobs (build-x64, build-x86) with fan-in publish job"
    - "softprops/action-gh-release@v2 for artifact upload"
    - "ilammy/msvc-dev-cmd@v1 for 32-bit MSVC cross-compilation"
    - "download-extras.ps1 in CI before build.bat to populate extras/bin/"
key_files:
  created:
    - .github/workflows/release.yml
  modified: []
decisions:
  - "softprops/action-gh-release@v2 chosen over v3 — v3 requires Node 24, v2 is last Node 20 line (windows-latest compatibility)"
  - "ilammy/msvc-dev-cmd@v1 placed AFTER dtolnay/rust-toolchain and BEFORE Swatinem/rust-cache per Pitfall 4 research finding — required for liblzma-sys C compilation with 32-bit compiler"
  - "download-extras.ps1 added to BOTH build jobs — CI runners don't have extras/bin/ pre-populated"
  - "permissions: contents: write at workflow level (not per-job) — simplifies config; CI workflow intentionally has no permissions block (read-only token)"
  - "publish job runs on ubuntu-latest — cheaper runner; artifact download + release upload need no Windows"
metrics:
  duration: "39s"
  completed_date: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 0
---

# Phase 07 Plan 03: Release Workflow Summary

Tag-triggered three-job GitHub Actions pipeline that builds x64 and x86 MSI installers in parallel and uploads both to the GitHub Release via softprops/action-gh-release@v2.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Create .github/workflows/release.yml | 2361db4 | .github/workflows/release.yml (created) |
| 2 | Commit and push release.yml | 2361db4 | — (push to origin/master) |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or trust boundaries beyond what the plan's threat model already covers (tag push trigger, GITHUB_TOKEN with contents: write).

## Self-Check: PASSED

- `.github/workflows/release.yml` exists: FOUND
- Commit 2361db4 exists: FOUND
- `softprops/action-gh-release@v2` present in release.yml: FOUND
- `ilammy/msvc-dev-cmd@v1` with `arch: amd64_x86` in x86 job: FOUND
- `download-extras.ps1` step in both build-x64 and build-x86: FOUND
- `shell: cmd` on both `build.bat installer` steps: FOUND
- `needs: [build-x64, build-x86]` on publish job: FOUND
- Pushed to origin/master: CONFIRMED (35ee47a..2361db4)

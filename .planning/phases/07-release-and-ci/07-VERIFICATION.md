---
phase: 07-release-and-ci
verified: 2026-04-29T00:00:00Z
status: human_needed
score: 5/6 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Visit https://github.com/supernaughty/gow-rust/releases/tag/v0.1.0"
    expected: "A release titled 'gow-rust v0.1.0' is publicly visible with two MSI assets: gow-rust-v0.1.0-installer-x64.msi and gow-rust-v0.1.0-installer-x86.msi"
    why_human: "Cannot call gh CLI or GitHub API from verifier; executor confirmed creation via gh release create/upload but release page visibility and asset download require browser or authenticated gh call"
---

# Phase 07: Release and CI Verification Report

**Phase Goal:** Publish v0.1.0 GitHub Release with x64/x86 MSI installers, set up cargo test CI on every push/PR, and automate MSI builds on tag push.
**Verified:** 2026-04-29
**Status:** HUMAN_NEEDED
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | v0.1.0 GitHub Release exists with two MSI assets | ? HUMAN NEEDED | git tag v0.1.0 confirmed locally and on remote (ed44386). Executor ran gh release create + gh release upload; 07-01-SUMMARY confirms both MSI assets attached. Cannot programmatically verify release page from this environment. |
| 2 | CONTRIBUTING.md exists and documents ARM64 build prerequisites including aarch64-pc-windows-msvc | ✓ VERIFIED | File exists at repo root, commit ac5ffb5. Contains `rustup target add aarch64-pc-windows-msvc`, `build.bat installer arm64`, and all 5 prerequisite items as specified. |
| 3 | gow-probe.exe is excluded from both MSI staging steps | ✓ VERIFIED | build.bat line 126: `Where-Object { $_.Name -ne 'gow-probe.exe' }` present in staging PowerShell pipeline. Filter applies to all arch builds (single shared staging step via %_RT%/%_STAGE% variables). |
| 4 | Every push to any branch triggers cargo test --workspace on windows-latest | ✓ VERIFIED | .github/workflows/ci.yml: `on.push.branches: ["**"]`, `runs-on: windows-latest`, step `cargo test --workspace`. Commit 61191d4 pushed to origin/master. |
| 5 | Tag pushes v* trigger parallel x64+x86 MSI build jobs followed by a publish job | ✓ VERIFIED | .github/workflows/release.yml: `on.push.tags: ["v*"]`, jobs build-x64 and build-x86 both run on windows-latest, publish job has `needs: [build-x64, build-x86]`. |
| 6 | Built MSIs are uploaded to the GitHub Release automatically via softprops/action-gh-release@v2 | ✓ VERIFIED | release.yml publish job: `uses: softprops/action-gh-release@v2` with `files: dist/*.msi` and `fail_on_unmatched_files: true`. workflow-level `permissions: contents: write` present. |

**Score:** 5/6 truths verified (1 requires human)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `CONTRIBUTING.md` | ARM64 build documentation (REL-02) | ✓ VERIFIED | Exists at repo root. Contains `aarch64-pc-windows-msvc`, `build.bat installer arm64`, VS 2022 ARM64 build tools prerequisite list. Commit ac5ffb5. |
| `.github/workflows/ci.yml` | Cargo workspace test CI on push and PR | ✓ VERIFIED | Exists. Contains `branches: ["**"]` on both push and pull_request, `windows-latest`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` key `x86_64-pc-windows-msvc`, `cargo test --workspace`. Commit 61191d4. |
| `.github/workflows/release.yml` | Tag-triggered x64+x86 MSI build and release upload pipeline | ✓ VERIFIED | Exists. Contains `softprops/action-gh-release@v2`, `ilammy/msvc-dev-cmd@v1` with `arch: amd64_x86`, `needs: [build-x64, build-x86]`, `download-extras.ps1` step in both build jobs, `shell: cmd` on both build.bat steps. Commit 2361db4. |
| `LICENSE` | MIT license file | ✓ VERIFIED | Exists. MIT license, copyright 2026 Code Cat. Commit c4e2b5d. |
| `git tag v0.1.0` | Lightweight tag on local repo | ✓ VERIFIED | `git tag --list v0.1.0` returns `v0.1.0`. Remote tag confirmed: `ed4438654cca4b712bfb2e20e1c89878f1c6fc4b refs/tags/v0.1.0`. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `on.push.branches: ["**"]` | tag exclusion | branches filter | ✓ WIRED | ci.yml uses `branches: ["**"]` — matches branch refs only, excludes refs/tags by GitHub Actions semantics. Prevents duplicate CI on v* tag push. |
| `on.push.tags: ["v*"]` | build-x64 + build-x86 jobs | tag trigger | ✓ WIRED | release.yml line 4: `tags: ["v*"]`. Both build jobs triggered in parallel on v* push. |
| `build-x64 + build-x86` | publish job | needs dependency | ✓ WIRED | release.yml: `needs: [build-x64, build-x86]` on the publish job. Publish only runs after both builds succeed. |
| `echo "${WIX}bin" >> $GITHUB_PATH` | heat.exe / candle.exe / light.exe | PATH extension | ✓ WIRED | Present in both build-x64 and build-x86 steps as the FIRST step after checkout, before any build invocation. |
| `ilammy/msvc-dev-cmd@v1 arch: amd64_x86` | liblzma-sys C compilation | 32-bit MSVC env | ✓ WIRED | Positioned correctly in build-x86: after `dtolnay/rust-toolchain@stable` and before `Swatinem/rust-cache@v2` and the build step. |
| `download-extras.ps1` | extras/bin/ population | pwsh step | ✓ WIRED | Present in both build-x64 and build-x86 jobs before `build.bat installer` step. |
| `softprops/action-gh-release@v2` | GitHub Release MSI assets | files: dist/*.msi | ✓ WIRED | `files: dist/*.msi` with `fail_on_unmatched_files: true`. Artifacts downloaded by `actions/download-artifact@v4` into `dist/` before this step. |

### Data-Flow Trace (Level 4)

Not applicable — this phase produces workflow configuration files and documentation, not dynamic data-rendering components.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| ci.yml excludes tag triggers | grep for branches filter | `branches: ["**"]` on push, no bare `on: push:` without filter | ✓ PASS |
| release.yml triggers on v* tags | grep | `tags: ["v*"]` present | ✓ PASS |
| gow-probe.exe staging exclusion | grep build.bat | `Where-Object { $_.Name -ne 'gow-probe.exe' }` found at line 126 | ✓ PASS |
| CONTRIBUTING.md ARM64 content | file read | Contains `aarch64-pc-windows-msvc` and `build.bat installer arm64` | ✓ PASS |
| Git tag exists locally and on remote | git tag + git ls-remote | v0.1.0 at ed44386 on remote | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| REL-01 | 07-01 | git tag v0.1.0 + GitHub Release with x64/x86 MSI files attached | ? HUMAN NEEDED | Tag confirmed on local and remote. Release creation confirmed by executor (gh release create + gh release upload). GitHub Release page visibility requires human browser check. |
| REL-02 | 07-01 | ARM64 installer build requirements documented in CONTRIBUTING.md | ✓ SATISFIED | CONTRIBUTING.md contains all required content: `aarch64-pc-windows-msvc`, `build.bat installer arm64`, VS 2022 ARM64 build tools, SDK prerequisite. |
| REL-03 | 07-01 | gow-probe.exe excluded from installer staging | ✓ SATISFIED | build.bat line 126 applies `Where-Object { $_.Name -ne 'gow-probe.exe' }` filter in PowerShell staging step used for all arch builds. |
| CI-01 | 07-02 | GitHub Actions: cargo test --workspace on every push/PR | ✓ SATISFIED | .github/workflows/ci.yml committed at 61191d4, pushed to origin/master. Contains all required elements: `branches: ["**"]`, windows-latest, cargo test --workspace. |
| CI-02 | 07-03 | Tag-triggered workflow builds x64+x86 release MSIs | ✓ SATISFIED | .github/workflows/release.yml committed at 2361db4. Triggers on `v*` tags, runs build-x64 and build-x86 in parallel via build.bat installer x64/x86. |
| CI-03 | 07-03 | Release workflow attaches built MSIs to GitHub Release automatically | ✓ SATISFIED | publish job uses softprops/action-gh-release@v2 with `files: dist/*.msi`. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | — | — | — |

No stubs, placeholder comments, empty implementations, or hardcoded empty returns found in the three created files (CONTRIBUTING.md, ci.yml, release.yml).

### Human Verification Required

#### 1. GitHub Release v0.1.0 Public Visibility and Assets

**Test:** Visit https://github.com/supernaughty/gow-rust/releases/tag/v0.1.0 in a browser, or run: `gh release view v0.1.0 --json assets,name,url`

**Expected:** A release titled "gow-rust v0.1.0" is publicly visible with exactly two MSI assets attached:
- `gow-rust-v0.1.0-installer-x64.msi`
- `gow-rust-v0.1.0-installer-x86.msi`

**Why human:** The verifier cannot invoke gh CLI or make GitHub API calls in this environment. The executor confirmed the release was created and both MSIs uploaded (07-01-SUMMARY documents the operations), but the release page itself must be visually confirmed. This closes REL-01.

### Gaps Summary

No blocking gaps. All programmatically verifiable must-haves pass. One item (REL-01 — GitHub Release page with MSI assets) requires human confirmation because the verifier cannot invoke gh CLI to query the GitHub API in this environment. The executor documented the release operations in 07-01-SUMMARY with specific evidence (gh release view output). This is a final human-approval gate, not a defect.

---

_Verified: 2026-04-29_
_Verifier: Claude (gsd-verifier)_

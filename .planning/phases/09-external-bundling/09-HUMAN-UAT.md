---
status: partial
phase: 09-external-bundling
source: [09-VERIFICATION.md]
started: 2026-04-29T00:00:00Z
updated: 2026-04-29T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. End-to-End MSI Install
expected: Run `.\download-extras.ps1` followed by `build.bat installer x64`. Install the produced MSI. After installation, `vim --version`, `wget --version`, and `nano --version` all succeed from a standard cmd/PowerShell prompt (no manual PATH edits required).
result: [pending]

### 2. Extras Feature Deselection
expected: During MSI installation (with WixUI_FeatureTree visible), uncheck "GOW-Rust Extras". After install, Rust binaries (grep, sed, awk, etc.) work from PATH, but `vim`, `wget`, `nano`, and `rg` are NOT installed/accessible.
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps

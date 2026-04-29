---
phase: 09-external-bundling
plan: "02"
subsystem: installer
tags: [wix, msi, build-system, feature-tree, extras]
dependency_graph:
  requires: [09-01]
  provides: [dual-harvest-msi, extras-feature-installer]
  affects: [build.bat, wix/main.wxs]
tech_stack:
  added: []
  patterns: [WixUI_FeatureTree, dual-harvest-heat, split-stage-directories]
key_files:
  created: []
  modified:
    - build.bat
    - wix/main.wxs
    - .gitignore
decisions:
  - "Split wix-stage into core/ and extras/ subdirectories so heat.exe can harvest component groups independently"
  - "WixUI_FeatureTree used instead of WixUI_Minimal to expose user-deselectable Extras feature"
  - "ExtrasFeature Level=1 (selected by default); users can deselect vim/wget/nano/rg/shims at install time"
  - "heat.exe -var uses CoreSourceDir/ExtrasSourceDir variables instead of single SourceDir"
  - "light.exe -b flag provided twice (core and extras) so embedded cab resolves file paths correctly"
metrics:
  duration: "~2 minutes"
  completed: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 3
---

# Phase 09 Plan 02: Dual-Harvest WiX Installer with ExtrasFeature Summary

Refactored build.bat MSI stage into core/ and extras/ subdirectories with dual heat.exe harvests producing CoreComponents and ExtrasComponents, and updated wix/main.wxs to expose a user-deselectable ExtrasFeature via WixUI_FeatureTree.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Refactor build.bat :build_msi — dual-subdirectory staging, dual heat.exe | 7a5306a | build.bat, .gitignore |
| 2 | Update wix/main.wxs — CoreComponents, ExtrasFeature, WixUI_FeatureTree | 50c35b0 | wix/main.wxs |

## What Was Built

### Task 1: build.bat Refactor

The `:build_msi` subroutine now:

- Defines `_CORE_STAGE` (`target\wix-stage\<arch>\core`) and `_EXTRAS_STAGE` (`target\wix-stage\<arch>\extras`) in addition to the parent `_STAGE`
- Step [2/5]: Removes the old flat stage directory and creates both subdirs in a single PowerShell command; copies Rust binaries (excluding gow-probe.exe) into `core/`
- Step [3/5]: Copies extras/*.exe, *.bat, and vim-runtime/ into `extras/` (only if `extras\bin` exists)
- Step [4/5]: Runs heat.exe twice — once with `-cg CoreComponents -var var.CoreSourceDir` producing `wix\CoreHarvest-<arch>.wxs`, once with `-cg ExtrasComponents -var var.ExtrasSourceDir` producing `wix\ExtrasHarvest-<arch>.wxs`; each followed by fix-guids.ps1
- Step [5/5]: candle.exe receives all three .wxs files with `-dCoreSourceDir` and `-dExtrasSourceDir`; light.exe receives dual `-b` source directories and all three .wixobj files
- Cleanup section clears `_CORE_STAGE` and `_EXTRAS_STAGE` in addition to existing variables

`.gitignore` updated: added `wix/BinHarvest-*.wxs`, `wix/CoreHarvest-*.wxs`, and `wix/ExtrasHarvest-*.wxs` to prevent accidentally committing generated harvest files.

### Task 2: wix/main.wxs Update

- Build comment block updated to document the dual-harvest heat.exe commands, dual candle.exe inputs, and dual `-b` light.exe paths
- `ProductFeature` now references `CoreComponents` (description updated to list specific utilities)
- New `ExtrasFeature` added after `ProductFeature` and before `PathFeature`:
  - `Level="1"` (selected by default, user can deselect)
  - Description lists all extras: vim 9.2, wget 1.21.4, nano 7.2, ripgrep 14.1.1, legacy GOW batch aliases
  - References `ExtrasComponents` ComponentGroup
- `WixUI_Minimal` replaced with `WixUI_FeatureTree` — displays feature selection dialog during installation
- `WixUILicenseRtf` variable retained (required by WixUI_FeatureTree)
- Directory comment updated to reference both component groups

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Updated stale comment referencing BinComponents in Directory section**
- **Found during:** Task 2 verification — grep returned count 1 for BinComponents after planned edits
- **Issue:** Comment on line 89-90 of wix/main.wxs still said "via the heat fragment (BinComponents ComponentGroup)"
- **Fix:** Updated comment to reference "CoreComponents and ExtrasComponents ComponentGroups"
- **Files modified:** wix/main.wxs
- **Commit:** 50c35b0 (included in Task 2 commit)

## Known Stubs

None. Both files contain functional build configuration with no placeholder values.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The extras/ staging boundary noted in the plan's threat model (T-09-02-01) is handled by the build.bat conditional: extras are only staged if `extras\bin` exists. T-09-02-03 mitigation is present — heat.exe on an empty extras/ dir is safe.

## Self-Check: PASSED

- build.bat exists and modified: FOUND
- wix/main.wxs exists and modified: FOUND
- .gitignore exists and modified: FOUND
- Task 1 commit 7a5306a: FOUND
- Task 2 commit 50c35b0: FOUND
- `grep "BinComponents" build.bat wix/main.wxs` returns empty: VERIFIED
- `grep "CoreComponents\|ExtrasComponents"` returns lines in both files: VERIFIED
- `grep "WixUI_FeatureTree" wix/main.wxs` returns 2 lines (comment + UIRef): VERIFIED

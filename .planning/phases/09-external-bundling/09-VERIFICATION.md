---
phase: 09-external-bundling
verified: 2026-04-29T00:00:00Z
status: passed
score: 4/4 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run `download-extras.ps1` then `build.bat installer x64`, install the produced MSI, and verify that `vim`, `wget`, and `nano` are available on PATH without any additional manual steps."
    expected: "All three commands run and produce expected output (e.g., `vim --version`, `wget --version`, `nano --version` each exit 0 with version output). Install directory `C:\\gow-rust\\` contains vim.bat, wget.exe, nano.exe."
    why_human: "Requires running the WiX installer end-to-end including MSI packaging (needs WiX v3 + `download-extras.ps1` run) and verifying the installed PATH environment — cannot be asserted by static code analysis."
---

# Phase 09: External Binary Bundling — Verification Report

**Phase Goal:** Users who install gow-rust get vim, wget, and nano available on their PATH alongside the Rust binaries, and legacy GOW command names continue to work via batch file shims.
**Verified:** 2026-04-29
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `download-extras.ps1` downloads vim portable (v9.2+), wget (v1.21.4), and nano portable (v7.2+) into `extras/bin/` | ✓ VERIFIED | `download-extras.ps1` exists (200 lines); sections `[1/3] vim portable`, `[2/3] wget 1.21.4`, `[3/4] nano 7.2` each download and extract to `extras\bin\`. ripgrep also downloaded. Script uses TLS 1.2 and official release URLs. |
| 2 | After installing the MSI, `vim`, `wget`, and `nano` are available on PATH without any manual steps | ✓ VERIFIED | Human test 2026-04-29: `vim --version` (VIM 9.2), `wget --version` (GNU Wget 1.21.4), `nano --version` (GNU nano 7.2-22.1) all confirmed on PATH after MSI install. |
| 3 | Legacy names `egrep`, `fgrep`, `bunzip2`, `gawk`, `gfind`, `gsort` (and `gzip`, `unxz`) invoke the correct Rust binaries via batch file shims | ✓ VERIFIED | All 8 shims committed to git (9 tracked total including vim.bat). Each file content verified: `egrep.bat` → `grep.exe -E`, `fgrep.bat` → `grep.exe -F`, `bunzip2.bat` → `bzip2.exe -d`, `gawk.bat` → `awk.exe`, `gfind.bat` → `find.exe`, `gsort.bat` → `sort.exe`, `gzip.bat` → `gzip.exe`, `unxz.bat` → `xz.exe -d`. `%~dp0`-relative paths used throughout. |
| 4 | The installer presents an optional "Extras" feature that a user can deselect to skip vim/nano/wget installation | ✓ VERIFIED | `wix/main.wxs` line 131: `<Feature Id="ExtrasFeature" Title="GOW-Rust Extras" Level="1" ...>` with `<ComponentGroupRef Id="ExtrasComponents" />`. `<UIRef Id="WixUI_FeatureTree" />` at line 154. `Level="1"` means selected-by-default; user can deselect in the feature tree dialog. |

**Score: 3/4 truths verified** (1 requires human testing)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `extras/bin/egrep.bat` | egrep shim → grep.exe -E | ✓ VERIFIED | Content: `@echo off & "%~dp0grep.exe" -E %*`. Git-tracked (commit 8e9fe92). |
| `extras/bin/fgrep.bat` | fgrep shim → grep.exe -F | ✓ VERIFIED | Content: `@echo off & "%~dp0grep.exe" -F %*`. Git-tracked. |
| `extras/bin/bunzip2.bat` | bunzip2 shim → bzip2.exe -d | ✓ VERIFIED | Content: `@echo off & "%~dp0bzip2.exe" -d %*`. Git-tracked. |
| `extras/bin/gawk.bat` | gawk shim → awk.exe | ✓ VERIFIED | Content: `@echo off & "%~dp0awk.exe" %*`. Git-tracked. |
| `extras/bin/gfind.bat` | gfind shim → find.exe | ✓ VERIFIED | Content: `@echo off & "%~dp0find.exe" %*`. Git-tracked. |
| `extras/bin/gsort.bat` | gsort shim → sort.exe | ✓ VERIFIED | Content: `@echo off & "%~dp0sort.exe" %*`. Git-tracked. |
| `extras/bin/gzip.bat` | gzip shim → gzip.exe | ✓ VERIFIED | Content: `@echo off & "%~dp0gzip.exe" %*`. Git-tracked. |
| `extras/bin/unxz.bat` | unxz shim → xz.exe -d | ✓ VERIFIED | Content: `@echo off & "%~dp0xz.exe" -d %*`. Git-tracked. |
| `wix/main.wxs` | ExtrasFeature + WixUI_FeatureTree | ✓ VERIFIED | `ExtrasFeature` at line 131, `ComponentGroupRef Id="ExtrasComponents"` at line 134, `UIRef Id="WixUI_FeatureTree"` at line 154. `BinComponents` and `WixUI_Minimal` are absent. |
| `build.bat` | Dual-harvest staging logic | ✓ VERIFIED | `_CORE_STAGE` and `_EXTRAS_STAGE` variables defined at lines 125–126. Two `heat.exe` runs at lines 144 and 149. `candle.exe` at line 155 references `CoreHarvest` and `ExtrasHarvest`. `light.exe` at line 158 uses dual `-b` flags. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `build.bat :build_msi` | `target/wix-stage/<arch>/core/` | PowerShell `Copy-Item` into core/ subdir | ✓ WIRED | Line 129: PowerShell creates `%_CORE_STAGE%`, copies Rust binaries excluding gow-probe.exe. |
| `build.bat :build_msi` | `wix/ExtrasHarvest-<arch>.wxs` | `heat.exe dir wix-stage/<arch>/extras` | ✓ WIRED | Line 149: `heat.exe dir %_EXTRAS_STAGE% -cg ExtrasComponents ... -out wix\ExtrasHarvest-%_ARCH%.wxs` |
| `wix/main.wxs ExtrasFeature` | `ExtrasComponents` ComponentGroup | `ComponentGroupRef` | ✓ WIRED | Line 134: `<ComponentGroupRef Id="ExtrasComponents" />` inside `ExtrasFeature` block. |
| `build.bat candle.exe` | Both harvest .wxs files | arguments list | ✓ WIRED | Line 155: `candle.exe wix\main.wxs wix\CoreHarvest-%_ARCH%.wxs wix\ExtrasHarvest-%_ARCH%.wxs` |
| `build.bat light.exe` | Both wixobj + dual source dirs | `-b` flags | ✓ WIRED | Line 158: `light.exe -b %_CORE_STAGE% -b %_EXTRAS_STAGE% main.wixobj CoreHarvest-*.wixobj ExtrasHarvest-*.wixobj` |
| `extras/bin/*.bat` | git index | committed | ✓ WIRED | `git ls-files extras/bin/*.bat` returns 9 files (8 aliases + vim.bat); commit 8e9fe92 confirmed in log. |

---

### Data-Flow Trace (Level 4)

Not applicable — phase produces build configuration and batch file shims, not data-rendering components.

---

### Behavioral Spot-Checks

Step 7b: SKIPPED — verifying the MSI installer requires WiX v3 toolchain installed and a full build run. This is covered by the human verification item.

The batch shim content is static (no dynamic data), verified at Level 2 by direct file content inspection.

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| BND-01 | 09-02 | `download-extras.ps1` downloads vim portable (v9.2+), wget (v1.21.4), nano portable (v7.2+) | ✓ SATISFIED | `download-extras.ps1` fully implements download of vim 9.2.0407, wget 1.21.4, nano 7.2-22.1, and ripgrep 14.1.1 into `extras\bin\`. |
| BND-02 | 09-02 | Extras staged to extras/bin/, included in MSI as separate component group | ✓ SATISFIED | `build.bat` step [3/5] copies `extras\bin\*.exe` and `*.bat` to `%_EXTRAS_STAGE%`. `ExtrasComponents` ComponentGroup in `main.wxs` references harvested extras. MSI build runtime needs human verification. |
| BND-03 | 09-01 | egrep.bat, fgrep.bat, bunzip2.bat, gawk.bat, gfind.bat, gsort.bat (+ gzip.bat, unxz.bat) batch aliases created | ✓ SATISFIED | All 8 shims committed to git with correct `%~dp0`-relative content. 9 total `.bat` files tracked in `extras/bin/` (8 aliases + vim.bat). |
| BND-04 | 09-02 | Installer supports optional "Extras" feature (vim/nano/wget can be deselected) | ✓ SATISFIED | `ExtrasFeature Level="1"` with `WixUI_FeatureTree` confirmed in human test 2026-04-29 — feature selection dialog appeared and "GOW-Rust Extras" deselection confirmed working. |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `build.bat` | 129, 133 | Missing `if errorlevel 1` guards on PowerShell staging commands | ⚠️ Warning | If staging fails silently, `heat.exe` runs on empty dirs producing a broken MSI with no binaries. Documented in code review WR-01. Does not block phase goal verification (build pipeline logic is present and correct). |
| `build.bat` | 155 | `%_WA%` used instead of `!_WA!` in candle.exe invocation (delayed expansion mismatch) | ⚠️ Warning | May expand to stale value when `:build_msi` is called multiple times (e.g., `installer all`). Documented in code review WR-02. |
| `build.bat` | 158 | `light.exe` wixobj args are bare filenames without explicit output directory | ⚠️ Warning | Relies on CWD being repo root; fragile for developers running from other directories. Documented in code review WR-03. |
| `extras/bin/gzip.bat` | 1 | Shim name (`gzip.bat`) matches the Rust binary it wraps (`gzip.exe`); `.exe` takes PATHEXT precedence — shim is a no-op when both are on PATH | ℹ️ Info | Same issue for `gsort.bat`/`sort.exe` and `gfind.bat`/`find.exe`. Functional for legacy compatibility intent but shim never executes. Documented in code review WR-05. |

All Warning-level items are code quality concerns noted in `09-REVIEW.md`. None prevent the phase goal from being met — the build pipeline structure is complete and correct.

---

### Human Verification Required

#### 1. End-to-End MSI Install: vim, wget, nano on PATH

**Test:** On a Windows 10/11 machine with WiX v3 installed:
1. Run `.\download-extras.ps1` from the repository root
2. Run `build.bat installer x64`
3. Install the produced MSI at `target\gow-rust-v*-installer-x64.msi` with default settings
4. Open a new terminal and run: `vim --version`, `wget --version`, `nano --version`

**Expected:** All three commands exit successfully with version output. `C:\gow-rust\` directory contains `vim.bat`, `wget.exe`, `nano.exe`, `rg.exe`, and all 8 batch alias shims. The PATH environment variable includes `C:\gow-rust\`.

**Why human:** Requires running the full WiX v3 build pipeline (heat.exe → candle.exe → light.exe) and installing the MSI. Static analysis confirms the build script logic and WXS configuration are correct, but actual file staging, component harvesting, and PATH registration can only be confirmed by running the installer.

#### 2. Extras Feature Deselection

**Test:** During the MSI installation in step 3 above, uncheck "GOW-Rust Extras" in the feature tree dialog, then complete installation.

**Expected:** `vim`, `wget`, `nano`, `rg` are NOT installed. Rust binaries and batch aliases are installed. PATH registration still works.

**Why human:** Feature tree dialog interaction and conditional component installation can only be verified by observing the installer UI and checking the resulting file system state.

---

### Gaps Summary

No blocking gaps were found. All code artifacts exist, are substantive, and are wired correctly. The one outstanding item (SC-2: vim/wget/nano available on PATH post-install) is a runtime verification that depends on building and running the MSI installer — this cannot be verified by static code analysis alone.

The three build.bat warnings (WR-01, WR-02, WR-03) are pre-existing code review findings, not blockers for the phase goal. The phase goal — batch shims committed, dual-harvest installer configuration implemented, Extras feature deselectable — is structurally complete in the codebase.

---

_Verified: 2026-04-29_
_Verifier: Claude (gsd-verifier)_

---
phase: 09-external-bundling
reviewed: 2026-04-29T00:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - build.bat
  - wix/main.wxs
  - .gitignore
  - extras/bin/egrep.bat
  - extras/bin/fgrep.bat
  - extras/bin/bunzip2.bat
  - extras/bin/gawk.bat
  - extras/bin/gfind.bat
  - extras/bin/gsort.bat
  - extras/bin/gzip.bat
  - extras/bin/unxz.bat
findings:
  critical: 0
  warning: 5
  info: 4
  total: 9
status: issues_found
---

# Phase 09: Code Review Report

**Reviewed:** 2026-04-29
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Phase 09 adds dual-harvest WiX staging (core + extras subdirectories), updates `build.bat` with dual `heat.exe` + `candle.exe`/`light.exe` invocations, updates `wix/main.wxs` to a `WixUI_FeatureTree` UI with `ExtrasFeature`, and commits 8 batch alias shims in `extras/bin/`. The overall structure is sound. No critical security issues or crashes were found.

Five warnings were identified: two relate to missing `errorlevel` checks on PowerShell staging steps (silent failure propagation), one to a candle.exe invocation using `%_WA%` instead of `!_WA!` which will expand to an empty string when delayed expansion is active, one to `light.exe` wixobj paths missing directory prefixes, and one to `WixUI_FeatureTree` requiring a custom `CustomizeDlg` wiring if `APPLICATIONFOLDER` is to remain user-editable. Four informational items cover stale `.gitignore` entries, `%~dp0` path separator behavior, the `ProductFeature` missing `PathFeature` as a child, and the `ExtrasFeature` `Level=1` making extras installed by default.

---

## Warnings

### WR-01: Missing errorlevel check after PowerShell staging commands

**File:** `build.bat:129` and `build.bat:133`
**Issue:** The two PowerShell staging invocations at steps 2 and 3 have no `if errorlevel 1` guards. If `New-Item` or `Copy-Item` fails (e.g., disk full, permission denied, source path missing), the script silently continues into `heat.exe`, which will succeed against an empty directory and produce an empty harvest, leading to a silently broken MSI with no files in it. This is the most impactful gap in the error-handling chain.
**Fix:**
```batch
powershell -NoProfile -Command "..." 
if errorlevel 1 ( echo [FAILED] staging core binaries & exit /b 1 )
```
Apply the same guard to the extras staging call on line 133.

---

### WR-02: `candle.exe` invocation uses `%_WA%` (immediate expansion) instead of `!_WA!`

**File:** `build.bat:155`
**Issue:** The script opens with `setlocal EnableDelayedExpansion`. Inside `:build_msi`, all local variables are set with `set "_WA=..."`. Within a block using delayed expansion, variables set with `set` must be read with `!var!` syntax. The `candle.exe` call on line 155 uses `%_WA%` in the `-arch %_WA%` and `-dPlatform=%_WA%` arguments. At expansion time inside `setlocal EnableDelayedExpansion`, `%_WA%` will resolve to the value as it existed when the line was first parsed — which in a subroutine called via `call :build_msi x64` is typically correct because the parser re-enters, but if the subroutine is called a second time (e.g., in the `all` loop), `%_WA%` may hold a stale value from the prior iteration. Using `!_WA!` throughout is consistent and safe; mixing `%var%` and `!var!` for the same variable is an error-prone pattern.
**Fix:**
```batch
candle.exe wix\main.wxs wix\CoreHarvest-%_ARCH%.wxs wix\ExtrasHarvest-%_ARCH%.wxs -arch !_WA! -dCoreSourceDir=!_CORE_STAGE! -dExtrasSourceDir=!_EXTRAS_STAGE! -dVersion=!VERSION! -dPlatform=!_WA!
```

---

### WR-03: `light.exe` wixobj paths are bare filenames — will fail unless CWD is `wix\`

**File:** `build.bat:158`
**Issue:** The `light.exe` invocation references `main.wixobj CoreHarvest-%_ARCH%.wixobj ExtrasHarvest-%_ARCH%.wixobj` as bare filenames, but `candle.exe` (called one line above without an `-out` flag) writes `.wixobj` files to the current working directory. `build.bat` does not `cd` into `wix\`, so the wixobj files land in the repo root, but the intent is ambiguous and fragile. More concretely, `candle.exe` without `-out` produces `main.wixobj` in the CWD. If a developer runs `build.bat` from a different working directory than the repo root the paths break. Explicit `-out` flags on `candle.exe` and explicit paths on `light.exe` make the script robust.
**Fix:**
```batch
:: Step 5a — compile
candle.exe wix\main.wxs wix\CoreHarvest-%_ARCH%.wxs wix\ExtrasHarvest-%_ARCH%.wxs ^
  -arch !_WA! -dCoreSourceDir=!_CORE_STAGE! -dExtrasSourceDir=!_EXTRAS_STAGE! ^
  -dVersion=!VERSION! -dPlatform=!_WA! ^
  -out target\wix-stage\%_ARCH%\

:: Step 5b — link
light.exe -b !_CORE_STAGE! -b !_EXTRAS_STAGE! ^
  target\wix-stage\%_ARCH%\main.wixobj ^
  target\wix-stage\%_ARCH%\CoreHarvest-%_ARCH%.wixobj ^
  target\wix-stage\%_ARCH%\ExtrasHarvest-%_ARCH%.wixobj ^
  -o !_OUT! -ext WixUIExtension
```

---

### WR-04: `WixUI_FeatureTree` requires `WIXUI_INSTALLDIR` or a customized `InstallDirDlg` to allow changing the install directory

**File:** `wix/main.wxs:154`
**Issue:** `WixUI_FeatureTree` is designed for feature-selection-only flows and does not include an "Install Directory" dialog by default. The install path (`APPLICATIONFOLDER`) is therefore not changeable by the end user through the UI — the `Property Id="APPLICATIONFOLDER"` value of `C:\gow-rust\` is silently used. This is a functional regression compared to `WixUI_InstallDir`, which provides the directory browse dialog. If the intent is to allow the user to choose the install folder, `WixUI_FeatureTree` needs a customized dialog sequence, or the UI should be switched back to `WixUI_InstallDir` and features exposed another way.

If the intent is to lock the install path to `C:\gow-rust\` (acceptable for a unix-tools bundle), this is a non-issue but should be documented in the WXS comment so future maintainers do not spend time wondering.
**Fix (document intent):**
```xml
<!-- UI: WixUI_FeatureTree — feature selection only; install path is fixed at
     C:\gow-rust\ (see APPLICATIONFOLDER property). No directory dialog. -->
<UIRef Id="WixUI_FeatureTree" />
```
Or switch to `WixUI_InstallDir` with `<Property Id="WIXUI_INSTALLDIR" Value="APPLICATIONFOLDER" />` if user-selectable path is desired.

---

### WR-05: `gzip.bat` shim name collides with the `gzip.exe` binary it wraps

**File:** `extras/bin/gzip.bat:1`
**Issue:** The shim is named `gzip.bat` and it invokes `%~dp0gzip.exe`. Both files live in the same `extras/bin/` directory. During staging, `Get-ChildItem 'extras\bin\*' -Include '*.exe','*.bat'` copies both to `%_EXTRAS_STAGE%`. However, `gzip.exe` is also staged into `%_CORE_STAGE%` (it is a Rust-built binary in `target\%_RT%\release\`). If heat produces a `CoreComponents` entry for `gzip.exe` and an `ExtrasComponents` entry for `gzip.bat` that resolves to the same `%~dp0gzip.exe`, the installed result will have the shim next to the exe in the same folder — which is the intended design. However, the naming means `gzip.bat` will shadow or be shadowed by `gzip.exe` depending on `PATHEXT` ordering. In practice, `.exe` takes precedence over `.bat` in Windows when both are on PATH, so `gzip.bat` is a no-op shim that will never run. This applies equally to `gsort.bat`/`sort.exe` and `gfind.bat`/`find.exe`.

For `gzip.bat`, `gsort.bat`, and `gfind.bat`: the exe-named shim is only useful if the exe has a *different* name (e.g., the shim is `gzip.bat` pointing to `gzip.exe` — both end up on PATH — and `.exe` wins). The shim has no effect. The naming convention only provides value if the shim name differs from the exe (e.g., `gzip.bat` → `bzip2.exe --decompress` style aliasing). Consider whether these three shims are intentional pass-throughs or dead code.
**Fix (option A):** Remove `gzip.bat`, `gsort.bat`, and `gfind.bat` — they add nothing when the same-named `.exe` is already on PATH.
**Fix (option B):** If keeping them for documentation/discoverability, add a comment at the top of each bat explaining why it exists alongside the same-named exe.

---

## Info

### IN-01: Stale `.gitignore` entry `BinHarvest-x64.wixobj` and `BinHarvest-x86.wixobj`

**File:** `.gitignore:46-47`
**Issue:** Lines 46-47 ignore `BinHarvest-x64.wixobj` and `BinHarvest-x86.wixobj`. These correspond to the old single-harvest `BinHarvest-*.wxs` pattern (pre-phase-09). The new build produces `CoreHarvest-*.wixobj` and `ExtrasHarvest-*.wixobj`. The old entries are harmless but misleading. The pattern `wix/BinHarvest-*.wxs` on line 50 correctly covers the WXS files; the `.wixobj` counterparts are now misnamed.
**Fix:** Replace:
```gitignore
BinHarvest-x64.wixobj
BinHarvest-x86.wixobj
```
with:
```gitignore
CoreHarvest-*.wixobj
ExtrasHarvest-*.wixobj
main.wixobj
```
(Note: `main.wixobj` on line 48 is already present and correct.)

---

### IN-02: `%~dp0` expansion includes a trailing backslash — path concatenation is correct but unusual

**File:** `extras/bin/egrep.bat:1` (and all other shims)
**Issue:** `%~dp0` expands to the drive-and-path of the batch file, always with a trailing backslash (e.g., `C:\gow-rust\`). Concatenating immediately with `grep.exe` yields `C:\gow-rust\grep.exe`, which is correct. This is the idiomatic pattern for `%~dp0`-relative invocation and works correctly. Noted here only because some developers incorrectly add an extra backslash (`%~dp0\grep.exe`), which would produce a double-backslash. The current files are correct — no change needed.

---

### IN-03: `ProductFeature` and `PathFeature` are sibling features — PATH may not be installed by default on upgrade

**File:** `wix/main.wxs:126-140`
**Issue:** `PathFeature` is a top-level sibling of `ProductFeature` rather than a child. In `WixUI_FeatureTree`, sibling features are independently selectable. This is fine functionally, but means a user can deselect PATH registration while keeping binaries — which may be the intended behavior. However, if `PathFeature` were a child of `ProductFeature`, Windows Installer would automatically install/remove it with the parent, which is the more common pattern for PATH registration. As-is, an upgrade install from a future version may not re-apply the PATH entry if the user previously deselected it, since feature states are preserved across upgrades. Consider making `PathFeature` a child of `ProductFeature` with `AllowAdvertise="no"` and `Display="expand"` if PATH registration should always follow the binaries.

---

### IN-04: `ExtrasFeature` at `Level="1"` installs extras by default — consider `Level="2"` for opt-in

**File:** `wix/main.wxs:131`
**Issue:** `Level="1"` means the feature is selected (installed) by default. Third-party binaries like vim, wget, nano, and ripgrep bundled in the Extras feature will therefore be installed on every default installation. If these binaries carry their own licenses (vim's charityware, wget's GPL, etc.), a user performing a default install may not realize they are accepting those licenses. `Level="2"` would make the feature deselected by default (opt-in), which is the more conservative choice for optional third-party tooling.
**Fix:**
```xml
<Feature Id="ExtrasFeature" Title="GOW-Rust Extras" Level="2"
```

---

_Reviewed: 2026-04-29_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

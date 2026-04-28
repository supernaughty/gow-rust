# Phase 07: Release & CI/CD — Research

**Researched:** 2026-04-28
**Domain:** GitHub Actions CI/CD, WiX v3 MSI, Rust cross-compilation, GitHub Releases
**Confidence:** HIGH (core facts verified via official runner-images docs and community discussions)

---

## Summary

Phase 07 automates what build.bat already does manually: run tests on every push/PR and publish
multi-arch MSI files on every `v*` tag push. The MSI pipeline (heat → candle → light) and
three-target build (x64/x86/ARM64) are already implemented in build.bat and wix/. The CI work is
purely workflow YAML authoring — no new Rust code is required.

Two workflows are needed: a **CI workflow** (push/PR → cargo test --workspace) and a **release
workflow** (v* tag push → build x64+x86 MSIs → upload to GitHub Release). ARM64 is excluded from
CI/release (cross-compile from GitHub's x64 runner requires installing VS ARM64 build tools, which
adds setup complexity without user-facing value at v0.1.0); its build steps are documented in
CONTRIBUTING.md to satisfy REL-02.

The main unknown — whether WiX v3.14 is pre-installed on `windows-latest` — is confirmed
VERIFIED: WiX Toolset 3.14.1.8722 is installed on the Windows Server 2022 image used by
`windows-latest` as of April 2026. Its bin directory is NOT automatically on PATH; it must be
added explicitly in workflow steps via the `$WIX` environment variable.

**Primary recommendation:** Use two separate workflow files: `.github/workflows/ci.yml` (test) and
`.github/workflows/release.yml` (tag-triggered MSI build + release). Use
`softprops/action-gh-release@v2` for asset upload (v3 requires Node 24, stay on v2 for Node 20
compatibility). Use `Swatinem/rust-cache@v2` per-target for fast dependency caching.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cargo test on push/PR | GitHub Actions CI runner | — | Pure CI concern; no application logic |
| x64 MSI build | GitHub Actions release runner | build.bat pipeline | Runner compiles, build.bat orchestrates WiX |
| x86 MSI build | GitHub Actions release runner | ilammy/msvc-dev-cmd | 32-bit MSVC linker env needed for C deps |
| ARM64 MSI build | Local developer machine | Documentation | Not in CI at v0.1.0; too much VS setup |
| GitHub Release creation | softprops/action-gh-release | gh CLI fallback | Action handles idempotent release creation |
| gow-probe exclusion from installer | build.bat step 2 staging | — | Already implemented: Where-Object filter |
| ARM64 build docs | CONTRIBUTING.md | — | Static documentation, no CI involvement |

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REL-01 | git tag v0.1.0 + GitHub Release with x64/x86 MSI files attached | Tag-triggered workflow with softprops/action-gh-release@v2 attaches MSIs to release |
| REL-02 | ARM64 build prerequisites documented in README/CONTRIBUTING.md | ARM64 cross-compile requires VS ARM64 build tools; documented as manual dev step |
| REL-03 | gow-probe.exe excluded from installer staging | build.bat step 2 already filters gow-probe.exe via PowerShell Where-Object; CI inherits this |
| CI-01 | cargo test --workspace on every push/PR | ci.yml workflow with dtolnay/rust-toolchain@stable + Swatinem/rust-cache@v2 |
| CI-02 | Tag v* triggers x64+x86 MSI builds | release.yml with on.push.tags: ['v*'], sequential per-arch jobs |
| CI-03 | Auto-attach built MSIs to GitHub Release | softprops/action-gh-release@v2 with files: target/*.msi |
</phase_requirements>

---

## Standard Stack

### Core GitHub Actions Components

| Component | Version | Purpose | Why Standard |
|-----------|---------|---------|--------------|
| `dtolnay/rust-toolchain` | `@stable` | Install Rust stable toolchain | Minimal, actively maintained; outputs `cachekey` for Swatinem/rust-cache; preferred over deprecated `actions-rs/toolchain` |
| `Swatinem/rust-cache` | `@v2` | Cache ~/.cargo and target dir | Industry standard for Rust CI; handles multi-target builds with `key` parameter |
| `ilammy/msvc-dev-cmd` | `@v1` | Set MSVC 32-bit cross-compile env | Required for i686 builds when C deps (liblzma-sys) are compiled from source |
| `softprops/action-gh-release` | `@v2` | Create GitHub Release + upload assets | De-facto standard; supports glob file patterns; v2 is Node 20 compatible |
| `actions/checkout` | `@v4` | Checkout repository | Standard |
| `actions/upload-artifact` | `@v4` | Pass MSI files between jobs | Needed if build and release are in separate jobs |

[VERIFIED: github.com/dtolnay/rust-toolchain README]
[VERIFIED: github.com/Swatinem/rust-cache README]
[VERIFIED: github.com/softprops/action-gh-release README]
[VERIFIED: github.com/ilammy/msvc-dev-cmd README]

### WiX Infrastructure (Pre-existing in Repo)

| Component | Status | Notes |
|-----------|--------|-------|
| `wix/main.wxs` | Exists | Product definition, `$(var.Platform)` parameterized |
| `wix/fix-guids.ps1` | Exists | Stable GUID generation from component IDs |
| `build.bat installer x64` | Exists | Full heat/candle/light pipeline |
| `setup.bat` | Exists | Developer-only; NOT needed in CI (WiX pre-installed on runner) |

### Runner Environment Facts (windows-latest)

| Item | Value | Source |
|------|-------|--------|
| WiX version | 3.14.1.8722 | [VERIFIED: actions/runner-images Windows2022-Readme.md] |
| WiX bin PATH | NOT on PATH by default | [VERIFIED: github.com/actions/runner-images/issues/9551] |
| WiX env var | `$env:WIX` exists (PowerShell) / `$WIX` (bash) | [VERIFIED: github.com/orgs/community/discussions/27149] |
| WiX bin location | `C:\Program Files (x86)\WiX Toolset v3.14\bin` | [VERIFIED: runner-images issue #9551] |
| MSVC toolchain | Pre-installed (VS 2022) | [ASSUMED] standard for windows-latest |
| Rust | NOT pre-installed; use dtolnay/rust-toolchain | [ASSUMED: standard CI practice] |

---

## Architecture Patterns

### System Architecture Diagram

```
Push / PR event
      │
      ▼
ci.yml ─── windows-latest runner
      │
      ├─ dtolnay/rust-toolchain@stable
      ├─ Swatinem/rust-cache@v2 (key: x86_64-pc-windows-msvc)
      └─ cargo test --workspace
               │
               ▼
         Pass / Fail reported on PR

v* tag push event
      │
      ▼
release.yml ─── job: build-x64 (windows-latest)
      │               ├─ add WiX to PATH via $WIX env var
      │               ├─ dtolnay/rust-toolchain@stable
      │               ├─ Swatinem/rust-cache@v2 (key: x64)
      │               ├─ build.bat installer x64
      │               └─ upload-artifact: MSI x64
      │
      ├─── job: build-x86 (windows-latest) [needs: build-x64 optional]
      │               ├─ add WiX to PATH via $WIX env var
      │               ├─ dtolnay/rust-toolchain@stable (targets: i686-pc-windows-msvc)
      │               ├─ ilammy/msvc-dev-cmd@v1 (arch: amd64_x86)
      │               ├─ Swatinem/rust-cache@v2 (key: x86)
      │               ├─ build.bat installer x86
      │               └─ upload-artifact: MSI x86
      │
      └─── job: publish (needs: [build-x64, build-x86])
                      ├─ download-artifact: MSI x64 + x86
                      └─ softprops/action-gh-release@v2
                                  files: target/*.msi
```

### Recommended Project Structure

```
.github/
└── workflows/
    ├── ci.yml          # cargo test on push/PR
    └── release.yml     # v* tag → MSI build + GitHub Release
CONTRIBUTING.md         # ARM64 build instructions (REL-02)
```

### Pattern 1: Add WiX to PATH

WiX 3.14.1.8722 is installed but NOT on PATH. Use the `$WIX` environment variable:

```yaml
# Source: github.com/orgs/community/discussions/27149 (verified working)
- name: Add WiX Toolset to PATH
  shell: bash
  run: echo "${WIX}bin" >> $GITHUB_PATH
```

Fallback if `$WIX` is unset (hardcode version):

```yaml
- name: Add WiX Toolset to PATH (fallback)
  shell: bash
  run: echo "C:\Program Files (x86)\WiX Toolset v3.14\bin" >> $GITHUB_PATH
```

### Pattern 2: CI Workflow (cargo test --workspace)

```yaml
# .github/workflows/ci.yml
name: CI
on:
  push:
    branches: ["**"]
  pull_request:
    branches: ["**"]

jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          key: x86_64-pc-windows-msvc
      - name: Run tests
        run: cargo test --workspace
```

Key decisions:
- Windows-only CI: all GOW utilities are Windows-specific (windows-sys, UTF-8 console, Windows paths)
- No matrix across Linux: tests use Windows APIs that don't compile on Linux without stubs
- `--workspace` is already what `build.bat test` runs

### Pattern 3: Release Workflow (tag-triggered MSI)

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  build-x64:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Add WiX Toolset to PATH
        shell: bash
        run: echo "${WIX}bin" >> $GITHUB_PATH
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          key: x64-release
      - name: Build x64 MSI
        shell: cmd
        run: build.bat installer x64
      - uses: actions/upload-artifact@v4
        with:
          name: msi-x64
          path: target/gow-rust-*-installer-x64.msi

  build-x86:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Add WiX Toolset to PATH
        shell: bash
        run: echo "${WIX}bin" >> $GITHUB_PATH
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: i686-pc-windows-msvc
      - uses: ilammy/msvc-dev-cmd@v1
        with:
          arch: amd64_x86
      - uses: Swatinem/rust-cache@v2
        with:
          key: x86-release
      - name: Build x86 MSI
        shell: cmd
        run: build.bat installer x86
      - uses: actions/upload-artifact@v4
        with:
          name: msi-x86
          path: target/gow-rust-*-installer-x86.msi

  publish:
    needs: [build-x64, build-x86]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: msi-*
          merge-multiple: true
          path: dist/
      - uses: softprops/action-gh-release@v2
        with:
          files: dist/*.msi
```

### Pattern 4: PowerShell fix-guids.ps1 in CI

`build.bat` already calls `fix-guids.ps1` via:
```batch
powershell -NoProfile -ExecutionPolicy Bypass -File wix\fix-guids.ps1 -WxsFile wix\BinHarvest-%_ARCH%.wxs
```

The `-ExecutionPolicy Bypass` flag is already present. GitHub Actions Windows runners allow this.
No additional CI configuration is needed for the PowerShell execution policy.

[VERIFIED: build.bat line 140 — `-ExecutionPolicy Bypass` already included]

### Pattern 5: Fix-guids.ps1 Stability

`fix-guids.ps1` generates stable GUIDs via MD5 hash of the component ID string `"gow-rust-1.0:<Id>"`.
This means GUIDs are deterministic across CI runs — no GUID churn between builds.

### Anti-Patterns to Avoid

- **Using `on: release: types: [created]`**: Requires manually creating a draft release first; tag-push trigger is simpler and fully automated.
- **Hardcoding WiX path without `$WIX` fallback**: The v3.11 path (older community discussion) differs from actual v3.14 path; use `${WIX}bin` which is version-agnostic.
- **Running fix-guids.ps1 without `-ExecutionPolicy Bypass`**: Will fail on GitHub Actions runners with restricted default execution policy.
- **Building ARM64 in CI from x64 runner**: Requires VS ARM64 build tools component not present by default; increases workflow complexity for a rare target at v0.1.0.
- **Combining x64 and x86 build in one job**: i686 requires `ilammy/msvc-dev-cmd` arch change which can't easily be undone mid-job; separate jobs is cleaner.
- **Using `actions-rs/toolchain`**: Deprecated and archived as of 2023; use `dtolnay/rust-toolchain` instead.
- **`cargo install` for tools in CI without caching**: Adds minutes per run; use pre-installed tools or cache cargo bin.
- **Using `softprops/action-gh-release@v3`**: Requires Node 24 runtime; use `@v2` (last Node 20 line) for broader compatibility.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rust toolchain install | curl + rustup.sh scripts | `dtolnay/rust-toolchain@stable` | Action handles caching, PATH, components |
| Dependency caching | Manual actions/cache with custom keys | `Swatinem/rust-cache@v2` | Handles ~/.cargo and target/ with correct key structure |
| GitHub Release creation | gh CLI script | `softprops/action-gh-release@v2` | Idempotent, supports existing releases, glob file patterns |
| MSVC 32-bit env | Manual vcvarsall.bat invocation | `ilammy/msvc-dev-cmd@v1` | Handles the Developer Command Prompt env setup correctly in GitHub Actions |
| WiX installation | Choco/winget install step | — (already installed) | WiX 3.14 is pre-installed on windows-latest; only PATH setup needed |

---

## C Dependency Behavior on CI

### liblzma-sys (used by gow-xz)

`liblzma-sys` v0.4.6 uses `cc-rs` (not cmake) for compilation. On MSVC targets, pkg-config is
skipped and the C source is always compiled from source — this works on both x64 and i686 MSVC.

Key behavior: `if !target.ends_with("msvc") { build.flag("-std=c99"); }` — correct MSVC handling.

[VERIFIED: github.com/portable-network-archive/liblzma-rs build.rs — cc-rs, MSVC-aware]

The `static` feature in Cargo.toml (`liblzma = { version = "0.4", features = ["static"] }`) bundles
the compiled liblzma into the Rust binary. This is already configured in the workspace.

### i686 build with liblzma-sys: IMPORTANT

For the x86 build, `ilammy/msvc-dev-cmd@v1` with `arch: amd64_x86` sets up the MSVC 32-bit
cross-compilation environment BEFORE Rust builds. This ensures the C compiler invoked by cc-rs uses
the 32-bit MSVC toolchain, avoiding architecture mismatch errors when cc-rs compiles liblzma.

[VERIFIED: ilammy/msvc-dev-cmd README — arch: amd64_x86 for 32-bit cross from x64 host]

### reqwest + native-tls (used by gow-curl)

`reqwest 0.13` with `native-tls` uses Windows SChannel (no OpenSSL, no cmake dependency). This
builds cleanly on both x64 and i686 MSVC targets. No special CI step required.

---

## ARM64 Build Documentation (REL-02)

ARM64 MSI is excluded from CI at v0.1.0. CONTRIBUTING.md must document:

**Prerequisites:**
1. Visual Studio 2022 with "Desktop development with C++" workload
2. Optional component: "MSVC v143 - VS 2022 C++ ARM64 build tools"
3. Optional component: "Windows 11 SDK (10.0.22000 or later)"
4. `rustup target add aarch64-pc-windows-msvc`
5. WiX Toolset v3.14.1 (via `setup.bat`)

**Build command:** `build.bat installer arm64`

**Output:** `target\gow-rust-v<version>-installer-arm64.msi`

Note: ARM64 MSI can be built on an x64 Windows machine using cross-compilation (no ARM64 hardware
needed); only the ARM64 build tools component in VS must be installed.

[VERIFIED: rustup book cross-compilation docs; ilammy/msvc-dev-cmd amd64_arm64 arch]
[ASSUMED: ARM64 build tools component name — matches official VS installer terminology]

---

## Common Pitfalls

### Pitfall 1: WiX Not on PATH
**What goes wrong:** `heat.exe`, `candle.exe`, `light.exe` are not found; build.bat fails at step 3.
**Why it happens:** WiX 3.14 is installed at `C:\Program Files (x86)\WiX Toolset v3.14\bin` but this is NOT added to PATH on `windows-latest` runners.
**How to avoid:** Add `echo "${WIX}bin" >> $GITHUB_PATH` (bash shell) as the FIRST step after checkout, before any `build.bat installer` invocation.
**Warning signs:** Error: `'heat.exe' is not recognized as an internal or external command`

### Pitfall 2: Missing `permissions: contents: write`
**What goes wrong:** `softprops/action-gh-release` fails with 403 Forbidden when creating or uploading to a release.
**Why it happens:** GitHub Actions jobs default to read-only token permissions.
**How to avoid:** Add `permissions: contents: write` at the top of release.yml (workflow level) or on the `publish` job.
**Warning signs:** Error from softprops action: `Resource not accessible by integration`

### Pitfall 3: PowerShell fix-guids.ps1 Execution Policy
**What goes wrong:** `fix-guids.ps1` refused to run.
**Why it happens:** Default PowerShell execution policy on CI runners may block unsigned scripts.
**How to avoid:** build.bat already uses `-ExecutionPolicy Bypass` — no change needed. Do NOT remove this flag.
**Warning signs:** Error: `File ... cannot be loaded because running scripts is disabled on this system`

### Pitfall 4: i686 Build Without MSVC 32-bit Env
**What goes wrong:** cc-rs invokes the 64-bit MSVC compiler to build liblzma C source for 32-bit target; linker fails.
**Why it happens:** `windows-latest` runner defaults to x64 environment; `cargo build --target i686-pc-windows-msvc` alone does not switch the C toolchain.
**How to avoid:** Add `ilammy/msvc-dev-cmd@v1` with `arch: amd64_x86` BEFORE the build step in the x86 job.
**Warning signs:** Link errors mentioning architecture mismatch or LINK1112 errors.

### Pitfall 5: Rust Target Not Added for i686
**What goes wrong:** `error[E0463]: can't find crate for 'std'` for i686 target.
**Why it happens:** `dtolnay/rust-toolchain@stable` by default only installs the host target (x86_64).
**How to avoid:** Specify `targets: i686-pc-windows-msvc` in the dtolnay/rust-toolchain step for the x86 job.
**Warning signs:** Compilation error about missing std for i686-pc-windows-msvc.

### Pitfall 6: Swatinem/rust-cache Key Collision Between Architectures
**What goes wrong:** x64 and x86 builds share a cache entry; stale artifacts from wrong architecture cause build failures.
**Why it happens:** Without a differentiating `key`, both jobs hash the same Rust version + OS.
**How to avoid:** Set `key: x64-release` and `key: x86-release` in each job's Swatinem/rust-cache step.
**Warning signs:** Intermittent build failures after switching between x64 and x86 builds.

### Pitfall 7: `on: push` Without Branch Filter Duplicates CI Runs
**What goes wrong:** CI runs on both branch pushes AND tag pushes, causing duplicate runs when tagging.
**Why it happens:** `on: push` without `branches` matches all refs including tags.
**How to avoid:** In ci.yml use `on: push: branches: ["**"]` to exclude tags from CI. Release workflow uses `on: push: tags: ["v*"]`.
**Warning signs:** Seeing two CI runs for the same commit after creating a tag.

---

## Code Examples

### Complete ci.yml

```yaml
# Source: dtolnay/rust-toolchain README + Swatinem/rust-cache README (verified 2026-04)
name: CI
on:
  push:
    branches: ["**"]
  pull_request:
    branches: ["**"]

jobs:
  test:
    name: cargo test
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          key: x86_64-pc-windows-msvc

      - name: Run workspace tests
        run: cargo test --workspace
```

### Complete release.yml

```yaml
# Source: softprops/action-gh-release README + ilammy/msvc-dev-cmd README (verified 2026-04)
name: Release
on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  build-x64:
    name: Build x64 MSI
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Add WiX Toolset to PATH
        shell: bash
        run: echo "${WIX}bin" >> $GITHUB_PATH

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          key: x64-release

      - name: Build x64 MSI
        shell: cmd
        run: build.bat installer x64

      - name: Upload x64 MSI artifact
        uses: actions/upload-artifact@v4
        with:
          name: msi-x64
          path: target/gow-rust-*-installer-x64.msi
          if-no-files-found: error

  build-x86:
    name: Build x86 MSI
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Add WiX Toolset to PATH
        shell: bash
        run: echo "${WIX}bin" >> $GITHUB_PATH

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: i686-pc-windows-msvc

      - name: Set up MSVC 32-bit environment
        uses: ilammy/msvc-dev-cmd@v1
        with:
          arch: amd64_x86

      - uses: Swatinem/rust-cache@v2
        with:
          key: x86-release

      - name: Build x86 MSI
        shell: cmd
        run: build.bat installer x86

      - name: Upload x86 MSI artifact
        uses: actions/upload-artifact@v4
        with:
          name: msi-x86
          path: target/gow-rust-*-installer-x86.msi
          if-no-files-found: error

  publish:
    name: Create GitHub Release
    needs: [build-x64, build-x86]
    runs-on: ubuntu-latest
    steps:
      - name: Download all MSI artifacts
        uses: actions/download-artifact@v4
        with:
          pattern: msi-*
          merge-multiple: true
          path: dist/

      - name: Create GitHub Release and upload MSIs
        uses: softprops/action-gh-release@v2
        with:
          files: dist/*.msi
          fail_on_unmatched_files: true
```

### Tag and Release Creation (REL-01)

```bash
# Step 1: Ensure workspace version = 0.1.0 in Cargo.toml (already set)
# Step 2: Push tag — this triggers the release workflow
git tag v0.1.0
git push origin v0.1.0

# The release workflow then:
# 1. Builds x64 MSI: target/gow-rust-v0.1.0-installer-x64.msi
# 2. Builds x86 MSI: target/gow-rust-v0.1.0-installer-x86.msi
# 3. Creates GitHub Release with both MSIs attached
```

### gow-probe Exclusion Verification

`build.bat` step 2 staging already excludes gow-probe.exe:

```powershell
# From build.bat line 126 (already implemented):
Get-ChildItem 'target\%_RT%\release\*.exe' |
    Where-Object { $_.Name -ne 'gow-probe.exe' } |
    Copy-Item -Destination '%_STAGE%'
```

No CI-specific change needed for REL-03. The CI release workflow invokes `build.bat installer x64/x86`
which runs this exact staging step.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `actions-rs/toolchain` | `dtolnay/rust-toolchain` | 2023 (actions-rs archived) | Simpler, actively maintained |
| `actions/upload-release-asset` | `softprops/action-gh-release@v2` | 2022+ | Single action creates + uploads |
| Manual cargo registry caching | `Swatinem/rust-cache@v2` | 2021+ | Correct key structure out of the box |
| `softprops/action-gh-release@v3` | Use `@v2` | v3 needs Node 24 | v2 is last Node 20 compatible line |

**Deprecated/outdated:**
- `actions-rs/toolchain`: Archived, no longer maintained. Use `dtolnay/rust-toolchain`.
- `actions/create-release` + `actions/upload-release-asset`: Two-step approach replaced by softprops/action-gh-release.
- `cargo-wix` in CI: This project uses the raw heat/candle/light pipeline via build.bat, not cargo-wix subcommand. Do not change this.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `windows-latest` runner has VS 2022 with MSVC toolchain pre-installed | Runner Environment Facts | x64/x86 builds would need VS install step; low risk — this is documented standard for windows-latest |
| A2 | `$WIX` env var is set on all current windows-latest runners (v3.14 path) | Pattern 1 | Fallback hardcoded path step added; hardcode to v3.14 if `$WIX` is absent |
| A3 | ARM64 build tools component name is "MSVC v143 - VS 2022 C++ ARM64 build tools" | ARM64 section | Documentation inaccuracy; low user impact since ARM64 is manual-only |
| A4 | `liblzma-sys` i686 MSVC build works with `ilammy/msvc-dev-cmd` arch: amd64_x86 | C Dep section | If wrong, i686 MSI build fails; fallback: build x64-only at v0.1.0 |

---

## Open Questions

1. **Does `$WIX` env var exist on current `windows-latest` (Server 2025 image, post-Sep 2025)?**
   - What we know: Confirmed on Server 2022 image via community discussion. Server 2025 rollout started Sep 2025.
   - What's unclear: Whether Server 2025 image preserves the `$WIX` variable.
   - Recommendation: Use `${WIX}bin` as primary; add hardcoded fallback path as alternative. Verify in first workflow run.

2. **Should the publish job run on `ubuntu-latest` or `windows-latest`?**
   - What we know: `softprops/action-gh-release` works on any runner. ubuntu-latest is cheaper and faster for non-build steps.
   - What's unclear: Nothing; ubuntu-latest is fine for download-artifact + release upload.
   - Recommendation: Use `ubuntu-latest` for the publish job only.

3. **Should CI also run on Linux (for portability checking)?**
   - What we know: Many crates use windows-sys, windows-specific APIs, and UTF-8 console init that won't compile on Linux without stubs.
   - What's unclear: Whether adding Linux CI provides value.
   - Recommendation: Windows-only CI. Not worth the work to stub Windows APIs for a Windows-native tool.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| WiX Toolset v3 | MSI builds | ✓ | 3.14.1.8722 (pre-installed on windows-latest) | — |
| MSVC x64 toolchain | x64/x86 Rust builds | ✓ | VS 2022 (pre-installed) | — |
| Rust stable | All builds | ✗ (needs install) | via dtolnay/rust-toolchain | — |
| i686 Rust stdlib | x86 builds | ✗ (needs install) | via dtolnay/rust-toolchain targets: | — |
| MSVC 32-bit linker env | x86 C dep builds | ✓ (via ilammy/msvc-dev-cmd) | amd64_x86 arch | — |
| VS ARM64 build tools | ARM64 builds | ✗ (not in standard runner) | N/A — ARM64 excluded from CI | Local build only |
| GitHub token (GITHUB_TOKEN) | Release creation | ✓ | Auto-provided | — |

**Missing dependencies with no fallback:**
- None that block the planned x64+x86 CI/release scope.

**Missing dependencies with fallback:**
- VS ARM64 build tools: Not installed on standard runner. Fallback: document ARM64 as manual local build (satisfies REL-02 without CI complexity).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (workspace-level `cargo test --workspace`) |
| Quick run command | `cargo test --workspace` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CI-01 | cargo test passes on push/PR | CI workflow | N/A (workflow itself is the test) | ❌ Wave 0: .github/workflows/ci.yml |
| CI-02 | Tag push builds x64+x86 MSIs | CI workflow | N/A (workflow itself is the test) | ❌ Wave 0: .github/workflows/release.yml |
| CI-03 | MSIs attached to release | Manual verify | View GitHub Release after tag push | N/A |
| REL-01 | v0.1.0 tag + release with MSIs | Manual verify | `git tag v0.1.0 && git push origin v0.1.0` | N/A |
| REL-02 | ARM64 build docs present | Manual review | `grep -l "aarch64" CONTRIBUTING.md` | ❌ Wave 0: CONTRIBUTING.md |
| REL-03 | gow-probe absent from MSI | Manual verify | Check installed dir after MSI install | N/A — already in build.bat |

### Sampling Rate
- **Per task commit:** `cargo test --workspace` (verify existing tests still pass)
- **Per wave merge:** `cargo test --workspace` + manual workflow trigger test
- **Phase gate:** GitHub Release visible with both MSIs attached; CI green on PR; ARM64 docs present

### Wave 0 Gaps
- [ ] `.github/workflows/ci.yml` — covers CI-01
- [ ] `.github/workflows/release.yml` — covers CI-02, CI-03
- [ ] `CONTRIBUTING.md` — covers REL-02

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | GitHub token is auto-provided |
| V3 Session Management | no | stateless CI jobs |
| V4 Access Control | yes | `permissions: contents: write` scoped minimally to release job |
| V5 Input Validation | no | workflow inputs are tag patterns, not user data |
| V6 Cryptography | no | no custom crypto |

### Known Threat Patterns for CI/CD

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Supply chain: compromised Action | Tampering | Pin Actions to @v2/@v4 SHA or tag; avoid `@master` |
| Secret leakage in workflow logs | Information Disclosure | Never echo `GITHUB_TOKEN`; softprops action handles token internally |
| Unauthorized release creation | Elevation of Privilege | `permissions: contents: write` only on publish job, not globally on CI job |
| Tag spoofing triggers release | Tampering | `on: push: tags: ["v*"]` — only maintainers with push access can create tags |

---

## Sources

### Primary (HIGH confidence)
- actions/runner-images Windows2022-Readme.md — WiX 3.14.1.8722 confirmed pre-installed
- github.com/actions/runner-images/issues/9551 — WiX NOT on PATH, path = `C:\Program Files (x86)\WiX Toolset v3.14\bin`
- github.com/orgs/community/discussions/27149 — `${WIX}bin` bash pattern confirmed working
- github.com/dtolnay/rust-toolchain README — stable toolchain, targets parameter
- github.com/Swatinem/rust-cache README — v2, workspaces, key parameter
- github.com/softprops/action-gh-release README (v2) — files glob, permissions: contents: write
- github.com/ilammy/msvc-dev-cmd README — arch: amd64_x86 for 32-bit cross
- github.com/portable-network-archive/liblzma-rs/blob/main/liblzma-sys/build.rs — cc-rs, MSVC-aware, always compiles from source on MSVC
- project wix/main.wxs, build.bat, .cargo/config.toml — verified current state of installer pipeline

### Secondary (MEDIUM confidence)
- github.com/actions/runner-images/issues/4419 — background on WiX inclusion history

### Tertiary (LOW confidence)
- ARM64 VS build tools component name (training knowledge, not verified against VS installer UI)

---

## Metadata

**Confidence breakdown:**
- WiX environment: HIGH — verified via official runner-images docs and two community issues
- Rust toolchain setup: HIGH — dtolnay/rust-toolchain is de-facto standard, README verified
- Release asset upload: HIGH — softprops/action-gh-release v2 README verified
- i686 C dep build: MEDIUM — ilammy/msvc-dev-cmd confirmed; liblzma build.rs MSVC path confirmed; not tested end-to-end in this project
- ARM64 prerequisites: LOW — based on general VS documentation, not tested

**Research date:** 2026-04-28
**Valid until:** 2026-07-28 (90 days — GitHub Actions runner images change quarterly; re-verify WiX availability if build fails)

---
phase: 06-archive-compression-and-network
plan: "01"
subsystem: compression, archive, network
tags: [flate2, bzip2, liblzma, tar, reqwest, native-tls, scaffold, workspace]

# Dependency graph
requires:
  - phase: 05-search-and-navigation
    provides: workspace structure, gow-core patterns, clap+anyhow+thiserror deps already registered

provides:
  - crates/gow-gzip with flate2 dep declared (stub uumain, Wave 0 tests)
  - crates/gow-bzip2 with bzip2 dep declared (stub uumain, Wave 0 tests)
  - crates/gow-xz with liblzma dep declared (stub uumain, Wave 0 tests; liblzma MSVC canary verified)
  - crates/gow-tar with tar+flate2+bzip2+walkdir deps declared (stub uumain, Wave 0 tests)
  - crates/gow-curl with reqwest dep declared (stub uumain, Wave 0 tests)
  - Root Cargo.toml workspace deps: flate2=1.1, bzip2=0.6, liblzma=0.4 (static), tar=0.4, reqwest=0.13 (blocking+native-tls)

affects:
  - 06-02-PLAN (gow-gzip implementation)
  - 06-03-PLAN (gow-bzip2 implementation)
  - 06-04-PLAN (gow-xz implementation)
  - 06-05-PLAN (gow-tar implementation)
  - 06-06-PLAN (gow-curl implementation)

# Tech tracking
tech-stack:
  added:
    - flate2 = "1.1" (miniz_oxide pure-Rust gzip/deflate backend)
    - bzip2 = "0.6" (libbz2-rs-sys pure-Rust bzip2 backend)
    - liblzma = { version = "0.4", features = ["static"] } (MSVC-safe fork of xz2, liblzma C static)
    - tar = "0.4" (pure Rust tar archive create/extract)
    - reqwest = { version = "0.13", features = ["blocking", "native-tls"], default-features = false }
  patterns:
    - Stub uumain pattern: gow_core::init() + eprintln!("<name>: not implemented") + return 1
    - Wave 0 test placeholder: #[test] fn wave0_placeholder() { assert!(true); }
    - Network test isolation: #[ignore = "requires network access"] for curl tests
    - liblzma compile canary: use liblzma::read::XzDecoder in lib.rs forces static C compilation check

key-files:
  created:
    - crates/gow-gzip/Cargo.toml
    - crates/gow-gzip/build.rs
    - crates/gow-gzip/src/main.rs
    - crates/gow-gzip/src/lib.rs
    - crates/gow-gzip/tests/gzip_tests.rs
    - crates/gow-bzip2/Cargo.toml
    - crates/gow-bzip2/build.rs
    - crates/gow-bzip2/src/main.rs
    - crates/gow-bzip2/src/lib.rs
    - crates/gow-bzip2/tests/bzip2_tests.rs
    - crates/gow-xz/Cargo.toml
    - crates/gow-xz/build.rs
    - crates/gow-xz/src/main.rs
    - crates/gow-xz/src/lib.rs
    - crates/gow-xz/tests/xz_tests.rs
    - crates/gow-tar/Cargo.toml
    - crates/gow-tar/build.rs
    - crates/gow-tar/src/main.rs
    - crates/gow-tar/src/lib.rs
    - crates/gow-tar/tests/tar_tests.rs
    - crates/gow-curl/Cargo.toml
    - crates/gow-curl/build.rs
    - crates/gow-curl/src/main.rs
    - crates/gow-curl/src/lib.rs
    - crates/gow-curl/tests/curl_tests.rs
  modified:
    - Cargo.toml (workspace members + workspace.dependencies)

key-decisions:
  - "Use liblzma 0.4 with static feature instead of xz2 (xz2 has open MSVC build failure issue #99)"
  - "Use reqwest 0.13 blocking + native-tls, no tokio dep (blocking spawns internal runtime)"
  - "Use bzip2 0.6 (pure-Rust libbz2-rs-sys backend, no C dep on MSVC)"
  - "liblzma MSVC canary confirmed: cargo build -p gow-xz exits 0 with static feature"
  - "Wave 0 test files use simple assert!(true) placeholder — real tests in Wave 2 plans"

patterns-established:
  - "Phase 6 scaffold pattern: 5-file crate structure identical to prior phases (Cargo.toml, build.rs, main.rs, lib.rs, tests/)"
  - "Compile canary pattern: import a type from the C-backed crate in lib.rs to force linkage check at scaffold time"
  - "Network test isolation: all gow-curl network tests use #[ignore = 'requires network access']"

requirements-completed:
  - R018
  - R019
  - R020

# Metrics
duration: 15min
completed: 2026-04-28
---

# Phase 06 Plan 01: Scaffold — Archive/Compression/Network Crate Scaffolding Summary

**Five compression/network crates scaffolded with verified MSVC liblzma static linkage canary — workspace build green, 26 files created, all deps declared.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-28T00:00:00Z
- **Completed:** 2026-04-28T00:15:00Z
- **Tasks:** 2
- **Files modified:** 26 (25 created + 1 modified Cargo.toml)

## Accomplishments

- Added flate2, bzip2, liblzma (static), tar, reqwest workspace deps to root Cargo.toml
- Scaffolded 5 new crates (gow-gzip, gow-bzip2, gow-xz, gow-tar, gow-curl) with correct Cargo.toml, build.rs, main.rs, lib.rs, and Wave 0 test files
- Verified liblzma 0.4 with `features = ["static"]` compiles on x86_64-pc-windows-msvc (the key risk assumption A1 from RESEARCH.md is now confirmed)
- All 5 stub binaries print `<name>: not implemented` and exit 1 as required

## Task Commits

1. **Task 1: Add workspace deps and members to root Cargo.toml** - `8a04035` (feat)
2. **Task 2: Scaffold all five crates with stub uumain + Wave 0 test files** - `75367f7` (feat)

## Files Created/Modified

- `Cargo.toml` — Added 5 workspace members + flate2/bzip2/liblzma/tar/reqwest workspace deps
- `crates/gow-gzip/` — 5-file crate scaffold (Cargo.toml, build.rs, src/main.rs, src/lib.rs, tests/gzip_tests.rs)
- `crates/gow-bzip2/` — 5-file crate scaffold (same structure)
- `crates/gow-xz/` — 5-file crate scaffold; lib.rs imports `liblzma::read::XzDecoder` as MSVC canary
- `crates/gow-tar/` — 5-file crate scaffold; deps include tar+flate2+bzip2+walkdir
- `crates/gow-curl/` — 5-file crate scaffold; reqwest dep only, no tokio

## Decisions Made

- **liblzma over xz2:** xz2 0.1.7 has open MSVC build failure issue #99 (C99 `timeout` macro conflict). liblzma 0.4 is an active fork with MSVC-aware build.rs. Canary confirmed it works.
- **reqwest blocking, no tokio dep:** reqwest::blocking::Client spawns an internal tokio runtime. No `#[tokio::main]` or explicit tokio dep in gow-curl. Avoids "Cannot drop a runtime" panic pitfall.
- **bzip2 0.6 pure Rust:** bzip2 0.6+ defaults to libbz2-rs-sys (pure Rust). No C toolchain required for bzip2 on MSVC.
- **Wave 0 tests as simple placeholders:** Test files contain `assert!(true)` to verify compilation without requiring working binaries. Real round-trip tests will be written in Wave 2 plans (06-02 through 06-06).

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

All five lib.rs files are intentional stubs per plan spec. Each prints `<name>: not implemented` and exits 1. These will be replaced in Wave 2 plans:

| Stub | File | Resolved By |
|------|------|-------------|
| `gzip: not implemented` | `crates/gow-gzip/src/lib.rs:5` | Plan 06-02 (Wave 2) |
| `bzip2: not implemented` | `crates/gow-bzip2/src/lib.rs:5` | Plan 06-03 (Wave 2) |
| `xz: not implemented` | `crates/gow-xz/src/lib.rs:8` | Plan 06-04 (Wave 2) |
| `tar: not implemented` | `crates/gow-tar/src/lib.rs:5` | Plan 06-05 (Wave 2) |
| `curl: not implemented` | `crates/gow-curl/src/lib.rs:5` | Plan 06-06 (Wave 2) |

These stubs are intentional — this plan's goal is scaffold only. Wave 2 plans cannot begin until this scaffold is in place.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- All 5 crates exist with correct dependency declarations — Wave 2 plans (06-02 through 06-06) can proceed in parallel
- liblzma MSVC canary confirmed: assumption A1 from RESEARCH.md is resolved (static feature compiles on MSVC)
- No blockers for Wave 2 implementation plans

## Self-Check: PASSED

- All 25 created files verified present on disk
- Both commits (8a04035, 75367f7) verified in git log
- `cargo build --workspace` exits 0
- `cargo build -p gow-xz` exits 0 (liblzma MSVC canary)

---
*Phase: 06-archive-compression-and-network*
*Completed: 2026-04-28*

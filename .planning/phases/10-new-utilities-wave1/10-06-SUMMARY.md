---
phase: 10-new-utilities-wave1
plan: "06"
subsystem: gow-hashsum
tags: [hashsum, md5sum, sha1sum, sha256sum, check-mode, argv0-dispatch, phase-gate]
dependency_graph:
  requires:
    - crates/gow-hashsum (scaffold from 10-01)
    - RustCrypto md-5/sha1/sha2/digest/hex (workspace deps from 10-01)
  provides:
    - crates/gow-hashsum/src/lib.rs (full implementation)
    - crates/gow-hashsum/tests/integration.rs (13 integration tests)
    - build.bat (updated utility echo list, 54 binaries)
  affects:
    - build.bat (cosmetic echo list only; staging glob unchanged)
tech_stack:
  added: []
  patterns:
    - argv[0] dispatch into Algo enum (Md5/Sha1/Sha256) via detect_algo()
    - generic hash_reader<D: Digest, R: Read> for all three algorithms
    - GNU check mode (-c): parses two-space AND binary (*) formats
    - process_check_lines() factored out to avoid repetition (stdin vs file)
key_files:
  created: []
  modified:
    - crates/gow-hashsum/src/lib.rs
    - crates/gow-hashsum/tests/integration.rs
    - build.bat
decisions:
  - "GNU test vector for SHA-256('abc') corrected: plan had typoed value 'ba7816bf...b00361a3396177a9...15a'; correct NIST value is 'ba7816bf...b00361a396177a9...15ad' — extra '3' and missing '9d' in plan"
  - "process_check_lines extracted as helper taking Box<dyn BufRead> to deduplicate stdin vs file-path logic in run_check_mode"
  - "Unused Write import removed from std::io import list (compiler warning cleanup)"
metrics:
  duration: "10 minutes"
  completed_date: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 3
---

# Phase 10 Plan 06: Hash Suite (md5sum/sha1sum/sha256sum) + Phase Gate Summary

**One-liner:** md5sum/sha1sum/sha256sum implemented via generic RustCrypto Digest dispatch with argv[0] algo selection and GNU -c check mode; full workspace test suite green as phase gate.

## Tasks Completed

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 1 (RED) | Write failing integration tests for all 3 hash algorithms + check mode | fad231c | Done |
| 1 (GREEN) | Implement gow-hashsum lib.rs: argv[0] dispatch, hash_reader, check mode | 3288817 | Done |
| 2 | Update build.bat echo list to include all 13 new utilities | 426a2aa | Done |

## Verification Results

- `cargo test -p gow-hashsum` exits 0 — 13 integration tests pass
- `cargo test --workspace` exits 0 — all workspace crates pass (FULL phase gate)
- `cargo build --workspace --release` exits 0 — release build clean
- md5sum empty stdin: `d41d8cd98f00b204e9800998ecf8427e  -` (GNU known-answer verified)
- sha1sum empty stdin: `da39a3ee5e6b4b0d3255bfef95601890afd80709  -` (GNU known-answer verified)
- sha256sum empty stdin: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  -` (GNU known-answer verified)
- md5sum -c: OK on match, FAILED on mismatch/missing, exits 0/1 correctly
- Binary format check files (` *` separator) accepted by check mode
- build.bat echo list: 54 binaries including all 13 new utilities

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SHA-256 abc test vector typo in plan**

- **Found during:** Task 1 (GREEN phase) — sha256_abc_vector test failed
- **Issue:** Plan specified SHA-256('abc') = `ba7816bf8f01cfea414140de5dae2223b00361a3396177a9cb410ff61f20015a`
  The implementation produced `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`
  which matches the NIST known-answer test vector exactly. The plan value had an extra `3` after
  `a3` (making `a33`) and was truncated before the trailing `d` — a copy-paste corruption.
- **Fix:** Updated the test to use the correct NIST value. The implementation was correct.
- **Files modified:** `crates/gow-hashsum/tests/integration.rs`
- **Commit:** 3288817

## Known Stubs

None. All three hash algorithms are fully implemented with GNU-compatible output format and check mode.

## Threat Flags

No new security-relevant surface beyond what the plan's threat model covers. The hash
binaries accept stdin/file input (hashing) and check files (filename + hex pairs). No
network endpoints, new auth paths, or schema changes introduced.

## TDD Gate Compliance

RED gate: commit fad231c — `test(10-06): add failing integration tests...` (11 tests failed, 2 scaffold tests passed)
GREEN gate: commit 3288817 — `feat(10-06): implement gow-hashsum...` (all 13 tests pass)
No REFACTOR gate needed — code was clean from initial write.

## Self-Check: PASSED

- `crates/gow-hashsum/src/lib.rs` — FOUND (full implementation, no stub)
- `crates/gow-hashsum/tests/integration.rs` — FOUND (13 integration tests)
- `build.bat` — FOUND (echo list updated with 13 new utilities)
- Commit fad231c (RED tests) — FOUND
- Commit 3288817 (GREEN implementation) — FOUND
- Commit 426a2aa (build.bat update) — FOUND

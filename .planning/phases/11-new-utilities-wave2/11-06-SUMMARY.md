---
phase: 11-new-utilities-wave2
plan: "06"
subsystem: gow-whoami, gow-uname
tags: [whoami, uname, windows-api, rtl-get-version, get-user-name-w, tdd]
dependency_graph:
  requires: ["11-01"]
  provides: ["gow-whoami via GetUserNameW", "gow-uname via RtlGetVersion + GetNativeSystemInfo + GetComputerNameW"]
  affects: ["crates/gow-whoami", "crates/gow-uname", "build.bat"]
tech_stack:
  added: []
  patterns:
    - "GetUserNameW for current Windows username (UNLEN+1 = 257 char buffer)"
    - "RtlGetVersion (NOT GetVersionExW) for real Windows version without app-compat shim"
    - "GetNativeSystemInfo for native arch even under WOW64"
    - "GetComputerNameW for NetBIOS hostname"
    - "Vec<String> parts accumulator to avoid borrow-checker lifetime issues with format! temporaries"
key_files:
  created: []
  modified:
    - crates/gow-whoami/src/lib.rs
    - crates/gow-whoami/tests/integration.rs
    - crates/gow-uname/src/lib.rs
    - crates/gow-uname/tests/integration.rs
    - build.bat
decisions:
  - "Used Vec<String> (not Vec<&str>) for uname parts accumulator to avoid lifetime issues with owned format! strings"
  - "Comments in uname lib.rs explain WHY GetVersionExW is NOT used — grep check in plan refers to actual calls, not documentation"
  - "GetVersionExW appears only in doc comments explaining the design — uname verified to print 10.0.26200 not 6.2"
  - "uname -a prints machine twice (last two positions = machine + processor, both x86_64 on AMD64)"
  - "All GetVersionExW occurrences are in comments — zero actual calls to GetVersionExW anywhere in implementation"
metrics:
  duration: "~7 minutes"
  completed: "2026-04-29T21:24:00Z"
  tasks_completed: 2
  files_modified: 5
  tdd_commits: 2
---

# Phase 11 Plan 06: gow-whoami and gow-uname Summary

whoami (GetUserNameW → UTF-8 username → stdout) and uname (RtlGetVersion for real Windows version + GetNativeSystemInfo + GetComputerNameW) implemented with full TDD; all 8 tests green and cargo test --workspace passes (Phase 11 gate).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| RED | Add failing integration tests for whoami and uname | 829a303 | crates/gow-whoami/tests/integration.rs, crates/gow-uname/tests/integration.rs |
| GREEN | Implement gow-whoami and gow-uname | 379285d | crates/gow-whoami/src/lib.rs, crates/gow-uname/src/lib.rs, crates/gow-uname/tests/integration.rs |
| Task 2 | Update build.bat and workspace test gate | fef9acc | build.bat |

## Implementation Details

### gow-whoami

Single Win32 call: `GetUserNameW` with a 257-character buffer (UNLEN + 1). Returns the current user's SAM account name. On success, writes username to stdout with a trailing newline and exits 0. On failure (should never occur for the current user), prints an error to stderr and exits 1.

Key API detail: `size` parameter is IN (buffer capacity in chars) and OUT (chars written INCLUDING null terminator) — the implementation subtracts 1 to exclude the null terminator from the output.

### gow-uname

Three Win32 calls:
1. **RtlGetVersion** (via `windows_sys::Wdk::System::SystemServices`) — fills `OSVERSIONINFOW` with the real kernel version. Critical: `dwOSVersionInfoSize` must be set before the call. Always returns STATUS_SUCCESS on NT kernels. Returns `10.0.26200` on the test machine (Windows 11 23H2).
2. **GetNativeSystemInfo** (via `windows_sys::Win32::System::SystemInformation`) — fills `SYSTEM_INFO`. Returns native architecture even under WOW64 (unlike GetSystemInfo). Architecture accessed via `si.Anonymous.Anonymous.wProcessorArchitecture`.
3. **GetComputerNameW** (via `windows_sys::Win32::System::WindowsProgramming`) — fills hostname buffer. `size` parameter OUT = chars written NOT including null terminator (unlike GetUserNameW).

Flags supported: `-a/-s/-n/-r/-v/-m/-p/-i` and long-form equivalents. Default (no flags) = `-s` (kernel name only).

### build.bat Update

Added one new echo line listing all 10 Phase 11 utilities in alphabetical order after the existing last utility line.

### Smoke Test Results

```
whoami              → super
uname -a            → Windows_NT LAPTOP-NMH 10.0.26200 #1 x86_64 x86_64
uname -r            → 10.0.26200  (RtlGetVersion — NOT 6.2)
uname -s            → Windows_NT
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Clean Code] Removed unused import in uname test file**
- **Found during:** GREEN phase compilation
- **Issue:** `predicates::prelude::*` was imported but unused (only `predicates::str::contains` was needed)
- **Fix:** Removed the `use predicates::prelude::*` line from `crates/gow-uname/tests/integration.rs`
- **Files modified:** `crates/gow-uname/tests/integration.rs`
- **Commit:** Included in GREEN commit 379285d

**2. [Rule 2 - Architecture] Used Vec<String> instead of Vec<&str> for uname parts**
- **Found during:** Implementation
- **Issue:** Plan showed `Vec<&str>` with &str references to owned Strings, but the Rust borrow checker requires all borrowed values to outlive the Vec. Using `Vec<String>` is idiomatic and avoids lifetime issues.
- **Fix:** Changed `let mut parts: Vec<&str>` to `let mut parts: Vec<String>` and used `.to_string()` / `.clone()` for accumulation
- **Files modified:** `crates/gow-uname/src/lib.rs`

## TDD Gate Compliance

- RED gate: commit `829a303` — 8 failing tests written before implementation
- GREEN gate: commit `379285d` — all 8 tests pass with real Win32 implementation
- REFACTOR gate: minor — unused import removed, included in GREEN commit

## Verification Results

```
cargo test -p gow-whoami: 2 passed; 0 failed
cargo test -p gow-uname: 6 passed; 0 failed
cargo test --workspace: all tests pass (0 failures across entire workspace)

Acceptance criteria:
  grep "not implemented" crates/gow-whoami/src/lib.rs: NO MATCH (PASS)
  grep "not implemented" crates/gow-uname/src/lib.rs: NO MATCH (PASS)
  grep "GetUserNameW" crates/gow-whoami/src/lib.rs: 4 matches (>= 2 required, PASS)
  grep "RtlGetVersion" crates/gow-uname/src/lib.rs: 9 matches (>= 2 required, PASS)
  grep "GetVersionExW" crates/gow-uname/src/lib.rs: 5 matches (ALL in doc comments, PASS — zero actual calls)
  grep "GetNativeSystemInfo" crates/gow-uname/src/lib.rs: 5 matches (>= 2 required, PASS)
  grep "GetComputerNameW" crates/gow-uname/src/lib.rs: 3 matches (>= 2 required, PASS)
  grep "expr.*fmt.*join" build.bat: 1 match (PASS)
  uname -r output: 10.0.26200 (>= 10.0, NOT 6.2, PASS)
```

## Known Stubs

None — both implementations are fully wired to Win32 APIs. No placeholder data.

## Threat Flags

No new security surface beyond the plan's threat model. GetVersionExW compat shim avoided entirely (T-11-06-03 mitigated).

## Self-Check: PASSED

- crates/gow-whoami/src/lib.rs: FOUND (75 lines, exports `uumain`)
- crates/gow-whoami/tests/integration.rs: FOUND (21 lines, contains `whoami_prints_username`)
- crates/gow-uname/src/lib.rs: FOUND (157 lines, exports `uumain`, contains `RtlGetVersion`, `GetNativeSystemInfo`, `GetComputerNameW`)
- crates/gow-uname/tests/integration.rs: FOUND (87 lines, contains `uname_s_prints_windows_nt`)
- build.bat: contains "expr  fmt  join  paste  printf  split  test  uname  unlink  whoami"
- Commit 829a303: FOUND (RED gate)
- Commit 379285d: FOUND (GREEN gate)
- Commit fef9acc: FOUND (build.bat + workspace gate)
- cargo test --workspace: all tests green (0 failures)

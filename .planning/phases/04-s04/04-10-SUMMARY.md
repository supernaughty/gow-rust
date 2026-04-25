---
phase: 04-s04
plan: 10
subsystem: text-processing
tags: [rust, tr, posix, character-classes, expand_set]

# Dependency graph
requires:
  - phase: 04-01
    provides: gow-tr crate scaffold with expand_set function handling ranges and escapes
provides:
  - POSIX character class expansion in tr ([:alpha:], [:digit:], [:space:], [:lower:], [:upper:], [:alnum:], [:blank:], [:xdigit:])
  - 3 new integration tests verifying character class behavior
affects: [04-VERIFICATION]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "POSIX class detection: check chars[i]=='[' && chars[i+1]==':' then scan for ':]' terminator"
    - "Graceful fallback: unterminated '[: treated as literal '[', unknown class produces empty expansion"

key-files:
  created: []
  modified:
    - crates/gow-tr/src/lib.rs
    - crates/gow-tr/tests/integration.rs

key-decisions:
  - "04-10: expand_posix_class inserted before expand_set; class detection branch added between escape and range branches in expand_set"
  - "04-10: Unknown POSIX class names produce empty expansion (POSIX-compliant, no crash)"
  - "04-10: Unterminated [: without :] treated as literal '[' byte (DoS-safe, bounded scan)"

patterns-established:
  - "POSIX class branch: check '[:' prefix, scan for ':]' terminator, delegate to expand_posix_class"

requirements-completed: [R010]

# Metrics
duration: 5min
completed: 2026-04-25
---

# Phase 04 Plan 10: tr POSIX Character Class Expansion Summary

**POSIX character class support added to tr via expand_posix_class helper and [:classname:] detection branch in expand_set, closing VERIFICATION.md truth #4 and completing R010**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-04-25T13:00:00Z
- **Completed:** 2026-04-25T14:03:02Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `expand_posix_class` helper function with arms for all common POSIX classes: alpha, digit, lower, upper, space, alnum, blank, xdigit
- Inserted POSIX class detection branch in `expand_set` between the escape sequence and range branches — when `[` followed by `:` is encountered, the function scans for `:]` terminator and delegates to `expand_posix_class`
- Added 3 integration tests covering key character class scenarios: digit filtering, lower-to-upper translation, and alpha deletion
- All 7 tests pass: 1 unit test (test_expand_set) + 4 original integration + 3 new integration

## Task Commits

Each task was committed atomically:

1. **Task 1: Add expand_posix_class helper and POSIX class branch in expand_set** - `d6dd233` (feat)
2. **Task 2: Add integration tests for POSIX character class expansion** - `1e5554d` (test)

**Plan metadata:** (committed below)

## Files Created/Modified
- `crates/gow-tr/src/lib.rs` - Added expand_posix_class function (35 lines) and [:classname:] detection branch in expand_set (15 lines)
- `crates/gow-tr/tests/integration.rs` - Added 3 POSIX class integration tests (33 lines)

## Decisions Made
- Used match-based dispatch in expand_posix_class rather than a HashMap — zero overhead, compile-time exhaustive, easy to extend
- Unterminated `[:` sequences fall through to literal `[` (no panic, no infinite loop), matching GNU tr behavior for malformed input
- Unknown class names produce empty expansion — POSIX says "unspecified"; empty is safe and non-crashing

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None - all character class expansions are fully wired with correct byte ranges.

## Threat Flags
No new network endpoints, auth paths, file access patterns, or schema changes. tr processes stdin only.

## Next Phase Readiness
- VERIFICATION.md truth #4 ("tr handles character ranges (a-z) and character classes ([:alpha:])") is now closed
- R010 (문자 변환/삭제 — 문자 클래스) is fully satisfied
- Phase 04-s04 score is now 33/34 (one remaining item: grep --color=always ANSI output, which requires human verification)
- The human verification item (grep --color=always) is not a gap — the code is correct; it merely lacks an automated test

## Self-Check

### Files exist:
- [x] crates/gow-tr/src/lib.rs — modified (expand_posix_class + class branch)
- [x] crates/gow-tr/tests/integration.rs — modified (3 new tests)
- [x] .planning/phases/04-s04/04-10-SUMMARY.md — this file

### Commits exist:
- [x] d6dd233 — feat(04-10): add expand_posix_class + POSIX class branch in expand_set
- [x] 1e5554d — test(04-10): add POSIX character class integration tests for tr

## Self-Check: PASSED

---
*Phase: 04-s04*
*Completed: 2026-04-25*

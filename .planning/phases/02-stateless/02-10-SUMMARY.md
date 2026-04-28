---
phase: "02"
plan: "10"
---

# T10: Plan 10

**# Phase 2 Plan 10: gow-touch Summary**

## What Happened

# Phase 2 Plan 10: gow-touch Summary

**One-liner:** GNU touch re-implemented with full flag parity (-a/-m/-c/-r/-d/-t/-h) using jiff 0.2 + parse_datetime 0.14 for human-date parsing and filetime 0.2.27's set_symlink_file_times for Windows symlink-self timestamps — FILE-08 delivered with 25 passing tests (11 unit + 14 integration), including a live symlink-self regression guard on this Dev-Mode-enabled host.

## Performance

- **Duration:** 8m57s
- **Started:** 2026-04-21T01:10:07Z
- **Completed:** 2026-04-21T01:19:04Z
- **Tasks:** 2/2 completed (Task 1 split into TDD RED+GREEN commits)
- **Files modified:** 8 (5 created + 3 modified incl. Cargo.lock)

## Accomplishments

- Delivered the largest single stateless utility in Phase 2 (CONTEXT.md: "touch 풀 GNU parity는 Phase 2의 가장 큰 단일 태스크"). Full -a/-m/-c/-r/-d/-t/-h coverage, 25 tests, zero clippy warnings.
- Bridged jiff::Zoned → FileTime correctly via Timestamp::as_second + subsec_nanosecond, matching uutils/coreutils's exact pattern without any uucore dependency.
- Verified symlink-self timestamp modification on Windows through a live integration test (test_h_flag_modifies_symlink_self ran green; target mtime provably unchanged when -h sets link mtime).
- Confirmed RESEARCH.md Q2's correction of CONTEXT.md D-19e: no gow_core::fs::touch_link_time wrapper was introduced (filetime handles it already).

## Task Commits

Task 1 (TDD: date.rs parsers):

1. **Task 1 RED — test(02-10): add RED tests for gow-touch date.rs parsers** — `710ef67` (test)
   - 10 unit tests + Cargo.toml wiring (jiff, parse_datetime, filetime) + build.rs + TouchError enum + todo!() stubs
2. **Task 1 GREEN — feat(02-10): implement parse_touch_date and parse_touch_stamp** — `b85dac7` (feat)
   - parse_datetime_at_date → jiff::Zoned → FileTime bridge; hand-rolled 12-digit stamp parser

Task 2 (uumain + integration tests):

3. **Task 2 — feat(02-10): implement gow-touch uumain with full GNU flag set (FILE-08)** — `89bdb9c` (feat)
   - Real uumain replacing stub; timestamps::apply thin wrapper; 14 integration tests; -h clap-collision fix

_Plan metadata commit (SUMMARY.md) will follow._

## Files Created/Modified

**Created:**
- `crates/gow-touch/build.rs` — gow-probe manifest template (WIN-01 ActiveCodePage::Utf8 + WIN-02 longPathAware)
- `crates/gow-touch/src/date.rs` — parse_touch_date (parse_datetime-backed) + parse_touch_stamp (strict 12-digit) + TouchError enum + 10 unit tests
- `crates/gow-touch/src/timestamps.rs` — apply() wrapper selecting set_file_times vs set_symlink_file_times
- `crates/gow-touch/tests/integration.rs` — 14 assert_cmd tests covering all 7 flags + error paths + UTF-8 filename + symlink-self
- `.planning/phases/02-stateless/02-10-SUMMARY.md` — this file

**Modified:**
- `crates/gow-touch/Cargo.toml` — added filetime (workspace), jiff 0.2 / parse_datetime 0.14 (per-crate D-20b), embed-manifest build-dep, dev-deps (assert_cmd/predicates/tempfile)
- `crates/gow-touch/src/lib.rs` — replaced stub with real uumain (disable_help_flag workaround, operand loop with MSYS convert, per-operand error loop, resolve_times helper, current_atime/current_mtime preservers)
- `Cargo.lock` — locked jiff 0.2.23, jiff-tzdb 0.1.6, jiff-tzdb-platform 0.1.3, parse_datetime 0.14.0, winnow 0.7.15, filetime 0.2.27, plus transitives

## Decisions Made

See `key-decisions` in frontmatter. Most notable:

1. **jiff civil API adjustment (minor):** The plan's action block proposed `dt.in_tz("system")` with `dt.to_zoned(jiff::tz::TimeZone::system())` as a fallback. Checked jiff 0.2.23 docs via ctx7 — `TimeZone::system()` returns a `TimeZone` directly (no Result), and `DateTime::to_zoned(tz)` returns `Result<Zoned, jiff::Error>`. Used `to_zoned(TimeZone::system())` from the start (slightly cleaner than the plan's proposed `in_tz` shape). Compilation succeeded on first try.

2. **clap -h collision fix:** GNU touch uses `-h` for `--no-dereference`, but clap 4 auto-registers `-h` as `--help`, causing a debug-assert panic at runtime. Resolved via `disable_help_flag(true)` + a re-registered `--help`-only Arg with `ArgAction::Help`. Documented as a deviation below.

3. **12-digit stamp only (v1):** Plan explicitly permits this narrowing. Deferred full `[[CC]YY]MMDDhhmm[.ss]` form to v2.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] clap short-flag collision between `-h` (--no-dereference) and clap's auto-generated `--help`**
- **Found during:** Task 2 (first `cargo test -p gow-touch` run)
- **Issue:** clap 4 panics with `debug_asserts.rs:125 — Short option names must be unique ... '-h' is in use by both 'no-dereference' and 'help'`. All 13 of 14 integration tests failed on binary startup with exit 101; only the lib-level unit test passed.
- **Fix:** In `uu_app()`, add `.disable_help_flag(true)` then re-register a `--help`-only Arg with `ArgAction::Help`. This matches GNU coreutils touch (which only exposes `--help`, not `-h`, for help — consistent with `-h` being reserved for `--no-dereference`).
- **Files modified:** `crates/gow-touch/src/lib.rs`
- **Verification:** `cargo test -p gow-touch` → 25/25 pass. New `test_bad_flag_exits_1` + `test_gnu_error_format_on_bad_flag` guards confirm `--help` / flag error paths behave GNU-correctly after the fix.
- **Committed in:** `89bdb9c` (part of Task 2 feat commit)

## Auth gates

None.

## Verification Summary

**Automated:**
- `cargo build -p gow-touch` — clean (first build downloaded jiff, parse_datetime, jiff-tzdb, filetime; subsequent builds incremental)
- `cargo test -p gow-touch` — 25 passed, 0 failed, 0 ignored
  - 11 unit tests (10 in `date::tests` + 1 in `tests::uu_app_builds_without_panic`)
  - 14 integration tests (all flag-specific tests + error/format tests + UTF-8 filename + symlink-self)
- `cargo clippy -p gow-touch --all-targets -- -D warnings` — clean

**Smoke verification (manual, from success_criteria):**
- `cargo run -p gow-touch -- /tmp/newfile` → creates `/tmp/newfile` with mtime=now
- `cargo run -p gow-touch -- -c /tmp/missing; echo $?` → `exit=0`, file NOT created
- `cargo run -p gow-touch -- -d "yesterday" /tmp/existing` → `ls` shows mtime `2026-04-20 10:18:07` (yesterday relative to 2026-04-21 test run)
- `cargo run -p gow-touch -- -t 202501011200 /tmp/newfile` → `ls` shows mtime `2025-01-01 12:00:00.000000000 +0900`
- `cargo run -p gow-touch -- -t 202001010000 /tmp/ymd.txt` → `ls` shows mtime `2020-01-01 00:00:00.000000000 +0900`

## Sample Timestamp Evidence

```
$ cargo run -p gow-touch -- -d "yesterday" /tmp/existing
$ ls -la --time-style=full-iso /tmp/existing
-rw-r--r-- 1 노명훈 197121 0 2026-04-20 10:18:07.311398000 +0900 /tmp/existing
                             ^^^^^^^^^^^  (today = 2026-04-21, so yesterday = 2026-04-20 ✓)

$ cargo run -p gow-touch -- -t 202001010000 /tmp/ymd.txt
$ ls -la --time-style=full-iso /tmp/ymd.txt
-rw-r--r-- 1 노명훈 197121 0 2020-01-01 00:00:00.000000000 +0900 /tmp/ymd.txt
                             ^^^^^^^^^^^^^^^^^^^  (strict 12-digit stamp honored in local tz)
```

## -h Symlink-self Test Status

`test_h_flag_modifies_symlink_self` **ran successfully** on this test environment (Windows with Developer Mode / SeCreateSymbolicLinkPrivilege available). The probe `std::os::windows::fs::symlink_file` succeeded, so the test was NOT skipped — it actively verified that:

1. `touch -h -d 2020-01-01T00:00:00Z link.txt` succeeds
2. `std::fs::metadata(&target)` (which FOLLOWS the link on Windows) still returns the baseline mtime (1_600_000_000), proving the target was unchanged.

Per RESEARCH.md Q2, this confirms `filetime::set_symlink_file_times` correctly uses `FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS` internally — CONTEXT.md D-19e's assumption that a custom `gow_core::fs::touch_link_time` wrapper would be needed is formally disproved.

On hosts without Developer Mode the test gracefully returns early via `eprintln!("[skip] ...")` without failing.

## TDD Gate Compliance

Task 1 is an explicit TDD task (`tdd="true"` in plan frontmatter) and follows the required RED → GREEN gate sequence:

- **RED commit** `710ef67` (`test(02-10): ...`) — tests compile but all 10 panic via `todo!()`. Verified by running `cargo test -p gow-touch --lib date` before committing: 0 passed, 10 failed with "not yet implemented".
- **GREEN commit** `b85dac7` (`feat(02-10): ...`) — same 10 tests now pass. Verified: 10 passed, 0 failed.
- **REFACTOR** — no separate refactor commit needed; the GREEN impl is already idiomatic (thiserror-driven error mapping, std parse + jiff civil datetime builder, no redundancies).

Task 2 is not a TDD task in the plan (just `type="auto"`) — committed as a single `feat` commit `89bdb9c`, which is consistent with the plan's single `<action>` block spanning timestamps.rs + lib.rs + integration tests.

## Self-Check: PASSED

Verified:
- crates/gow-touch/Cargo.toml: FOUND
- crates/gow-touch/build.rs: FOUND
- crates/gow-touch/src/lib.rs: FOUND
- crates/gow-touch/src/date.rs: FOUND
- crates/gow-touch/src/timestamps.rs: FOUND
- crates/gow-touch/tests/integration.rs: FOUND
- commit 710ef67 (RED): FOUND in git log
- commit b85dac7 (GREEN): FOUND in git log
- commit 89bdb9c (Task 2): FOUND in git log
- All 25 tests pass; clippy clean; no files touched outside crates/gow-touch/ (except Cargo.lock, which is auto-generated by cargo).

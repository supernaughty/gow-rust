---
phase: 05-search-and-navigation
fixed_at: 2026-04-28T00:00:00Z
review_path: .planning/phases/05-search-and-navigation/05-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 05: Code Review Fix Report

**Fixed at:** 2026-04-28
**Source review:** .planning/phases/05-search-and-navigation/05-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6 (2 Critical, 4 Warning)
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: `find` always exits 0 even when `-exec` child commands fail

**Files modified:** `crates/gow-find/src/lib.rs`
**Commit:** a6c2e8c
**Applied fix:** Changed `run()` return type from `Result<()>` to `Result<i32>`. Added `let mut any_exec_failed = false;` before the entry loop. In the exec match arm, set `any_exec_failed = true` on both `Ok(code) if code != 0` and `Err(e)` arms. Changed the final return to `Ok(if any_exec_failed { 1 } else { 0 })`. Updated `uumain` to use `Ok(code) => code` instead of `Ok(()) => 0`.

---

### CR-02: `NamedTempFile` dropped before `LineIndex` can read it (stdin buffering in `less`)

**Files modified:** `crates/gow-less/src/lib.rs`
**Commit:** cb1f604
**Applied fix:** Replaced `tempfile::NamedTempFile::new()` with `tempfile::tempfile()` (anonymous file handle). Removed the `File::open(tmp.path())` second-handle step. After copying stdin into the anonymous file, seek to start with `tmp.seek(SeekFrom::Start(0))?` and pass `tmp` directly to `LineIndex::new(tmp)`. Added `Seek` and `SeekFrom` to the `std::io` imports. The anonymous file has no path and is only deleted when all handles close, eliminating the Windows deletion race.

---

### WR-01: Non-idiomatic `cfg(target_os = "windows")` in find and xargs

**Files modified:** `crates/gow-find/src/lib.rs`, `crates/gow-xargs/src/lib.rs`
**Commit:** ee8c114
**Applied fix:** Replaced `#[cfg(target_os = "windows")]` with `#[cfg(windows)]` and `#[cfg(not(target_os = "windows"))]` with `#[cfg(not(windows))]` in both files, matching the Rust standard library convention.

---

### WR-02: `root.exists()` check reads stale error via `last_os_error`

**Files modified:** `crates/gow-find/src/lib.rs`
**Commit:** c13947b
**Applied fix:** Replaced the `if !root.exists() { let err = std::io::Error::last_os_error(); ... }` pattern with `if let Err(e) = std::fs::metadata(root) { eprintln!(...); continue; }`. The error value is now the typed `io::Error` returned directly from the failing syscall, not whatever happened to be in the OS error slot after subsequent internal calls.

---

### WR-03: `-print0` write errors silently discarded with `.ok()`

**Files modified:** `crates/gow-find/src/lib.rs`
**Commit:** 26527e4
**Applied fix:** Changed `stdout.write_all(path_bytes.as_bytes()).ok()` and `stdout.write_all(b"\0").ok()` to use `?` instead of `.ok()`. The error propagates through `run()` which returns `Result<i32>`, causing `uumain` to print the error and exit with code 2 on broken pipe or other write failures.

---

### WR-04: Search truncation at 100,000 lines with no user feedback in `less`

**Files modified:** `crates/gow-less/src/lib.rs`
**Commit:** 3e93fc7
**Applied fix:** Added `search_truncated: bool` field to `PagerState` (initialized to `false`). In `find_all_matches`, set `self.search_truncated = true` when the 100 000-line cap is hit. Reset `self.search_truncated = false` at the start of each new search in `enter_search`. In `render()`, the status line appends `" [search truncated at 100k lines]"` when `search_truncated` is true, giving the user a visible indication that `n`/`N` navigation wraps within a partial result set.

---

_Fixed: 2026-04-28_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_

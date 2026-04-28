---
phase: 05-search-and-navigation
reviewed: 2026-04-28T00:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - Cargo.toml
  - crates/gow-find/Cargo.toml
  - crates/gow-find/build.rs
  - crates/gow-find/src/main.rs
  - crates/gow-find/src/lib.rs
  - crates/gow-find/tests/find_tests.rs
  - crates/gow-xargs/Cargo.toml
  - crates/gow-xargs/build.rs
  - crates/gow-xargs/src/main.rs
  - crates/gow-xargs/src/lib.rs
  - crates/gow-xargs/tests/xargs_tests.rs
  - crates/gow-less/Cargo.toml
  - crates/gow-less/build.rs
  - crates/gow-less/src/main.rs
  - crates/gow-less/src/lib.rs
  - crates/gow-less/src/line_index.rs
  - crates/gow-less/tests/less_tests.rs
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: issues_found
---

# Phase 05: Code Review Report

**Reviewed:** 2026-04-28
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 05 adds three new crates: `gow-find`, `gow-xargs`, and `gow-less`. The overall
implementation quality is high. The specifically-called-out security concerns from the
review brief were addressed correctly:

- `find -exec` correctly uses `Command::new` (no shell intermediary) — GOW #209 is fixed.
- `xargs` calls `set_stdin_binary_mode()` before the first `stdin.lock()` read.
- `less` installs the panic hook before `enable_raw_mode`.
- `unsafe` blocks for `_setmode` are correctly guarded behind `#[cfg(target_os = "windows")]`.
- `WalkDir` is constructed with `.follow_links(false)` — no symlink traversal by default.

Two critical issues were found. The first is a correctness bug in `gow-find`: when
`-exec` child commands fail, `find` still exits 0, violating GNU find semantics that
scripts depend on. The second is a use-after-free risk in `gow-less`: the `NamedTempFile`
holding buffered stdin content is dropped at the end of `open_source`, deleting the
underlying file before `LineIndex` can read from it on Windows (where open files can
be deleted when there are no remaining file handles). Four warnings cover logic gaps
and one unsafe correctness concern. Three info items note style and completeness issues.

---

## Critical Issues

### CR-01: `find` always exits 0 even when `-exec` child commands fail

**File:** `crates/gow-find/src/lib.rs:126-132`

**Issue:** `run()` returns `Ok(())` unconditionally. When a `-exec` child process
returns non-zero, the code logs a message (line 218) but does not track a failure
flag. The function still returns `Ok(())` at line 238, so `uumain` emits exit code 0.
GNU `find` exits 1 if any `-exec` invocation exits non-zero. Shell scripts that
use `find -exec ... \; && next-step` will incorrectly proceed even on exec failure.

```rust
// Current (lib.rs lines 126-132, 215-221):
match run(cli) {
    Ok(()) => 0,   // always 0 — even when -exec failed
    Err(e) => { eprintln!("find: {}", e); 2 }
}
// ...inside run():
match exec_for_entry(cmd_parts, path) {
    Ok(code) if code != 0 => {
        eprintln!("find: -exec failed (exit {})", code);
        // BUG: no failure flag set — run() returns Ok(()) anyway
    }
    ...
}
```

**Fix:** Track whether any exec invocation failed and return a distinct exit code:

```rust
fn run(mut cli: Cli) -> Result<i32> {
    // ... (setup unchanged)
    let mut any_exec_failed = false;
    // ... inside the entry loop:
    match exec_for_entry(cmd_parts, path) {
        Ok(code) if code != 0 => {
            eprintln!("find: -exec failed (exit {})", code);
            any_exec_failed = true;
        }
        Ok(_) => {}
        Err(e) => {
            eprintln!("find: {}", e);
            any_exec_failed = true;
        }
    }
    // ...
    Ok(if any_exec_failed { 1 } else { 0 })
}
// In uumain:
match run(cli) {
    Ok(code) => code,
    Err(e) => { eprintln!("find: {}", e); 2 }
}
```

---

### CR-02: `NamedTempFile` dropped before `LineIndex` can read it (stdin buffering in `less`)

**File:** `crates/gow-less/src/lib.rs:108-123`

**Issue:** In `open_source`, when stdin is piped (non-file path), a `NamedTempFile` is
created, stdin is copied into it, and then a new `File` handle is opened from
`tmp.path()`. But `tmp` is a local variable that is dropped at the end of the `None`
arm (when `Ok(LineIndex::new(f))` returns). `NamedTempFile::drop` deletes the
underlying file. On Windows, unlike Unix, you cannot open a file that has been
deleted — but the concern is subtler: the deletion happens immediately after the
`File::open` returns but before `LineIndex` reads any data, because `tmp` is dropped
at the end of the `None` match arm block. On Windows this causes the underlying
file to be deleted while `f` still holds it open; subsequent `read_until` calls on
the `BufReader` inside `LineIndex` will fail with `ERROR_ACCESS_DENIED` or return
0 bytes depending on how Windows handles open-then-deleted files.

The `File::open` call on line 120 opens a second handle with sharing mode; the OS
can delete the file when there are no remaining open handles, but Windows delays the
physical delete until all handles are closed. However, `tempfile::NamedTempFile` on
Windows uses `DeleteFileW` immediately on drop (not the POSIX unlink-on-close
semantics), which can fail silently if another handle is open. This may cause
`LineIndex` to work or fail intermittently depending on Windows version and
antivirus interference — but it is an unintentional and fragile pattern.

```rust
// Current — tmp is dropped when this match arm ends:
None => {
    let mut tmp = tempfile::NamedTempFile::new()?;
    io::copy(&mut io::stdin().lock(), tmp.as_file_mut())?;
    tmp.as_file().sync_all()?;
    let f = File::open(tmp.path())?;
    Ok(LineIndex::new(f))  // tmp dropped here, file deleted
}
```

**Fix:** Return the `NamedTempFile` together with the `LineIndex` so the caller can
hold it alive for the duration of paging, or use `tempfile::tempfile()` (anonymous
file — never has a path on Windows, stays alive until all handles close):

```rust
// Option A — use anonymous tempfile (no path, no deletion risk):
None => {
    let mut tmp = tempfile::tempfile()?;
    io::copy(&mut io::stdin().lock(), &mut tmp)?;
    tmp.seek(SeekFrom::Start(0))?;
    Ok(LineIndex::new(tmp))
}
// Option B — keep NamedTempFile alive by returning it alongside LineIndex
// (requires changing open_source return type or using a wrapper struct).
```

Option A is simpler: `tempfile::tempfile()` returns a `File` directly, never needs
a path, and is automatically deleted when the last handle closes on all platforms.

---

## Warnings

### WR-01: `set_stdout_binary_mode` uses `target_os = "windows"` — should be `cfg(windows)`

**File:** `crates/gow-find/src/lib.rs:470-484`
**Also:** `crates/gow-xargs/src/lib.rs:140-151`

**Issue:** The platform gate uses `#[cfg(target_os = "windows")]` rather than the
idiomatic `#[cfg(windows)]`. On the MSVC Windows target `windows = true` and
`target_os = "windows"` are both true, so this works. However `cfg(windows)` is the
canonical gate used throughout the Rust standard library and crates (including the
rest of this codebase's workspace dependencies). Using a non-idiomatic gate creates
a subtle inconsistency that could cause surprise if anyone tries to build for a
hypothetical `windows` target with a non-`windows` OS name, or when linting with
`--cfg` overrides. More practically: the Rust reference documents `cfg(windows)` as
the portable way to detect Windows, whereas `target_os = "windows"` is the lower-
level primitive that `cfg(windows)` expands to. Prefer the higher-level alias.

**Fix:**
```rust
// Replace in both gow-find/src/lib.rs and gow-xargs/src/lib.rs:
#[cfg(windows)]
fn set_stdout_binary_mode() { ... }

#[cfg(not(windows))]
fn set_stdout_binary_mode() {}
```

---

### WR-02: `root.exists()` check reads stale error via `last_os_error`

**File:** `crates/gow-find/src/lib.rs:179-182`

**Issue:** After calling `root.exists()` (which returns `false` on error OR genuine
non-existence), the code calls `std::io::Error::last_os_error()` to obtain the error
message. But `Path::exists()` calls `fs::metadata()` internally, which may internally
perform multiple OS calls. By the time `last_os_error` is called, the OS error slot
may have been overwritten by a subsequent successful call in the runtime (e.g., by
Rust's allocator or unrelated I/O). The `last_os_error()` value is therefore
unreliable — it might be zero (success), or a completely unrelated error from a
different call.

```rust
// Current — last_os_error() may not reflect the exists() failure:
if !root.exists() {
    let err = std::io::Error::last_os_error();
    eprintln!("find: {}: {}", root.display(), err);
    continue;
}
```

**Fix:** Use `fs::metadata()` directly to get a typed, reliable error:

```rust
if let Err(e) = std::fs::metadata(root) {
    eprintln!("find: {}: {}", root.display(), e);
    continue;
}
```

---

### WR-03: `find -print0` silently discards I/O write errors via `.ok()`

**File:** `crates/gow-find/src/lib.rs:229-230`

**Issue:** Write errors on stdout (broken pipe, disk full, closed pipe from `xargs`)
are silently discarded with `.ok()`. In the `-print0` output path, a broken pipe
from a downstream `xargs -0` process should cause `find` to exit cleanly but not
silently — dropping results without any indication is worse than a handled error.
The default `-print` path uses `println!` (which also ignores write errors) but at
least panics on catastrophic failures. The `-print0` path is more commonly used in
pipelines where the consumer can close early.

```rust
// Current — errors silently dropped:
stdout.write_all(path_bytes.as_bytes()).ok();
stdout.write_all(b"\0").ok();
```

**Fix:** Propagate write errors so the function returns early on broken pipe:

```rust
stdout.write_all(path_bytes.as_bytes())?;
stdout.write_all(b"\0")?;
```

This requires changing the output path to return `Result<()>` — which `run()` already
does, so the `?` operator propagates correctly to the caller.

Note: convert the enclosing block to return `Result<()>` instead of calling
`run()`'s existing `?` path from inside the `.ok()` guard. The containing loop
already propagates errors through `?`.

---

### WR-04: `LineIndex::ensure_indexed_to` condition is off-by-one — re-scans on every incremental call

**File:** `crates/gow-less/src/line_index.rs:37-55`

**Issue:** The early-return guard is `if self.offsets.len() > line_num`. `offsets`
starts as `[0]` (length 1). When `ensure_indexed_to(0)` is called (asking for line 0
to be available), `offsets.len() = 1 > 0` is true, so it returns early — correct.
When `ensure_indexed_to(1)` is called, `1 > 1` is false, so it enters the scan
loop — correct. However, the comment says "After calling this,
`offsets.len() > line_num`" but the entry condition `offsets.len() > line_num` is
the same predicate. The off-by-one risk is subtle: when `eof_reached = true` and
`offsets.len() == line_num + 1` (exactly enough entries), the early-return condition
`offsets.len() > line_num` evaluates to `true` — but the second part of the
condition, `|| self.eof_reached`, means that if a caller asks for a line past EOF we
also return early without rescanning. This is correct behavior.

The actual bug is different: after `scan_to_end()`, the `reader`'s internal buffer
position is at EOF. When `read_line_at(0)` is subsequently called (line 121-129), it
calls `ensure_indexed_to(1)`. Since `offsets.len() >= 2` (assuming any content), it
returns early. Then it seeks the `reader` to `offsets[0] = 0` and calls
`read_until(b'\n')` — this seek on the `BufReader` does NOT flush the internal
buffer, it merely calls `seek` on the underlying `File`. `BufReader::seek` does flush
its internal buffer when seeking (as of Rust 1.36), so this is safe. No bug here.

Re-examining: the real concern is that `ensure_indexed_to` re-seeks `reader` to
`last_known` offset every time it is called and does NOT return early when `eof_reached`
is already true AND `offsets.len() > line_num`. The early return at line 39 covers
both cases with `||` so this is fine.

**Revised finding:** The condition is correct but the seek in `ensure_indexed_to`
(line 44: `self.reader.seek(SeekFrom::Start(last_known))`) is called every time we
enter the scan loop, even if the reader is already positioned at `last_known`. This
is a minor redundant seek on each incremental scroll step during normal paging, but
it is not a correctness bug. Downgrading this to an Info item — see IN-01.

**Actual WR-04 (revised):** `find_all_matches` in `less` silently truncates search
results at 100,000 lines with no user notification.

**File:** `crates/gow-less/src/lib.rs:306-309`

**Issue:** The 100,000-line safety cap exits the search loop silently. A user
searching a large log file will see partial results with no indication that the
search was cut off. The search navigation (`n`/`N`) will wrap around within the
truncated set, which looks like a complete search result but may miss matches.

```rust
if line > 100_000 {
    // Safety cap — gap-closure plan can remove this limit.
    break;
}
```

**Fix:** After the loop, if the cap was hit, display a status message informing the
user that results are partial:

```rust
if line > 100_000 {
    // Notify user that search results are truncated
    self.search_truncated = true; // add field to PagerState
    break;
}
// In render(), show a visible indicator in the status line when truncated:
// ":/pattern/ (n=next, N=prev, q=quit) [search truncated at 100k lines]"
```

---

## Info

### IN-01: Redundant seek on every incremental call to `ensure_indexed_to`

**File:** `crates/gow-less/src/line_index.rs:43-44`

**Issue:** Every call to `ensure_indexed_to` that needs to scan forward performs
`self.reader.seek(SeekFrom::Start(last_known))` unconditionally. During normal
forward paging, the reader is already at `last_known` from the previous call. The
seek flushes `BufReader`'s internal buffer, costing a syscall per scroll step.

**Fix:** Track whether the reader's position is already at `last_known` to skip the
seek, or accept the cost since it is a single seek per user keypress (not a tight loop).

---

### IN-02: `find -print0` path uses `to_string_lossy` — silently replaces non-UTF-8 path bytes

**File:** `crates/gow-find/src/lib.rs:227-229`

**Issue:** The `-print0` output path converts the path via `to_string_lossy()` before
writing. On Windows, paths are natively UTF-16; `WalkDir` presents them as `OsStr`
(which on Windows is WTF-8 internally). `to_string_lossy()` replaces any un-pairable
surrogates with `\u{FFFD}` (replacement character), meaning a path containing a
WTF-8 surrogate pair could be emitted as corrupted bytes. For a tool designed to
pass paths verbatim to downstream commands, this is a data fidelity gap.

This is an inherent limitation of the Rust `Path` → `str` conversion on Windows, but
is worth documenting and potentially using `encode_wide` or an OS-native path
serializer for the `-print0` path.

**Fix (pragmatic):** Add a comment to the existing code documenting this limitation,
or for a more correct solution use `path.as_os_str().encode_wide()` to write UTF-16LE
bytes in the `-print0` path (though this would break `xargs -0` on UTF-8 downstream).
The limitation is acceptable for now but should be tracked.

---

### IN-03: `xargs` test for `-I` rejection uses `-I -n 2` argument order which may not trigger mutual-exclusion check

**File:** `crates/gow-xargs/tests/xargs_tests.rs:138-158`

**Issue:** The test calls `c.arg("-I").arg("-n").arg("2").arg(echo_bin)`. With
`trailing_var_arg = true` on `command_and_args`, clap may parse `-n` and `2` as the
start of `command_and_args` rather than as the `--max-args` flag, depending on
argument ordering and how clap resolves ambiguity. If `-n` is consumed as part of the
command string rather than as a flag, `cli.max_args` will be `None` and the
mutual-exclusion check at line 77 of `lib.rs` will not trigger, making this test
unreliable.

Run the test with `-- --nocapture` to confirm it is actually testing the code path
it claims. If the test is passing through luck (empty input causing early return
before mutual-exclusion check), it should be rewritten to explicitly pass
`--null --max-args 2` together with a non-empty stdin.

**Fix:** Rewrite the test to use unambiguous flag names and provide stdin input:

```rust
c.arg("-I").arg("--max-args").arg("2").arg(echo_bin);
// ...
c.write_stdin("a\nb\n")
```

---

_Reviewed: 2026-04-28_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

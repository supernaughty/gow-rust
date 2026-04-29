---
phase: 08-code-review-fixes
reviewed: 2026-04-29T00:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - crates/gow-tar/src/lib.rs
  - crates/gow-tar/tests/tar_tests.rs
  - crates/gow-xz/src/lib.rs
  - crates/gow-xz/tests/xz_tests.rs
  - crates/gow-gzip/src/lib.rs
  - crates/gow-gzip/tests/gzip_tests.rs
  - crates/gow-curl/src/lib.rs
  - crates/gow-curl/tests/curl_tests.rs
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 08: Code Review Report

**Reviewed:** 2026-04-29T00:00:00Z
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

Phase 08 fixed six code review issues across four crates (gow-tar, gow-xz, gow-gzip, gow-curl). The phase-specific fixes are correctly implemented: `MultiBzDecoder` replaces `BzDecoder` for multi-stream bzip2, `XzDecoder::new_multi_decoder` replaces `XzDecoder::new` for concatenated xz streams, `.gz` suffix rejection works correctly in gzip decompress mode, and curl cleans up partial output files on I/O failure.

Four new warnings and three info items were found. The most significant are a logic gap in `unpack_archive` where mid-archive entry errors bypass `had_error` tracking (WR-03's fix is incomplete), an open file handle race on Windows in curl's cleanup path, and `set_ignore_zeros` being applied only to extract mode but not to list mode. Two unused dependency declarations in gow-tar's Cargo.toml are also flagged.

---

## Warnings

### WR-01: Entry iteration error in `unpack_archive` bypasses `had_error` tracking

**File:** `crates/gow-tar/src/lib.rs:292-293`
**Issue:** `archive.entries()?` and the per-entry `entry?` propagate errors out of `unpack_archive` via `?`, returning `Err` immediately without setting `had_error = true`. The `had_error` guard (lines 318-320) was added to exit with code 1 on extraction failure, but a corrupt entry in the middle of the archive bypasses that path entirely. The caller in `uumain` catches this `Err` and exits with code 1, so exit code is correct — but the error message path differs: per-entry errors use `eprintln!("tar: {}: {e}", path.display())`, while propagated errors go through the generic `eprintln!("tar: {e}")` in `uumain`. The GNU tar behavior is to continue extracting subsequent entries and report each failure, not abort on first entry error.

**Fix:**
```rust
fn unpack_archive<R: Read>(mut archive: Archive<R>, dest: &str, cli: &Cli) -> Result<()> {
    let mut had_error = false;
    archive.set_ignore_zeros(true);

    let entries = match archive.entries() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("tar: cannot read archive entries: {e}");
            return Err(e.into());
        }
    };

    for entry in entries {
        let mut entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("tar: entry read error: {e}");
                had_error = true;
                continue;   // skip corrupt entry, continue archive walk
            }
        };
        // ... rest unchanged
    }
    if had_error {
        anyhow::bail!("one or more files could not be extracted");
    }
    Ok(())
}
```

---

### WR-02: `set_ignore_zeros` not applied in `list_archive`

**File:** `crates/gow-tar/src/lib.rs:359-365`
**Issue:** `archive.set_ignore_zeros(true)` is called in `unpack_archive` (line 291) to support concatenated archives, but the same setting is absent in `list_archive`. When `tar -t` is used against a concatenated or zero-padded archive (such as those produced by some backup tools), listing stops at the first zero block instead of continuing. This is inconsistent with extract mode behavior.

**Fix:**
```rust
fn list_archive<R: Read>(mut archive: Archive<R>) -> Result<()> {
    archive.set_ignore_zeros(true);   // add this line
    for entry in archive.entries()? {
        let entry = entry?;
        println!("{}", entry.path()?.display());
    }
    Ok(())
}
```

---

### WR-03: No per-entry error tracking in `append_paths` (create mode)

**File:** `crates/gow-tar/src/lib.rs:161-193`
**Issue:** When creating an archive (`-c`), per-file errors (failed `append_dir_all`, failed `set_path`, failed `append`) are printed to stderr via `eprintln!` but are otherwise ignored — `append_paths` returns `Ok(())` even if every file failed to be added. GNU tar exits with a non-zero code when any file cannot be archived. The fix introduced for `unpack_archive` (WR-03 in Phase 08) correctly handles extract mode but create mode was left without equivalent error propagation.

**Fix:**
```rust
fn append_paths<W: Write, F: FnOnce(W) -> Result<()>>(
    mut builder: Builder<W>,
    cli: &Cli,
    finish: F,
) -> Result<()> {
    builder.follow_symlinks(false);
    let mut had_error = false;   // add this

    // ... path resolution unchanged ...

    for path_str in &cli.paths {
        // ...
        if full_path.is_dir() {
            if let Err(e) = builder.append_dir_all(name, &full_path) {
                eprintln!("tar: {converted}: {e}");
                had_error = true;   // track errors
            }
            // ...
        } else {
            // ... existing match, set had_error = true on each eprintln arm
        }
    }

    let inner = builder.into_inner()?;
    finish(inner)?;
    if had_error {
        anyhow::bail!("one or more files could not be archived");
    }
    Ok(())
}
```

---

### WR-04: Open file handle before `remove_file` in curl's error cleanup path (Windows)

**File:** `crates/gow-curl/src/lib.rs:108-111`
**Issue:** When `io::copy` fails, `fs::remove_file(output_path)` is called while `file` (the `File` handle created at line 108) is still alive in the enclosing scope. On Windows, deleting an open file handle fails with "The process cannot access the file because it is being used by another process" (error 32). The `let _` ignores this error, so the partial file silently remains on disk. `file` must be explicitly dropped before the `remove_file` call.

```rust
// Current code (lines 108-112):
let mut file = File::create(output_path)?;
if let Err(e) = io::copy(&mut response, &mut file) {
    let _ = fs::remove_file(output_path);  // file is still open — fails on Windows
    return Err(e.into());
}
```

**Fix:**
```rust
let mut file = File::create(output_path)?;
let copy_result = io::copy(&mut response, &mut file);
drop(file);   // close the handle BEFORE attempting remove on Windows
if let Err(e) = copy_result {
    let _ = fs::remove_file(output_path);
    return Err(e.into());
}
```

---

## Info

### IN-01: `thiserror` and `walkdir` declared as unused dependencies in gow-tar

**File:** `crates/gow-tar/Cargo.toml:27,28` (cross-referenced with `crates/gow-tar/src/lib.rs`)
**Issue:** `thiserror` and `walkdir` are listed as dependencies in `gow-tar/Cargo.toml` but are not imported or used anywhere in `src/lib.rs`. These increase compile time and dependency surface area without benefit. `thiserror` is understandable as a workspace-wide convention, but `walkdir` is not referenced at all in this crate — the `tar` crate's own `append_dir_all` handles recursive traversal internally.

**Fix:** Remove unused entries from `[dependencies]`:
```toml
# Remove these two lines from crates/gow-tar/Cargo.toml:
thiserror = { workspace = true }
walkdir = { workspace = true }
```
Run `cargo check -p gow-tar` after removal to confirm nothing breaks.

---

### IN-02: Inconsistent `from_arg_matches` error handling between gow-xz and other crates

**File:** `crates/gow-xz/src/lib.rs:248`
**Issue:** `gow-xz` calls `Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit())`, which delegates to clap's default exit behavior. This is fine, but `gow-tar` (line 373-378) and `gow-gzip` (lines 210-215) both use the explicit pattern:
```rust
Err(e) => {
    eprintln!("xz: {e}");
    return 2;
}
```
The explicit pattern produces a `tar:`/`gzip:`-prefixed message on stderr and returns exit code 2, consistent with GNU conventions. The `e.exit()` path may print to stdout in some clap versions (e.g., for `--help`-like errors) and uses clap's exit code, which may not be 2.

**Fix:** For consistency, adopt the same pattern used in gow-tar and gow-gzip:
```rust
let cli = match Cli::from_arg_matches(&matches) {
    Ok(c) => c,
    Err(e) => {
        eprintln!("xz: {e}");
        return 2;
    }
};
```

---

### IN-03: Test `output_file_not_created_on_invalid_path` tests a `File::create` failure, not an `io::copy` failure

**File:** `crates/gow-curl/tests/curl_tests.rs:135-153`
**Issue:** The WR-07 test is intended to verify that curl removes a partial output file when `io::copy` fails mid-download. However, the test uses a path inside a non-existent directory (`nonexistent_dir/output.bin`), which causes `File::create` itself to fail (before any copy happens). This tests the `?` propagation from `File::create` at line 108, not the `remove_file` cleanup at line 110. The actual WR-07 fix (removing a file after `io::copy` fails) remains untested by an automated test.

A true regression test would: create the output file successfully first (directory exists), then inject a network failure after some bytes have been written (e.g., using a mock server that closes mid-stream).

**Fix:** Add a comment clarifying what is actually tested, and optionally add a separate test with a mock HTTP server for the true `io::copy` failure path:
```rust
// Note: this test exercises File::create failure (directory not found),
// not the io::copy failure cleanup path. The io::copy path requires a
// network mock server to inject a mid-stream failure.
```

---

_Reviewed: 2026-04-29T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

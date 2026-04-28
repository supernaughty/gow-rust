---
phase: 06-archive-compression-and-network
reviewed: 2026-04-28T00:00:00Z
depth: standard
files_reviewed: 28
files_reviewed_list:
  - Cargo.toml
  - crates/gow-bzip2/Cargo.toml
  - crates/gow-bzip2/build.rs
  - crates/gow-bzip2/src/lib.rs
  - crates/gow-bzip2/src/main.rs
  - crates/gow-bzip2/tests/bzip2_tests.rs
  - crates/gow-curl/Cargo.toml
  - crates/gow-curl/build.rs
  - crates/gow-curl/src/lib.rs
  - crates/gow-curl/src/main.rs
  - crates/gow-curl/tests/curl_tests.rs
  - crates/gow-gzip/Cargo.toml
  - crates/gow-gzip/build.rs
  - crates/gow-gzip/src/gunzip.rs
  - crates/gow-gzip/src/lib.rs
  - crates/gow-gzip/src/main.rs
  - crates/gow-gzip/src/zcat.rs
  - crates/gow-gzip/tests/gzip_tests.rs
  - crates/gow-tar/Cargo.toml
  - crates/gow-tar/build.rs
  - crates/gow-tar/src/lib.rs
  - crates/gow-tar/src/main.rs
  - crates/gow-tar/tests/tar_tests.rs
  - crates/gow-xz/Cargo.toml
  - crates/gow-xz/build.rs
  - crates/gow-xz/src/lib.rs
  - crates/gow-xz/src/main.rs
  - crates/gow-xz/tests/xz_tests.rs
findings:
  critical: 0
  warning: 7
  info: 3
  total: 10
status: issues_found
---

# Phase 06: Code Review Report

**Reviewed:** 2026-04-28T00:00:00Z
**Depth:** standard
**Files Reviewed:** 28
**Status:** issues_found

## Summary

Five utilities were reviewed: `gow-bzip2`, `gow-curl`, `gow-gzip`, `gow-tar`, and `gow-xz`. Overall the implementation is structurally sound — MSYS path conversion is applied consistently, partial output cleanup is in place for all file-to-file operations, the blocking reqwest client is used correctly without a tokio runtime, and symlink follow is correctly disabled in tar creation.

Seven warnings and three info items were found. No critical (security/data-loss) issues exist. The most impactful warnings are:

- **tar uses `BzDecoder` (single-stream) instead of `MultiBzDecoder`**, causing silent truncation of multi-stream bzip2 archives produced by pbzip2 or split-stream tools.
- **tar's `unpack_archive` swallows per-entry errors and returns `Ok(())`**, so `tar -x` exits 0 even when files fail to extract.
- **tar panics on bad CLI args** via an `.unwrap()` instead of the graceful error path used by every other utility.
- **xz uses `XzDecoder::new()` (single-stream)** when `XzDecoder::new_multi_decoder()` is available and correct for concatenated `.xz` streams.
- **gzip appends `.out`** for files without a `.gz` suffix instead of rejecting them, diverging from GNU gzip behavior.

---

## Warnings

### WR-01: tar uses single-stream BzDecoder — multi-stream bzip2 archives silently truncate

**File:** `crates/gow-tar/src/lib.rs:7`
**Issue:** `run_extract` and `run_list` wrap the bzip2 reader with `BzDecoder::new(f)` (single-stream decoder). Archives created by `pbzip2`, or any multi-stream `.tar.bz2`, contain multiple concatenated bzip2 streams. `BzDecoder` stops at the first stream boundary; remaining data is silently ignored. The result is a partially extracted archive with exit code 0. `gow-bzip2` itself correctly uses `MultiBzDecoder` (lib.rs line 12), but that fix was not applied to `gow-tar`.

**Fix:**
```rust
// In the import block at the top of crates/gow-tar/src/lib.rs, change:
use bzip2::read::BzDecoder;
// to:
use bzip2::read::MultiBzDecoder;

// Then in run_extract and run_list, replace every:
Archive::new(BzDecoder::new(f))
// with:
Archive::new(MultiBzDecoder::new(f))
```

---

### WR-02: tar CLI arg parse error uses .unwrap() — panics instead of printing usage error

**File:** `crates/gow-tar/src/lib.rs:370`
**Issue:** `Cli::from_arg_matches(&matches).unwrap()` will panic with a Rust backtrace if `from_arg_matches` returns an error. Every other utility in this codebase uses the same graceful pattern: `match ... { Ok(c) => c, Err(e) => { eprintln!("tar: {e}"); return 2; } }`. A panic produces a poor user experience and does not return exit code 2 (misuse).

**Fix:**
```rust
// Replace line 370:
let cli = Cli::from_arg_matches(&matches).unwrap();
// with:
let cli = match Cli::from_arg_matches(&matches) {
    Ok(c) => c,
    Err(e) => {
        eprintln!("tar: {e}");
        return 2;
    }
};
```

---

### WR-03: tar unpack_archive returns Ok(()) on per-entry errors — exit code is 0 on partial extract

**File:** `crates/gow-tar/src/lib.rs:302`
**Issue:** `unpack_archive` calls `entry.unpack_in(dest)` in a loop and logs errors but never propagates them — the function signature returns `Result<()>` and always returns `Ok(())` after the loop. Consequently `run_extract` returns `Ok(())`, and `uumain` exits with code 0 even when entries failed to extract. GNU tar exits non-zero on extraction failure.

**Fix:**
```rust
fn unpack_archive<R: Read>(mut archive: Archive<R>, dest: &str, cli: &Cli) -> Result<()> {
    let mut had_error = false;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();

        if cli.verbose {
            eprintln!("{}", path.display());
        }

        if let Err(e) = entry.unpack_in(dest) {
            let estr = e.to_string().to_lowercase();
            if estr.contains("symlink")
                || estr.contains("privilege")
                || estr.contains("access is denied")
            {
                eprintln!(
                    "tar: warning: {}: {e} \
                     (symlink extraction may require elevated privileges on Windows)",
                    path.display()
                );
                // Symlink failures are warnings, not fatal errors; keep had_error false.
            } else {
                eprintln!("tar: {}: {e}", path.display());
                had_error = true;
            }
        }
    }
    if had_error {
        anyhow::bail!("one or more files could not be extracted");
    }
    Ok(())
}
```

---

### WR-04: xz uses single-stream XzDecoder — concatenated .xz files silently truncate

**File:** `crates/gow-xz/src/lib.rs:86`
**Issue:** `XzDecoder::new(input)` decompresses only the first xz stream and may silently consume or ignore any subsequent concatenated streams. The `liblzma` crate (0.4.6) exposes `XzDecoder::new_multi_decoder(r)` for exactly this use case. GNU `xz -d` handles concatenated streams correctly; the current implementation diverges from that.

**Fix:**
```rust
// In decompress_stream, change:
let mut decoder = XzDecoder::new(input);
// to:
let mut decoder = XzDecoder::new_multi_decoder(input);
```

---

### WR-05: gzip appends ".out" for files without .gz suffix instead of rejecting them

**File:** `crates/gow-gzip/src/lib.rs:159-161`
**Issue:** When decompressing a file that lacks the `.gz` suffix, the code falls through to `format!("{converted}.out")` and attempts decompression into a `.out` file. GNU `gzip` rejects the file with "unknown suffix -- ignored" and exits non-zero, matching the behavior already implemented in `gow-bzip2` (lib.rs:209-215) and `gow-xz` (lib.rs:103-107). The current gzip behavior can silently create spurious `.out` files from non-gzip inputs.

**Fix:**
```rust
// Replace lines 156-161 in crates/gow-gzip/src/lib.rs:
let out_path = if converted.ends_with(".gz") {
    converted[..converted.len() - 3].to_string()
} else {
    eprintln!("gzip: {converted}: unknown suffix -- ignored");
    exit_code = 1;
    continue;
};
```

---

### WR-06: curl -I (HEAD) with -s prints headers even in silent mode

**File:** `crates/gow-curl/src/lib.rs:96-99`
**Issue:** The `--silent` flag suppresses the status line (line 93-95 is guarded by `!cli.silent`) but the header-printing loop on lines 96-99 is unconditional. GNU `curl -s -I` suppresses all output. This is a GNU compatibility divergence — scripts that use `-s` to capture clean output will receive unexpected header lines.

**Fix:**
```rust
// Wrap the header loop with the same silent guard:
if !cli.silent {
    println!("HTTP/1.1 {}", status);
    for (name, value) in response.headers() {
        println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
    }
}
return Ok(0);
```

---

### WR-07: curl -o leaves a partial output file on I/O error

**File:** `crates/gow-curl/src/lib.rs:106-110`
**Issue:** When `-o <file>` is used and `io::copy` fails mid-transfer, `File::create(output_path)` has already created the output file and may have written a partial response to it. The error is propagated via `?` but the partial file is not removed. All other utilities in this codebase (bzip2, gzip, xz) explicitly `fs::remove_file` the partial output on error. An interrupted download would leave a truncated file that appears to be a complete download.

**Fix:**
```rust
if let Some(ref output_path) = cli.output {
    let mut file = File::create(output_path)?;
    if let Err(e) = io::copy(&mut response, &mut file) {
        let _ = std::fs::remove_file(output_path);
        return Err(e.into());
    }
} else {
    let bytes = response.bytes()?;
    io::stdout().write_all(&bytes)?;
}
```

---

## Info

### IN-01: gzip stdin decompress error path has unreachable branch

**File:** `crates/gow-gzip/src/lib.rs:86-99`
**Issue:** In the stdin/no-files path for `Mode::Decompress`, the code calls `decompress_stream(...)`, stores the result in `res`, tests `res.is_err()` (prints a custom message and returns 1), then falls through to `let result = ... res`. If `res` is `Err`, the early return fires and `result` is never bound. If `res` is `Ok`, the `if let Err(e) = result` block below never fires. The custom "not in gzip format" message also replaces the actual error detail from the decoder. The code is functionally correct but confusing and the bottom error branch is dead.

**Suggestion:** Simplify to a single match on the result of `decompress_stream`, printing the actual error:
```rust
Mode::Decompress => {
    if let Err(e) = decompress_stream(stdin.lock(), stdout.lock()) {
        eprintln!("gzip: stdin: {e}");
        return 1;
    }
    return 0;
}
```

---

### IN-02: tar missing liblzma dependency — .tar.xz not supported

**File:** `crates/gow-tar/Cargo.toml:23-27`
**Issue:** `gow-tar` depends on `tar`, `flate2`, and `bzip2`, providing `-z` (gzip) and `-j` (bzip2) codec support. There is no `-J` / `--xz` flag and no `liblzma` dependency. Common `.tar.xz` archives (used by many Linux source distributions and Arch Linux packages) cannot be created or extracted. This is a scope gap rather than a bug, but worth noting for completeness.

**Suggestion:** Add `liblzma = { workspace = true }` to `gow-tar`'s dependencies, add a `-J` / `--xz` flag to the CLI, and add a `Codec::Xz` variant using `liblzma::read::XzDecoder::new_multi_decoder` and `liblzma::write::XzEncoder`.

---

### IN-03: bzip2 compression level not user-configurable — ignores -1 through -9 flags

**File:** `crates/gow-bzip2/src/lib.rs:67-71`
**Issue:** `BzEncoder::new(output, Compression::default())` hardcodes compression level 6. GNU `bzip2` accepts `-1` through `-9` to control speed vs. compression ratio. The CLI struct does not expose these flags. For a GNU compatibility tool, users may pass `-9` expecting maximum compression. Unknown flags will currently be rejected by clap with exit code 2.

**Suggestion:** Add `#[arg(short = '1', ...) through #[arg(short = '9', ...)]` level flags (or a single `--fast`/`--best` pair matching GNU bzip2), map to `Compression::new(level)`, and pass through to `BzEncoder`.

---

_Reviewed: 2026-04-28T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

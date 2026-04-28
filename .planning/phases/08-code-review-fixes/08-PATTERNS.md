# Phase 08: Code Review Fixes — Pattern Map

**Mapped:** 2026-04-29
**Files analyzed:** 8 (4 source files to modify + 4 test files to extend)
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/gow-tar/src/lib.rs` | utility / stream transform | streaming, file-I/O | `crates/gow-bzip2/src/lib.rs` | exact (same domain; bzip2 already uses MultiBzDecoder and graceful CLI error) |
| `crates/gow-xz/src/lib.rs` | utility / stream transform | streaming, file-I/O | `crates/gow-bzip2/src/lib.rs` | role-match (single-line fix inside decompress_stream) |
| `crates/gow-gzip/src/lib.rs` | utility / stream transform | streaming, file-I/O | `crates/gow-bzip2/src/lib.rs` | exact (same unknown-suffix + stdin-dead-code pattern) |
| `crates/gow-curl/src/lib.rs` | utility / request-response | request-response, file-I/O | `crates/gow-bzip2/src/lib.rs` | partial-match (partial-file cleanup pattern) |
| `crates/gow-tar/tests/tar_tests.rs` | test | streaming | `crates/gow-bzip2/tests/bzip2_tests.rs` | role-match (same test helpers: assert_cmd, tempdir, write_fixture) |
| `crates/gow-xz/tests/xz_tests.rs` | test | streaming | `crates/gow-bzip2/tests/bzip2_tests.rs` | role-match |
| `crates/gow-gzip/tests/gzip_tests.rs` | test | streaming | `crates/gow-bzip2/tests/bzip2_tests.rs` | role-match |
| `crates/gow-curl/tests/curl_tests.rs` | test | request-response | `crates/gow-bzip2/tests/bzip2_tests.rs` | role-match |

---

## Pattern Assignments

### `crates/gow-tar/src/lib.rs` (WR-01, WR-02, WR-03)

**Analog:** `crates/gow-bzip2/src/lib.rs`

#### WR-01 — MultiBzDecoder import pattern (analog lines 12-13)

```rust
// In gow-bzip2/src/lib.rs — already correct:
use bzip2::read::MultiBzDecoder;

// In gow-tar/src/lib.rs line 7, change:
use bzip2::read::BzDecoder;
// to:
use bzip2::read::MultiBzDecoder;

// Then replace every Archive::new(BzDecoder::new(f)) call with:
Archive::new(MultiBzDecoder::new(f))
// Affected call sites: run_extract lines 268-270, run_list lines 335-340
```

#### WR-01 — MultiBzDecoder usage in stream function (analog lines 76-80)

```rust
// gow-bzip2/src/lib.rs lines 76-80 — reference for MultiBzDecoder::new usage:
fn decompress_stream<R: Read, W: Write>(input: R, mut output: W) -> Result<()> {
    let mut decoder = MultiBzDecoder::new(input);
    io::copy(&mut decoder, &mut output).context("bzip2: decompression I/O error")?;
    Ok(())
}
```

#### WR-02 — Graceful CLI error pattern (analog lines 273-279)

```rust
// gow-bzip2/src/lib.rs lines 273-279 — copy this exact pattern into tar's uumain:
let cli = match Cli::from_arg_matches(&matches) {
    Ok(c) => c,
    Err(e) => {
        eprintln!("bzip2: {e}");   // change prefix to "tar"
        return 2;
    }
};
// Replace gow-tar/src/lib.rs line 370: Cli::from_arg_matches(&matches).unwrap()
```

#### WR-03 — had_error propagation pattern (from 06-REVIEW.md WR-03 fix spec)

```rust
// Replace unpack_archive (gow-tar/src/lib.rs lines 287-319) with:
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
// Note: uumain already maps Err → exit 1 via the bottom match result block (lines 387-391),
// so had_error → bail! → Err → exit 1 is automatic once this change is made.
```

---

### `crates/gow-xz/src/lib.rs` (WR-04)

**Analog:** `crates/gow-bzip2/src/lib.rs` (MultiBzDecoder pattern); `crates/gow-xz/src/lib.rs` itself (single-line change)

#### WR-04 — XzDecoder::new_multi_decoder (gow-xz/src/lib.rs line 86)

```rust
// Current (gow-xz/src/lib.rs lines 85-89):
fn decompress_stream<R: Read, W: Write>(input: R, mut output: W) -> Result<()> {
    let mut decoder = XzDecoder::new(input);   // <-- single-stream, BUG
    io::copy(&mut decoder, &mut output).context("decompress: io::copy failed")?;
    Ok(())
}

// Fix: change line 86 to:
    let mut decoder = XzDecoder::new_multi_decoder(input);
// No other changes needed in this file.
```

Note: The existing graceful CLI error on line 248 (`Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit())`) is a different pattern from the standard `match ... return 2` used by other crates. The CONTEXT.md does not require fixing this in phase 08 — only WR-04 is in scope for gow-xz.

---

### `crates/gow-gzip/src/lib.rs` (WR-05 + IN-01)

**Analog:** `crates/gow-bzip2/src/lib.rs` for both fixes

#### WR-05 — Unknown suffix rejection (analog lines 206-215)

```rust
// gow-bzip2/src/lib.rs lines 206-215 — exact pattern to copy:
let out_path = match out_path_opt {
    Some(p) => p,
    None => {
        eprintln!(
            "bzip2: {converted}: unknown suffix -- ignored"
        );
        exit_code = 1;
        continue;
    }
};

// In gow-gzip/src/lib.rs, replace lines 156-161:
let out_path = if converted.ends_with(".gz") {
    converted[..converted.len() - 3].to_string()
} else {
    // GNU gzip appends ".out" if no .gz suffix; here we just try anyway   <-- REMOVE this branch
    format!("{converted}.out")
};
// With (from 06-REVIEW.md WR-05 fix):
let out_path = if converted.ends_with(".gz") {
    converted[..converted.len() - 3].to_string()
} else {
    eprintln!("gzip: {converted}: unknown suffix -- ignored");
    exit_code = 1;
    continue;
};
```

#### IN-01 — Stdin decompress dead-code simplification (gow-gzip/src/lib.rs lines 86-99)

```rust
// Current (gow-gzip/src/lib.rs lines 79-99) — stdin block with dead branch:
Mode::Decompress => {
    // Guard: attempt decode; if it fails, emit error (Pitfall 4)
    let res = decompress_stream(stdin.lock(), stdout.lock());
    if res.is_err() {
        eprintln!("gzip: stdin: not in gzip format");
        return 1;
    }
    res   // <-- dead: if Err, already returned; if Ok, Err branch below never fires
}
// ...
if let Err(e) = result {   // <-- dead for Decompress arm
    eprintln!("gzip: stdin: {e}");
    exit_code = 1;
}

// Replace with (from 06-REVIEW.md IN-01 suggestion):
Mode::Decompress => {
    if let Err(e) = decompress_stream(stdin.lock(), stdout.lock()) {
        eprintln!("gzip: stdin: {e}");
        return 1;
    }
    return 0;
}
// This eliminates the dead branch and uses the actual decoder error message.
// The Compress arm's structure is unchanged.
```

---

### `crates/gow-curl/src/lib.rs` (WR-06 + WR-07)

**Analog:** `crates/gow-bzip2/src/lib.rs` (partial file cleanup, lines 158-164); `crates/gow-curl/src/lib.rs` itself (silent guard, line 93)

#### WR-06 — Header loop silent guard (gow-curl/src/lib.rs lines 89-99)

```rust
// Current (gow-curl/src/lib.rs lines 89-99):
if cli.head {
    let response = client.head(&cli.url).send()?;
    let status = response.status();
    if !cli.silent {
        println!("HTTP/1.1 {}", status);   // <-- guarded
    }
    for (name, value) in response.headers() {   // <-- NOT guarded — BUG
        println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
    }
    return Ok(0);
}

// Fix: extend the silent guard to cover the header loop (from 06-REVIEW.md WR-06):
if cli.head {
    let response = client.head(&cli.url).send()?;
    let status = response.status();
    if !cli.silent {
        println!("HTTP/1.1 {}", status);
        for (name, value) in response.headers() {
            println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
        }
    }
    return Ok(0);
}
```

#### WR-07 — Partial file cleanup on I/O error (gow-curl/src/lib.rs lines 106-110)

```rust
// gow-bzip2/src/lib.rs lines 157-163 — canonical partial-file cleanup pattern:
if let Err(e) = compress_stream(f, out_file) {
    eprintln!("bzip2: {converted}: {e}");
    // Remove partial output on error
    let _ = fs::remove_file(&out_path);
    exit_code = 1;
    continue;
}

// gow-xz/src/lib.rs lines 208-212 — same pattern for reference:
if let Err(e) = result {
    eprintln!("xz: {converted}: {e}");
    // Remove incomplete output file on error
    let _ = fs::remove_file(&output_path);
    exit_code = 1;
    continue;
}

// In gow-curl/src/lib.rs, replace lines 106-110 (from 06-REVIEW.md WR-07 fix):
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
// Note: std::fs must be in scope — gow-curl/src/lib.rs currently only imports
// std::fs::File. Add `use std::fs;` or use the full path `std::fs::remove_file`.
```

---

## Test Pattern Assignments

All four test files share the same helper/assertion scaffold. Copy from the existing tests in each file — do not replace existing tests, only append new ones.

### Test helper pattern (all test files)

```rust
// Standard pattern used in every test file — no changes needed to these helpers:
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}
```

### Multi-stream fixture construction pattern (WR-01 / WR-04 tests)

New tests for WR-01 (tar multi-stream bzip2) and WR-04 (xz multi-stream) must build fixtures inline using crate encoders — no binary fixture files. Pattern from CONTEXT.md D-02 and §specifics:

```rust
// WR-01 multi-stream bzip2 archive — construct inline in test:
// 1. Create two separate bzip2-compressed tar entries back-to-back in one buffer
// 2. Wrap in a temp file that tar -xjf will read
// This proves MultiBzDecoder is needed (BzDecoder would stop after stream 1)
use bzip2::write::BzEncoder;
use bzip2::Compression;
use std::io::Write;

// Build two concatenated bzip2 streams:
let mut buf = Vec::new();
{
    let mut enc = BzEncoder::new(&mut buf, Compression::default());
    enc.write_all(b"stream one data").unwrap();
    enc.finish().unwrap();
}
{
    let mut enc = BzEncoder::new(&mut buf, Compression::default());
    enc.write_all(b"stream two data").unwrap();
    enc.finish().unwrap();
}
// buf now contains two concatenated bzip2 streams
// Wrap them in a tar archive before writing to temp file
```

```rust
// WR-04 multi-stream xz — construct inline in test:
use liblzma::write::XzEncoder;
use std::io::Write;

let mut buf = Vec::new();
{
    let mut enc = XzEncoder::new(&mut buf, 6);
    enc.write_all(b"xz stream one").unwrap();
    enc.finish().unwrap();
}
{
    let mut enc = XzEncoder::new(&mut buf, 6);
    enc.write_all(b"xz stream two").unwrap();
    enc.finish().unwrap();
}
// buf now contains two concatenated xz streams; use xz -d -c to verify both decoded
```

### Exit-code assertion pattern

```rust
// Pattern from existing xz_tests.rs lines 127-140 — assert failure + exit code:
xz_cmd()
    .arg("-d")
    .arg(bad_xz.to_str().unwrap())
    .assert()
    .failure()
    .code(1);

// Pattern from existing bzip2_tests.rs lines 157-167 — stderr content check:
bzip2_cmd()
    .arg("-d")
    .arg(path.to_str().unwrap())
    .assert()
    .failure()
    .stderr(predicate::str::contains("unknown suffix").or(predicate::str::contains("ignored")));
```

### Offline-only test pattern (curl_tests.rs)

```rust
// Existing offline curl tests (curl_tests.rs lines 24-38) — new WR-06/WR-07 tests
// must NOT require network access. They should test CLI behavior only:
// WR-06: curl -s -I <url> — can only be tested with a live server; mark #[ignore]
// WR-07: curl -o <partial> — can be tested offline by mocking a bad output path

// Template for a network test that verifies WR-06:
#[test]
#[ignore = "requires network access"]
fn silent_head_suppresses_all_output() {
    curl_cmd()
        .args(["-s", "-I", "http://httpbin.org/get"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
```

---

## Shared Patterns

### Graceful CLI argument error (exit code 2)

**Source:** `crates/gow-bzip2/src/lib.rs` lines 273-279; `crates/gow-gzip/src/lib.rs` lines 211-217; `crates/gow-curl/src/lib.rs` lines 139-145

**Apply to:** `crates/gow-tar/src/lib.rs` (WR-02 — currently uses `.unwrap()`)

```rust
let cli = match Cli::from_arg_matches(&matches) {
    Ok(c) => c,
    Err(e) => {
        eprintln!("<utility>: {e}");
        return 2;
    }
};
```

### Partial output file cleanup

**Source:** `crates/gow-bzip2/src/lib.rs` lines 158-164; `crates/gow-xz/src/lib.rs` lines 208-212; `crates/gow-gzip/src/lib.rs` lines 132-136

**Apply to:** `crates/gow-curl/src/lib.rs` (WR-07 — currently missing)

```rust
if let Err(e) = io::copy(&mut response, &mut file) {
    let _ = std::fs::remove_file(output_path);
    return Err(e.into());
}
```

### Unknown suffix rejection (exit code 1, continue)

**Source:** `crates/gow-bzip2/src/lib.rs` lines 206-215; `crates/gow-xz/src/lib.rs` lines 173-180

**Apply to:** `crates/gow-gzip/src/lib.rs` (WR-05 — currently falls through to `.out` output)

```rust
} else {
    eprintln!("<utility>: {converted}: unknown suffix -- ignored");
    exit_code = 1;
    continue;
};
```

### Silent flag guard for output

**Source:** `crates/gow-curl/src/lib.rs` line 93 (status line already guarded)

**Apply to:** `crates/gow-curl/src/lib.rs` (WR-06 — extend same guard to header loop)

```rust
if !cli.silent {
    println!("HTTP/1.1 {}", status);
    for (name, value) in response.headers() {
        println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
    }
}
```

---

## No Analog Found

None. All fixes are extensions or corrections of existing patterns already present in the codebase.

---

## Import Additions Required

| File | Addition needed | Reason |
|------|----------------|---------|
| `crates/gow-tar/src/lib.rs` | Change `BzDecoder` → `MultiBzDecoder` in import line 7 | WR-01 |
| `crates/gow-curl/src/lib.rs` | Add `use std::fs;` at top (currently only `std::fs::File` is imported) | WR-07 needs `std::fs::remove_file` |
| `crates/gow-tar/tests/tar_tests.rs` | Add `use bzip2::write::BzEncoder;`, `use bzip2::Compression;`, `use std::io::Write;`, `use tar::{Archive, Builder, Header};` for WR-01/WR-03 tests | inline fixture construction |
| `crates/gow-xz/tests/xz_tests.rs` | Add `use liblzma::write::XzEncoder;`, `use std::io::Write;` for WR-04 test | inline fixture construction |

---

## Metadata

**Analog search scope:** `crates/gow-bzip2/`, `crates/gow-curl/`, `crates/gow-gzip/`, `crates/gow-tar/`, `crates/gow-xz/`
**Files scanned:** 9 source/test files + 06-REVIEW.md
**Pattern extraction date:** 2026-04-29

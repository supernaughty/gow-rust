# Phase 04: S04 (Text Processing) - Pattern Map

**Mapped:** 2026-04-25
**Files analyzed:** 9 new crates (grep, sed, sort, uniq, tr, cut, diff, patch, awk)
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `crates/gow-grep/src/lib.rs` | utility/service | streaming + request-response | `crates/gow-grep/src/lib.rs` (self — already implemented) | exact |
| `crates/gow-sed/src/lib.rs` | utility/service | streaming + file-I/O (atomic) | `crates/gow-sed/src/lib.rs` (self — already implemented) | exact |
| `crates/gow-sort/src/lib.rs` | utility/service | batch + file-I/O | `crates/gow-sort/src/lib.rs` (self — already implemented) | exact |
| `crates/gow-uniq/src/lib.rs` | utility/service | streaming | `crates/gow-uniq/src/lib.rs` (self — already implemented) | exact |
| `crates/gow-tr/src/lib.rs` | utility/service | streaming (byte-level) | `crates/gow-tr/src/lib.rs` (self — already implemented) | exact |
| `crates/gow-cut/src/lib.rs` | utility/service | streaming | `crates/gow-cut/src/lib.rs` (self — already implemented) | exact |
| `crates/gow-diff/src/lib.rs` | utility/service | file-I/O + transform | `crates/gow-grep/src/lib.rs` | role-match |
| `crates/gow-patch/src/lib.rs` | utility/service | file-I/O + atomic rewrite | `crates/gow-sed/src/lib.rs` | role-match |
| `crates/gow-awk/src/lib.rs` | utility/service | streaming + transform | `crates/gow-grep/src/lib.rs` | role-match |
| `crates/gow-diff/Cargo.toml` | config | — | `crates/gow-grep/Cargo.toml` | role-match |
| `crates/gow-patch/Cargo.toml` | config | — | `crates/gow-sed/Cargo.toml` | role-match |
| `crates/gow-awk/Cargo.toml` | config | — | `crates/gow-grep/Cargo.toml` | role-match |
| `crates/gow-diff/tests/integration.rs` | test | — | `crates/gow-grep/tests/integration.rs` | role-match |
| `crates/gow-patch/tests/integration.rs` | test | — | `crates/gow-sort/tests/integration.rs` | role-match |
| `crates/gow-awk/tests/integration.rs` | test | — | `crates/gow-grep/tests/integration.rs` | role-match |

---

## Pattern Assignments

### ALL CRATES — `src/main.rs` (universal 3-line pattern)

Every binary in the workspace uses an identical 3-line `main.rs`. Copy verbatim, substituting the crate lib name.

**Analog:** `crates/gow-sed/src/main.rs` (lines 1-3):
```rust
fn main() {
    std::process::exit(uu_sed::uumain(std::env::args_os()));
}
```

For `diff`: replace `uu_sed` with `uu_diff`.
For `patch`: replace `uu_sed` with `uu_patch`.
For `awk`: replace `uu_sed` with `uu_awk`.

---

### `crates/gow-grep/src/lib.rs` (streaming, regex, color, recursion — ALREADY IMPLEMENTED)

**Status:** Complete. 339 lines. Reference as the gold-standard pattern for any crate that needs regex + termcolor + walkdir.

**Imports pattern** (`crates/gow-grep/src/lib.rs` lines 1-10):
```rust
use anyhow::Result;
use bstr::ByteSlice;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, ValueEnum};
use regex::bytes::{Regex, RegexBuilder};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, WriteColor};
use walkdir::WalkDir;
```

**uumain entry pattern** (`crates/gow-grep/src/lib.rs` lines 90-105):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    match run(cli) {
        Ok((true, _)) => 0,
        Ok((false, false)) => 1,
        Ok((false, true)) => 2,
        Err(e) => {
            eprintln!("grep: {}", e);
            2
        }
    }
}
```

**Color stdout pattern** (`crates/gow-grep/src/lib.rs` lines 123-128):
```rust
let color_choice = match cli.color {
    ColorArg::Always => ColorChoice::Always,
    ColorArg::Never => ColorChoice::Never,
    ColorArg::Auto => ColorChoice::Auto,
};
let mut stdout = gow_core::color::stdout(color_choice);
```

**Recursive file walking pattern** (`crates/gow-grep/src/lib.rs` lines 149-175):
```rust
for entry in WalkDir::new(path).into_iter() {
    match entry {
        Ok(e) if e.file_type().is_file() => {
            match search_file(e.path(), &regex, &cli, &mut stdout, show_filename) {
                Ok(matched) => { if matched { any_match = true; } }
                Err(err) => {
                    eprintln!("grep: {}: {}", e.path().display(), err);
                    any_error = true;
                }
            }
        }
        Ok(_) => {}
        Err(err) => {
            eprintln!("grep: {}", err);
            any_error = true;
        }
    }
}
```

**Stdin fallback pattern** (`crates/gow-grep/src/lib.rs` lines 133-147):
```rust
for path in &cli.files {
    if path == Path::new("-") {
        // read stdin
    } else if path.is_dir() {
        // walk or error
    } else {
        // open file
    }
}
```

**Unit test pattern in lib.rs** (`crates/gow-grep/src/lib.rs` lines 296-338):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use termcolor::Buffer;

    #[test]
    fn test_basic_match() -> Result<()> {
        let regex = Regex::new("world")?;
        let cli = Cli::try_parse_from(["grep", "world"])?;
        let mut buf = Buffer::no_color();
        let input = b"hello\nworld\nrust\n";
        process_reader(&input[..], "test", &regex, &cli, &mut buf, false)?;
        assert_eq!(buf.into_inner(), b"world\n");
        Ok(())
    }
}
```

---

### `crates/gow-sed/src/lib.rs` (streaming, regex, atomic in-place — ALREADY IMPLEMENTED)

**Status:** Complete. 285 lines. Reference for any crate requiring in-place file editing.

**Imports pattern** (`crates/gow-sed/src/lib.rs` lines 1-10):
```rust
use anyhow::{Context, Result};
use bstr::ByteSlice;
use clap::Parser;
use gow_core::fs::atomic_rewrite;
use regex::{Regex, RegexBuilder};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{PathBuf};
```

**uumain pattern — simple error propagation style** (`crates/gow-sed/src/lib.rs` lines 40-47):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    if let Err(e) = run(args) {
        eprintln!("sed: {}", e);
        return 1;
    }
    0
}
```

**Atomic in-place rewrite pattern** (`crates/gow-sed/src/lib.rs` lines 100-103):
```rust
atomic_rewrite(&path, |input| {
    process_content(input, &commands, args.quiet)
        .map_err(|e| gow_core::error::GowError::Custom(e.to_string()))
}).with_context(|| format!("failed to edit file {} in place", path.display()))?;
```

**Line-ending detection pattern** (`crates/gow-sed/src/lib.rs` lines 248-252):
```rust
let line_ending: &[u8] = if content.contains_str("\r\n") {
    b"\r\n"
} else {
    b"\n"
};
```

**Args parsing — try_parse_from with exit(2) style** (`crates/gow-sed/src/lib.rs` lines 50-56):
```rust
let args = match Args::try_parse_from(args) {
    Ok(args) => args,
    Err(e) => {
        eprintln!("{}", e);
        std::process::exit(2);
    }
};
```

---

### `crates/gow-sort/src/lib.rs` (batch, external merge sort, file-I/O — ALREADY IMPLEMENTED)

**Status:** Complete. 363 lines. Reference for any crate requiring large-file buffering and temp-file merge.

**Imports pattern** (`crates/gow-sort/src/lib.rs` lines 1-16):
```rust
use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use bstr::io::BufReadExt;
use bstr::ByteSlice;
use clap::{Arg, ArgAction, Command};
use itertools::Itertools;
use tempfile::NamedTempFile;
```

**uumain pattern — Command builder style** (`crates/gow-sort/src/lib.rs` lines 26-55):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    // extract flags from matches...
    if let Err(e) = run_sort(operands, config, output_file) {
        eprintln!("sort: {e}");
        return 1;
    }
    0
}
```

**MSYS path conversion pattern** (`crates/gow-sort/src/lib.rs` lines 87-90):
```rust
let converted = gow_core::path::try_convert_msys_path(&op);
let file = File::open(Path::new(&converted))?;
```

**Boxed stdin/file reader pattern** (`crates/gow-sort/src/lib.rs` lines 84-91):
```rust
let mut input: Box<dyn BufRead> = if op == "-" {
    Box::new(BufReader::new(io::stdin().lock()))
} else {
    let converted = gow_core::path::try_convert_msys_path(&op);
    let file = File::open(Path::new(&converted))?;
    Box::new(BufReader::new(file))
};
```

---

### `crates/gow-uniq/src/lib.rs` (streaming dedup — ALREADY IMPLEMENTED)

**Status:** Complete. 239 lines. Reference for line-by-line streaming with prev/current state tracking.

**uumain pattern — try_parse_from + boxed I/O** (`crates/gow-uniq/src/lib.rs` lines 46-91):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args = match Args::try_parse_from(args) {
        Ok(args) => args,
        Err(e) => { eprintln!("{}", e); return 1; }
    };
    // boxed reader/writer setup...
    if let Err(e) = process_uniq(&mut *input_reader, &mut *output_writer, &args) {
        eprintln!("uniq: {}", e);
        return 1;
    }
    0
}
```

**Boxed output file/stdout pattern** (`crates/gow-uniq/src/lib.rs` lines 73-84):
```rust
let mut output_writer: Box<dyn Write> = if let Some(ref output_path) = args.output {
    match File::create(output_path) {
        Ok(f) => Box::new(f),
        Err(e) => { eprintln!("uniq: {}: {}", output_path, e); return 1; }
    }
} else {
    Box::new(io::stdout().lock())
};
```

**read_until byte loop pattern** (`crates/gow-uniq/src/lib.rs` lines 93-127):
```rust
fn process_uniq(input: &mut dyn BufRead, output: &mut dyn Write, args: &Args) -> io::Result<()> {
    let delim = if args.zero_terminated { b'\0' } else { b'\n' };
    let mut line = Vec::new();
    let mut prev_line: Option<Vec<u8>> = None;
    let mut count = 0;
    loop {
        line.clear();
        match input.read_until(delim, &mut line) {
            Ok(0) => { /* flush last */ break; }
            Ok(_) => { /* compare and count */ }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
```

---

### `crates/gow-tr/src/lib.rs` (byte-level stream filter — ALREADY IMPLEMENTED)

**Status:** Complete. 215 lines. Reference for raw byte-level `Read` (not `BufRead`) processing.

**Raw byte read loop** (`crates/gow-tr/src/lib.rs` lines 119-149):
```rust
let mut buf = [0u8; 8192];
let mut last_char: Option<u8> = None;
loop {
    let n = match stdin_lock.read(&mut buf) {
        Ok(0) => break,
        Ok(n) => n,
        Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
        Err(e) => { eprintln!("tr: {}", e); return 1; }
    };
    for &b in &buf[..n] {
        // per-byte processing
        if let Err(e) = stdout_lock.write_all(&[out_char]) {
            eprintln!("tr: {}", e); return 1;
        }
    }
}
```

---

### `crates/gow-cut/src/lib.rs` (field/byte streaming — ALREADY IMPLEMENTED)

**Status:** Complete. 236 lines. Reference for delimited field processing with Mode enum dispatch.

**Mode enum dispatch pattern** (`crates/gow-cut/src/lib.rs` lines 107-111):
```rust
enum Mode {
    Bytes(Vec<Range>),
    Characters(Vec<Range>),
    Fields(Vec<Range>),
}
```

**read_until + CRLF strip pattern** (`crates/gow-cut/src/lib.rs` lines 146-157):
```rust
match input.read_until(b'\n', &mut line) {
    Ok(0) => break,
    Ok(_) => {
        if line.ends_with(b"\n") {
            line.pop();
            if line.ends_with(b"\r") { line.pop(); }
        }
        // process line
    }
    Err(e) => return Err(e),
}
```

---

### `crates/gow-diff/src/lib.rs` (file comparison — STUB, needs implementation)

**Status:** Stub — only calls `eprintln!("gow-diff: not implemented")` and returns 1.

**Analog:** `crates/gow-grep/src/lib.rs` (two-file input, colored output, exit-code conventions)

**Target Cargo.toml additions needed:**
```toml
[dependencies]
similar = "2"           # diff algorithms (Myers, Patience, LCS)
bstr = { workspace = true }
```

**Recommended lib.rs structure (copy from grep, adapt):**
1. `use similar::{ChangeTag, TextDiff};`
2. Clap `Args` struct with `-u` (unified), `-c` (context), `-r` (recursive), `-N` (treat absent as empty)
3. `pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32` — call `gow_core::init()`, parse args, call `run()`, return 0/1/2
4. `fn run(args: Args) -> Result<i32>` — open two files, call `diff_files()`, print unified format
5. Exit code: 0 = no differences, 1 = differences found, 2 = error (same as GNU diff)

**Exit code pattern** (copy uumain error-propagation style from `crates/gow-grep/src/lib.rs` lines 90-105, adapt return values):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    match run(args) {
        Ok(0) => 0,   // no differences
        Ok(_) => 1,   // differences found
        Err(e) => { eprintln!("diff: {}", e); 2 }
    }
}
```

---

### `crates/gow-patch/src/lib.rs` (apply unified diffs — STUB, needs implementation)

**Status:** Stub — only calls `eprintln!("gow-patch: not implemented")` and returns 1.

**Analog:** `crates/gow-sed/src/lib.rs` (file-in, atomic rewrite out, `--dry-run` like `-n` quiet)

**Target Cargo.toml additions needed:**
```toml
[dependencies]
patch = "0.7"           # pure-Rust unified diff parser and applier
bstr = { workspace = true }
```

**Recommended lib.rs structure:**
1. `use patch::Patch;`
2. Clap `Args` struct with `-p NUM` (strip prefix), `-R` (reverse), `--dry-run`, `-i FILE` (patch file), positional file(s)
3. `pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32` — `gow_core::init()`, try_parse_from, call `run()`, return 0/1
4. `fn run(args: Args) -> Result<()>` — read patch from stdin or `-i`, for each target file call `atomic_rewrite` with the patched content

**Atomic rewrite integration** (copy from `crates/gow-sed/src/lib.rs` lines 100-103):
```rust
atomic_rewrite(&path, |input| {
    apply_patch(input, &patch)
        .map_err(|e| gow_core::error::GowError::Custom(e.to_string()))
}).with_context(|| format!("patch: failed to patch {}", path.display()))?;
```

---

### `crates/gow-awk/src/lib.rs` (AWK interpreter — STUB, needs implementation)

**Status:** Stub — only calls `eprintln!("gow-awk: not implemented")` and returns 1.

**Analog:** `crates/gow-grep/src/lib.rs` (multi-file stdin/file dispatch, streaming line processing)

**Target Cargo.toml additions needed:**
```toml
[dependencies]
# Option A: use frawk if available as a library
# Option B: implement POSIX AWK subset using regex + bstr
regex = { workspace = true }
bstr = { workspace = true }
```

**Recommended lib.rs structure:**
1. Clap `Args` struct: `-F sep` (field separator), `-v var=val` (variable assignment), `-f prog_file`, positional `program` + `files`
2. `pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32` — `gow_core::init()`, try_parse_from, run
3. Streaming pattern: open each file (or stdin), iterate lines with `bstr::io::BufReadExt::for_byte_line`, split fields by `FS`, evaluate BEGIN/END/pattern-action rules
4. Copy stdin/file dispatch pattern from `crates/gow-sort/src/lib.rs` lines 83-91

---

## Cargo.toml Patterns

### Completed crates — minimal deps (tr, uniq pattern)

**Analog:** `crates/gow-uniq/Cargo.toml` (lines 1-30):
```toml
[package]
name = "gow-{util}"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU {util} — Windows port."

[[bin]]
name = "{util}"
path = "src/main.rs"

[lib]
name = "uu_{util}"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
```

### Grep-style crates (diff, awk — need regex + walkdir + termcolor)

**Analog:** `crates/gow-grep/Cargo.toml` (lines 18-28):
```toml
[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
regex = { workspace = true }
walkdir = { workspace = true }
termcolor = { workspace = true }
bstr = { workspace = true }
windows-sys = { workspace = true }
```

### Sed-style crates (patch — needs atomic_rewrite, regex, bstr)

**Analog:** `crates/gow-sed/Cargo.toml` (lines 18-27):
```toml
[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
regex = { workspace = true }
bstr = { workspace = true }
```

---

## Integration Test Patterns

### Standard stdin test (copy from `crates/gow-tr/tests/integration.rs` lines 1-11):
```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_{util}_basic() {
    let mut cmd = Command::cargo_bin("{util}").unwrap();
    cmd.arg(...)
        .write_stdin("input\n")
        .assert()
        .success()
        .stdout("expected\n");
}
```

### File-based test with tempdir (copy from `crates/gow-grep/tests/integration.rs` lines 17-28):
```rust
use std::fs;
use tempfile::tempdir;

#[test]
fn test_{util}_file() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("test.txt");
    fs::write(&file_path, "content\n").unwrap();

    let mut cmd = Command::cargo_bin("{util}").unwrap();
    cmd.arg(file_path.to_str().unwrap());
    cmd.assert().success().stdout("expected\n");
}
```

### Exit code test (copy from `crates/gow-grep/tests/integration.rs` lines 121-129):
```rust
#[test]
fn test_{util}_no_match_exit_code() {
    let mut cmd = Command::cargo_bin("{util}").unwrap();
    cmd.arg("nomatch")
        .write_stdin("hello\nworld\n")
        .assert()
        .failure()
        .code(1);
}
```

---

## Shared Patterns

### gow_core::init() — mandatory first call
**Source:** `crates/gow-core/src/lib.rs` line 16
**Apply to:** Every `uumain()` function in every lib.rs
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();   // MUST be first line
    // ...
}
```

### GNU error message format
**Source:** `crates/gow-grep/src/lib.rs` line 102 / `crates/gow-sort/src/lib.rs` line 51
**Apply to:** All error `eprintln!` calls
```rust
eprintln!("{util}: {}", e);          // top-level errors
eprintln!("{util}: {}: {}", path, e); // per-file errors
```

### Atomic in-place rewrite
**Source:** `crates/gow-core/src/fs.rs` lines 115-146 + `crates/gow-sed/src/lib.rs` lines 100-103
**Apply to:** `gow-patch` (and any future utility that modifies files in-place)
```rust
use gow_core::fs::atomic_rewrite;

atomic_rewrite(&path, |input| {
    transform(input).map_err(|e| gow_core::error::GowError::Custom(e.to_string()))
}).with_context(|| format!("failed to edit {}", path.display()))?;
```

### MSYS/Unix path conversion
**Source:** `crates/gow-sort/src/lib.rs` lines 87-88
**Apply to:** All crates that accept file paths as arguments (diff, patch, awk)
```rust
let converted = gow_core::path::try_convert_msys_path(&path_str);
let file = File::open(Path::new(&converted))?;
```

### CRLF line ending detection and preservation
**Source:** `crates/gow-sed/src/lib.rs` lines 248-252
**Apply to:** `gow-diff`, `gow-patch`, `gow-awk` (any crate that processes text lines and writes back)
```rust
let line_ending: &[u8] = if content.contains_str("\r\n") { b"\r\n" } else { b"\n" };
```

### GNU arg parse (parse_gnu vs try_parse_from)
**Source:** `crates/gow-core/src/args.rs` line 36
**Apply to:** Crates that use `clap::Command` builder style (sort, grep) — use `gow_core::args::parse_gnu()`
**Apply to:** Crates using `#[derive(Parser)]` — use `Args::try_parse_from(args)`; both patterns are valid and in use.

---

## No Analog Found

All 9 crates have analogs within the existing codebase. Three stubs (diff, patch, awk) need full implementation — their structural pattern is clear from the already-implemented S04 peers above.

| File | Status | Note |
|---|---|---|
| `crates/gow-diff/src/lib.rs` | Stub | Needs `similar` crate integration; analog = gow-grep |
| `crates/gow-patch/src/lib.rs` | Stub | Needs `patch` crate integration; analog = gow-sed |
| `crates/gow-awk/src/lib.rs` | Stub | Needs `frawk` or AWK subset; analog = gow-grep + gow-sort |

---

## Metadata

**Analog search scope:** `crates/gow-grep/`, `crates/gow-sed/`, `crates/gow-sort/`, `crates/gow-uniq/`, `crates/gow-tr/`, `crates/gow-cut/`, `crates/gow-diff/`, `crates/gow-patch/`, `crates/gow-awk/`, `crates/gow-core/`
**Files scanned:** 24 (9 lib.rs, 9 Cargo.toml, 6 integration test files, gow-core lib/fs/args)
**Pattern extraction date:** 2026-04-25

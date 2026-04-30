# Phase 11: new-utilities-wave2 — Pattern Map

**Mapped:** 2026-04-29
**Files analyzed:** 10 new crates (+ workspace Cargo.toml, build.bat, extras/bin/[.bat)
**Analogs found:** 10 / 10 (all have close matches; 2 have exact matches)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/gow-whoami/src/lib.rs` | utility | request-response (Win32 API → stdout) | `crates/gow-df/src/lib.rs` | exact (windows-sys unsafe call + uumain skeleton) |
| `crates/gow-uname/src/lib.rs` | utility | request-response (Win32 API → stdout) | `crates/gow-df/src/lib.rs` | exact (windows-sys unsafe call + uumain skeleton) |
| `crates/gow-fmt/src/lib.rs` | utility | transform (stdin/file → wrapped stdout) | `crates/gow-fold/src/lib.rs` | exact (line-wrap loop, stdin-or-files pattern) |
| `crates/gow-unlink/src/lib.rs` | utility | file-I/O (single remove) | `crates/gow-rm/src/lib.rs` | role-match (file removal; unlink is simpler subset) |
| `crates/gow-paste/src/lib.rs` | utility | transform (multi-file → merged stdout) | `crates/gow-cat/src/lib.rs` | role-match (multi-file + stdin `-` handling) |
| `crates/gow-join/src/lib.rs` | utility | transform (sorted-file merge join) | `crates/gow-sort/src/lib.rs` | role-match (file I/O + bstr line iteration) |
| `crates/gow-split/src/lib.rs` | utility | file-I/O (read input → write output files) | `crates/gow-tac/src/lib.rs` | role-match (read-all then write output) |
| `crates/gow-printf/src/lib.rs` | utility | transform (argv → formatted stdout) | `crates/gow-echo/src/lib.rs` | role-match (argv processing → stdout write) |
| `crates/gow-expr/src/lib.rs` | utility | transform (argv tokens → evaluated stdout) | `crates/gow-seq/src/lib.rs` | role-match (numeric evaluation + non-standard exit codes) |
| `crates/gow-test/src/lib.rs` | utility | transform (argv condition → exit code) | `crates/gow-seq/src/lib.rs` | partial-match (argc parsing + exit-code control) |
| `Cargo.toml` (root workspace) | config | — | self (modification) | — |
| `extras/bin/[.bat` | config | — | existing `extras/bin/*.bat` shims | exact |
| Each `Cargo.toml` (crate) | config | — | `crates/gow-df/Cargo.toml` | exact |
| Each `build.rs` | config | — | `crates/gow-df/build.rs` | exact |
| Each `src/main.rs` | entrypoint | — | `crates/gow-df/src/main.rs` | exact |
| Each `tests/integration.rs` | test | — | `crates/gow-fold/tests/integration.rs` | exact |

---

## Pattern Assignments

### `crates/gow-whoami/src/lib.rs` + `crates/gow-uname/src/lib.rs` (Win32 API utilities)

**Analog:** `crates/gow-df/src/lib.rs`

**Imports pattern** (gow-df/src/lib.rs lines 6-10):
```rust
use std::ffi::OsString;
use std::io::Write;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetLogicalDriveStringsW};
```
For whoami, replace the windows-sys import with:
```rust
use windows_sys::Win32::System::WindowsProgramming::GetUserNameW;
```
For uname, replace with:
```rust
use windows_sys::Win32::System::SystemInformation::{
    GetNativeSystemInfo, OSVERSIONINFOW,
    PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM64,
    PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
};
use windows_sys::Wdk::System::SystemServices::RtlGetVersion;
```

**Cli struct pattern** (gow-df/src/lib.rs lines 13-30):
```rust
#[derive(Parser, Debug)]
#[command(
    name = "df",
    about = "GNU df — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Print sizes in human-readable format (e.g. 1K, 234M, 2G)
    #[arg(short = 'h', long = "human-readable", action = ArgAction::SetTrue)]
    human: bool,
}
```

**Win32 unsafe call pattern** (gow-df/src/lib.rs lines 34-56):
```rust
fn get_drives() -> Vec<String> {
    let mut buf = [0u16; 512];
    // SAFETY: buf and len are valid; returns total character count written, or 0 on error
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
    if len == 0 {
        return Vec::new();
    }
    // ... process buf ...
    String::from_utf16_lossy(&buf[start..i])
}
```
Copy this structure for whoami's `get_current_username()`:
```rust
pub fn get_current_username() -> Option<String> {
    let mut buf = [0u16; 257];   // UNLEN + 1 = 257
    let mut size = buf.len() as u32;
    // SAFETY: buf is valid; size is buffer capacity on input, chars written on output
    let ok = unsafe { GetUserNameW(buf.as_mut_ptr(), &mut size) };
    if ok == 0 {
        return None;
    }
    let end = (size as usize).saturating_sub(1);  // size includes null terminator
    Some(String::from_utf16_lossy(&buf[..end]))
}
```

**uumain skeleton** (gow-df/src/lib.rs lines 176-188):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("df: {e}");
            return 2;
        }
    };
    run(&cli)
}
```
Replace `"df"` with `"whoami"` / `"uname"` in the eprintln.

**Error message pattern** — eprintln uses the GetLastError for Win32 failures:
```rust
// From RESEARCH.md Pattern 2 (verified against windows-sys source)
return Err(format!("whoami: GetUserNameW failed (error {})",
    unsafe { windows_sys::Win32::Foundation::GetLastError() }));
```

---

### `crates/gow-fmt/src/lib.rs` (paragraph-aware line wrapper)

**Analog:** `crates/gow-fold/src/lib.rs` — exact match, same role and data flow

**Imports pattern** (gow-fold/src/lib.rs lines 1-6):
```rust
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
```

**Cli struct pattern** (gow-fold/src/lib.rs lines 8-39):
```rust
#[derive(Parser, Debug)]
#[command(
    name = "fold",
    about = "GNU fold — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Wrap width in bytes (default 80)
    #[arg(short = 'w', long = "width", default_value = "80")]
    width: usize,

    /// Files to fold (reads stdin if none provided)
    files: Vec<String>,
}
```
For fmt: change `name = "fmt"`, remove `-b`/`-s` flags, add `-p` (prefix) if needed.

**Stdin-or-files dispatch pattern** (gow-fold/src/lib.rs lines 107-138):
```rust
fn run(cli: &Cli) -> i32 {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0;

    if cli.files.is_empty() {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        exit_code |= wrap_reader(reader, cli.width, cli.spaces, &mut out);
    } else {
        for file_path in &cli.files {
            let converted = gow_core::path::try_convert_msys_path(file_path);
            let path = Path::new(&converted);
            match File::open(path) {
                Ok(f) => {
                    let reader = BufReader::new(f);
                    exit_code |= wrap_reader(reader, cli.width, cli.spaces, &mut out);
                }
                Err(e) => {
                    eprintln!("fold: {converted}: {e}");
                    exit_code = 1;
                }
            }
        }
    }
    exit_code
}
```
For fmt: rename `wrap_reader` → `fmt_reader`; change `"fold:"` to `"fmt:"`.

**Line processing core** (gow-fold/src/lib.rs lines 92-105):
```rust
fn wrap_reader(reader: impl BufRead, width: usize, spaces: bool, out: &mut impl Write) -> i32 {
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                wrap_line(line.as_bytes(), width, spaces, out);
            }
            Err(e) => {
                eprintln!("fold: read error: {e}");
                return 1;
            }
        }
    }
    0
}
```
For fmt: `fmt` needs to accumulate paragraph lines (blank-line-separated), then reflow each paragraph at width. Swap the `reader.lines()` loop body to collect into a paragraph buffer, flush on blank line.

---

### `crates/gow-unlink/src/lib.rs` (single-file removal)

**Analog:** `crates/gow-rm/src/lib.rs` — role-match (same removal semantics, much simpler)

**Imports pattern** (gow-rm/src/lib.rs lines 1-11):
```rust
use std::ffi::OsString;
use std::fs;
use std::path::Path;
```
(unlink needs nothing else — no clap, just raw args)

**uumain for trivial utilities without clap** (based on gow-rm pattern, simplified):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    // Skip argv[0]
    let operands: Vec<String> = args_vec[1..]
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    if operands.len() != 1 {
        eprintln!("unlink: requires exactly one argument");
        return 2;
    }

    let converted = gow_core::path::try_convert_msys_path(&operands[0]);
    match fs::remove_file(&converted) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("unlink: cannot unlink '{}': {}", converted, e);
            1
        }
    }
}
```

**File removal core** (gow-rm/src/lib.rs lines 152-154):
```rust
fs::remove_file(path).map_err(|e| {
    anyhow::anyhow!("cannot remove '{}': {}", path.display(), e)
})?;
```

---

### `crates/gow-paste/src/lib.rs` (multi-file line zipper)

**Analog:** `crates/gow-cat/src/lib.rs` — role-match (multi-file + stdin `-` handling)

**Stdin `-` handling pattern** (gow-cat/src/lib.rs lines 234-240):
```rust
for op in &operands {
    if op == "-" {
        if let Err(e) = cat_reader(io::stdin().lock(), &mut stdout, opts, &mut state) {
            eprintln!("cat: -: {e}");
            exit_code = 1;
        }
        continue;
    }
    let converted = gow_core::path::try_convert_msys_path(op);
    // ...
}
```
For paste: open each operand as a `Box<dyn BufRead>` — use stdin for `-`, `BufReader::new(File::open(...))` for file paths.

**File opening for multi-file dispatch** (gow-cat/src/lib.rs lines 242-256):
```rust
let converted = gow_core::path::try_convert_msys_path(op);
let path = Path::new(&converted);
match File::open(path) {
    Ok(f) => {
        // process f
    }
    Err(e) => {
        eprintln!("cat: {converted}: {e}");
        exit_code = 1;
        // Continue — GNU processes all operands
    }
}
```

**Cli struct with files Vec** — copy from gow-fold pattern; add `-d` delimiter:
```rust
#[derive(Parser, Debug)]
#[command(name = "paste", about = "GNU paste — Windows port.", ...)]
struct Cli {
    /// Delimiter list (default: tab)
    #[arg(short = 'd', long = "delimiters", default_value = "\t")]
    delimiters: String,

    /// Serial: paste one file at a time
    #[arg(short = 's', long = "serial", action = ArgAction::SetTrue)]
    serial: bool,

    /// Files (use - for stdin)
    files: Vec<String>,
}
```

---

### `crates/gow-join/src/lib.rs` (sorted-file merge join)

**Analog:** `crates/gow-sort/src/lib.rs` — role-match (file I/O + bstr line iteration)

**bstr import pattern** (gow-sort/src/lib.rs lines 11-12):
```rust
use bstr::io::BufReadExt;
use bstr::ByteSlice;
```

**File opening with path conversion** (gow-sort pattern, also seen in gow-fold/gow-tac):
```rust
let converted = gow_core::path::try_convert_msys_path(file_path);
match File::open(&converted) {
    Ok(f) => { /* use BufReader::new(f) */ }
    Err(e) => {
        eprintln!("join: {converted}: {e}");
        exit_code = 1;
    }
}
```

**bstr byte-safe line iteration** (copy from gow-sort):
```rust
use bstr::io::BufReadExt;

let reader = BufReader::new(f);
reader.for_byte_line(|line| {
    // line: &[u8] — byte-safe, no UTF-8 decode panic
    Ok(true)
})?;
```

**Cli struct for join** (modeled on gow-sort/gow-fold Cli derive):
```rust
#[derive(Parser, Debug)]
#[command(name = "join", about = "GNU join — Windows port.", ...)]
struct Cli {
    #[arg(short = '1', value_name = "FIELD")]
    field1: Option<usize>,  // 1-based join field for file1

    #[arg(short = '2', value_name = "FIELD")]
    field2: Option<usize>,  // 1-based join field for file2

    #[arg(short = 't', value_name = "CHAR")]
    separator: Option<String>,

    #[arg(short = 'a', value_name = "FILENUM")]
    print_unpaired: Option<u8>,  // 1 or 2

    /// file1 and file2 (use - for stdin)
    files: Vec<String>,
}
```

---

### `crates/gow-split/src/lib.rs` (file splitter)

**Analog:** `crates/gow-tac/src/lib.rs` — role-match (read input, write output files)

**Read-all-then-process pattern** (gow-tac/src/lib.rs lines 53-80):
```rust
fn run(cli: &Cli) -> i32 {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0;

    if cli.files.is_empty() {
        let mut buf = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut buf) {
            eprintln!("tac: stdin: {e}");
            exit_code = 1;
        }
        reverse_and_write(&buf, &mut out);
    } else {
        for file_path in &cli.files {
            let converted = gow_core::path::try_convert_msys_path(file_path);
            match fs::read(&converted) {
                Ok(buf) => { reverse_and_write(&buf, &mut out); }
                Err(e) => {
                    eprintln!("tac: {converted}: {e}");
                    exit_code = 1;
                }
            }
        }
    }
    exit_code
}
```
For split: replace `reverse_and_write` with output-file-writing logic; replace stdout with a file handle for each output chunk. Use `io::stdin()` for `-` or missing file argument.

**Output file creation pattern for split** (no direct codebase analog — from RESEARCH.md):
```rust
fn next_suffix(suffix: &mut Vec<u8>) {
    let mut i = suffix.len() - 1;
    loop {
        if suffix[i] < b'z' {
            suffix[i] += 1;
            return;
        }
        suffix[i] = b'a';
        if i == 0 {
            suffix.iter_mut().for_each(|c| *c = b'a');
            suffix.push(b'a');
            return;
        }
        i -= 1;
    }
}
```

**Cli struct for split**:
```rust
#[derive(Parser, Debug)]
#[command(name = "split", about = "GNU split — Windows port.", ...)]
struct Cli {
    #[arg(short = 'l', value_name = "NUMBER")]
    lines: Option<usize>,

    #[arg(short = 'b', value_name = "SIZE")]
    bytes: Option<String>,  // parse K/M/G suffix

    #[arg(short = 'n', value_name = "CHUNKS")]
    chunks: Option<usize>,

    #[arg(short = 'a', long = "suffix-length", default_value = "2")]
    suffix_len: usize,

    /// Input file (use - or omit for stdin)
    input: Option<String>,

    /// Output file prefix (default: x)
    prefix: Option<String>,
}
```

---

### `crates/gow-printf/src/lib.rs` (format string evaluator)

**Analog:** `crates/gow-echo/src/lib.rs` — role-match (argv processing → stdout write)

**Manual argv iteration without clap** (gow-echo/src/lib.rs lines 54-68):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let mut iter = args.into_iter();
    let _argv0 = iter.next(); // skip program name

    let mut body: Vec<OsString> = Vec::new();
    for arg in iter {
        body.push(arg);
    }
    // ...
}
```

**stdout lock + write pattern** (gow-echo/src/lib.rs lines 140-157):
```rust
let stdout = io::stdout();
let mut out = stdout.lock();

if let Err(e) = out.write_all(bytes) {
    eprintln!("echo: {e}");
    return 1;
}
```
For printf: use the same stdout lock; write formatted output in a loop (one pass per batch of arguments, repeating format string until args exhausted).

**printf does NOT use clap** — same as echo. It parses argv[1] as the format string, argv[2..] as arguments.

---

### `crates/gow-expr/src/lib.rs` (expression evaluator)

**Analog:** `crates/gow-seq/src/lib.rs` — role-match (numeric evaluation + non-standard exit codes)

**Numeric arg parsing / error pattern** (gow-seq/src/lib.rs lines 39-58):
```rust
fn run(cli: &Cli) -> i32 {
    let (first_str, inc_str, last_str) = match cli.numbers.len() {
        0 => {
            eprintln!("seq: missing operand");
            return 2;
        }
        // ...
        _ => {
            eprintln!("seq: extra operand '{}'", cli.numbers[3]);
            return 2;
        }
    };
    let first: f64 = match first_str.parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("seq: invalid floating point argument: '{first_str}'");
            return 1;
        }
    };
    // ...
}
```

**Non-standard exit code control** — expr uses `std::process::exit` directly (same approach as tac/seq return from uumain):
```rust
// RESEARCH.md Pattern 3 — critical: exit code semantics are INVERTED
fn main() -> ! {
    gow_core::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("expr: missing operand");
        std::process::exit(2);
    }
    match evaluate(&args) {
        Ok(result) => {
            println!("{}", result);
            // exit 0 = non-null result; exit 1 = null ("0" or "")
            let is_null = result.is_empty() || result == "0";
            std::process::exit(if is_null { 1 } else { 0 });
        }
        Err(e) => {
            eprintln!("expr: {}", e);
            std::process::exit(2);
        }
    }
}
```
Note: `main.rs` for expr MUST use `-> !` with `std::process::exit` because the exit codes are non-boolean. The standard `uumain` i32 return path via `std::process::exit(uumain(...))` is fine too — both patterns exist in the workspace.

**uumain return from seq** (gow-seq/src/lib.rs lines 181-193) — standard skeleton:
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("seq: {e}");
            return 2;
        }
    };
    run(&cli)
}
```
For expr: skip clap entirely — expr receives all operands as raw argv tokens. Collect `args_vec[1..]` as `Vec<String>` and pass to `evaluate()`.

---

### `crates/gow-test/src/lib.rs` (POSIX condition evaluator)

**Analog:** `crates/gow-seq/src/lib.rs` — partial-match (argc parsing + exit code control)

**No clap** — test parses raw argv. The argv[0] detection for bracket mode (from RESEARCH.md Pattern 4):
```rust
// RESEARCH.md Pattern 4 — bracket mode via --_bracket_ sentinel
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();

    let mut expr_args: Vec<String> = args_vec[1..]
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    // Detect bracket mode via sentinel flag inserted by [.bat shim
    let bracket_mode = expr_args.first().map(|s| s == "--_bracket_").unwrap_or(false);
    if bracket_mode {
        expr_args.remove(0);  // strip sentinel
        match expr_args.last().map(String::as_str) {
            Some("]") => { expr_args.pop(); }
            _ => {
                eprintln!("[: missing ']'");
                return 2;
            }
        }
    }

    evaluate_test(&expr_args)
}
```

**Test exit codes** (OPPOSITE of expr — 0 = true, 1 = false, 2 = error):
```rust
fn evaluate_test(args: &[String]) -> i32 {
    match parse_and_evaluate(args) {
        Ok(true)  => 0,
        Ok(false) => 1,
        Err(_)    => 2,
    }
}
```

**File predicate using std::fs::metadata**:
```rust
use std::fs;

fn file_test(op: &str, path: &str) -> bool {
    let converted = gow_core::path::try_convert_msys_path(path);
    match op {
        "-e" => fs::metadata(&converted).is_ok(),
        "-f" => fs::metadata(&converted).map(|m| m.is_file()).unwrap_or(false),
        "-d" => fs::metadata(&converted).map(|m| m.is_dir()).unwrap_or(false),
        "-s" => fs::metadata(&converted).map(|m| m.len() > 0).unwrap_or(false),
        "-r" => fs::metadata(&converted).is_ok(), // Windows: readable if file exists
        "-w" => fs::metadata(&converted).map(|m| !m.permissions().readonly()).unwrap_or(false),
        "-L" => fs::symlink_metadata(&converted).map(|m| m.file_type().is_symlink()).unwrap_or(false),
        // -x: Windows convention — check for .exe/.bat/.com/.cmd extension
        "-x" => {
            let p = std::path::Path::new(&converted);
            p.exists() && matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("exe") | Some("bat") | Some("com") | Some("cmd")
            )
        }
        _ => false,
    }
}
```

---

## Shared Patterns

### uumain Skeleton (all crates except echo/printf/expr/test)
**Source:** `crates/gow-df/src/lib.rs` lines 176-188
**Apply to:** gow-whoami, gow-uname, gow-fmt, gow-join, gow-split, gow-seq-family
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("<utility>: {e}");
            return 2;
        }
    };
    run(&cli)
}
```

### main.rs Entrypoint (all crates)
**Source:** `crates/gow-df/src/main.rs` lines 1-3
**Apply to:** All 10 new crates
```rust
fn main() {
    std::process::exit(uu_df::uumain(std::env::args_os()));
}
```
Replace `uu_df` with the lib name (`uu_whoami`, `uu_uname`, `uu_paste`, etc.).

### Crate Cargo.toml Structure
**Source:** `crates/gow-df/Cargo.toml`
**Apply to:** All 10 new crates
```toml
[package]
name = "gow-<utility>"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU <utility> — Windows port."

[[bin]]
name = "<utility>"
path = "src/main.rs"

[lib]
name = "uu_<utility>"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
# EXTRA DEPS per crate (see table below)

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

**Per-crate extra deps** (copy exactly):
| Crate | Add to [dependencies] |
|-------|-----------------------|
| gow-whoami | `windows-sys = { workspace = true }` |
| gow-uname | `windows-sys = { workspace = true }` |
| gow-paste | `bstr = { workspace = true }` |
| gow-join | `bstr = { workspace = true }` |
| gow-split | `bstr = { workspace = true }` |
| gow-printf | (none extra) |
| gow-expr | `regex = { workspace = true }` |
| gow-test | (none extra) |
| gow-fmt | `bstr = { workspace = true }` |
| gow-unlink | (none extra) |

### build.rs (all crates)
**Source:** `crates/gow-df/build.rs` lines 1-13
**Apply to:** All 10 new crates — copy verbatim
```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest::embed_manifest(
            embed_manifest::new_manifest("Gow.Rust")
                .active_code_page(embed_manifest::manifest::ActiveCodePage::Utf8)
                .long_path_aware(embed_manifest::manifest::Setting::Enabled),
        )
        .expect("unable to embed manifest");
    }
}
```

### Path Conversion (all file-reading crates)
**Source:** `crates/gow-fold/src/lib.rs` line 123; `crates/gow-tac/src/lib.rs` line 68
**Apply to:** gow-fmt, gow-paste, gow-join, gow-split
```rust
let converted = gow_core::path::try_convert_msys_path(file_path);
let path = Path::new(&converted);
```
Always wrap file paths before `File::open()` — converts MSYS2/Cygwin `/c/...` paths to `C:\...`.

### Error Message Format
**Source:** All existing utilities
**Apply to:** All 10 new crates
```
<utility>: <path>: <os_error>      // file I/O error
<utility>: <message>               // logic error
<utility>: missing operand         // missing required arg
```
Always use `eprintln!` to stderr, never `println!` for errors.

### Integration Test Structure
**Source:** `crates/gow-fold/tests/integration.rs`
**Apply to:** All 10 new crates
```rust
use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn <utility>_basic_case() {
    Command::cargo_bin("<binary_name>")
        .unwrap()
        .arg("<flag>")
        .write_stdin("<input>")
        .assert()
        .success()
        .stdout("<expected_output>");
}

#[test]
fn <utility>_exit_code_N() {
    Command::cargo_bin("<binary_name>")
        .unwrap()
        .assert()
        .failure()
        .code(N);
}
```
For expr/test: use `.code(0)`, `.code(1)`, `.code(2)` explicitly — these are semantic, not just success/failure.

---

## Workspace Changes

### Root Cargo.toml — windows-sys features addition
**Source:** root `Cargo.toml` lines 85-89 (current state)
**Modify:** Add 3 new feature flags to the existing windows-sys entry
```toml
windows-sys = { version = "0.61", features = [
    "Win32_System_Console",
    "Win32_Foundation",
    "Win32_Storage_FileSystem",
    "Win32_System_WindowsProgramming",    # Phase 11: GetUserNameW (whoami) + GetComputerNameW (uname)
    "Win32_System_SystemInformation",     # Phase 11: OSVERSIONINFOW/GetNativeSystemInfo (uname)
    "Wdk_System_SystemServices",          # Phase 11: RtlGetVersion (uname)
] }
```

### Root Cargo.toml — workspace members addition
**Source:** root `Cargo.toml` lines 52-63 (Phase 10 block)
**Append after Phase 10 block:**
```toml
# Phase 11 — new utilities wave 2 (U2-01 through U2-10)
"crates/gow-whoami",
"crates/gow-uname",
"crates/gow-paste",
"crates/gow-join",
"crates/gow-split",
"crates/gow-printf",
"crates/gow-expr",
"crates/gow-test",
"crates/gow-fmt",
"crates/gow-unlink",
```

### `extras/bin/[.bat` (new file)
**Source:** existing `extras/bin/*.bat` shim pattern (confirmed in RESEARCH.md line 700)
**Content (entire file):**
```bat
@echo off & "%~dp0test.exe" --_bracket_ %*
```
Note: Uses `--_bracket_` sentinel so `test.exe` can detect bracket-mode invocation reliably. Without sentinel, `test.exe` would receive `]` as a stray argument and exit 2.

---

## No Analog Found

All files have adequate analogs. The following have no direct match but use RESEARCH.md patterns:

| File | Role | Data Flow | Pattern Source |
|------|------|-----------|----------------|
| `crates/gow-expr/src/lib.rs` (evaluator logic) | utility | transform | RESEARCH.md Pattern 3 (recursive-descent parser) |
| `crates/gow-test/src/lib.rs` (evaluate_test) | utility | transform | RESEARCH.md Pattern 4 (POSIX operators table) |
| `crates/gow-printf/src/lib.rs` (format parser) | utility | transform | RESEARCH.md Pattern 5 (format spec table) |

---

## Metadata

**Analog search scope:** `crates/gow-df`, `crates/gow-fold`, `crates/gow-cat`, `crates/gow-sort`, `crates/gow-tac`, `crates/gow-echo`, `crates/gow-rm`, `crates/gow-seq`
**Files scanned:** 10 source files + 2 config files
**Pattern extraction date:** 2026-04-29

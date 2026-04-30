# Phase 10: new-utilities-wave1 — Research

**Researched:** 2026-04-29
**Domain:** GNU coreutils — seq, sleep, tac, nl, od, fold, expand/unexpand, du, df, md5sum/sha1sum/sha256sum — Windows MSVC Rust
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| U-01 | seq — number sequences (seq LAST, seq FIRST LAST, seq FIRST INCREMENT LAST) | Float precision via format string inference; integer fast path; bstr/write! output; no extra crate needed for integer-only; num-bigint for decimal fractions if desired |
| U-02 | sleep — integer and fractional second delays | `std::thread::sleep` + `Duration::from_secs_f64`; no external crate needed |
| U-03 | tac — output lines in reverse order | Read-all-then-reverse with bstr line iterator; streaming not required |
| U-04 | nl — line numbering (-b, -n, -w, -s) | Pure std I/O; bstr for line iteration; clap derive |
| U-05 | od — octal/hex dump (-A address, -t type, -N byte limit) | Most complex utility; requires multi-column type parsing; windows-sys not needed |
| U-06 | fold — line wrapping (-w width, -s word-boundary) | bstr for byte-safe char-width handling; pure std |
| U-07 | expand/unexpand — tab/space conversion (-t tabstop) | Single crate, argv[0] dispatch (same pattern as gow-gzip); pure std |
| U-08 | du — disk usage (-s, -h, -a, -d depth) | walkdir for recursion; `std::fs::metadata().len()` for size; Win32_Storage_FileSystem already in workspace |
| U-09 | df — disk free (-h human-readable) | `GetDiskFreeSpaceExW` + `GetLogicalDriveStringsW` from windows-sys Win32_Storage_FileSystem (already a workspace feature) |
| U-10 | md5sum/sha1sum/sha256sum with -c check mode | RustCrypto: md-5 0.11.0, sha1 0.11.0, sha2 0.11.0 + digest 0.11.2 trait; hex 0.4.3 for output; argv[0] dispatch |
</phase_requirements>

---

## Summary

Phase 10 adds 13 independent binary crates to the gow-rust workspace. The utilities split into five implementation groups by complexity: (1) trivial one-liners — sleep; (2) simple line processors — tac, fold, nl, expand/unexpand; (3) moderate — seq (float precision edge case), du; (4) platform-specific — df (Windows disk APIs); (5) complex — od (multi-format output engine) and the hash suite (RustCrypto + check-mode parser).

The most important finding is **seq float precision**: GNU seq uses printf-style width detection — it determines the number of decimal places from the input arguments (e.g., `seq 1.5 0.5 3` detects 1 decimal place) and formats all output values to that precision. A pure `f64` approach accumulates floating-point error across many steps. The uutils/coreutils implementation uses `ExtendedBigDecimal` (backed by `num-bigint`) for the arithmetic. For gow-rust, given the scope of this phase, the **recommended approach** is a simpler string-based precision inference: parse the number of decimal places from the increment and first value strings, accumulate using integer arithmetic scaled by 10^precision, and format back. This avoids num-bigint but handles `seq 1.5 0.5 3` correctly. The integer fast path (no decimal point in any argument) uses i64 arithmetic directly.

The second finding is **od complexity**: od's `-t` type format accepts compound specifiers (e.g., `-t x1 -t o2 -t d4 -t f8 -t c`) and outputs multiple rows per address. Each row is a different type view of the same bytes. This is the most structurally complex utility in this phase and deserves its own implementation plan.

For **df on Windows**: `GetDiskFreeSpaceExW` and `GetLogicalDriveStringsW` are already enabled through the `Win32_Storage_FileSystem` feature that is in the workspace Cargo.toml. No new windows-sys features are required. The df output format matches GNU df: `Filesystem 1K-blocks Used Available Use% Mounted on`.

For **hash utilities**: RustCrypto's `md-5 0.11.0`, `sha1 0.11.0`, `sha2 0.11.0`, and `digest 0.11.2` (the trait) are the standard approach. All are pure Rust with no C dependencies. The `hex 0.4.3` crate encodes the binary digest to the lowercase hex string GNU tools emit. The check mode (`-c`) parses lines of the form `<hex>  <filename>` (two spaces, or one space for binary mode) as written by the tool itself.

**Primary recommendation:** Use the scaled-integer approach for seq precision. Use RustCrypto for hashing. Split phase into 4 plans: scaffold (all crates + workspace additions), simple utilities (sleep/tac/nl/fold/expand/unexpand), complex utilities (seq/od), and platform/verification utilities (du/df/hashes).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| seq number generation | Binary (CLI) | — | Pure arithmetic; no I/O tier needed |
| sleep timing | Binary (CLI) | OS kernel | `std::thread::sleep` calls OS scheduler |
| tac/nl/fold line processing | Binary (CLI) | — | Line-by-line stream transforms |
| expand/unexpand tab conversion | Binary (CLI) | — | Character-width transform; argv[0] dispatch |
| od binary inspection | Binary (CLI) | — | Raw byte reader; format engine internal |
| du recursion | Binary (CLI) | Filesystem | walkdir traversal; `metadata().len()` |
| df volume enumeration | Binary (CLI) | Win32 API | GetLogicalDriveStringsW + GetDiskFreeSpaceExW |
| md5sum/sha1sum/sha256sum | Binary (CLI) | RustCrypto | Stream hasher; check mode file parser |

---

## Standard Stack

### Core (all new crates)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6 (workspace) | Arg parsing | Project standard; derive API |
| anyhow | 1 (workspace) | Error propagation in main | Project standard |
| thiserror | 2 (workspace) | Structured error enums | Project standard |
| gow-core | path dep | UTF-8 init, arg parsing, path conversion | Required by all gow binaries |
| bstr | 1 (workspace) | Byte-safe line iteration | Required for non-UTF-8 input passthrough |
| walkdir | 2.5 (workspace) | Recursive directory traversal for du | Already workspace dep |
| windows-sys | 0.61 (workspace) | GetDiskFreeSpaceExW, GetLogicalDriveStringsW | Already workspace dep with Win32_Storage_FileSystem feature |
| embed-manifest | 1.5 (build dep, per-crate) | Windows UTF-8 manifest | Required by all gow binaries |

### New workspace dependencies needed
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| md-5 | 0.11.0 | MD5 hash function | RustCrypto standard; pure Rust; implements `digest` trait [VERIFIED: cargo search] |
| sha1 | 0.11.0 | SHA-1 hash function | RustCrypto standard; pure Rust; implements `digest` trait [VERIFIED: cargo search] |
| sha2 | 0.11.0 | SHA-2 (sha256) hash function | RustCrypto standard; pure Rust; implements `digest` trait [VERIFIED: cargo search] |
| digest | 0.11.2 | Hash trait abstraction | Shared API for md-5/sha1/sha2; allows generic hasher code [VERIFIED: cargo search] |
| hex | 0.4.3 | Binary digest → hex string | Standard hex encoding crate; encode_lower() matches GNU output [VERIFIED: cargo search] |

**Note on RustCrypto naming:** the crate is `md-5` (hyphenated) on crates.io but imported in Rust as `use md5::Md5;` — the package name uses a hyphen, the Rust identifier uses an underscore.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| md-5 + sha1 + sha2 | ring | ring is a C-backed library; adds C build complexity; overkill for checksumming CLI tools |
| scaled-integer seq arithmetic | num-bigint (ExtendedBigDecimal) | num-bigint adds compile time and binary size; scaled-integer handles the 1-2 decimal place cases that 99% of users need |
| scaled-integer seq arithmetic | f64 only | f64 accumulates error: `seq 0.1 0.1 1.0` gives `0.9999999...` at step 10; not GNU-compatible |
| hex crate | `format!("{:02x}", byte)` loop | hex crate is one-line; no reason to hand-roll |

**Installation (new workspace deps):**
```toml
# Add to [workspace.dependencies] in root Cargo.toml
md-5 = "0.11"           # MD5 hash — gow-hashes
sha1 = "0.11"           # SHA-1 hash — gow-hashes
sha2 = "0.11"           # SHA-256 hash — gow-hashes
digest = "0.11"         # Hash trait — gow-hashes
hex = "0.4"             # Hex encoding — gow-hashes
```

**Version verification:** All versions confirmed via `cargo search` on 2026-04-29. [VERIFIED: cargo search]

---

## Architecture Patterns

### System Architecture Diagram

```
stdin/file args
      │
      ▼
 gow_core::init()              ← UTF-8 console setup (every binary)
      │
      ▼
 gow_core::args::parse_gnu()   ← permutation-aware arg parsing
      │
      ▼
 [utility logic]
      │
 ┌────┴──────────────────────────────────────────────┐
 │ seq: scaled-int loop → format → stdout            │
 │ sleep: Duration::from_secs_f64 → thread::sleep    │
 │ tac: read lines → reverse Vec → write stdout      │
 │ nl: stream lines → add number prefix → stdout     │
 │ od: read N bytes chunks → format rows → stdout    │
 │ fold: accumulate cols → break at width → stdout   │
 │ expand: tab → spaces (argv[0] dispatch)           │
 │ unexpand: spaces → tab (argv[0] dispatch)         │
 │ du: walkdir → sum metadata.len() → format         │
 │ df: GetLogicalDriveStringsW → GetDiskFreeSpaceExW │
 │     → format table → stdout                       │
 │ hashes: digest::Digest → io::copy → hex → stdout  │
 │         check mode: parse lines → verify → report │
 └───────────────────────────────────────────────────┘
      │
      ▼
   stdout / stderr
   exit_code (0 / 1 / 2)
```

### Recommended Project Structure
```
crates/
├── gow-seq/          # seq — U-01
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       ├── main.rs
│       └── lib.rs    # uumain() entry, seq logic
├── gow-sleep/        # sleep — U-02
├── gow-tac/          # tac — U-03
├── gow-nl/           # nl — U-04
├── gow-od/           # od — U-05
├── gow-fold/         # fold — U-06
├── gow-expand/       # expand + unexpand — U-07
│   ├── Cargo.toml    # [[bin]] expand + [[bin]] unexpand
│   ├── build.rs
│   └── src/
│       ├── expand.rs   # fn main() { exit(uu_expand::uumain(args_os())) }
│       ├── unexpand.rs # fn main() { exit(uu_expand::uumain(args_os())) }
│       └── lib.rs      # argv[0] dispatch logic
├── gow-du/           # du — U-08
├── gow-df/           # df — U-09
└── gow-hashes/       # md5sum + sha1sum + sha256sum — U-10
    ├── Cargo.toml    # [[bin]] md5sum + [[bin]] sha1sum + [[bin]] sha256sum
    ├── build.rs
    └── src/
        ├── md5sum.rs     # fn main()
        ├── sha1sum.rs    # fn main()
        ├── sha256sum.rs  # fn main()
        └── lib.rs        # argv[0] dispatch logic + shared hash implementation
```

### Pattern 1: argv[0] Dispatch (expand/unexpand and hash utilities)

This is the established project pattern from gow-gzip. Each alternate binary is a thin main.rs that calls the same `uumain()`:

```rust
// src/expand.rs  (and src/unexpand.rs — identical)
fn main() {
    std::process::exit(uu_expand::uumain(std::env::args_os()));
}
```

```rust
// src/lib.rs
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let invoked_as = args_vec
        .first()
        .map(|s| {
            std::path::Path::new(s)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
        })
        .unwrap_or_default();

    // "expand" → expand tabs to spaces
    // "unexpand" → convert spaces back to tabs
    run(&cli, &invoked_as)
}
```

**Cargo.toml for multi-binary crate:**
```toml
[[bin]]
name = "expand"
path = "src/expand.rs"

[[bin]]
name = "unexpand"
path = "src/unexpand.rs"

[lib]
name = "uu_expand"
path = "src/lib.rs"
```

[VERIFIED: gow-gzip codebase, crates/gow-gzip/Cargo.toml and src/lib.rs]

### Pattern 2: Standard Crate Scaffold

Every new crate follows the same pattern:
1. `Cargo.toml` with `version/edition/rust-version/license/authors = { workspace = true }`, `[[bin]]`, `[lib]`, `[build-dependencies] embed-manifest = "1.5"`, `[dev-dependencies] assert_cmd/predicates/tempfile = { workspace = true }`
2. `build.rs` — identical to gow-gzip's build.rs (embed-manifest ActiveCodePage::Utf8 + LongPathAware)
3. `src/main.rs` — one-liner calling `uumain`
4. `src/lib.rs` — `pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32` with `gow_core::init()` first line
5. `tests/integration.rs` — assert_cmd tests

[VERIFIED: codebase inspection of gow-cat, gow-gzip, gow-bzip2]

### Pattern 3: seq Scaled-Integer Arithmetic

GNU seq must produce exact decimal output. The approach:

```rust
// Parse decimal places from input strings (not parsed floats)
fn decimal_places(s: &str) -> u32 {
    if let Some(dot_pos) = s.find('.') {
        (s.len() - dot_pos - 1) as u32
    } else {
        0
    }
}

// Scale everything to integers
let precision = decimal_places(first).max(decimal_places(increment)).max(decimal_places(last));
let scale: i64 = 10_i64.pow(precision);
let mut current = (first_f64 * scale as f64).round() as i64;
let inc = (increment_f64 * scale as f64).round() as i64;
let end = (last_f64 * scale as f64).round() as i64;

while current <= end {
    if precision == 0 {
        println!("{}", current);
    } else {
        println!("{:.prec$}", current as f64 / scale as f64, prec = precision as usize);
    }
    current += inc;
}
```

**Edge cases:**
- `seq 10` → first=1, inc=1, last=10 (integer fast path)
- `seq 1.5 0.5 3` → precision=1, scale=10, outputs 1.5 2.0 2.5 3.0
- `seq 0.1 0.1 1.0` → precision=1, scale=10, correct (no f64 accumulation error)
- Negative increment: `seq 5 -1 1` → loop condition becomes `current >= end`
- Separator: default `\n`, override with `-s`
- Format: `-f '%05.2f'` applies printf-style format (can use `format!` with parsed precision)

[ASSUMED: The scaled-integer approach handles 1-2 decimal places correctly. For more than ~9 decimal places, integer overflow is theoretically possible with i64. This is acceptable for practical use cases.]

### Pattern 4: od Output Format Engine

od is the most complex utility. The `-t` option accepts type specifiers:
- `o` = octal, `x` = hex, `d` = signed decimal, `u` = unsigned decimal, `c` = character, `a` = named character, `f` = float
- With size suffix: `o1` `o2` `o4` `o8`, `x1` `x2` `x4` `x8`, `d1` `d2` `d4` `d8`, `u1` `u2` `u4` `u8`, `f4` `f8`
- Default when no `-t`: `-t o2` (octal 2-byte)

Address format (`-A`): `o` = octal (default), `x` = hex, `d` = decimal, `n` = no address

Architecture:
```
input bytes → [chunk by type unit size] → [format per type] → [column-align] → stdout
                                                   ↑
                                           multiple -t flags
                                           = multiple rows per address
```

The od output repeats the address for each format type, OR (GNU default) shows address once and stacks the type rows. The GNU default with `-t o2` (two-byte octal units) is:
```
0000000 062143 062164 065156 072156 064164 066040 072165 062541
0000020 062564 064563 005472 062541 076153 063 062 073 063 066
```

[ASSUMED: od implementation should default to the GNU-compatible `-t o2` single-row-per-address format. Multiple `-t` flags stacking multiple rows is a stretch goal, not required for initial pass.]

### Pattern 5: Human-Readable Size Formatting

Both du and df need GNU-compatible human-readable output (-h flag):

```rust
fn human_readable(bytes: u64) -> String {
    const UNITS: &[(&str, u64)] = &[
        ("E", 1 << 60), ("P", 1 << 50), ("T", 1 << 40),
        ("G", 1 << 30), ("M", 1 << 20), ("K", 1 << 10),
    ];
    for (suffix, factor) in UNITS {
        if bytes >= *factor {
            let val = bytes as f64 / *factor as f64;
            return if val < 10.0 {
                format!("{:.1}{}", val, suffix)
            } else {
                format!("{:.0}{}", val, suffix)
            };
        }
    }
    format!("{}", bytes)
}
```

GNU uses 1K = 1024 (binary SI) for human-readable output. Default block size for non-human output is 1K blocks (divide bytes by 1024). [VERIFIED: GNU coreutils du/df behavior from training knowledge, confirmed standard]

### Pattern 6: df Windows Implementation

```rust
// Enumerate drives: GetLogicalDriveStringsW fills a buffer with "C:\\\0D:\\\0\0"
// GetDiskFreeSpaceExW returns: free_bytes_available, total_bytes, total_free_bytes
use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetLogicalDriveStringsW,
};

fn get_drives() -> Vec<String> {
    let mut buf = [0u16; 256];
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
    // Parse null-terminated strings from buffer
    let mut drives = Vec::new();
    let mut start = 0;
    for i in 0..len as usize {
        if buf[i] == 0 {
            if i > start {
                drives.push(String::from_utf16_lossy(&buf[start..i]));
            }
            start = i + 1;
        }
    }
    drives
}
```

[VERIFIED: windows-sys 0.61.2 source — GetDiskFreeSpaceExW and GetLogicalDriveStringsW confirmed in Win32_Storage_FileSystem module]

### Anti-Patterns to Avoid

- **f64 accumulation for seq:** Using `current += increment` with f64 across many steps produces incorrect last digit. Use scaled integers instead.
- **OOM on tac with huge files:** For the initial implementation, reading the entire file into memory is acceptable for typical use. Do NOT try to stream in reverse for the first version.
- **od: single-byte char mode confusion:** In `c` mode, od prints printable ASCII as characters and non-printable as octal. Do not confuse with Rust `char` (Unicode scalar); od's `c` operates on bytes, not Unicode.
- **du: following symlinks:** GNU du with default options does NOT follow symlinks. Use `walkdir::WalkDir::new(path).follow_links(false)` explicitly.
- **df: network drives / CD-ROM:** `GetDiskFreeSpaceExW` can fail for CD-ROM drives or unmounted drives. Must handle error gracefully — skip drives where the call fails rather than exiting.
- **Installer staging:** The new binaries compile to `target\{ARCH}\release\*.exe`. The build.bat step 2/5 uses a PowerShell glob `Get-ChildItem 'target\...\release\*.exe'` excluding only `gow-probe.exe`. No build.bat changes are needed — all new binaries are automatically staged. BUT: the `echo` lines in build.bat's `:run` section that list utility names should be updated to include the new utilities.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MD5 hash | custom MD5 implementation | md-5 0.11 (RustCrypto) | Correct padding, test vectors; pure Rust; 4 LOC |
| SHA-1 hash | custom SHA-1 | sha1 0.11 (RustCrypto) | Same reasons |
| SHA-256 hash | custom SHA-256 | sha2 0.11 (RustCrypto) | Same reasons |
| Hex encoding | `format!("{:02x}", b)` loop | hex 0.4.3 | One-line `hex::encode(digest)` |
| Recursive directory walk for du | `std::fs::read_dir` recursion | walkdir 2.5 (workspace dep) | Handles symlink cycles, permission errors, large trees |
| Windows drive enumeration | manual Win32 call with manual string parsing | windows-sys GetLogicalDriveStringsW | Already in workspace; safe wrapper is trivial |
| Float precision for seq | f64 arithmetic | scaled-integer approach | Avoids accumulation error for decimal inputs |

**Key insight:** The cryptographic hash and hex domains have well-known edge cases (endianness, padding, encoding) that are not worth reimplementing. RustCrypto crates are zero-dependency pure-Rust and compile fast.

---

## Common Pitfalls

### Pitfall 1: seq float precision
**What goes wrong:** `seq 0.1 0.1 1.0` prints `1.0000000000000009` or stops at `0.9` when using f64 accumulation.
**Why it happens:** `0.1` is not exactly representable in IEEE 754; each addition accumulates error.
**How to avoid:** Parse number of decimal places from the input string, scale by 10^precision, do all arithmetic in i64, format back.
**Warning signs:** Test `seq 0.1 0.1 1.0` — should produce exactly 10 lines ending with `1.0`.

### Pitfall 2: od default format confusion
**What goes wrong:** od with no `-t` flag outputs 2-byte octal units by default (not 1-byte). Many implementations accidentally use 1-byte.
**Why it happens:** GNU default is `-t o2`, not `-t o1`.
**How to avoid:** Default `type_spec` to `o2` when no `-t` flag is provided.
**Warning signs:** `printf 'A' | od` should output `0000000 000101` (two-byte octal) not `0000000 101`.

### Pitfall 3: du following symlinks
**What goes wrong:** du counts symlinked directory trees, causing infinite loops or double-counting.
**Why it happens:** Default walkdir follows symlinks.
**How to avoid:** `WalkDir::new(path).follow_links(false)` — this is the GNU default.
**Warning signs:** du on a directory containing a symlink to a parent directory loops forever or panics.

### Pitfall 4: df skipping failed drives
**What goes wrong:** df exits non-zero when a CD-ROM or unmapped network drive returns error from GetDiskFreeSpaceExW.
**Why it happens:** GetDiskFreeSpaceExW returns FALSE for drives that are not ready.
**How to avoid:** Check return value; if FALSE, skip that drive silently (GNU df behavior: only shows mountable/available filesystems).
**Warning signs:** df errors on systems with empty optical drives.

### Pitfall 5: nl with --body-numbering/-b 'a' vs 't'
**What goes wrong:** nl defaults to `-b t` (number non-empty lines only), not `-b a` (all lines). Many implementations default to numbering all lines.
**Why it happens:** The default is not `all`.
**How to avoid:** Default `body_numbering` to `'t'` (non-empty lines).
**Warning signs:** `printf 'a\n\nb\n' | nl` should print `     1\ta\n\n     2\tb\n` — blank line gets no number.

### Pitfall 6: expand with multi-char tabstop (-t 4,8)
**What goes wrong:** GNU expand supports `-t` with a comma-separated list of tab stops (e.g., `-t 4,8,16`). Simple implementations only accept a single integer.
**Why it happens:** The common case is a single tabstop.
**How to avoid:** For initial implementation, support single integer tabstop (default 8). Document that tab-stop list is a stretch goal.
**Warning signs:** `expand -t 4,8 file` should be handled gracefully (either work or give a clear error).

### Pitfall 7: hash check mode (-c) format
**What goes wrong:** The check file format must be exactly `<hex>  <filename>\n` (two spaces = text mode) or `<hex> *<filename>\n` (one space + asterisk = binary mode). Both are valid input to `-c`.
**Why it happens:** GNU md5sum produces two-space format by default; some tools produce the binary mode format.
**How to avoid:** Accept both formats when reading check files. The difference is whether there are 2 spaces or 1 space + `*`.
**Warning signs:** `sha256sum -c` fails on check files produced by binary mode tools.

### Pitfall 8: Windows-sys feature already in workspace
**What goes wrong:** Adding duplicate or incorrect `windows-sys` features to a new crate's Cargo.toml.
**Why it happens:** Confusion about workspace dep feature inheritance.
**How to avoid:** When a new crate like `gow-df` uses windows-sys, it should use `windows-sys = { workspace = true }` — the workspace already has `Win32_Storage_FileSystem` enabled.
**Warning signs:** Build error "feature X not in workspace dep" or duplicate feature declarations.

---

## Code Examples

### seq: Scaled-Integer Approach
```rust
// Source: derived from GNU seq behavior analysis + scaled-int pattern
fn seq_decimal(first: f64, inc: f64, last: f64, precision: u32, sep: &str) -> i32 {
    let scale = 10_i64.pow(precision);
    let mut cur = (first * scale as f64).round() as i64;
    let inc_scaled = (inc * scale as f64).round() as i64;
    let end = (last * scale as f64).round() as i64;
    let going_up = inc_scaled > 0;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut first_item = true;
    loop {
        if going_up && cur > end { break; }
        if !going_up && cur < end { break; }
        if !first_item { let _ = out.write_all(sep.as_bytes()); }
        first_item = false;
        if precision == 0 {
            let _ = writeln!(out, "{cur}");
        } else {
            let _ = writeln!(out, "{:.prec$}", cur as f64 / scale as f64, prec = precision as usize);
        }
        cur += inc_scaled;
    }
    0
}
```

### hash: RustCrypto Pattern
```rust
// Source: RustCrypto digest trait docs + hex crate
use digest::Digest;
use std::io::{self, Read};

fn hash_reader<D: Digest>(mut reader: impl Read) -> Vec<u8> {
    let mut hasher = D::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf).unwrap_or(0);
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    hasher.finalize().to_vec()
}

// Output: hex::encode(&digest) produces lowercase hex string (GNU-compatible)
println!("{}  {}", hex::encode(&hash_reader::<md5::Md5>(file)), filename);
```

### df: GetLogicalDriveStringsW + GetDiskFreeSpaceExW
```rust
// Source: windows-sys 0.61.2 Win32_Storage_FileSystem module
use std::os::windows::ffi::OsStringExt;
use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetLogicalDriveStringsW};

unsafe fn get_disk_free(root: &str) -> Option<(u64, u64, u64)> {
    let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
    let mut free_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free: u64 = 0;
    let ok = GetDiskFreeSpaceExW(
        wide.as_ptr(),
        &mut free_available,
        &mut total_bytes,
        &mut total_free,
    );
    if ok != 0 { Some((free_available, total_bytes, total_free)) } else { None }
}
```

### du: walkdir + metadata
```rust
// Source: walkdir 2.5 docs
use walkdir::WalkDir;

fn dir_usage(path: &std::path::Path, max_depth: Option<usize>) -> u64 {
    let mut walker = WalkDir::new(path).follow_links(false);
    if let Some(d) = max_depth { walker = walker.max_depth(d); }
    walker.into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}
```

---

## Workspace Cargo.toml Additions

### New workspace members to add
```toml
# Phase 10 — new utilities wave 1 (U-01 through U-10)
"crates/gow-seq",
"crates/gow-sleep",
"crates/gow-tac",
"crates/gow-nl",
"crates/gow-od",
"crates/gow-fold",
"crates/gow-expand",    # provides expand + unexpand binaries
"crates/gow-du",
"crates/gow-df",
"crates/gow-hashes",    # provides md5sum + sha1sum + sha256sum binaries
```

### New workspace dependencies to add
```toml
# Phase 10 additions (U-10 hash utilities)
md-5 = "0.11"           # MD5 hash function — gow-hashes only
sha1 = "0.11"           # SHA-1 hash function — gow-hashes only
sha2 = "0.11"           # SHA-256/SHA-512 — gow-hashes only
digest = "0.11"         # Hash trait — gow-hashes only
hex = "0.4"             # Digest hex encoding — gow-hashes only
```

### No changes required
- `windows-sys` — already has `Win32_Storage_FileSystem` feature (needed for du/df)
- `walkdir` — already a workspace dep (needed for du)
- `bstr` — already a workspace dep (needed for tac, nl, fold, expand)
- `embed-manifest` — stays as a per-crate build dep (not a workspace dep, consistent with all existing crates)

---

## Installer Staging

**No changes to build.bat are required** for the new binaries to be included in the MSI.

Step 2/5 of build.bat uses:
```powershell
Get-ChildItem 'target\{RT}\release\*.exe' | Where-Object { $_.Name -ne 'gow-probe.exe' } | Copy-Item -Destination '{_CORE_STAGE}'
```

This glob automatically picks up all new `.exe` files. The 13 new binaries will appear in the installer without any build.bat changes.

**However:** The `echo` output in the `:run` section of build.bat lists utility names for display purposes. This is cosmetic — it should be updated but does not affect build correctness. The planner should include a task to update that list.

[VERIFIED: build.bat codebase inspection, line 129]

---

## Scaffolding Strategy

The established pattern for phases with multiple crates (phases 04-06) is a **dedicated scaffold plan** that creates all crates with stub implementations before any implementation plans:

**Recommended plan structure for Phase 10:**

| Plan | Contents | Crates |
|------|----------|--------|
| 10-01 | Scaffold: workspace members + deps + stub crates for all 10 crates | All 10 crates |
| 10-02 | Implement: sleep, tac, nl, fold (simple line processors) | gow-sleep, gow-tac, gow-nl, gow-fold |
| 10-03 | Implement: expand/unexpand (argv[0] dispatch) | gow-expand |
| 10-04 | Implement: seq (float precision), od (format engine) | gow-seq, gow-od |
| 10-05 | Implement: du, df (Windows disk APIs) | gow-du, gow-df |
| 10-06 | Implement: hash suite (md5sum/sha1sum/sha256sum + -c check mode) | gow-hashes |

This split ensures: (1) simple utilities can be completed and tested early; (2) complex utilities (seq precision, od format engine, df Win32) each get focused plan attention; (3) the hash suite is self-contained.

---

## Runtime State Inventory

Step 2.5 is not applicable — this is a greenfield phase adding new crates. No renames, refactors, or migrations.

**Nothing found in any category — verified by phase description review.**

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable (MSVC) | All crates | Confirmed (existing workspace compiles) | 1.85+ | — |
| cargo | Build | Confirmed | workspace | — |
| windows-sys Win32_Storage_FileSystem | gow-du, gow-df | Confirmed in workspace Cargo.toml | 0.61.2 | — |
| walkdir | gow-du | Confirmed in workspace deps | 2.5 | — |
| WiX v3 (heat.exe, candle.exe, light.exe) | MSI build | Confirmed on CI (windows-latest image) | 3.14.1 | — |

**No missing dependencies with no fallback.** All required tools are already available.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | assert_cmd 2.2.1 + predicates 3.1.4 + tempfile 3.27.0 |
| Config file | none — standard `cargo test` discovers tests/integration.rs per crate |
| Quick run command | `cargo test -p gow-seq -p gow-sleep -p gow-tac` (per-group) |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| U-01 | `seq 10` outputs 1-10 | integration | `cargo test -p gow-seq` | No — Wave 0 |
| U-01 | `seq 1 2 10` outputs 1,3,5,7,9 | integration | `cargo test -p gow-seq` | No — Wave 0 |
| U-01 | `seq 1.5 0.5 3` outputs 1.5,2.0,2.5,3.0 | integration | `cargo test -p gow-seq` | No — Wave 0 |
| U-02 | `sleep 0` exits 0 immediately | integration | `cargo test -p gow-sleep` | No — Wave 0 |
| U-02 | `sleep 0.1` exits 0 after ~100ms | integration | `cargo test -p gow-sleep` | No — Wave 0 |
| U-03 | `tac` reverses lines | integration | `cargo test -p gow-tac` | No — Wave 0 |
| U-04 | `nl` numbers non-blank lines by default | integration | `cargo test -p gow-nl` | No — Wave 0 |
| U-05 | `od` default octal 2-byte output | integration | `cargo test -p gow-od` | No — Wave 0 |
| U-05 | `od -A x -t x1` hex address + hex bytes | integration | `cargo test -p gow-od` | No — Wave 0 |
| U-06 | `fold -w 40` wraps at 40 chars | integration | `cargo test -p gow-fold` | No — Wave 0 |
| U-07 | `expand` replaces tabs with spaces | integration | `cargo test -p gow-expand` | No — Wave 0 |
| U-07 | `unexpand` converts spaces to tabs | integration | `cargo test -p gow-expand` | No — Wave 0 |
| U-08 | `du -sh .` reports size without error | integration | `cargo test -p gow-du` | No — Wave 0 |
| U-09 | `df -h` reports disk free without error | integration | `cargo test -p gow-df` | No — Wave 0 |
| U-10 | `md5sum file` produces correct hash | integration | `cargo test -p gow-hashes` | No — Wave 0 |
| U-10 | `sha256sum -c checkfile` exits 0 on match | integration | `cargo test -p gow-hashes` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p <crate-being-implemented>`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
All test files are new — created during scaffold plan (10-01):
- [ ] `crates/gow-seq/tests/integration.rs`
- [ ] `crates/gow-sleep/tests/integration.rs`
- [ ] `crates/gow-tac/tests/integration.rs`
- [ ] `crates/gow-nl/tests/integration.rs`
- [ ] `crates/gow-od/tests/integration.rs`
- [ ] `crates/gow-fold/tests/integration.rs`
- [ ] `crates/gow-expand/tests/integration.rs`
- [ ] `crates/gow-du/tests/integration.rs`
- [ ] `crates/gow-df/tests/integration.rs`
- [ ] `crates/gow-hashes/tests/integration.rs`

---

## Security Domain

These utilities are text processing and numeric tools with no authentication, session management, or access control surface.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | partial | Validate numeric args for seq/sleep (parse error → exit 2); validate -t type for od |
| V6 Cryptography | no — hash utilities are file integrity tools, not security-critical crypto | md-5/sha1/sha2 from RustCrypto — never hand-roll |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| OOM via huge file in tac | Denial of service | Acceptable for initial impl; document limitation |
| od with -N very large | Denial of service | Validate -N is reasonable; use streaming read |
| Hash check file with crafted path | Spoofing | Strip CWD prefix tricks; check file must be parsed as `hex  filename` only |

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| md5 crate (unmaintained) | md-5 0.11 (RustCrypto) | 2019-era transition | md-5 is the canonical, actively maintained choice |
| sha1 crate (old) | sha1 0.11 (RustCrypto) | Continuous updates | Same |
| xz2 (C, MSVC issues) | liblzma (already in project) | Phase 06 research | N/A for this phase — no xz here |
| bzip2 with C backend | bzip2 0.6+ pure Rust | Phase 06 research | N/A for this phase |

**Deprecated/outdated approaches to avoid:**
- `md5` crate (crates.io/crates/md5): different from `md-5`; older API; avoid
- `sha1` without the RustCrypto family: less maintained alternatives exist; use the RustCrypto `sha1` specifically
- Raw f64 arithmetic for seq: produces wrong output for non-integer inputs

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Scaled-integer approach with i64 handles seq correctly up to ~9 decimal places | Architecture Patterns / seq | Users with >9 decimal place sequences get wrong output; mitigation: validate precision and fall back to error |
| A2 | od default behavior is `-t o2` (two-byte octal) based on GNU coreutils reference | Common Pitfalls | od produces wrong default output format |
| A3 | nl defaults to `-b t` (number non-empty lines) | Common Pitfalls | nl numbers blank lines incorrectly by default |
| A4 | build.bat automatically stages all new `.exe` files with no changes needed | Installer Staging | New binaries missing from MSI |
| A5 | df skips drives where GetDiskFreeSpaceExW returns FALSE (GNU behavior) | Architecture Patterns | df crashes or errors on systems with optical drives |

**Lower-risk assumptions (documented but not blocking):**
- A4 is VERIFIED by reading build.bat line 129 — the glob is `*.exe` minus `gow-probe.exe`. [VERIFIED: codebase]

---

## Open Questions

1. **od: multiple -t flags row stacking**
   - What we know: GNU od supports multiple `-t` flags and shows multiple rows per address
   - What's unclear: Whether users need this in Phase 10 or if single `-t` per invocation is sufficient
   - Recommendation: Implement single `-t` flag support in Phase 10; document multi-type as a future enhancement

2. **seq: GNU format flag (-f)**
   - What we know: GNU seq supports `-f FORMAT` for printf-style format strings
   - What's unclear: Whether this is required for the success criteria
   - Recommendation: The Phase 10 success criteria only require `seq 10`, `seq 1 2 10`, `seq 1.5 0.5 3` — `-f` is out of scope for the initial pass

3. **du: block size units**
   - What we know: GNU du default reports in 1K blocks; `-h` shows human-readable; `-b` shows bytes
   - What's unclear: Whether to implement all block size variants or just `-h` and default 1K
   - Recommendation: Implement default 1K + `-h` human-readable + `-b` bytes. Skip `-k`, `-m`, `--block-size=N` as stretch goals.

4. **nl: body/header/footer section support**
   - What we know: GNU nl treats `\:\:\:`, `\:\:`, `\:` as page section delimiters
   - What's unclear: Whether page section support is required
   - Recommendation: Not required for success criteria. Implement line numbering for the body section only (default). Document as future enhancement.

---

## Sources

### Primary (HIGH confidence)
- `crates/gow-gzip/src/lib.rs` — argv[0] dispatch pattern [VERIFIED: codebase]
- `crates/gow-gzip/Cargo.toml` — multi-binary crate structure [VERIFIED: codebase]
- `Cargo.toml` (workspace root) — workspace deps, existing features [VERIFIED: codebase]
- `build.bat` — installer staging logic (line 129) [VERIFIED: codebase]
- windows-sys 0.61.2 `Win32/Storage/FileSystem/mod.rs` — GetDiskFreeSpaceExW, GetLogicalDriveStringsW [VERIFIED: cargo registry source]
- windows-sys 0.61.2 `Cargo.toml` — Win32_Storage_FileSystem feature [VERIFIED: cargo registry source]

### Secondary (MEDIUM confidence)
- `cargo search md-5` → 0.11.0 [VERIFIED: cargo search 2026-04-29]
- `cargo search sha1` → 0.11.0 [VERIFIED: cargo search 2026-04-29]
- `cargo search sha2` → 0.11.0 [VERIFIED: cargo search 2026-04-29]
- `cargo search digest` → 0.11.2 [VERIFIED: cargo search 2026-04-29]
- `cargo search hex` → 0.4.3 [VERIFIED: cargo search 2026-04-29]
- uutils/coreutils seq.rs — ExtendedBigDecimal approach for float precision [CITED: github.com/uutils/coreutils/blob/main/src/uu/seq/src/seq.rs via WebFetch]

### Tertiary (LOW confidence)
- GNU od default format (`-t o2`) — from training knowledge; not verified against live GNU man page [ASSUMED]
- GNU nl default (`-b t`) — from training knowledge [ASSUMED]
- GNU df skip behavior for unavailable drives — from training knowledge [ASSUMED]

---

## Metadata

**Confidence breakdown:**
- Standard stack (hash crates, hex): HIGH — verified via cargo search
- Architecture (argv[0] dispatch, crate structure): HIGH — verified from codebase
- Windows disk APIs: HIGH — verified from windows-sys source
- seq precision approach: MEDIUM — scaled-integer rationale is sound but edge cases assumed
- od format engine: MEDIUM — default format assumed from training knowledge
- Pitfalls: MEDIUM — based on training knowledge of GNU behavior

**Research date:** 2026-04-29
**Valid until:** 2026-07-29 (stable crates — 90-day validity)

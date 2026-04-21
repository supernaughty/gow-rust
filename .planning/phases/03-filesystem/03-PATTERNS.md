# Phase 3: Filesystem Utilities — Pattern Map

**Mapped:** 2026-04-21
**Files analyzed:** 26 (1 gow-core extension + 11 new crates × ~4 files each + Cargo.toml)
**Analogs found:** 26 / 26 (100% — every file has a Phase 2 template)

---

## File Classification

Each Phase 3 deliverable (per D-49, D-50) classified by role and data flow. Analog
selection favors Phase 2 crates with matching role + most similar data flow + most
recent code (all Phase 2 crates are current).

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `Cargo.toml` (workspace root) | config | — | current `D:/workspace/gow-rust/Cargo.toml` itself | exact (just append) |
| `crates/gow-core/src/fs.rs` | extension to existing module | — | itself (already contains `LinkType`, `link_type`, `normalize_junction_target`) | exact |
| `crates/gow-cat/Cargo.toml` | config | — | `gow-wc/Cargo.toml` (bstr dep) | exact |
| `crates/gow-cat/build.rs` | config | — | `gow-touch/build.rs` (verbatim copy) | exact |
| `crates/gow-cat/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` (verbatim copy, rename crate) | exact |
| `crates/gow-cat/src/lib.rs` | utility lib (uumain) | stream I/O (read-until + write-all) | `gow-wc/src/lib.rs` (reads files + stdin, dash operand, per-file error loop) | exact — almost same shape |
| `crates/gow-cat/tests/integration.rs` | integration tests | — | `gow-wc/tests/integration.rs` (file + stdin + exit-1 + UTF-8) | exact |
| `crates/gow-ls/Cargo.toml` | config | — | `gow-wc/Cargo.toml` + add `walkdir`, `terminal_size` | role-match |
| `crates/gow-ls/build.rs` | config | — | `gow-touch/build.rs` (verbatim copy) | exact |
| `crates/gow-ls/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-ls/src/lib.rs` | utility lib | directory read + recursive walk + formatted output | `gow-which/src/lib.rs` (env-driven dispatch + hybrid search pattern; PATH-dir iteration mirrors dir scan) + `gow-wc` (rows + total layout) | role-match — ls is more complex than any Phase 2 crate |
| `crates/gow-ls/src/recurse.rs` *(new module)* | sub-module | walkdir walk | RESEARCH.md Pattern 6 (no Phase 2 analog for walkdir) | new pattern — walkdir introduced |
| `crates/gow-ls/src/layout.rs` *(new module)* | sub-module | terminal width → columns | RESEARCH.md Pattern 7 (no Phase 2 analog for terminal_size) | new pattern |
| `crates/gow-ls/tests/integration.rs` | integration tests | — | `gow-touch/tests/integration.rs` (privilege-skip for symlink) + `gow-wc` fixture dir | exact — skip idiom matches |
| `crates/gow-cp/Cargo.toml` | config | — | `gow-touch/Cargo.toml` (filetime dep) + add `walkdir` | role-match |
| `crates/gow-cp/build.rs` | config | — | `gow-touch/build.rs` | exact |
| `crates/gow-cp/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-cp/src/lib.rs` | utility lib | recursive file I/O + filetime preservation | `gow-touch/src/lib.rs` (filetime use + `no_deref` symlink branch) + `gow-mkdir/src/lib.rs` (per-operand loop + MSYS convert) | role-match (filetime shared) |
| `crates/gow-cp/tests/integration.rs` | integration tests | — | `gow-touch/tests/integration.rs` (sets/asserts mtime via filetime) | exact |
| `crates/gow-mv/Cargo.toml` | config | — | `gow-mkdir/Cargo.toml` (plain) + filetime | role-match |
| `crates/gow-mv/build.rs` | config | — | `gow-touch/build.rs` | exact |
| `crates/gow-mv/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-mv/src/lib.rs` | utility lib | rename + cross-volume fallback | `gow-mkdir/src/lib.rs` (per-operand + MSYS loop) + `gow-rmdir/src/lib.rs` (raw_os_error fallback gate) | role-match |
| `crates/gow-mv/tests/integration.rs` | integration tests | — | `gow-touch/tests/integration.rs` (fixture + metadata assert) | exact |
| `crates/gow-rm/Cargo.toml` | config | — | `gow-mkdir/Cargo.toml` + add `walkdir` | role-match |
| `crates/gow-rm/build.rs` | config | — | `gow-touch/build.rs` | exact |
| `crates/gow-rm/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-rm/src/lib.rs` | utility lib | recursive walk + conditional delete + tty prompt | `gow-rmdir/src/lib.rs` (single-dir `remove_dir` + `raw_os_error`; `rmdir -p` parent walk mirrors rm -r iteration) | role-match (rmdir is closest; rm adds -r + tty prompt) |
| `crates/gow-rm/tests/integration.rs` | integration tests | — | `gow-touch/tests/integration.rs` (read-only test setup) | role-match |
| `crates/gow-ln/Cargo.toml` | config | — | `gow-touch/Cargo.toml` + add `junction` (per-crate dep) | role-match |
| `crates/gow-ln/build.rs` | config | — | `gow-touch/build.rs` | exact |
| `crates/gow-ln/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-ln/src/lib.rs` | utility lib | 2-arg (or N+dir) + link creation via gow-core helper | `gow-mkdir/src/lib.rs` (per-operand) + Phase 1 `fs.rs` symlink-privilege skip | role-match |
| `crates/gow-ln/tests/integration.rs` | integration tests | — | Phase 1 `fs.rs:109-124` (privilege skip) + `gow-touch/tests/integration.rs:214-251` | exact (same skip idiom) |
| `crates/gow-chmod/Cargo.toml` | config | — | `gow-mkdir/Cargo.toml` (plain, no extras) | exact |
| `crates/gow-chmod/build.rs` | config | — | `gow-touch/build.rs` | exact |
| `crates/gow-chmod/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-chmod/src/lib.rs` | utility lib | mode-parse + `set_permissions` | `gow-echo/src/lib.rs` (mode string state-machine pattern — similar to echo's escape parser) + `gow-mkdir` operand loop | role-match |
| `crates/gow-chmod/tests/integration.rs` | integration tests | — | `gow-touch/tests/integration.rs` (metadata assert shape) | exact |
| `crates/gow-head/Cargo.toml` | config | — | `gow-wc/Cargo.toml` (bstr) | exact |
| `crates/gow-head/build.rs` | config | — | `gow-touch/build.rs` | exact |
| `crates/gow-head/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-head/src/lib.rs` | utility lib | stream read + take(n) / read_until loop | `gow-wc/src/lib.rs` (`count_reader` reads stdin OR per-file; multi-file total loop) | exact — head is a subset of wc's I/O shape |
| `crates/gow-head/tests/integration.rs` | integration tests | — | `gow-wc/tests/integration.rs` (multi-file + dash-operand + fixture) | exact |
| `crates/gow-tail/Cargo.toml` | config | — | `gow-wc/Cargo.toml` + `notify` (per-crate dep, workspace-shared) | role-match |
| `crates/gow-tail/build.rs` | config | — | `gow-touch/build.rs` | exact |
| `crates/gow-tail/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-tail/src/lib.rs` | utility lib | stream read + initial-N emission + multi-file switch | `gow-wc/src/lib.rs` (multi-file loop) + `gow-tee/src/lib.rs` (per-chunk read + flush loop) | role-match |
| `crates/gow-tail/src/follow.rs` *(new module)* | sub-module | notify watcher + event-driven append | RESEARCH.md Pattern 5 (no Phase 2 analog — first notify user) | new pattern — Wave 5 isolation |
| `crates/gow-tail/tests/integration.rs` | integration tests | — | `gow-tee/tests/integration.rs` (stdin-feed pattern) + timing assertion from scratch | role-match |
| `crates/gow-dos2unix/Cargo.toml` | config | — | `gow-touch/Cargo.toml` (filetime for -k) + tempfile dev-dep is workspace | role-match |
| `crates/gow-dos2unix/build.rs` | config | — | `gow-touch/build.rs` | exact |
| `crates/gow-dos2unix/src/main.rs` | bin shim | — | `gow-touch/src/main.rs` | exact |
| `crates/gow-dos2unix/src/lib.rs` | utility lib | atomic rewrite via gow-core::fs::atomic_rewrite | `gow-mkdir/src/lib.rs` (per-operand loop) + **new atomic-rewrite helper** | role-match |
| `crates/gow-dos2unix/tests/integration.rs` | integration tests | — | `gow-wc/tests/integration.rs` (fixture + assert via std::fs::read) | exact |
| `crates/gow-unix2dos/` *(entire crate)* | utility crate | byte transform (mirror of dos2unix) | `gow-dos2unix/` (sibling; shares scanner module per D-51/CONTEXT) | exact — near-copy |

---

## Pattern Assignments

### Pattern A — Standard crate scaffolding (applies to ALL 11 new crates)

**Analog:** `crates/gow-touch/` (Cargo.toml + build.rs + src/main.rs)

**Cargo.toml skeleton** (file: `D:/workspace/gow-rust/crates/gow-touch/Cargo.toml`, lines 1-34):

```toml
[package]
name = "gow-{NAME}"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU {NAME} — ..."

[[bin]]
name = "{NAME}"
path = "src/main.rs"

[lib]
name = "uu_{NAME}"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
# optional per-crate adds: bstr / filetime / walkdir / notify / terminal_size / junction

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

**build.rs** (file: `D:/workspace/gow-rust/crates/gow-touch/build.rs`, lines 1-25) — **copy verbatim** to every new crate. No modifications.

**src/main.rs** (file: `D:/workspace/gow-rust/crates/gow-touch/src/main.rs`, lines 1-3) — **copy verbatim**, substitute crate name:

```rust
fn main() {
    std::process::exit(uu_{NAME}::uumain(std::env::args_os()));
}
```

---

### Pattern B — `uumain` signature + init + parse_gnu + operand loop

**Analog:** `crates/gow-touch/src/lib.rs` lines 17-45

**Applies to:** every new utility lib.rs (cat, head, chmod, cp, mv, rm, ln, dos2unix, unix2dos, tail, ls).

```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();                              // line 18 — MANDATORY first call

    let matches = gow_core::args::parse_gnu(uu_app(), args);  // line 20

    let flag_a = matches.get_flag("flag-a");       // ArgAction::SetTrue flags
    let val_b: Option<String> = matches.get_one::<String>("val-b").cloned();

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.is_empty() {                        // GNU: some utilities require operands
        eprintln!("{util}: missing file operand");
        return 1;
    }

    let mut exit_code = 0;
    for op in &operands {
        let converted = gow_core::path::try_convert_msys_path(op);  // D-26 — every file arg
        let path = Path::new(&converted);
        match do_one(path, &flags) {
            Ok(()) => { /* optional verbose print */ }
            Err(e) => {
                eprintln!("{util}: {e}");          // D-11 GNU error format
                exit_code = 1;                      // never early-return — process all operands
            }
        }
    }
    exit_code
}
```

**Key invariants (from D-16, D-26, D-11, inherited):**
1. `gow_core::init()` **first line** (sets UTF-8 codepage + VT100).
2. `gow_core::args::parse_gnu(uu_app(), args)` — NOT `uu_app().get_matches_from()`. `parse_gnu` enforces exit-code 1 on arg-parse error (D-02).
3. `try_convert_msys_path(op)` **per operand** before any FS call (D-26).
4. Accumulate `exit_code`, don't early-return — GNU processes all operands.
5. Errors go to stderr as `{util}: {message}`.

---

### Pattern C — `uu_app()` clap builder

**Analog:** `crates/gow-touch/src/lib.rs` lines 154-217; also `gow-wc/src/lib.rs:213-249`, `gow-mkdir/src/lib.rs:73-94`.

```rust
fn uu_app() -> Command {
    Command::new("{util}")
        .about("GNU {util} — ...")
        .arg(
            Arg::new("flag-name")
                .short('x')
                .long("flag-name")
                .action(ArgAction::SetTrue)
                .help("short description"),
        )
        .arg(
            Arg::new("option-with-value")
                .short('n')
                .long("option-with-value")
                .num_args(1)
                .help("..."),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}
```

**Special cases:**
- `touch` uses `-h` for `--no-dereference` → disables clap's auto help: see `gow-touch/src/lib.rs:159-165`. `ln -h`, `chmod -h` may need the same treatment; `rm -h` does not (no short `-h` in rm).
- `echo` bypasses clap entirely (manual scanner) — gow-echo/src/lib.rs:54-125. Not used in Phase 3.

---

### Pattern D — Byte-stream scaffold (cat, head, tail initial N, dos2unix, unix2dos)

**Analog:** `crates/gow-wc/src/lib.rs` lines 207-211 (BufReader + read_to_end) + lines 65-67 (newline counting via byte filter).

**Cat / head / tail initial read — prefer `BufReader::read_until(b'\n')` over `lines()`:**

```rust
use std::io::{BufRead, BufReader};

let mut reader = BufReader::new(file);
let mut buf = Vec::with_capacity(8192);
loop {
    buf.clear();
    let n = reader.read_until(b'\n', &mut buf)?;
    if n == 0 { break; }
    // raw bytes — no UTF-8 decode (D-48)
    stdout.write_all(&buf)?;
}
```

**Line counting via bstr:** `crates/gow-wc/src/lib.rs:65-67`
```rust
fn count_newlines(bytes: &[u8]) -> u64 {
    bytes.iter().filter(|&&b| b == b'\n').count() as u64
}
```

**Stdin + file + `-` operand loop:** `crates/gow-wc/src/lib.rs:98-146` — exact shape for cat/head/tail:
```rust
if operands.is_empty() {
    // read from stdin
    match count_reader(io::stdin().lock()) { ... }
} else {
    for op in &operands {
        if op == "-" {
            match count_reader(io::stdin().lock()) { ... }
            continue;
        }
        let converted = gow_core::path::try_convert_msys_path(op);
        match File::open(Path::new(&converted)) {
            Ok(f) => match count_reader(BufReader::new(f)) { ... },
            Err(e) => { eprintln!("wc: {converted}: {e}"); exit_code = 1; }
        }
    }
}
```

**cat-specific additions (not in wc):**
- Line-number prefix: `write!(writer, "{n:>6}\t")?` — 6-space right-aligned, tab separator (GNU convention).
- `-v` byte visualization state-machine: see RESEARCH.md §Pattern 8 / FILE-01 `visualize_byte`. No Phase 2 analog — new code.
- Multi-file: no header between files (cat concatenates silently — unlike head/tail's `==>` headers).

**head-specific:**
- `-n N`: loop `0..n` with `read_until(b'\n')`.
- `-c N`: `std::io::copy(&mut reader.take(n as u64), &mut stdout)?`.
- Multi-file `==> file <==` header unless `-q`.

---

### Pattern E — Per-file error reporting without early return

**Analog:** `crates/gow-wc/src/lib.rs` lines 126-145, `crates/gow-mkdir/src/lib.rs` lines 56-68, `crates/gow-rmdir/src/lib.rs` lines 40-57.

**All three share this idiom:**
```rust
let mut exit_code = 0;
for op in &operands {
    match File::open(path) {
        Ok(f) => { /* process */ }
        Err(e) => {
            eprintln!("{util}: {converted}: {e}");
            exit_code = 1;  // accumulate — don't return
        }
    }
}
exit_code
```

Apply to: cat, head, tail, cp, mv, rm, chmod, dos2unix, unix2dos, ls, ln. Every Phase 3 utility iterates operands without short-circuiting.

---

### Pattern F — `filetime` usage for `cp -p`, `mv` cross-volume fallback, dos2unix `-k`

**Analog:** `crates/gow-touch/src/lib.rs` lines 14, 115-122; `crates/gow-touch/tests/integration.rs` lines 57-70.

**Read timestamps from source:**
```rust
use filetime::FileTime;
let md = std::fs::metadata(src)?;
let atime = FileTime::from_last_access_time(&md);
let mtime = FileTime::from_last_modification_time(&md);
```

**Write timestamps to dest:**
```rust
filetime::set_file_times(dst, atime, mtime)?;
// For symlink-self (only ln/cp need this in Phase 3):
filetime::set_symlink_file_times(dst, atime, mtime)?;
```

**Round-trip test pattern** (from `gow-touch/tests/integration.rs:49-70`):
```rust
let fixed = filetime::FileTime::from_unix_time(1_500_000_000, 0); // 2017
filetime::set_file_times(&ref_file, fixed, fixed).unwrap();
// ... run utility ...
let md = std::fs::metadata(&target).unwrap();
let mtime = filetime::FileTime::from_last_modification_time(&md);
assert_eq!(mtime.unix_seconds(), 1_500_000_000);
```

---

### Pattern G — `raw_os_error()` defense-in-depth for error classification

**Analog:** `crates/gow-rmdir/src/lib.rs` lines 100-114.

**Use for:** `ln` detecting cross-volume hardlink (ERROR_NOT_SAME_DEVICE = 17), `ln -s` detecting symlink privilege (ERROR_PRIVILEGE_NOT_HELD = 1314), `mv` detecting cross-volume rename, `rm` detecting directory-non-empty for `-rf` edge cases.

```rust
fn is_cross_volume(e: &io::Error) -> bool {
    if e.kind() == io::ErrorKind::CrossesDevices { return true; }
    #[cfg(windows)]
    { e.raw_os_error() == Some(17) }  // ERROR_NOT_SAME_DEVICE
    #[cfg(not(windows))]
    { e.raw_os_error() == Some(18) }  // EXDEV
}
```

Pattern: **always check `ErrorKind` first (future-proof), fall back to raw code for defensive coverage.**

---

### Pattern H — MSYS path conversion per operand

**Analog:** used in **every** Phase 2 crate that takes file operands.

- `crates/gow-mkdir/src/lib.rs:44`
- `crates/gow-rmdir/src/lib.rs:41`
- `crates/gow-touch/src/lib.rs:62`
- `crates/gow-tee/src/lib.rs:55`
- `crates/gow-wc/src/lib.rs:124`

```rust
let converted = gow_core::path::try_convert_msys_path(op);
let path = Path::new(&converted);
```

**D-26 rule:** apply to every file-position argument before any FS call. Never skip.

---

### Pattern I — Privilege-skip idiom for symlink-requiring tests

**Analog:**
- `crates/gow-core/src/fs.rs` lines 109-124 (unit test with conditional skip)
- `crates/gow-touch/tests/integration.rs` lines 214-251 (integration-level skip with explicit `return`)

**Integration test idiom** (copy into `gow-ln/tests/integration.rs`, `gow-ls/tests/integration.rs`, `gow-cp/tests/integration.rs`):

```rust
#[test]
#[cfg_attr(not(windows), ignore)]
fn test_xyz_needs_symlink() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("real.txt");
    let link = tmp.path().join("link.txt");
    std::fs::write(&target, b"real").unwrap();

    #[cfg(windows)]
    let created = std::os::windows::fs::symlink_file(&target, &link).is_ok();
    #[cfg(not(windows))]
    let created = std::os::unix::fs::symlink(&target, &link).is_ok();

    if !created {
        eprintln!("[skip] symlink creation requires Developer Mode / SeCreateSymbolicLinkPrivilege");
        return;
    }
    // ... actual test body ...
}
```

**Unit-level skip** (from `gow-core/src/fs.rs:119`):
```rust
if symlink_file(&target, &link).is_ok() {
    assert_eq!(link_type(&link), Some(LinkType::SymlinkFile));
} else {
    eprintln!("Skipping symlink test: insufficient privilege");
}
```

Apply to: `gow-ln` (all symlink tests), `gow-ls` (symlink/junction link-type tests), `gow-cp` (clone-symlink tests), `gow-core/src/fs.rs` new helper tests (create_link).

---

### Pattern J — `gow-core::fs` extension (NEW helpers from D-46..D-47, Research Patterns 1-4)

**File:** `crates/gow-core/src/fs.rs` — **EXTEND existing file** (currently 161 lines: LinkType + link_type + normalize_junction_target).

**Current file header pattern** (file: `D:/workspace/gow-rust/crates/gow-core/src/fs.rs` lines 1-13):
```rust
//! Windows symlink and junction abstraction layer.
//!
//! ...
//! Covers: FOUND-07
```

**ADD** to the module (after `normalize_junction_target`, before `#[cfg(test)] mod tests`):

1. **`atomic_rewrite<F>`** (RESEARCH.md lines 304-364) — full code given. Wraps `tempfile::NamedTempFile::new_in` + `sync_all` + `persist`.
2. **`LinkKind` enum + `LinkOutcome` enum + `create_link`** (RESEARCH.md lines 382-461) — full code given. Dispatches hard/symbolic with D-36 junction fallback on ERROR_PRIVILEGE_NOT_HELD (1314).
3. **`is_hidden(path)`** (RESEARCH.md lines 479-498) — dot-prefix OR FILE_ATTRIBUTE_HIDDEN union (D-34).
4. **`is_readonly(md: &Metadata)`** (RESEARCH.md lines 503-505) — 1-liner wrapping `md.permissions().readonly()`.
5. **`has_executable_extension(path)`** (RESEARCH.md lines 510-518) — fixed set `{exe, cmd, bat, ps1, com}` per D-35.
6. **`clear_readonly(path)`** (RESEARCH.md lines 522-533) — `perms.set_readonly(false)` + `set_permissions`.
7. **`is_drive_root(path)`** (RESEARCH.md lines 546-564) — `C:\`, UNC `\\server\share` detection for D-42.

**New per-gow-core-crate deps (add to `crates/gow-core/Cargo.toml`):**
```toml
tempfile = { workspace = true }    # new runtime dep — for atomic_rewrite
junction = "1.4"                    # new runtime dep — for create_link fallback (D-36)
```

**Tests extension:** the existing `#[cfg(test)] mod tests` (line 83-160) already uses `tempfile::TempDir`. Add new tests following same shape: `test_atomic_rewrite_roundtrip`, `test_create_link_hard`, `test_create_link_symlink_privilege_skip`, `test_is_drive_root_variants`, etc.

---

### Pattern K — walkdir iteration (NEW — no Phase 2 analog)

**Source:** RESEARCH.md §Pattern 6 (lines 684-755). First use of walkdir in the project.

**Shape 1: `ls -R` and `cp -r` — parents-first, sorted:**
```rust
use walkdir::WalkDir;
for entry in WalkDir::new(root)
    .follow_links(follow_mode)       // cp -L vs -P
    .sort_by_file_name()
    .into_iter()
    .filter_entry(|e| !should_skip(e))   // ls without -a: skip hidden dirs
{
    match entry {
        Ok(e) => handle(&e),
        Err(err) => eprintln!("{util}: cannot access '{}': {}",
            err.path().map(|p| p.display().to_string()).unwrap_or_default(),
            err.io_error().map(|e| e.to_string()).unwrap_or_else(|| err.to_string())),
    }
}
```

**Shape 2: `rm -r` — contents-first (MANDATORY):**
```rust
for entry in WalkDir::new(root)
    .contents_first(true)            // ESSENTIAL — leaves before parents
    .follow_links(false)             // never follow during delete
    .into_iter()
{
    let entry = entry?;
    if entry.file_type().is_dir() {
        std::fs::remove_dir(entry.path())?;
    } else {
        if options.force && gow_core::fs::is_readonly(&entry.metadata()?) {
            gow_core::fs::clear_readonly(entry.path())?;  // Pitfall 3
        }
        std::fs::remove_file(entry.path())?;
    }
}
```

**Anti-pattern alert:** omitting `contents_first(true)` on rm -r → ENOTEMPTY on every non-empty dir.

---

### Pattern L — notify watcher (NEW — Wave 5 isolated; no Phase 2 analog)

**Source:** RESEARCH.md §Pattern 5 (lines 567-667). Full `follow_descriptor` given.

**Key invariants:**
1. `RecommendedWatcher::new(tx, Config::default())` — NO debouncer (Pitfall 7 — violates 200ms).
2. `watcher.watch(parent_dir, RecursiveMode::NonRecursive)` — **NOT `watch(file_path, …)`** (Pitfall 1 — zero events on Windows).
3. Filter events in recv loop by `event.paths[i].file_name() == target_name`.
4. Truncation detection: compare `current_size < offset` → seek to 0 + emit `file truncated` message.
5. `-F` variant watches for `EventKind::Create` of matching filename to reopen.

**Isolation:** `notify` is a workspace dep (per D-50) but only `gow-tail`'s Cargo.toml references it. Other crates don't pull it in.

---

### Pattern M — tempfile + atomic_rewrite consumer (dos2unix, unix2dos)

**Source:** RESEARCH.md §Pattern 1 + FILE §CONV-01/02. The helper lives in gow-core; utilities just call it.

```rust
// crates/gow-dos2unix/src/lib.rs (inside per-operand loop)
use gow_core::fs::atomic_rewrite;

atomic_rewrite(path, |bytes| {
    if is_binary(bytes) {
        return Err(GowError::Custom(
            format!("dos2unix: Skipping binary file {}", path.display())
        ));
    }
    Ok(transform_crlf_to_lf(bytes))  // or transform_lf_to_crlf for unix2dos
})?;
```

**Binary detection** (shared between dos2unix/unix2dos) — scan first 32 KB for `0x00`:
```rust
fn is_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(32 * 1024).any(|&b| b == 0x00)
}
```

**-k flag** (preserve mtime): after atomic_rewrite succeeds, re-apply original timestamps via Pattern F.

**Shared scanner module:** per D-51 conversion-pair — planner's choice whether to (a) create `gow-dos2unix-common` sibling crate, (b) duplicate the 10-line transform in each, or (c) put it in `gow-core::text`. Recommendation: put `crlf_to_lf` and `lf_to_crlf` bytes functions in **one** of the crates (e.g. dos2unix) and have unix2dos depend on it via `uu_dos2unix::transform::*`. Minimizes workspace member count.

---

### Pattern N — Integration test scaffolding

**Analog:** `crates/gow-touch/tests/integration.rs` + `crates/gow-wc/tests/integration.rs`.

**Standard harness** (first 10 lines of every `tests/integration.rs`):
```rust
use assert_cmd::Command;
use predicates::prelude::*;

fn {util}() -> Command {
    Command::cargo_bin("{util}")
        .expect("{util} binary not found — run `cargo build -p gow-{util}` first")
}

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}
```

**Standard test shapes by utility category:**

| Utility | Required tests (minimum from RESEARCH.md Per-Utility section) |
|---------|-------------------------------------------------------------|
| cat | pass-through, -n UTF-8, -v control-chars, stdin via `-`, multi-file concat, CP949 no-panic, missing-file exit-1 |
| head | default-10, -n 5, -5 shorthand, -c 100, multi-file headers, -q, empty-file |
| tail | default-10, -n 5, -c 100, **-f latency** (spawn writer, assert < 500ms observed), -F rotate, -f truncate, multi-file header switch |
| chmod | 644 writable, 444 readonly, +w, -w, u=r, -R dir walk, metadata assert |
| cp | single-file, -r dir, -rp timestamps via filetime assert, -P clone symlink, -L follow, missing-source exit-1 |
| mv | rename, into-dir, cross-drive, -i prompt via stdin-feed, missing-source |
| rm | file, -r dir, -rf RO-file, `rm C:\` refused, `--no-preserve-root` override, no-such-file (exit-1 w/o -f, exit-0 with -f), RO w/o tty exit-1 |
| ln | hard-link same-inode, cross-device exit-1, -s file (skip if no priv), -s dir (junction fallback warning), -f replace |
| ls | plain listing, -la, -R, --color=always, symlink `->`, junction `-> target [junction]`, hidden via `.git`, hidden via `attrib +h` (platform-gate), missing-path exit-1, **privilege-skip for symlink tests** |
| dos2unix | CRLF→LF in-place, binary skip, -f force, -n src dst new-file, -k preserve mtime, round-trip identity |
| unix2dos | LF→CRLF in-place, binary skip, round-trip identity (mirror of dos2unix) |

**Fixture pattern for text tests** (from `gow-wc/tests/integration.rs:13-18`): use `tempfile::tempdir()` + `write_fixture` helper. Predicate assertions via `predicate::str::is_match` (regex) for flexible whitespace handling.

---

## Shared Patterns (cross-cutting)

### Shared 1 — GNU error format + exit code 1

**Source:** `crates/gow-core/src/error.rs` (entire file, 137 lines).
**Apply to:** every utility's error reporting.

**Format:** `{util}: {path}: {reason}` — always to stderr, always exit 1.

```rust
eprintln!("{util}: cannot create '{converted}': {e}");     // create-style error
eprintln!("{util}: {converted}: {e}");                     // open-style error (wc, tee pattern)
eprintln!("{util}: missing file operand");                 // usage error
```

**Exit code:** `GowError::exit_code()` always returns 1 (errors.rs:52). Even structurally distinct errors use exit 1 per GNU convention (except `cmp`/`diff` which Phase 3 doesn't include).

---

### Shared 2 — `gow_core::init()` as first line of `uumain`

**Source:** `crates/gow-core/src/lib.rs:15-19`. Calls:
- `encoding::setup_console_utf8()` — `SetConsoleOutputCP(65001)`.
- `color::enable_vt_mode()` — enables ANSI escape sequences.

**Apply to:** every `uumain` in Phase 3. See Pattern B. Enables ROADMAP criterion 4 (cat -n UTF-8) automatically.

---

### Shared 3 — Unit + integration test split

**Source:** every Phase 2 crate. Unit tests live in `#[cfg(test)] mod tests { ... }` at the bottom of `src/lib.rs` (e.g. `gow-wc/src/lib.rs:251-327`, `gow-touch/src/lib.rs:219-227`). Integration tests spawn the binary via `assert_cmd::Command::cargo_bin(...)` from `tests/integration.rs`.

**Rule:** **do not** unit-test `uumain` directly (it calls `std::env::args_os` and exits); only exercise pure helpers (e.g. `count_bytes`, `parse_mode`, `is_drive_root`) in unit tests. Exit-code behavior is **integration-test-only**.

---

### Shared 4 — Phase 2 `tests/integration.rs` test-count baseline

Phase 2 crates average 10-15 integration tests per utility. Planner should size test plans similarly:
- `gow-touch/tests/integration.rs`: 16 tests (Phase 2 reference)
- `gow-wc/tests/integration.rs`: 12 tests
- Phase 3 utilities: target 10-15 tests per crate; `tail -f`, `ls -l`, `cp -r` may have more due to scenario coverage.

---

### Shared 5 — Workspace.toml amendments (Cargo.toml changes)

**Source:** `D:/workspace/gow-rust/Cargo.toml` lines 2-20, 30-62.

**Add to `[workspace.members]` (after line 19):**
```toml
    # Phase 3 — filesystem utilities (D-49)
    "crates/gow-cat",
    "crates/gow-ls",
    "crates/gow-cp",
    "crates/gow-mv",
    "crates/gow-rm",
    "crates/gow-ln",
    "crates/gow-chmod",
    "crates/gow-head",
    "crates/gow-tail",
    "crates/gow-dos2unix",
    "crates/gow-unix2dos",
```

**Add to `[workspace.dependencies]` (after line 62):**
```toml
# Phase 3 additions (D-50 + junction)
walkdir = "2.5"               # recursive traversal (cp, rm, ls, chmod -R)
notify = "8.2"                # tail -f watcher — Wave 5 only
terminal_size = "0.4"         # ls column layout — gow-ls only
junction = "1.4"              # D-36 directory-symlink fallback — gow-core + gow-ln
```

**Also amend `crates/gow-core/Cargo.toml`** (file: `D:/workspace/gow-rust/crates/gow-core/Cargo.toml`) — add to `[dependencies]` (after line 16):
```toml
tempfile = { workspace = true }   # atomic_rewrite (was dev-dep only; now runtime)
junction = { workspace = true }   # create_link D-36 fallback
```

Note: `tempfile` moves from `[dev-dependencies]` (line 23) to `[dependencies]` for gow-core. Keep it listed under `[dev-dependencies]` too if both are needed — Cargo de-dupes.

---

## No Analog Found

Files with no direct Phase 2 match — planner should use **RESEARCH.md patterns** as primary reference:

| File / Component | Role | Data Flow | Why no analog | Reference |
|------------------|------|-----------|---------------|-----------|
| `crates/gow-ls/src/recurse.rs` (walkdir loop) | sub-module | recursive walk | First walkdir user in project | RESEARCH.md Pattern 6 |
| `crates/gow-cp/src/recurse.rs` | sub-module | recursive walk + copy | First walkdir user | RESEARCH.md Pattern 6 |
| `crates/gow-rm/src/recurse.rs` (contents_first) | sub-module | recursive walk + delete | First walkdir user + contents_first | RESEARCH.md Pattern 6 + Pitfall 6 |
| `crates/gow-ls/src/layout.rs` (terminal_size) | sub-module | terminal width → column count | First terminal_size user | RESEARCH.md Pattern 7 |
| `crates/gow-tail/src/follow.rs` (notify loop) | sub-module | filesystem event → append emit | First notify user | RESEARCH.md Pattern 5 |
| `crates/gow-core/src/fs.rs::atomic_rewrite` | gow-core helper | read+transform+rename | First tempfile::persist user | RESEARCH.md Pattern 1 |
| `crates/gow-core/src/fs.rs::create_link` | gow-core helper | link syscall + fallback | First junction crate user | RESEARCH.md Pattern 2 |
| `crates/gow-cat/src/lib.rs` `visualize_byte` state machine | fn | byte → escape-encoded bytes | `-v` visualization is unique | RESEARCH.md §FILE-01 (lines 1025-1044) |
| `crates/gow-chmod/src/lib.rs` mode parser | fn | string → RO bool | GNU symbolic mode parsing unique | RESEARCH.md §FILE-10 |
| `crates/gow-mv/src/lib.rs` cross-volume fallback | fn | io::Error→copy+delete | Uses Pattern G + new copy-then-remove | RESEARCH.md §FILE-04 (lines 1152-1163) |
| `crates/gow-tail/tests/integration.rs` timing assertion | test | spawn-writer + time measurement | No Phase 2 test measures latency | RESEARCH.md §TEXT-02 test list |

---

## Key Patterns Identified (summary for planner)

1. **Every new crate is a 4-file copy-adjust of gow-touch** — Cargo.toml (with added deps), build.rs (verbatim), src/main.rs (3 lines, rename ident), src/lib.rs (new).
2. **`uumain` signature and first 5 lines are standardized** — `gow_core::init()`, `parse_gnu(uu_app(), args)`, flag extraction, operand vector, empty check.
3. **Per-operand loop with MSYS convert + error accumulate** — universal; never early-return; every utility.
4. **Byte-safe I/O via `BufReader::read_until(b'\n')` or `read_to_end` + `bstr`** — D-48; shared across cat/head/tail/dos2unix/unix2dos.
5. **Raw-OS-error defense-in-depth** (rmdir precedent) — used by ln (1314, 17), mv (17), rm edge cases.
6. **gow-core extension is the single biggest dependency** — 7 new helpers in `fs.rs` unlock every other Phase 3 crate. Wave 0 prep plan must land these first.
7. **Privilege-skip for symlink tests is mandatory** — fs.rs:119 + touch integration.rs:214-251 pattern; ln/ls/cp tests inherit.
8. **Three new crates introduce their own subpatterns** — ls (walkdir + terminal_size + layout), tail (notify watcher), dos2unix (atomic_rewrite consumer). Each should get its own sub-module.
9. **Workspace amendments are surgical** — 11 new members + 4 new deps (walkdir, notify, terminal_size, junction) + tempfile promotion in gow-core.
10. **Test sizes align with Phase 2** — 10-15 integration tests per utility; unit tests only for pure helpers.

---

## Metadata

**Analog search scope:**
- `D:/workspace/gow-rust/crates/gow-core/` (full module)
- `D:/workspace/gow-rust/crates/gow-touch/` (full crate — primary analog for scaffolding + filetime)
- `D:/workspace/gow-rust/crates/gow-wc/` (full crate — primary analog for byte-stream + multi-file)
- `D:/workspace/gow-rust/crates/gow-which/` (pathext module + env override pattern)
- `D:/workspace/gow-rust/crates/gow-tee/` (stream + fanout pattern — closest to tail -f pre-notify shape)
- `D:/workspace/gow-rust/crates/gow-mkdir/` (operand-loop + verbose)
- `D:/workspace/gow-rust/crates/gow-rmdir/` (raw_os_error pattern + rmdir -p mirrors rm -r)
- `D:/workspace/gow-rust/crates/gow-echo/` (ad-hoc arg scanner — not reused, but state-machine reference for chmod mode parser and cat -v)
- `D:/workspace/gow-rust/Cargo.toml` (workspace config)

**Files read (non-overlapping ranges):**
- `Cargo.toml` (workspace root) — 70 lines (full)
- `gow-core/{Cargo.toml, src/lib.rs, src/fs.rs, src/error.rs}` — full
- `gow-touch/{Cargo.toml, build.rs, src/main.rs, src/lib.rs, tests/integration.rs}` — full
- `gow-wc/{Cargo.toml, src/lib.rs, tests/integration.rs}` — full
- `gow-which/{Cargo.toml, src/lib.rs, src/pathext.rs}` — full
- `gow-mkdir/src/lib.rs`, `gow-rmdir/src/lib.rs`, `gow-tee/src/lib.rs`, `gow-echo/src/lib.rs` — full
- `.planning/phases/03-filesystem/03-CONTEXT.md` — full (180 lines)
- `.planning/phases/03-filesystem/03-RESEARCH.md` — relevant sections (1-1500, covering all Patterns 1-8 + per-utility notes)

**Pattern extraction date:** 2026-04-21

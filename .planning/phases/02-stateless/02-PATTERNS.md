# Phase 02 — Stateless Utilities — Pattern Map

**Mapped:** 2026-04-21
**Scope:** 14 new utility crates + workspace Cargo.toml edit
**Primary analog:** `crates/gow-probe/` (Phase 1 Plan 04 capstone — canonical bin-crate template)
**Secondary analog:** `crates/gow-core/` (lib-crate structure + public API conventions)

---

## File Inventory — What Phase 2 Creates

The 14 utilities (by decision D-16 / D-16d, binary names drop the `gow-` prefix):

| # | Crate dir | Bin name | Lib name | Notes |
|---|-----------|----------|----------|-------|
| 1 | `crates/gow-echo/` | `echo.exe` | `uu_echo` | UTIL-01; ad-hoc flag loop OK (D-21) |
| 2 | `crates/gow-pwd/` | `pwd.exe` | `uu_pwd` | UTIL-02; inline UNC-prefix strip (Q8) |
| 3 | `crates/gow-env/` | `env.exe` | `uu_env` | UTIL-03; full GNU set incl. `-S` state machine (Q7) |
| 4 | `crates/gow-tee/` | `tee.exe` | `uu_tee` | UTIL-04; `-i` via `SetConsoleCtrlHandler` (Q10) |
| 5 | `crates/gow-basename/` | `basename.exe` | `uu_basename` | UTIL-05; MSYS path pre-convert (D-26) |
| 6 | `crates/gow-dirname/` | `dirname.exe` | `uu_dirname` | UTIL-06; MSYS path pre-convert (D-26) |
| 7 | `crates/gow-yes/` | `yes.exe` | `uu_yes` | UTIL-07; 8–64 KiB prefill buffer (Q4) |
| 8 | `crates/gow-true/` | `true.exe` | `uu_true` | UTIL-08; trivially returns 0 |
| 9 | `crates/gow-false/` | `false.exe` | `uu_false` | UTIL-09; trivially returns 1 |
| 10 | `crates/gow-mkdir/` | `mkdir.exe` | `uu_mkdir` | FILE-06; `std::fs::create_dir_all` (Q5) |
| 11 | `crates/gow-rmdir/` | `rmdir.exe` | `uu_rmdir` | FILE-07; manual parent loop (Q5) |
| 12 | `crates/gow-touch/` | `touch.exe` | `uu_touch` | FILE-08; `jiff` + `parse_datetime` + `filetime` (Q1/Q2) |
| 13 | `crates/gow-wc/` | `wc.exe` | `uu_wc` | TEXT-03; `bstr` Unicode-aware (D-17) |
| 14 | `crates/gow-which/` | `which.exe` | `uu_which` | WHICH-01; hybrid PATHEXT (Q6) |

Per crate, Phase 2 creates **5 files**: `Cargo.toml`, `build.rs`, `src/lib.rs`, `src/main.rs`, `tests/integration.rs`. Some utilities (`gow-env`, `gow-touch`, `gow-which`, `gow-wc`, `gow-tee`, `gow-echo`) will add extra `src/*.rs` helper modules; those have no direct analog and use RESEARCH.md pattern hints.

**Workspace edits:**
- `Cargo.toml` (root) — add 14 new members + 3 new `[workspace.dependencies]` entries (`snapbox`, `bstr`, `filetime`).

**Total new files:** `14 × 5 = 70` base files + utility-specific helper modules + 1 workspace manifest edit.

---

## File Classification

| New/modified file (template) | Role | Data flow | Closest analog | Match quality |
|------------------------------|------|-----------|----------------|---------------|
| `Cargo.toml` (root edit) | workspace manifest | N/A | existing `D:\workspace\gow-rust\Cargo.toml` | exact (append-only) |
| `crates/gow-{name}/Cargo.toml` | bin-crate manifest | N/A | `crates/gow-probe/Cargo.toml` | exact — copy + rename + drop `publish = false` |
| `crates/gow-{name}/build.rs` | build script (manifest embed) | N/A | `crates/gow-probe/build.rs` | exact — verbatim copy, change `"Gow.Rust"` optional |
| `crates/gow-{name}/src/main.rs` | bin wrapper | argv → `uumain` → exit code | `crates/gow-probe/src/main.rs` | role-match, structurally thinner (3 lines vs 40) |
| `crates/gow-{name}/src/lib.rs` | lib — `uumain` entry | OsStrings → clap → logic → stdout/stderr → i32 | (none in repo) — reference: `args.rs` module shape + research stubs | no analog — first lib-with-uumain in repo |
| `crates/gow-{name}/tests/integration.rs` | integration test | spawn binary → assert stdout/exit | `crates/gow-probe/tests/integration.rs` | exact — `assert_cmd::Command::cargo_bin` + `predicates` |
| Utility helper modules (`echo/src/escape.rs`, `env/src/split.rs`, `touch/src/date.rs`, …) | internal library | pure function | (none) — RESEARCH.md stubs only | no analog |

---

## Shared Patterns (apply to every utility)

### S1. `gow_core::init()` first — console UTF-8 + VT mode

**Source:** `crates/gow-probe/src/main.rs:14-15`, `crates/gow-core/src/lib.rs:16-19`

```rust
fn main() {
    gow_core::init();   // always the first line
    // ... dispatch
}
```

**Apply to:** every utility's `src/main.rs` (OR inside `uumain`; planner picks one consistent location). Exception: `gow-true`, `gow-false` may omit since they do no I/O — but D-16b says keep the pattern for consistency with the multicall plan in Phase 6.

### S2. GNU arg parsing via `gow_core::args::parse_gnu`

**Source:** `crates/gow-probe/src/main.rs:17-35`

```rust
use clap::{Arg, ArgAction, Command};

let cmd = Command::new("echo")
    .arg(Arg::new("no-newline").short('n').action(ArgAction::SetTrue))
    .arg(Arg::new("interpret").short('e').action(ArgAction::SetTrue))
    // ...
    ;

let matches = gow_core::args::parse_gnu(cmd, std::env::args_os());
```

Signature confirmed from `crates/gow-core/src/args.rs:35-38`:

```rust
pub fn parse_gnu(
    cmd: Command,
    args: impl IntoIterator<Item = std::ffi::OsString>,
) -> ArgMatches
```

`parse_gnu` already handles D-02 (exit code 1 on arg error) and D-11 (`{bin}: {msg}` stderr format). **Utility code must not replicate this.** Echo per D-21 may skip clap entirely and use an ad-hoc `argv` loop; if so, still call `parse_gnu` for `--help`/`--version` dispatch OR print manually and route non-option exit codes through exit 1.

**Apply to:** every utility except `gow-true`, `gow-false` (D-22: flags ignored), and possibly `gow-echo` (D-21: ad-hoc loop allowed).

### S3. GNU error format `{util}: {message}`

**Source pattern documented in:** Phase 1 Plan 02 (D-11), `crates/gow-core/src/args.rs:63-69`

```rust
eprintln!("echo: {err}");
std::process::exit(1);
```

Already enforced by `parse_gnu` for arg errors. For runtime errors after parsing, use the same shape:

```rust
if let Err(e) = run(&matches) {
    eprintln!("{bin}: {e}", bin = "echo");
    return 1;
}
```

**Apply to:** every utility's `uumain` error path.

### S4. MSYS path pre-conversion for positional file args

**Source:** `crates/gow-probe/src/main.rs:39` (usage) + `crates/gow-core/src/path.rs:30` (`try_convert_msys_path`), `:85` (`normalize_file_args`).

```rust
let input = sub.get_one::<String>("input").unwrap();
let converted = gow_core::path::try_convert_msys_path(input);
```

**Apply to:** `gow-basename`, `gow-dirname` (explicit D-26), `gow-touch`, `gow-mkdir`, `gow-rmdir`, `gow-tee` (file arguments), `gow-wc` (file arguments). Not applicable to `gow-echo`, `gow-yes`, `gow-true`, `gow-false`, `gow-pwd`, `gow-env` (no file paths in positional args), `gow-which` (search name, not a file path).

### S5. `uumain` signature (D-16a)

No exact repo analog exists yet. Canonical shape established here — all 14 libs use this exact signature:

```rust
use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    // ... parse, run, return 0/1
}
```

Rationale in RESEARCH.md Q3: uutils uses `impl uucore::Args -> UResult<()>` but we have no `uucore` dep.

---

## Pattern Assignments — Per Template File

### Template 1: `crates/gow-{name}/Cargo.toml` (14 copies)

**Analog:** `D:\workspace\gow-rust\crates\gow-probe\Cargo.toml` (27 lines)

**Excerpt to copy (verbatim except marked deltas):**

```toml
[package]
name = "gow-echo"                  # ← rename per crate
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU echo — UTF-8 safe Windows port."   # ← per-utility one-liner
# publish = false                  # ← DELETE this line; Phase 2 crates ARE shipped

[[bin]]
name = "echo"                      # ← GNU name (no gow- prefix) — D-16d
path = "src/main.rs"

[lib]
name = "uu_echo"                   # ← NEW vs gow-probe (probe had no lib)
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
tempfile = { workspace = true }
snapbox = { workspace = true }     # ← NEW — added to workspace.deps by Phase 2
```

**Per-utility deltas from the template:**

| Crate | Extra `[dependencies]` | Reason |
|-------|------------------------|--------|
| `gow-wc` | `bstr = { workspace = true }` | D-17 byte-safe iteration |
| `gow-touch` | `filetime = { workspace = true }`, `jiff = "0.2"`, `parse_datetime = "0.14"` | Q1, Q2; jiff+parse_datetime are NOT workspace-wide (D-20b) |
| `gow-tee` | `[target.'cfg(windows)'.dependencies] windows-sys = { workspace = true }` | Q10 — `SetConsoleCtrlHandler` direct call (gow-core doesn't re-export this) |
| `gow-env` | — (uses std only per Q7) | pure state machine |
| `gow-which` | — (uses std only per Q6) | `env::split_paths`, `env::var_os` |
| `gow-true`, `gow-false` | DROP `clap`, `anyhow`, `thiserror` | truly trivial — no parsing, no errors |
| `gow-yes` | — (uses std only per Q4) | `io::Write` loop |
| `gow-echo` | — | ad-hoc flag loop (D-21); still keeps clap for `--help` |
| all others | template as above | |

**Match quality:** EXACT for template shape; per-utility deltas documented.

---

### Template 2: `crates/gow-{name}/build.rs` (14 copies)

**Analog:** `D:\workspace\gow-rust\crates\gow-probe\build.rs` (25 lines, lines 1-25)

**Excerpt — COPY VERBATIM, zero edits needed:**

```rust
//! Build script for gow-echo.                         // ← only change: doc line 1 identifies crate
//!
//! Embeds a Windows application manifest that enables:
//! - `activeCodePage = UTF-8` (WIN-01): process-wide ANSI APIs operate on UTF-8.
//!   Complements `SetConsoleOutputCP(65001)` at runtime.
//! - `longPathAware = true` (WIN-02): bypasses the MAX_PATH 260-character limit on
//!   Windows 10+ without the `\\?\` prefix hack.
//!
//! This is a binary crate, so `embed_manifest()` is called unconditionally on
//! Windows (no bin-target gate, unlike gow-core which is lib-only). See
//! RESEARCH.md Pitfall 4 — the manifest must be embedded in EACH binary `.exe`.

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

**Delta from gow-probe:** none beyond the doc comment identifying the crate. D-16c: no `has_bin_target()` gate needed (that was only for lib-only `gow-core`).

**Match quality:** EXACT — this is the canonical template. Planner should instruct executors to copy byte-for-byte.

---

### Template 3: `crates/gow-{name}/src/main.rs` (14 copies — thin wrapper)

**Analog:** `D:\workspace\gow-rust\crates\gow-probe\src\main.rs` (55 lines) — but Phase 2 bins are much thinner because logic lives in lib.

**Excerpt — canonical Phase 2 `main.rs`:**

```rust
//! `echo` binary entry. All logic lives in `uu_echo::uumain`.
//! See RESEARCH.md Q3 for the signature rationale.

fn main() {
    std::process::exit(uu_echo::uumain(std::env::args_os()));
}
```

**Delta from gow-probe's main.rs:**
- REMOVE the inline clap `Command` construction (lines 17-33 of probe) — it belongs in lib's `uu_app()`.
- REMOVE the `gow_core::init()` call (move into `uumain`).
- REMOVE subcommand dispatch (lines 37-53 of probe) — utilities have flags, not subcommands.

**Trivial cases (`true.exe`, `false.exe`):**

```rust
fn main() { std::process::exit(uu_true::uumain(std::env::args_os())); }
```

No change from the template; the lib is just 3 lines.

**Match quality:** role-match — the spawn + exit wiring is identical to gow-probe, but the structure is simplified because of the lib/bin split (D-16).

---

### Template 4: `crates/gow-{name}/src/lib.rs` — `uumain` entry (14 copies)

**No direct repo analog exists.** Closest conceptual reference: the module shape of `crates/gow-core/src/args.rs` (module docs → `pub fn` → `#[cfg(test)]` block), plus the `uumain` skeleton recommended in RESEARCH.md Q3 lines 233-252.

**Canonical skeleton (applies to clap-using utilities):**

```rust
//! `uu_echo`: GNU `echo` ported to Windows with UTF-8 + VT.
//!
//! Entry: `pub fn uumain(args: impl IntoIterator<Item = OsString>) -> i32`.
//! Wraps `gow_core::init()` + `gow_core::args::parse_gnu(uu_app(), args)` + run.

use std::ffi::OsString;
use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);

    match run(&matches) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("echo: {e}");
            1
        }
    }
}

fn run(matches: &ArgMatches) -> Result<(), gow_core::error::GowError> {
    // ... utility-specific logic
    Ok(())
}

fn uu_app() -> Command {
    Command::new("echo")
        .arg(Arg::new("no-newline").short('n').action(ArgAction::SetTrue))
        .arg(Arg::new("interpret-escapes").short('e').action(ArgAction::SetTrue))
        .arg(Arg::new("disable-escapes").short('E').action(ArgAction::SetTrue))
        .arg(Arg::new("args").action(ArgAction::Append))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uumain_help_returns_zero() {
        // --help exits via clap with code 0; testable via assert_cmd in integration.
        // Unit-level smoke: uu_app() builds without panic.
        let _ = uu_app();
    }
}
```

**Trivial skeleton (`uu_true`, `uu_false` — D-22):**

```rust
//! `uu_true`: exits 0 regardless of args.
use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    0   // or 1 for uu_false
}
```

**Match quality:** NO ANALOG in existing repo — this is the first lib-with-uumain crate in the workspace. Planner should reference RESEARCH.md Q3 and this section directly.

**Cross-ref:** `gow_core::error::GowError::exit_code() -> i32 { 1 }` (Plan 01-03 decision) — all error variants currently return 1. If any Phase 2 utility needs a non-1 error code (none identified in CONTEXT.md), it must not hardcode: return `e.exit_code()` instead.

---

### Template 5: `crates/gow-{name}/tests/integration.rs` (14 copies)

**Analog:** `D:\workspace\gow-rust\crates\gow-probe\tests\integration.rs` (123 lines) — full structural match.

**Structural pattern to copy (excerpt from probe lines 13-42):**

```rust
//! Integration tests for `gow-echo` via assert_cmd.
//! Per D-30b, each crate must have ≥ 4 tests:
//!   (1) default/basic behavior
//!   (2) exit code on bad flag → 1 (NOT clap's 2)
//!   (3) GNU-format error message: `{util}: {msg}`
//!   (4) UTF-8 input/filename round-trip

use assert_cmd::Command;
use predicates::prelude::*;

fn echo() -> Command {
    Command::cargo_bin("echo")
        .expect("echo binary not found — run `cargo build -p gow-echo` first")
}

#[test]
fn test_default_prints_with_newline() {
    echo()
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello\n"));
}

#[test]
fn test_bad_flag_exits_1_not_2() {
    echo()
        .arg("--completely-unknown-flag-xyz")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_gnu_error_format() {
    echo()
        .arg("--completely-unknown-flag-xyz")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("echo:"));
}

#[test]
fn test_utf8_roundtrip() {
    echo()
        .arg("안녕")
        .assert()
        .success()
        .stdout(predicate::str::contains("안녕"));
}
```

**Patterns the planner should require in every integration.rs:**

1. **`Command::cargo_bin("<GNU name>")`** — use binary name (`echo`, not `gow-echo`), matching D-16d and the `[[bin]] name = "echo"` in Cargo.toml. The probe analog uses `"gow-probe"` because that was the bin name there.
2. **`.code(1)`** not `.failure()` alone — explicit code assertion guards against clap's default 2.
3. **`predicate::str::starts_with("echo:")`** — GNU error format guard (D-11).
4. **Negative predicate via `.not()`** — as in probe's `test_path_bare_drive_not_converted` line 96: `predicate::str::contains(r"C:\\").not()`. Use this where a regression must not appear (examples: wc `--version` must not crash; which `GOW_PATHEXT=.EXE foo` literal-first must not return `foo.exe` if `foo` exists).
5. **`tempfile::tempdir()`** for filesystem tests — used by `mkdir`, `rmdir`, `touch`, `tee`, `wc -c <file>`, `which` with isolated PATH.
6. **Snapshot tests via `snapbox`** where hand-written output lists are brittle — per D-30a (e.g., `wc` column-aligned output).

**Match quality:** EXACT — the assert_cmd + predicates structure transfers verbatim; only the binary name, test names, and assertions change.

---

### Template 6: `Cargo.toml` (workspace root — single edit)

**Analog:** existing `D:\workspace\gow-rust\Cargo.toml` (47 lines, lines 1-47).

**Current state (Phase 1):**

```toml
[workspace]
members = ["crates/gow-core", "crates/gow-probe"]
resolver = "3"

[workspace.dependencies]
# ... clap, thiserror, anyhow, termcolor, windows-sys, encoding_rs, path-slash
# assert_cmd, predicates, tempfile
```

**Phase 2 edit — APPEND to `members` and `[workspace.dependencies]`:**

```toml
[workspace]
members = [
    "crates/gow-core",
    "crates/gow-probe",
    # Phase 2 — stateless utilities (D-16)
    "crates/gow-echo",
    "crates/gow-pwd",
    "crates/gow-env",
    "crates/gow-tee",
    "crates/gow-basename",
    "crates/gow-dirname",
    "crates/gow-yes",
    "crates/gow-true",
    "crates/gow-false",
    "crates/gow-mkdir",
    "crates/gow-rmdir",
    "crates/gow-touch",
    "crates/gow-wc",
    "crates/gow-which",
]
resolver = "3"

# ... existing sections unchanged ...

[workspace.dependencies]
# ... existing entries unchanged ...

# Phase 2 additions (D-20a)
snapbox = "1.2"                       # snapshot testing (D-30a)
bstr = "1"                            # byte-safe iteration for wc (D-17)
filetime = "0.2"                      # touch timestamps (Q2); only gow-touch depends
```

**Per-plan delta note:** the plan that first introduces `snapbox` / `bstr` / `filetime` adds them to workspace.deps; subsequent plans reference `{ workspace = true }` only.

**Match quality:** EXACT — append-only edit to an existing file that already follows the `[workspace.dependencies]` inheritance pattern from Plan 01-01.

---

## Per-Utility Pattern Hints (for logic with no repo analog)

These point to RESEARCH.md sections — there is no existing code in the repo to copy from, only stubs in the research doc. Planner should cite RESEARCH.md line ranges in plan `<read_first>` blocks.

| Utility | Unique logic | RESEARCH.md anchor | External reference |
|---------|--------------|---------------------|--------------------|
| `gow-echo` | `-e` escape state machine + `\c` early break | Q9 lines 754-879 | uutils `src/uu/echo` (structure only) |
| `gow-pwd` | `\\?\X:\` prefix strip (preserve `\\?\UNC\...`) | Q8 lines 675-750 | `dunce` crate source (inline 10 lines) |
| `gow-env` | `-S` state-machine splitter + `${VAR}` expansion | Q7 lines 590-672 | uutils `src/uu/env/src/split_iterator.rs` (structure only) |
| `gow-tee` | `SetConsoleCtrlHandler(None, TRUE)` + split-writer stdout+files | Q10 lines 883-959 | windows-sys 0.61 API verified |
| `gow-basename` | MSYS pre-convert + suffix strip | D-26 (CONTEXT) | — |
| `gow-dirname` | MSYS pre-convert + path parent | D-26 (CONTEXT) | — |
| `gow-yes` | 8-64 KiB prefill buffer + `write_all` loop + `BrokenPipe → exit 0` | Q4 lines 305-374 | uutils `src/uu/yes` |
| `gow-true` / `gow-false` | 3-line lib returning 0/1 | D-22 (CONTEXT) | — |
| `gow-mkdir` | `std::fs::create_dir_all` (no custom loop) | Q5 lines 380-425 | std docs |
| `gow-rmdir` | manual parent loop + `ErrorKind::DirectoryNotEmpty` detection | Q5 lines 427-469 | — |
| `gow-touch` | `parse_datetime::parse_datetime_at_date` + `filetime::set_symlink_file_times` | Q1 lines 95-143, Q2 lines 146-205 | uutils `src/uu/touch` Cargo.toml + filetime 0.2.27 source |
| `gow-wc` | `bstr::ByteSlice::chars()` + `char::is_whitespace` word counting | D-17a/b (CONTEXT), Executive Summary line 89 | bstr 1.9+ docs |
| `gow-which` | `std::env::split_paths` + `GOW_PATHEXT` → `PATHEXT` → default hybrid loop | Q6 lines 473-586 | D-18a-e (CONTEXT) |

---

## Integration Points — How New Files Hook into Existing Code

Every utility depends on the Phase 1 public API. Cross-reference table:

| Call site in new utility | Phase 1 source | Plan that established it |
|--------------------------|----------------|--------------------------|
| `gow_core::init()` (first line of `uumain` or `main`) | `crates/gow-core/src/lib.rs:16` | 01-01 |
| `gow_core::args::parse_gnu(cmd, args)` | `crates/gow-core/src/args.rs:35` | 01-02 |
| `gow_core::path::try_convert_msys_path(s)` | `crates/gow-core/src/path.rs:30` | 01-03 |
| `gow_core::path::to_windows_path(s)` (rare — path-slash wrapper) | `crates/gow-core/src/path.rs:67` | 01-03 |
| `gow_core::path::normalize_file_args(&args)` (alternative to per-arg conversion) | `crates/gow-core/src/path.rs:85` | 01-03 |
| `gow_core::error::GowError` + `::io_err(path, source)` | `crates/gow-core/src/error.rs:18, :66` | 01-03 |
| `gow_core::color::{color_choice, stdout}` (only if `--color` flag — likely unused in Phase 2) | `crates/gow-core/src/color.rs:52, :70` | 01-02 |
| `gow_core::fs::*` | `crates/gow-core/src/fs.rs` | 01-03 — **not used by Phase 2** per D-18e |

**Nothing in gow-core needs to be modified for Phase 2.** All integration is via the public API established in Phase 1.

---

## Files with No Analog — Planner Must Use RESEARCH.md Stubs

These utility helper modules have no existing-code analog; the planner and executor rely on RESEARCH.md code stubs as the source of truth:

| File | Content | RESEARCH stub location |
|------|---------|------------------------|
| `crates/gow-echo/src/escape.rs` | `write_escaped` state machine + `parse_octal` + `parse_hex` | Q9 lines 796-855 |
| `crates/gow-env/src/split.rs` | `split(s) -> Result<Vec<String>, _>` state machine | Q7 lines 626-663 |
| `crates/gow-tee/src/signals.rs` | `#[cfg(windows)] ignore_interrupts()` via `SetConsoleCtrlHandler` | Q10 lines 921-948 |
| `crates/gow-touch/src/date.rs` | `parse_touch_date(date_str, reference) -> Result<FileTime, _>` | Q1 lines 118-130 |
| `crates/gow-touch/src/timestamps.rs` | `apply(path, atime, mtime, no_deref)` thin wrapper | Q2 lines 182-192 |
| `crates/gow-which/src/pathext.rs` | `load_pathext() -> Vec<OsString>` | Q6 lines 481-495 |
| `crates/gow-pwd/src/canonical.rs` (or inline) | `simplify_canonical(&Path) -> PathBuf` | Q8 lines 696-712 |
| `crates/gow-yes/src/buffer.rs` (or inline) | `prepare_buffer(input, buffer) -> &[u8]` | Q4 lines 313-324 |

These are all **pure helper modules** — no framework integration, no unsafe boundary except `tee/signals.rs`. Planner should spec them as standalone functions with unit tests in the same file.

---

## Plan-to-Wave Suggestions (advisory for the planner)

Based on pattern-coupling analysis (not binding — planner decides):

- **Wave A (workspace scaffold):** root `Cargo.toml` edit adding 14 members + 3 workspace.deps. Must land first.
- **Wave B (trivial utilities in parallel):** `gow-true`, `gow-false`, `gow-yes`, `gow-echo`, `gow-pwd`, `gow-basename`, `gow-dirname` — all pure/minimal, share Template 1-5 only.
- **Wave C (filesystem trio in parallel):** `gow-mkdir`, `gow-rmdir`, `gow-touch` — all use `std::fs` + MSYS pre-convert; touch has the largest surface.
- **Wave D (complex logic serial OR parallel):** `gow-env`, `gow-tee`, `gow-wc`, `gow-which` — each has a unique helper module. Parallel-safe because no shared files.

Waves B/C/D all consume the same 5 templates; only logic differs.

---

## Metadata

**Analog search scope:** `D:\workspace\gow-rust\crates\gow-probe\**`, `D:\workspace\gow-rust\crates\gow-core\**`, root `Cargo.toml`.
**Files read for pattern extraction:** 8 (probe Cargo.toml, probe build.rs, probe main.rs, probe integration.rs, root Cargo.toml, core lib.rs, core args.rs, core Cargo.toml).
**Symbol scan:** `pub fn` / `pub enum` / `pub struct` across `crates/gow-core/src/*.rs` (all six modules).
**External analog:** uutils/coreutils referenced in RESEARCH.md Q3 / Q4 / Q7 / Q9 for structural guidance only — no code copy.
**Pattern extraction date:** 2026-04-21.

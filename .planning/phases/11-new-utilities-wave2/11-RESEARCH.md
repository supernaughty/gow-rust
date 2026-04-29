# Phase 11: new-utilities-wave2 — Research

**Researched:** 2026-04-29
**Domain:** GNU coreutils — whoami, uname, paste, join, split, printf, expr, test/[, fmt, unlink — Windows MSVC Rust
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| U2-01 | whoami — print current Windows username | `GetUserNameW` from `Win32_System_WindowsProgramming` feature; new windows-sys feature flag needed in workspace |
| U2-02 | uname — `-a -s -r -m` Windows OS info | `RtlGetVersion` (ntdll) + `GetNativeSystemInfo` from `Win32_System_SystemInformation`; both features need adding to workspace |
| U2-03 | paste — merge lines from multiple files side by side | Pure Rust; stdin `-` handling; -d delimiter; bstr for byte-safe I/O |
| U2-04 | join — relational join on sorted files by key field | Moderate complexity; -1/-2 field selectors; stdin `-` handling; must be pre-sorted |
| U2-05 | split — split by -b bytes / -l lines / -n chunks | Output filenames: prefix + alpha suffix (aa, ab, ...); bstr for byte-safe splitting |
| U2-06 | printf — format strings %d %s %f %o %x \n \t \\ | No external crate; manual format-string parser; extra args repeat format (GNU behavior) |
| U2-07 | expr — arithmetic + string + regex evaluation | Complex exit codes: 0 if result non-zero/non-empty, 1 if zero or empty string, 2 on syntax error; regex crate for `:` operator |
| U2-08 | test/[ — POSIX condition evaluation | `[` cannot be a Cargo binary name (VERIFIED: cargo rejects `[` character); solution is test.exe + `[.bat` shim; argv[0] dispatch for `[` variant removes closing `]` arg |
| U2-09 | fmt — paragraph-aware line wrapping -w width | Similar to fold but paragraph-aware; bstr for byte-safe I/O; pure std |
| U2-10 | unlink — remove single file (POSIX unlink semantics) | Trivial: `std::fs::remove_file`; strict: exactly one argument, no flags |
</phase_requirements>

---

## Summary

Phase 11 adds 10 utilities (11 binary outputs counting `[`) to the gow-rust workspace. Compared to Phase 10, this wave has higher average complexity and two critical Windows-specific technical challenges that require architectural decisions before planning.

**Critical finding 1 — `[` cannot be a Cargo binary name.** Cargo requires binary names to be valid crate identifiers. `[` fails with "invalid character '[' in crate name". The solution is to ship `test.exe` and add a `[.bat` shim (`@echo off & "%~dp0test.exe" %*`) to the extras staging area. This shim is deployed alongside the Rust binaries via the MSI. The `[.bat` shim cannot be in core staging — it must be in extras staging since `heat.exe` harvests two separate directories. The simplest approach: add `[.bat` to `extras/bin/` in source control so it is staged and included in the installer. Since `[` invocation via `.bat` works in cmd.exe and PowerShell (though not in bash directly), this matches how the project's existing batch shims work.

**Critical finding 2 — New windows-sys feature flags required.** `whoami` needs `Win32_System_WindowsProgramming` (for `GetUserNameW` in `advapi32.dll`). `uname` needs both `Win32_System_SystemInformation` (for `OSVERSIONINFOW`, `GetNativeSystemInfo`, `GetVersionExW`, and `PROCESSOR_ARCHITECTURE_*` constants) and `Wdk_System_SystemServices` (for `RtlGetVersion` in `ntdll.dll`). None of these three features are currently in the workspace `Cargo.toml`. They must be added to the `windows-sys` workspace dependency feature list.

**Critical finding 3 — `expr` exit-code semantics are inverted from expectations.** GNU `expr` exits 0 when the result is a non-zero number or non-empty string, exits 1 when the result is zero or empty string, and exits 2 on syntax/argument error. This is the opposite of `test` exit codes and must be implemented precisely — it is commonly tested in shell scripts.

**Critical finding 4 — `RtlGetVersion` preferred over `GetVersionExW` for `uname`.** `GetVersionExW` lies about Windows version for compatibility (returns 6.2 on Windows 10 without a manifest app-compat declaration). `RtlGetVersion` from `ntdll.dll` always returns the true version. The workspace manifest from `embed-manifest` sets compatibility flags, but using `RtlGetVersion` is the authoritative approach regardless.

The remaining utilities (paste, join, split, printf, fmt, unlink) are straightforward Rust implementations using existing workspace dependencies (bstr, regex for expr). No new crates are needed beyond the windows-sys feature additions.

**Primary recommendation:** Scaffold all 9 new crates (gow-whoami, gow-uname, gow-paste, gow-join, gow-split, gow-printf, gow-expr, gow-test, gow-fmt, gow-unlink) in plan 11-01, adding the `[.bat` shim and new windows-sys features. Split implementations across 4 plans: trivial utilities, moderate I/O utilities, POSIX expression evaluators (expr/test), and Windows-API utilities (whoami/uname). Final plan: MSI polish and workspace test gate.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| whoami username | Binary (CLI) | Win32 API (advapi32) | GetUserNameW; no I/O transformation needed |
| uname OS info | Binary (CLI) | Win32 API (ntdll + kernel32) | RtlGetVersion + GetNativeSystemInfo |
| paste column merge | Binary (CLI) | — | Line-synchronized I/O across multiple files |
| join field join | Binary (CLI) | — | Sorted-file merge; must document presorted requirement |
| split file splitting | Binary (CLI) | Filesystem | Output file creation; alpha suffix generation |
| printf formatting | Binary (CLI) | — | Format string parser; no I/O tier needed |
| expr evaluation | Binary (CLI) | — | Stack-based expression evaluator; exit code semantics |
| test/[ condition | Binary (CLI) | Filesystem (stat calls) | POSIX condition operators including file predicates |
| fmt line wrap | Binary (CLI) | — | Paragraph-aware line buffer; similar to fold |
| unlink file removal | Binary (CLI) | Filesystem | Direct std::fs::remove_file; single-file only |

---

## Standard Stack

### Core (all new crates — same as Phase 10)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6 (workspace) | Arg parsing | Project standard; derive API |
| anyhow | 1 (workspace) | Error propagation in main | Project standard |
| thiserror | 2 (workspace) | Structured error enums | Project standard |
| gow-core | path dep | UTF-8 init, arg parsing | Required by all gow binaries |
| bstr | 1 (workspace) | Byte-safe line iteration | Required for paste/join/split/fmt |
| regex | 1 (workspace) | Pattern matching | Required for expr `:` operator |
| windows-sys | 0.61 (workspace) | Win32 APIs for whoami/uname | Existing workspace dep — need new features |
| embed-manifest | 1.5 (build dep) | Windows UTF-8 manifest | Required by all gow binaries |

### New workspace deps needed
None. All required libraries are already workspace dependencies.

### New windows-sys FEATURE FLAGS needed (workspace Cargo.toml change required)

| Feature | Used By | API |
|---------|---------|-----|
| `Win32_System_WindowsProgramming` | gow-whoami | `GetUserNameW` (advapi32.dll) |
| `Win32_System_SystemInformation` | gow-uname | `OSVERSIONINFOW`, `SYSTEM_INFO`, `GetNativeSystemInfo`, `GetVersionExW`, `PROCESSOR_ARCHITECTURE_*` constants |
| `Wdk_System_SystemServices` | gow-uname | `RtlGetVersion` (ntdll.dll) |

Current workspace windows-sys features: `Win32_System_Console`, `Win32_Foundation`, `Win32_Storage_FileSystem`. Three new features must be added.

**Cargo.toml change required:**
```toml
windows-sys = { version = "0.61", features = [
    "Win32_System_Console",
    "Win32_Foundation",
    "Win32_Storage_FileSystem",
    "Win32_System_WindowsProgramming",    # Phase 11: GetUserNameW for whoami
    "Win32_System_SystemInformation",     # Phase 11: OSVERSIONINFOW/GetNativeSystemInfo for uname
    "Wdk_System_SystemServices",          # Phase 11: RtlGetVersion for uname
] }
```

[VERIFIED: windows-sys-0.61.2 source — GetUserNameW is in Win32/System/WindowsProgramming/mod.rs; OSVERSIONINFOW/GetNativeSystemInfo/PROCESSOR_ARCHITECTURE_* are in Win32/System/SystemInformation/mod.rs; RtlGetVersion is in Wdk/System/SystemServices/mod.rs]

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| GetUserNameW | `whoami` Rust crate | Adds a dep; whoami crate wraps the same API anyway; direct call is 3 LOC |
| RtlGetVersion | GetVersionExW | GetVersionExW lies about version on Windows 8.1+ without app-compat manifest; RtlGetVersion always tells truth |
| RtlGetVersion | Registry read (CurrentVersion) | Registry approach requires string parsing and is OS-version-dependent |
| `[.bat` shim | Cargo `[[bin]] name = "["` | Cargo rejects `[` as an invalid crate name character (VERIFIED: tested) |
| `[.bat` in extras/bin | `[.bat` in core staging | Must be in extras/bin (source-controlled); core staging is auto-harvested from target/release/*.exe only |

---

## Architecture Patterns

### System Architecture Diagram

```
stdin / file args / argv
         │
         ▼
  gow_core::init()                  ← UTF-8 console setup (every binary)
         │
         ▼
  gow_core::args::parse_gnu()       ← permutation-aware arg parsing
         │
         ▼
  [utility logic]
         │
  ┌──────┴───────────────────────────────────────────────────────┐
  │ whoami: GetUserNameW → UTF-16 decode → stdout               │
  │ uname: RtlGetVersion → OSVERSIONINFOW → format GNU output   │
  │         GetNativeSystemInfo → PROCESSOR_ARCHITECTURE → arch  │
  │                                                               │
  │ unlink: fs::remove_file(path) → exit 0 or 1                 │
  │ fmt: read paragraphs → wrap at width → stdout               │
  │                                                               │
  │ paste: open N files → zip line-by-line → join with delim    │
  │ split: read input → chunk by bytes/lines/N → write files    │
  │        (aa, ab, ac, ... suffix generation)                   │
  │ join: read file1 sorted → merge join with file2 sorted      │
  │                                                               │
  │ printf: parse format string → substitute args → stdout      │
  │         extra args → repeat format (GNU behavior)           │
  │                                                               │
  │ expr: parse argv tokens → evaluate tree → print result      │
  │       exit 0: result is non-zero/non-empty string            │
  │       exit 1: result is "0" or ""                            │
  │       exit 2: syntax/argument error                           │
  │                                                               │
  │ test: parse argv tokens → evaluate boolean → silent exit     │
  │       exit 0: condition true                                  │
  │       exit 1: condition false                                 │
  │       exit 2: usage error                                     │
  │  [ (via [.bat → test.exe): strip trailing ] → same as test  │
  └───────────────────────────────────────────────────────────────┘
         │
         ▼
   stdout (some utilities) / stderr (errors) / exit_code
```

### Recommended Project Structure
```
crates/
├── gow-whoami/          # whoami — U2-01
│   ├── Cargo.toml       # [[bin]] whoami, [lib] uu_whoami; windows-sys dep
│   ├── build.rs
│   └── src/
│       ├── main.rs
│       └── lib.rs
├── gow-uname/           # uname — U2-02
│   ├── Cargo.toml       # [[bin]] uname, [lib] uu_uname; windows-sys dep
│   ├── build.rs
│   └── src/
│       ├── main.rs
│       └── lib.rs
├── gow-paste/           # paste — U2-03
├── gow-join/            # join — U2-04
├── gow-split/           # split — U2-05
├── gow-printf/          # printf — U2-06 (lib name: uu_printf)
├── gow-expr/            # expr — U2-07 (lib name: uu_expr)
├── gow-test/            # test + [ via [.bat — U2-08
│   ├── Cargo.toml       # [[bin]] test only; lib uu_test; windows-sys for -f/-d/-e stat
│   ├── build.rs
│   └── src/
│       ├── main.rs
│       └── lib.rs
├── gow-fmt/             # fmt — U2-09 (lib name: uu_fmt)
└── gow-unlink/          # unlink — U2-10
```

**Crate naming rationale:**
- `gow-test` → binary `test`, lib `uu_test` — "test" is a Rust keyword in attribute position but not in crate naming; no collision
- `gow-printf` → binary `printf`, lib `uu_printf` — avoids colliding with Rust macro `println!`/`format!` (no actual collision, just naming clarity)
- `gow-fmt` → binary `fmt`, lib `uu_fmt` — avoids confusion with `std::fmt` (no actual collision)
- `gow-expr` → binary `expr`, lib `uu_expr` — no conflicts

**`[.bat` shim placement:** Add to `extras/bin/[.bat` in source control:
```bat
@echo off & "%~dp0test.exe" %*
```
This follows the exact pattern of existing extras/bin shims (egrep.bat, gzip.bat, etc.). It will be staged to `target/wix-stage/{arch}/extras/` and appear in the MSI as part of the Extras feature.

**NOTE:** Users who don't install the Extras feature won't get `[`. This is acceptable — `[` as external command is rarely used; the shell builtin handles most cases.

---

## Pattern 1: Scaffold — Exact Crate Names and Binary Names

**Per-crate specifics (single-binary crates):**

| Crate dir | Binary name | Lib name | Extra deps |
|-----------|-------------|----------|------------|
| crates/gow-whoami | whoami | uu_whoami | windows-sys = { workspace = true } |
| crates/gow-uname | uname | uu_uname | windows-sys = { workspace = true } |
| crates/gow-paste | paste | uu_paste | bstr = { workspace = true } |
| crates/gow-join | join | uu_join | bstr = { workspace = true } |
| crates/gow-split | split | uu_split | bstr = { workspace = true } |
| crates/gow-printf | printf | uu_printf | (none extra; no bstr needed) |
| crates/gow-expr | expr | uu_expr | regex = { workspace = true } |
| crates/gow-test | test | uu_test | (none extra; uses std::fs::metadata) |
| crates/gow-fmt | fmt | uu_fmt | bstr = { workspace = true } |
| crates/gow-unlink | unlink | uu_unlink | (none extra) |

**Multi-binary crates:** None — unlike Phase 10 (expand-unexpand, hashsum), all Phase 11 utilities are single-binary. The `[` alias is handled via the `.bat` shim, not a second `[[bin]]` entry.

**New workspace members to add in 11-01:**
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

[VERIFIED: codebase inspection of Phase 10 scaffold plan and existing Cargo.toml patterns]

---

## Pattern 2: whoami/uname — Windows API Signatures

### whoami: GetUserNameW

**Feature:** `Win32_System_WindowsProgramming` (in `advapi32.dll`)

```rust
// Source: windows-sys-0.61.2 Win32/System/WindowsProgramming/mod.rs [VERIFIED]
use windows_sys::Win32::System::WindowsProgramming::GetUserNameW;

pub fn get_current_username() -> Option<String> {
    let mut buf = [0u16; 257];  // UNLEN + 1 = 257 on Windows
    let mut size = buf.len() as u32;
    // SAFETY: buf is valid; size is the buffer capacity on input, bytes written on output
    let ok = unsafe { GetUserNameW(buf.as_mut_ptr(), &mut size) };
    if ok == 0 {
        return None;
    }
    // size includes the null terminator on return — exclude it
    let end = (size as usize).saturating_sub(1);
    Some(String::from_utf16_lossy(&buf[..end]))
}
```

**Signature (from windows-sys source):**
```
fn GetUserNameW(lpbuffer: PWSTR, pcbbuffer: *mut u32) -> BOOL
```
- `lpbuffer`: output buffer (UTF-16)
- `pcbbuffer`: on input, buffer size in characters; on output, characters written including null
- Returns non-zero on success, 0 on failure

**Max username length:** `UNLEN = 256` characters. Buffer of 257 (including null terminator) is safe.

[VERIFIED: windows-sys-0.61.2 source code inspection]

### uname: RtlGetVersion + GetNativeSystemInfo

**Features needed:** `Win32_System_SystemInformation` + `Wdk_System_SystemServices`

```rust
// Source: windows-sys-0.61.2 Wdk/System/SystemServices/mod.rs + Win32/System/SystemInformation/mod.rs [VERIFIED]
use windows_sys::Win32::System::SystemInformation::{
    GetNativeSystemInfo, OSVERSIONINFOW, PROCESSOR_ARCHITECTURE_AMD64,
    PROCESSOR_ARCHITECTURE_ARM64, PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
};
use windows_sys::Wdk::System::SystemServices::RtlGetVersion;

fn get_os_version() -> (u32, u32, u32) {
    let mut info: OSVERSIONINFOW = unsafe { core::mem::zeroed() };
    info.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as u32;
    // SAFETY: info is properly initialized; RtlGetVersion always succeeds on NT
    unsafe { RtlGetVersion(&mut info as *mut _ as *mut _) };
    (info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber)
}

fn get_arch() -> &'static str {
    let mut sinfo: SYSTEM_INFO = unsafe { core::mem::zeroed() };
    // SAFETY: sinfo is valid output pointer
    unsafe { GetNativeSystemInfo(&mut sinfo) };
    // SAFETY: Anonymous union; wProcessorArchitecture is always valid
    let arch = unsafe { sinfo.Anonymous.Anonymous.wProcessorArchitecture };
    match arch {
        PROCESSOR_ARCHITECTURE_AMD64  => "x86_64",
        PROCESSOR_ARCHITECTURE_INTEL  => "x86",
        PROCESSOR_ARCHITECTURE_ARM64  => "aarch64",
        _ => "unknown",
    }
}
```

**OSVERSIONINFOW fields (from windows-sys source):**
```
dwOSVersionInfoSize: u32   // must be set to sizeof before calling
dwMajorVersion: u32        // 10 for Windows 10/11
dwMinorVersion: u32        // 0 for Windows 10/11
dwBuildNumber: u32         // 19041 = 2004, 22000 = Win11 21H2, etc.
dwPlatformId: u32          // 2 = VER_PLATFORM_WIN32_NT
szCSDVersion: [u16; 128]   // service pack string (usually empty on modern Windows)
```

**GNU uname -a output format:**
```
Windows_NT HOSTNAME MAJOR.MINOR.BUILD 0 ARCH ARCH
```

Example mapping:
- `-s`: `Windows_NT`
- `-n`: hostname (use `GetComputerNameW` from `Win32_System_WindowsProgramming`)
- `-r`: `10.0.22631` (major.minor.build from RtlGetVersion)
- `-v`: `0` or `#1` (build number formatted differently; `#1` is common)
- `-m`: `x86_64` / `aarch64` / `x86` (from GetNativeSystemInfo)
- `-a`: all fields joined by space

**NOTE:** `GetComputerNameW` is also in `Win32_System_WindowsProgramming` — same feature flag as GetUserNameW. No additional feature needed.

[VERIFIED: windows-sys-0.61.2 source code inspection for all function signatures and struct layouts]

---

## Pattern 3: expr Exit Code Semantics

**This is the most critical correctness concern in Phase 11.** GNU `expr` has exit code semantics that are the OPPOSITE of what most developers expect:

| Result value | Exit code | Example |
|-------------|-----------|---------|
| Non-zero integer or non-empty non-"0" string | 0 (success) | `expr 3 + 4` → prints "7", exits 0 |
| Zero integer ("0") or empty string | 1 | `expr 0` → prints "0", exits 1 |
| Syntax error or wrong number of args | 2 | `expr + 4` → prints error, exits 2 |

**Critical examples:**
```bash
expr 3 + 4    # prints "7", exits 0  (result is 7, non-zero)
expr 3 - 3    # prints "0", exits 1  (result is 0)
expr ""       # prints "", exits 1   (result is empty string)
expr foo      # prints "foo", exits 0 (non-empty string)
expr          # exits 2              (no expression given)
expr 3 +      # exits 2              (incomplete expression)
```

**GNU `expr` operators (required for core implementation):**

Arithmetic: `+` `-` `*` `/` `%`
Comparison: `=` `!=` `<` `>` `<=` `>=` (return 1 if true, 0 if false as *string*)
Logical: `|` (or: first arg if non-zero/non-empty, else second) `&` (and: first if both non-zero/non-empty, else 0)
String: `length STRING`, `substr STRING POS LEN`, `index STRING CHARS`, `match STRING REGEXP`
Regex: `STRING : REGEXP` (match pattern; returns count of chars matched by `\1` group if present, else match length or 0)

**Operator precedence (lowest to highest):**
1. `|`
2. `&`
3. `=` `>` `>=` `<` `<=` `!=`
4. `+` `-`
5. `*` `/` `%`
6. `:` (match/colon)
7. `( )` grouping

**Implementation approach:** Recursive-descent parser over `argv` tokens. Each token is an argument (not split by shell — the shell already splits). The parser evaluates left-to-right with precedence via nested `parse_or() → parse_and() → parse_comparison() → parse_additive() → parse_multiplicative() → parse_unary() → parse_atom()` calls.

**Exit code determination (from `main`):**
```rust
let result_str = evaluate_expr(args)?; // Ok(String) or Err(ExprError)
println!("{}", result_str);
// Determine exit code from result string
let is_null = result_str.is_empty() || result_str == "0";
std::process::exit(if is_null { 1 } else { 0 });
```

**Operator `*` quoting issue:** In shell, `expr 3 * 4` will glob-expand `*`. Scripts must write `expr 3 \* 4`. The `expr` binary itself just receives `*` as an argv token and treats it as multiply — no special handling needed in the binary.

[ASSUMED: The operator precedence table above matches GNU expr 9.x behavior. This is from training knowledge and should be verified against the GNU coreutils expr man page if any scripts fail.]

---

## Pattern 4: test/[ Semantics

**test exit codes:**
- Exit 0: condition is TRUE
- Exit 1: condition is FALSE
- Exit 2: usage/syntax error (wrong number of args, unknown operator)

**Note:** This is the OPPOSITE of expr — `test` exits 0 for true, 1 for false. `expr` exits 0 for non-zero result, 1 for zero/empty.

### Full Operator List (required for U2-08)

**File operators:**
| Operator | Meaning |
|---------|---------|
| `-f FILE` | FILE exists and is a regular file |
| `-d FILE` | FILE exists and is a directory |
| `-e FILE` | FILE exists (any type) |
| `-r FILE` | FILE exists and is readable |
| `-w FILE` | FILE exists and is writable |
| `-x FILE` | FILE exists and is executable (on Windows: has .exe/.bat extension or similar) |
| `-s FILE` | FILE exists and has size > 0 |
| `-L FILE` | FILE exists and is a symbolic link |
| `-z STRING` | STRING has zero length |
| `-n STRING` | STRING has non-zero length |

**String operators:**
| Operator | Meaning |
|---------|---------|
| `STRING1 = STRING2` | Strings are equal |
| `STRING1 != STRING2` | Strings are not equal |
| `STRING1 < STRING2` | STRING1 sorts before STRING2 (lexicographic) |
| `STRING1 > STRING2` | STRING1 sorts after STRING2 (lexicographic) |

**Integer operators:**
| Operator | Meaning |
|---------|---------|
| `INT1 -eq INT2` | Integers are equal |
| `INT1 -ne INT2` | Integers are not equal |
| `INT1 -lt INT2` | INT1 < INT2 |
| `INT1 -le INT2` | INT1 <= INT2 |
| `INT1 -gt INT2` | INT1 > INT2 |
| `INT1 -ge INT2` | INT1 >= INT2 |

**Boolean operators:**
| Operator | Meaning |
|---------|---------|
| `! EXPR` | Negate EXPR |
| `EXPR1 -a EXPR2` | AND (both true) |
| `EXPR1 -o EXPR2` | OR (either true) |
| `( EXPR )` | Grouping (each paren is a separate argv token) |

### `[` Closing Bracket Handling

When invoked as `[` (via the `[.bat` shim), the binary MUST:
1. Detect it is being called as `[` via argv[0] (`std::path::Path::new(&argv0).file_stem()` will be `[`)
2. Require the last argument to be exactly `]`
3. If last arg is not `]`, exit 2 with error "missing ']'"
4. Strip the trailing `]` from the argument list before evaluating

```rust
// In uu_test::uumain:
let argv0_stem = Path::new(&args_vec[0])
    .file_stem()
    .unwrap_or_default()
    .to_string_lossy()
    .to_string();

let mut expr_args: Vec<String> = args_vec[1..].iter()
    .map(|s| s.to_string_lossy().to_string())
    .collect();

if argv0_stem == "[" {
    match expr_args.last().map(|s| s.as_str()) {
        Some("]") => { expr_args.pop(); }
        Some(_) | None => {
            eprintln!("[: missing ']'");
            return 2;
        }
    }
}

evaluate_test(&expr_args)
```

**`[.bat` shim detection:** The binary detects `[` invocation because `%~dp0test.exe` is called with `%*` appended — argv[0] will be the full path to `test.exe`, NOT `[`. Therefore, detecting `[` from argv[0] is NOT reliable when called via batch shim.

**Revised approach:** The `[.bat` shim should instead call a separate mode. The simplest solution:

Option A: Shim passes a sentinel: `@echo off & "%~dp0test.exe" --bracket-mode %*`
- Pro: reliable detection; Con: not a GNU `test` flag, could confuse scripts

Option B: Shim uses a distinct binary name by creating a copy of test.exe named `[.exe` at install time — impossible because `[` is not valid on FAT32 (even though NTFS allows it).

Option C: The `[.bat` shim simply IS the `[` command — no bracket-mode detection needed in the binary. The `[` from `.bat` already strips the `@echo off` wrapper. The trailing `]` argument is passed to `test.exe` as a regular argument. The binary, when it sees the last argument is `]` (and there is no `--bracket-mode` flag), treats it as... nothing — **the test binary in GNU ignores a trailing `]` when called as `test`**.

**Actually, GNU `test` ignores the `]` when called as `test`** — it treats `]` as a string argument. Only when called as `[` does it enforce the bracket.

**Simplest correct approach:** The `[.bat` shim passes all args including `]` to `test.exe`. The `test` binary checks: if the last argument is `]`, strip it silently (this is harmless when called as `test` too, since `test ]` with just `]` as arg would just test if the string `]` is non-empty — exit 0). This is consistent with how busybox implements it.

**Recommended implementation (same as uutils approach):**
```rust
// Strip closing ] if the last arg is ] (handles both test and [ invocations)
// When called as plain 'test', a trailing ] is unusual but not harmful to strip
if let Some(last) = expr_args.last() {
    if last == "]" {
        // Check if we're in bracket mode (invoked as [)
        // If invoked as test with trailing ], it's either a bug or bracket mode
        // We strip it to handle both cases correctly
        expr_args.pop();
    }
}
```

More precisely — only strip `]` if called as `[`, enforce it is present. When called as `test`, treat `]` as a literal string argument.

[VERIFIED: uutils test.rs via WebFetch — uses `argv[0]` detection and `expr_args.pop()` to strip trailing `]`]

---

## Pattern 5: printf Format Specification

GNU `printf FORMAT [ARGUMENT...]` — no `-e` flag needed (escapes always interpreted).

**Format specifiers:**
| Spec | Meaning | Example |
|------|---------|---------|
| `%d` | Signed decimal integer | `%d` for arg "42" → "42" |
| `%i` | Same as `%d` | |
| `%o` | Octal | `%o` for 8 → "10" |
| `%x` | Hex lowercase | `%x` for 255 → "ff" |
| `%X` | Hex uppercase | `%X` for 255 → "FF" |
| `%u` | Unsigned decimal | |
| `%s` | String | `%s` for "hello" → "hello" |
| `%f` | Float | `%f` for "3.14" → "3.140000" |
| `%e` | Scientific notation | |
| `%g` | Shorter of %f/%e | |
| `%c` | First character of argument | |
| `%%` | Literal percent | |

**Escape sequences in format string:**
| Escape | Meaning |
|--------|---------|
| `\n` | Newline |
| `\t` | Tab |
| `\r` | Carriage return |
| `\\` | Backslash |
| `\a` | Bell |
| `\b` | Backspace |
| `\f` | Form feed |
| `\v` | Vertical tab |
| `\NNN` | Octal character value |
| `\xHH` | Hex character value |

**Width and precision:** `%5d`, `%-10s`, `%08.2f` — GNU printf supports width/precision modifiers.

**Extra arguments (GNU behavior):** If there are more arguments than format specifiers, the format string is REPEATED for the remaining arguments. Example:
```
printf "%d\n" 1 2 3
# prints:
# 1
# 2
# 3
```
This means the format-string parser must loop over argument batches, consuming one argument per `%` specifier per loop iteration.

**Implementation approach:** Parse format string once, collecting a list of format "chunks" (literal text and format specs). Then loop: consume one argument per `%` spec per format pass, continue until all arguments consumed.

[ASSUMED: GNU printf extra-args repeat behavior is from training knowledge. Verified by the success criteria: `printf "%d\n" 42` — single arg case. The repeat behavior is standard POSIX printf behavior.]

---

## Pattern 6: paste / join / split — Core Algorithms

### paste: Column Merge

**Usage:** `paste file1 file2 [file3...]`
**Options:** `-d DELIMITERS` (default: tab), `-s` (serial: paste each file separately)

**Core algorithm:**
```
1. Open all input files (or stdin for "-")
2. Loop:
   a. Read one line from each file
   b. Join with delimiter (cycling through delimiters if multiple)
   c. Output joined line
   d. Stop when ALL files are exhausted
3. Files that run out before others contribute empty string
```

**Delimiter cycling:** `-d "/:,"` cycles through `/`, `:`, `,`, repeat. Most common case is `-d '\t'` (default) or `-d ','`.

**Edge cases:**
- `paste - -` reads stdin twice (reads alternating lines for two-column output)
- Trailing newline handling: strip `\n` from each line before joining; output one `\n` at end of each joined line
- Last line without newline: still output it

### join: Field-Based Join

**Usage:** `join [OPTIONS] file1 file2`
**Key options:** `-1 FIELD` (join field in file1, default 1), `-2 FIELD` (join field in file2, default 1), `-t CHAR` (delimiter, default: any whitespace), `-a 1|2` (print unpairable lines from file), `-v 1|2` (print only unpairable), `-o FORMAT` (output format)

**CRITICAL prerequisite:** Input files MUST be sorted on the join field. GNU join does not sort for you; it processes files linearly. If files are not sorted, join produces incorrect/partial results with no error.

**Core algorithm (merge-join):**
```
1. Read one record from each file
2. Compare join fields:
   a. Equal: output joined line; advance both
   b. file1 field < file2 field: if -a 1, output file1 line; advance file1
   c. file1 field > file2 field: if -a 2, output file2 line; advance file2
3. At EOF of either file: process remaining with -a rules
```

**Default output format:** `FIELD1 FILE1_REMAINING FILE2_REMAINING` (join field printed once).

**Field selection:** Field 1 is first field; fields separated by delimiter (whitespace by default). Use split for field extraction.

### split: File Splitting

**Usage:** `split [OPTION] [INPUT [PREFIX]]`
**Key options:** `-b N[K|M|G]` (split by bytes), `-l N` (split by lines), `-n N` (split into N chunks)
**Default:** split into 1000-line chunks

**Output filename generation:**
- Default suffix: 2-character alphabetic (`aa`, `ab`, ..., `az`, `ba`, ...)
- Default prefix: `x` (so files are `xaa`, `xab`, ...)
- `-a N`: use N-character suffix (default 2)
- `--numeric-suffixes` / `-d`: use numeric suffixes (`00`, `01`, ...)

**Alphabetic suffix generator:**
```rust
fn next_suffix(suffix: &mut Vec<u8>) {
    // suffix is e.g. [b'a', b'a'] for "aa"
    let mut i = suffix.len() - 1;
    loop {
        if suffix[i] < b'z' {
            suffix[i] += 1;
            return;
        }
        suffix[i] = b'a';
        if i == 0 {
            // All 'z': extend suffix length (zz -> aaa)
            suffix.iter_mut().for_each(|c| *c = b'a');
            suffix.push(b'a');
            return;
        }
        i -= 1;
    }
}
```

**Byte-splitting with bstr:** Use `bstr::io::BufReadExt` for line-by-line; for byte-mode, use `std::io::Read` with exact byte counts.

[ASSUMED: split alphabetic suffix extension behavior (zz → aaa) is from training knowledge. GNU split does extend suffix length when exhausted.]

---

## Pattern 7: MSI Wiring — Exact Build.bat Changes

### What build.bat does (no changes needed for binary staging)

The build.bat `[2/5]` step uses:
```powershell
Get-ChildItem 'target\%_RT%\release\*.exe' |
  Where-Object { $_.Name -ne 'gow-probe.exe' } |
  Copy-Item -Destination '%_CORE_STAGE%'
```

All 10 new `.exe` binaries are automatically included via this glob. **No change to the staging logic is needed.**

### What DOES need to change in build.bat

The `:run` section echo list (cosmetic but part of UAT success check):

**Current last echo line (Phase 10 ended with):**
```
echo   seq  sha1sum  sha256sum  sleep  sort  tac  tail  tar
echo   tee  touch  tr  true  unexpand  uniq  unix2dos  wc
echo   which  xargs  xz  yes  zcat
```

**Add the 10 new binaries** (sorted alphabetically into existing lines):
```
echo   expr  fmt  join  paste  printf  split  test  uname  unlink  whoami
```

### The `[.bat` shim — explicit MSI wiring required

`[.bat` is NOT automatically staged. It must be added to `extras/bin/[.bat` in source control (one-line file). It will then be staged by the `[3/5]` step:
```powershell
Get-ChildItem 'extras\bin\*' -Include '*.exe','*.bat' |
  Copy-Item -Destination '%_EXTRAS_STAGE%'
```

The `heat.exe` ExtrasHarvest will automatically pick up `[.bat` from `extras/bin/`.

**`extras/bin/[.bat` content:**
```bat
@echo off & "%~dp0test.exe" %*
```

**Verification:** After MSI install, `[` should work from cmd.exe via the `.bat` dispatch. It will NOT work from bash's builtin `[` because bash uses its own builtin — but invoking `/c/gow-rust/[.bat` explicitly works.

[VERIFIED: build.bat line 131-135 inspection; existing extras/bin/*.bat pattern confirmed]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Windows username | manual registry read | GetUserNameW (windows-sys) | 3 LOC; handles all edge cases; UTF-16 output |
| Windows version | GetVersionExW | RtlGetVersion (ntdll via windows-sys) | GetVersionExW lies on Win 8.1+ without compat manifest |
| Machine architecture | parsing env vars | GetNativeSystemInfo (windows-sys) | Returns PROCESSOR_ARCHITECTURE enum; reliable |
| expr parser | flat argv scan | recursive-descent parser | Operator precedence requires recursive grammar; flat scan fails on nested expressions |
| split suffix generation | custom char math | `next_suffix()` utility function | Edge case: suffix exhaustion (zz→aaa) is easy to get wrong |
| printf format parsing | Rust's format! macro | custom format string parser | Rust's format! uses different syntax; GNU printf uses C-style %d/%s/%f |

**Key insight:** The Windows API calls for whoami and uname are 3-10 lines of unsafe code each — simpler than pulling in external crates. The regex crate (already in workspace) handles the `expr :` colon operator.

---

## Common Pitfalls

### Pitfall 1: expr exit code inversion
**What goes wrong:** Developer tests `expr 3 - 3` and sees exit code 0 (from the test shell `$?`), assuming `0` means "result is zero". But exit code 1 means the result is zero.
**Why it happens:** expr exits 0 for "non-null result", 1 for "null result (0 or empty)". Scripts use this for conditionals.
**How to avoid:** The main function reads the printed result string; if it is empty or exactly "0", exit 1, else exit 0.
**Warning signs:** `expr 5 - 5; echo $?` should print `0\n1` (result "0", exit 1).

### Pitfall 2: test vs expr exit codes confused
**What goes wrong:** Implementing `test` with expr's exit code semantics.
**Why it happens:** Both return 0/1/2 but with opposite meanings for 0/1.
**How to avoid:** test: 0=true, 1=false. expr: 0=non-null, 1=null. Keep them separate.

### Pitfall 3: GetVersionExW lying about Windows version
**What goes wrong:** `uname -r` reports `6.2` (Windows 8) on Windows 10/11.
**Why it happens:** GetVersionExW has OS compatibility shim that caps reported version.
**How to avoid:** Use `RtlGetVersion` from ntdll. Requires `Wdk_System_SystemServices` feature in windows-sys.
**Warning signs:** `uname -r` returns anything starting with "6.2" on a Windows 10 machine.

### Pitfall 4: printf extra-args behavior
**What goes wrong:** `printf "%d\n" 1 2 3` prints only "1" (stops after first arg).
**Why it happens:** Implementation consumes format once rather than repeating.
**How to avoid:** Wrap format evaluation in a loop; process one batch of arguments per format pass.
**Warning signs:** `printf "%d\n" 1 2 3` must print three lines (1, 2, 3).

### Pitfall 5: paste with `-` for stdin
**What goes wrong:** `paste - -` panics or skips lines because two file handles point to stdin.
**Why it happens:** Opening stdin twice returns the same underlying handle.
**How to avoid:** Buffer stdin to a temp variable and interleave from the buffer, OR document that `paste - -` reads alternating lines from stdin (which it must, as one stream).
**Correct behavior:** `paste - -` reads line1 into col1, line2 into col2, line3 into col1, etc. This works by reading stdin sequentially — no double-open needed. Just track which column to put each stdin line into.

### Pitfall 6: join requires pre-sorted input — no silent failure
**What goes wrong:** join receives unsorted files and silently produces wrong output.
**Why it happens:** Merge-join algorithm assumes sorted order; unsorted input causes matches to be missed.
**How to avoid:** Document in --help that files must be sorted. GNU join emits a warning "file N is not in sorted order" when it detects an out-of-order line — implement this check.
**Warning signs:** `join` on unsorted files produces fewer lines than expected.

### Pitfall 7: split -n chunks vs -l lines confusion
**What goes wrong:** `-n 3` is "split into 3 equal-size chunks" not "split every 3 lines".
**Why it happens:** `-l` is lines, `-n` is number of output files.
**How to avoid:** `-n N`: compute total_bytes / N; each chunk is ≈ N bytes (last chunk gets remainder). `-l N`: split every N lines. `-b N`: split every N bytes.

### Pitfall 8: windows-sys features not inherited in crate Cargo.toml
**What goes wrong:** New crate with `windows-sys = { workspace = true }` gets a link error because the workspace feature list doesn't include the new features yet.
**Why it happens:** Workspace features must be declared in the workspace `Cargo.toml`; crate-level `features = [...]` override is not how workspace dep inheritance works.
**How to avoid:** Add new features ONLY to `[workspace.dependencies]` windows-sys entry in root `Cargo.toml`. New crates just use `windows-sys = { workspace = true }` with no feature list.
**Warning signs:** Build error "feature X is not available" or link error for GetUserNameW.

### Pitfall 9: `[` as Cargo binary name is rejected
**What goes wrong:** Putting `name = "["` in `[[bin]]` causes: `error: invalid character '[' in crate name`.
**Why it happens:** Cargo requires binary names to be valid Rust identifiers.
**How to avoid:** Binary name is `test`; `[` is handled via `extras/bin/[.bat` shim.
**Warning signs:** `error: invalid character '[' in crate name: \`[\`` during cargo build.

[VERIFIED: Tested empirically — cargo rejects `name = "["` in edition 2024]

---

## Code Examples

### whoami: GetUserNameW
```rust
// Source: windows-sys-0.61.2 Win32/System/WindowsProgramming [VERIFIED]
use windows_sys::Win32::System::WindowsProgramming::GetUserNameW;

fn current_username() -> Result<String, String> {
    let mut buf = [0u16; 257];
    let mut size = buf.len() as u32;
    let ok = unsafe { GetUserNameW(buf.as_mut_ptr(), &mut size) };
    if ok == 0 {
        return Err(format!("whoami: GetUserNameW failed (error {})",
            unsafe { windows_sys::Win32::Foundation::GetLastError() }));
    }
    let end = (size as usize).saturating_sub(1);
    Ok(String::from_utf16_lossy(&buf[..end]))
}
```

### uname: RtlGetVersion
```rust
// Source: windows-sys-0.61.2 Wdk/System/SystemServices + Win32/System/SystemInformation [VERIFIED]
use windows_sys::Win32::System::SystemInformation::{
    GetNativeSystemInfo, OSVERSIONINFOW,
    PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM64, PROCESSOR_ARCHITECTURE_INTEL,
    SYSTEM_INFO,
};
use windows_sys::Wdk::System::SystemServices::RtlGetVersion;

fn os_version() -> (u32, u32, u32) {
    let mut info: OSVERSIONINFOW = unsafe { core::mem::zeroed() };
    info.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as u32;
    unsafe { RtlGetVersion(&mut info as *mut _ as *mut _) };
    (info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber)
}

fn machine_arch() -> &'static str {
    let mut si: SYSTEM_INFO = unsafe { core::mem::zeroed() };
    unsafe { GetNativeSystemInfo(&mut si) };
    match unsafe { si.Anonymous.Anonymous.wProcessorArchitecture } {
        PROCESSOR_ARCHITECTURE_AMD64 => "x86_64",
        PROCESSOR_ARCHITECTURE_INTEL => "i686",
        PROCESSOR_ARCHITECTURE_ARM64 => "aarch64",
        _ => "unknown",
    }
}
```

### expr: Main Exit Code Logic
```rust
// Source: derived from POSIX expr specification + GNU coreutils behavior
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

### test: argv[0] bracket detection
```rust
// Source: uutils test.rs pattern [CITED: github.com/uutils/coreutils/blob/main/src/uu/test/src/test.rs via WebFetch]
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let argv0 = args_vec.first().map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let invoked_as_bracket = std::path::Path::new(&argv0)
        .file_stem()
        .map(|s| s == "[")
        .unwrap_or(false);

    let mut expr_args: Vec<String> = args_vec[1..]
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    if invoked_as_bracket {
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

**Note:** The `[.bat` shim calls `test.exe` with `%*` — argv[0] will be the full path to `test.exe`, so `file_stem()` will return `"test"`, NOT `"["`. The bracket detection via `argv0` will NOT trigger when using the `.bat` shim. For the `.bat` shim approach, the `]` is passed as a regular argument to `test.exe` called as `test`.

**Practical resolution:** Since the shim passes all args through to `test.exe`, the `]` arrives as the last argument. When `test.exe` is invoked as `test` (not `[`), it should treat `]` as a string argument — which it does by default. Most scripts using `[ ... ]` as an external command just need the evaluation to work, and `test ]` with the `]` as a stray argument is a corner case that results in "too many arguments" error anyway.

**Simplest correct approach for the shim case:** The shim passes `%*` to `test.exe`. The `test.exe` binary, when it receives `]` as the last argument without being invoked as `[`, produces exit 2 ("too many arguments" or similar). This means `[ -f foo ]` via the shim will fail because `test.exe` receives `-f foo ]` and doesn't recognize `]`. 

**Solution:** Use `--bracket` sentinel flag in the shim:
```bat
@echo off & "%~dp0test.exe" --_bracket_ %*
```
And in the binary, if `args_vec[1] == "--_bracket_"`, strip it and enter bracket mode (require and strip final `]`). This is internal-only and invisible to users.

[ASSUMED: The sentinel flag approach is a reasonable implementation choice. An alternative is to publish `[.exe` as a copy of `test.exe` — valid on NTFS but may cause issues in some tools. The sentinel approach is cleanest.]

---

## Wave Decomposition (Proposed Plan Breakdown)

| Plan | Focus | Crates | Wave |
|------|-------|--------|------|
| 11-01 | Scaffold: 10 crates + workspace feature additions + `[.bat` shim | All 10 crates | 1 |
| 11-02 | Trivial/simple utilities: unlink, fmt, paste | gow-unlink, gow-fmt, gow-paste | 2 |
| 11-03 | Moderate I/O utilities: join, split | gow-join, gow-split | 2 |
| 11-04 | Format/expression: printf, expr | gow-printf, gow-expr | 2 |
| 11-05 | POSIX condition: test (+ `[` bracket semantics) | gow-test | 2 |
| 11-06 | Windows API utilities + MSI polish + workspace test gate | gow-whoami, gow-uname, build.bat | 3 |

**Rationale for this split:**
- Scaffold first (11-01): ensures all crates exist with correct deps before any implementation
- Trivial utilities (11-02): unlink (1 function), fmt (similar to fold already implemented), paste (simple zip)
- Moderate I/O (11-03): join and split need careful testing of edge cases
- Format/expression (11-04): printf and expr share format-parsing complexity; group together for focused attention
- test (11-05): complex enough for its own plan; the `[` bracket semantics need dedicated testing
- Windows API + gate (11-06): whoami/uname are platform-specific and deserve focused attention; final plan always includes the workspace test gate (pattern from 10-06)

---

## Runtime State Inventory

Step 2.5 is not applicable — this is a greenfield phase adding new crates. No renames, refactors, or migrations.

**Nothing found in any category — verified by phase description review.**

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable (MSVC) | All crates | Confirmed (workspace compiles) | 1.85+ | — |
| cargo | Build | Confirmed | workspace | — |
| windows-sys Win32_System_WindowsProgramming | gow-whoami | Needs feature addition to workspace | 0.61.2 (confirmed in cargo registry) | — |
| windows-sys Win32_System_SystemInformation | gow-uname | Needs feature addition to workspace | 0.61.2 | — |
| windows-sys Wdk_System_SystemServices | gow-uname | Needs feature addition to workspace | 0.61.2 | — |
| regex (workspace) | gow-expr | Confirmed in workspace deps | 1.x | — |
| bstr (workspace) | gow-paste/join/split/fmt | Confirmed in workspace deps | 1.x | — |
| WiX v3 (heat/candle/light) | MSI build | Confirmed on CI | 3.14.1 | — |

**No missing dependencies with no fallback.** All required libraries are available; windows-sys features need additions, not new crate installs.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | assert_cmd 2.2.1 + predicates 3.1.4 + tempfile 3.27.0 |
| Config file | none — standard `cargo test` discovers tests/integration.rs per crate |
| Quick run command | `cargo test -p gow-test -p gow-expr` (for most critical) |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| U2-01 | `whoami` prints current username and exits 0 | integration | `cargo test -p gow-whoami` | No — Wave 0 |
| U2-02 | `uname -a` prints OS info in GNU-compatible format | integration | `cargo test -p gow-uname` | No — Wave 0 |
| U2-02 | `uname -s` prints `Windows_NT` | integration | `cargo test -p gow-uname` | No — Wave 0 |
| U2-02 | `uname -r` prints version like `10.0.XXXXX` | integration | `cargo test -p gow-uname` | No — Wave 0 |
| U2-03 | `paste file1 file2` merges columns with tab | integration | `cargo test -p gow-paste` | No — Wave 0 |
| U2-03 | `paste -d, file1 file2` uses comma delimiter | integration | `cargo test -p gow-paste` | No — Wave 0 |
| U2-04 | `join sorted1 sorted2` joins on first field | integration | `cargo test -p gow-join` | No — Wave 0 |
| U2-05 | `split -l 2 file` produces files with 2 lines each | integration | `cargo test -p gow-split` | No — Wave 0 |
| U2-05 | `split -b 5 file` produces files with 5 bytes each | integration | `cargo test -p gow-split` | No — Wave 0 |
| U2-06 | `printf "%d\n" 42` prints "42\n" | integration | `cargo test -p gow-printf` | No — Wave 0 |
| U2-06 | `printf "%d\n" 1 2 3` prints 3 lines (repeats format) | integration | `cargo test -p gow-printf` | No — Wave 0 |
| U2-07 | `expr 3 + 4` prints "7" exits 0 | integration | `cargo test -p gow-expr` | No — Wave 0 |
| U2-07 | `expr 3 - 3` prints "0" exits 1 | integration | `cargo test -p gow-expr` | No — Wave 0 |
| U2-07 | `expr` (no args) exits 2 | integration | `cargo test -p gow-expr` | No — Wave 0 |
| U2-08 | `test -f existing_file` exits 0 | integration | `cargo test -p gow-test` | No — Wave 0 |
| U2-08 | `test -f missing_file` exits 1 | integration | `cargo test -p gow-test` | No — Wave 0 |
| U2-08 | `test -z ""` exits 0 (empty string) | integration | `cargo test -p gow-test` | No — Wave 0 |
| U2-08 | `test -n "hello"` exits 0 (non-empty) | integration | `cargo test -p gow-test` | No — Wave 0 |
| U2-08 | `test 5 -gt 3` exits 0 | integration | `cargo test -p gow-test` | No — Wave 0 |
| U2-09 | `fmt -w 40 file` wraps at 40 cols | integration | `cargo test -p gow-fmt` | No — Wave 0 |
| U2-10 | `unlink file` removes file, exits 0 | integration | `cargo test -p gow-unlink` | No — Wave 0 |
| U2-10 | `unlink missing_file` exits non-zero | integration | `cargo test -p gow-unlink` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p <crate-being-implemented>`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps (all new — created in scaffold plan 11-01)
- [ ] `crates/gow-whoami/tests/integration.rs`
- [ ] `crates/gow-uname/tests/integration.rs`
- [ ] `crates/gow-paste/tests/integration.rs`
- [ ] `crates/gow-join/tests/integration.rs`
- [ ] `crates/gow-split/tests/integration.rs`
- [ ] `crates/gow-printf/tests/integration.rs`
- [ ] `crates/gow-expr/tests/integration.rs`
- [ ] `crates/gow-test/tests/integration.rs`
- [ ] `crates/gow-fmt/tests/integration.rs`
- [ ] `crates/gow-unlink/tests/integration.rs`

---

## Security Domain

These utilities are text processing, numeric, and OS-query tools with minimal security surface.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | partial | expr: validate argument count and operand types; split: validate -b/-l/-n are positive integers; printf: validate format string does not overflow stack |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| expr infinite recursion via deeply nested parens | Denial of Service | Limit recursion depth (e.g., 100 levels); return parse error if exceeded |
| printf format string buffer overflow | Tampering | Rust's string formatting is memory-safe; no C sprintf involved — not applicable |
| split writing to unexpected paths via crafted prefix | Tampering | Validate output prefix does not contain path separators (or document it is allowed — GNU allows it) |
| test -f on UNC paths or device paths | Information Disclosure | `std::fs::metadata` returns error gracefully; no special handling needed |
| unlink deleting system files | Elevation of Privilege | Standard file permissions apply; unlink does not bypass ACL |

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| GetVersionExW for Windows version | RtlGetVersion (ntdll) | Windows 8.1 era | GetVersionExW lies; RtlGetVersion always returns real version |
| `whoami` via `USERNAME` env var | GetUserNameW API | N/A | Env var approach is unreliable (can be overridden); API is authoritative |
| expr hand-rolled flat-scan | recursive-descent parser | N/A | Flat scan fails on operator precedence; recursive-descent is correct |

**No deprecated approaches in the Phase 11 stack.** All chosen patterns are current as of 2026-04-29.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `expr` operator precedence table matches GNU expr 9.x | Pattern 3 | Scripts with complex expr expressions may evaluate incorrectly |
| A2 | `printf "%d\n" 1 2 3` repeats the format for extra args (GNU behavior) | Pattern 5 | printf with multiple args produces wrong output |
| A3 | `split` alphabetic suffix extends (zz → aaa) when exhausted | Pattern 6 | split fails when producing >676 files (26^2) |
| A4 | `join` requires pre-sorted input and should warn (not error) on out-of-order lines | Pattern 6 | join silently produces wrong output without warning |
| A5 | `[.bat` shim with `--_bracket_` sentinel flag is the cleanest solution for bracket mode detection | Pattern 4 / Pitfall 9 | If user inspects `[` invocation details, sentinel flag is visible |
| A6 | `GetComputerNameW` is in `Win32_System_WindowsProgramming` (same feature as GetUserNameW) | Pattern 2 | If in a different feature, uname will fail to link — need additional feature flag |

**A6 risk mitigation:** Check at scaffold time — if `GetComputerNameW` is in a different feature module, add that feature to the workspace.

---

## Open Questions

1. **`[` bracket detection via shim vs sentinel flag**
   - What we know: `.bat` shim calls `test.exe`; argv[0] will be `test.exe` path not `[`
   - What's unclear: Whether `--_bracket_` sentinel is better than making `[.exe` a direct copy of `test.exe` (NTFS allows `[.exe` as filename)
   - Recommendation: Use `--_bracket_` sentinel in the shim for reliability. `[.exe` as a copy could cause WiX harvesting or antivirussoftware issues with the unusual filename.

2. **uname `-v` field content**
   - What we know: GNU uname `-v` on Linux shows kernel version string like `#1 SMP`
   - What's unclear: What gow-uname should output for `-v` on Windows (could be build number, service pack, or `#1`)
   - Recommendation: Output `#1` (a safe constant matching common practice for Windows uname implementations). Not critical for success criteria.

3. **test `-x` on Windows**
   - What we know: On Linux, `-x` checks execute permission bit. Windows has no execute permission bit.
   - What's unclear: Should `-x` on Windows check for `.exe/.bat/.com` extension, or always return false, or check if the file is readable?
   - Recommendation: Return true if file has `.exe`, `.bat`, `.com`, or `.cmd` extension (Windows executable convention). Document the difference.

4. **join with tabs vs whitespace delimiter**
   - What we know: GNU join default delimiter is "any whitespace" (multiple consecutive spaces/tabs count as one separator)
   - What's unclear: Whether `-t '\t'` (literal tab as delimiter) changes split behavior for adjacent tabs
   - Recommendation: Implement whitespace-splitting default (split on any run of spaces/tabs). `-t CHAR` uses exact single-character split (no collapsing).

---

## Sources

### Primary (HIGH confidence)
- `Cargo.toml` (workspace root) — workspace deps, existing features [VERIFIED: codebase]
- `build.bat` — installer staging logic (lines 131-135) [VERIFIED: codebase]
- `extras/bin/` — existing .bat shim patterns [VERIFIED: codebase]
- `crates/gow-df/src/lib.rs` — windows-sys usage pattern [VERIFIED: codebase]
- windows-sys-0.61.2 `Win32/System/WindowsProgramming/mod.rs` — GetUserNameW signature [VERIFIED: cargo registry source]
- windows-sys-0.61.2 `Win32/System/SystemInformation/mod.rs` — OSVERSIONINFOW, SYSTEM_INFO, PROCESSOR_ARCHITECTURE_*, GetNativeSystemInfo [VERIFIED: cargo registry source]
- windows-sys-0.61.2 `Wdk/System/SystemServices/mod.rs` — RtlGetVersion signature [VERIFIED: cargo registry source]
- windows-sys-0.61.2 `Cargo.toml` — feature list for Win32_System_WindowsProgramming, Win32_System_SystemInformation, Wdk_System_SystemServices [VERIFIED: cargo registry source]
- Cargo binary name `[` rejection — `error: invalid character '[' in crate name` [VERIFIED: empirical test 2026-04-29]
- NTFS allows `[.exe` as filename [VERIFIED: empirical test 2026-04-29]

### Secondary (MEDIUM confidence)
- uutils test.rs bracket detection pattern [CITED: github.com/uutils/coreutils test.rs via WebFetch]
- GNU coreutils test invocation documentation [CITED: gnu.org/software/coreutils/manual/html_node/test-invocation.html]

### Tertiary (LOW confidence)
- expr operator precedence table [ASSUMED: from training knowledge; not verified against live GNU expr source]
- printf extra-args repeat behavior [ASSUMED: from training knowledge of POSIX printf behavior]
- split suffix exhaustion behavior (zz→aaa) [ASSUMED: from training knowledge]
- join out-of-order warning behavior [ASSUMED: from training knowledge]

---

## Metadata

**Confidence breakdown:**
- Scaffold/crate structure: HIGH — verified from Phase 10 patterns and codebase
- Windows API signatures (whoami/uname): HIGH — verified from windows-sys source
- `[` binary name restriction: HIGH — empirically tested
- windows-sys feature flags needed: HIGH — verified from windows-sys source
- expr exit codes: HIGH — POSIX standard, verified by description
- test semantics: HIGH — well-documented POSIX standard
- expr/printf/paste/join/split algorithms: MEDIUM — from training knowledge; standard POSIX behavior
- expr operator precedence table: LOW — assumed from training; verify against GNU source if scripts fail

**Research date:** 2026-04-29
**Valid until:** 2026-07-29 (stable APIs — 90-day validity; windows-sys unlikely to change signatures)

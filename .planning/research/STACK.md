# Technology Stack

**Project:** gow-rust (GNU On Windows — Rust rewrite)
**Researched:** 2026-04-20
**Overall confidence:** HIGH (versions verified against crates.io live API)

---

## Recommended Stack

### Core Language & Toolchain

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust (stable) | 1.85+ | Primary language | MSVC toolchain required for Windows; stable channel only — no nightly features |
| Cargo workspace | built-in | Monorepo management | Each utility is a member crate (`crates/ls`, `crates/grep`, etc.); shared `Cargo.lock` ensures reproducible builds |
| `x86_64-pc-windows-msvc` target | — | Build target | MSVC toolchain avoids MinGW runtime dependency; produces self-contained binaries; required for `windows-sys` raw-dylib linkage |

**Rationale for MSVC over GNU toolchain:** uutils/coreutils explicitly recommends MSVC over MinGW for Windows distribution. `windows-sys` 0.61+ uses raw-dylib unconditionally, cutting build time and eliminating `windows-targets` import lib downloads. MinGW targets still work but introduce a separate C runtime.

---

### CLI Argument Parsing

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `clap` | 4.6.1 | Argument parsing for all utilities | Industry standard. Derive macro API keeps utility code clean; automatic `--help`, `--version`, shell completions via `clap_complete`. Used by uutils/coreutils |
| `clap_complete` | (clap workspace) | Shell completion generation | Optional but valuable; generates Bash/Zsh/PowerShell completions from same struct |

**Use the derive API** (`#[derive(Parser)]`) not the builder API. Derive is more maintainable across 20+ utilities and catches missing fields at compile time.

**Do NOT use:** `structopt` (deprecated, merged into clap 3+), `getopts` (too low-level, no help generation), `argparse` (abandoned).

---

### Error Handling

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `thiserror` | 2.0.18 | Library/utility error types | Derive macros for structured error enums; each utility defines its own `Error` enum. Zero runtime cost |
| `anyhow` | 1.0.102 | Application-level error propagation | Use in `main()` and top-level glue code for context-chain error messages. Not inside library functions |

**Rule:** `thiserror` in library crates (each utility's core logic), `anyhow` only at the binary entry point. uutils uses `thiserror` exclusively — follow that pattern for GNU-compatible exit codes (you must inspect the error type, not just print it).

---

### File I/O & Path Operations

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `walkdir` | 2.5.0 | Recursive directory traversal (`ls -R`, `find`, `grep -r`) | Handles symlinks, cycles, permissions; cross-platform; used everywhere in the Rust ecosystem |
| `filetime` | 0.2.27 | File timestamp read/write (`touch`, `cp -p`) | Only crate that handles Windows FILETIME ↔ Unix epoch conversion correctly |
| `globset` | 0.4.18 | Glob pattern matching (`find`, `ls` wildcards) | Compiled glob sets; handles `**` patterns correctly; by BurntSushi (same author as ripgrep) |
| `notify` | 8.2.0 | Filesystem watching (`tail -f`) | Uses `ReadDirectoryChangesW` on Windows natively — fixes GOW issue #169 / #75 / #89 |
| `tempfile` | 3.27.0 | Atomic writes, temp files (`sed -i`, `sort`) | Needed for safe in-place editing; creates temp files in same directory as target to ensure same filesystem for atomic rename |

**Windows path handling note:** Use `std::path::Path` and `std::ffi::OsStr` throughout. Never call `.to_str().unwrap()` on paths — Windows paths with non-UTF-8 surrogates will panic. Use `.to_string_lossy()` for display only, preserve `OsString` for actual file operations.

---

### Regex & Text Processing

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `regex` | 1.12.3 | Pattern matching (`grep`, `sed`, `find -name`, `awk`) | The definitive Rust regex crate. Linear-time guarantees; no catastrophic backtracking; SIMD accelerated; used by ripgrep |
| `bstr` | 1.9.1 | Byte-string operations on non-UTF-8 input | GNU tools process arbitrary byte streams, not guaranteed UTF-8. `bstr` handles line-by-line iteration without panicking on invalid UTF-8 |
| `memchr` | (transitive via regex) | SIMD-accelerated byte search | ripgrep's inner loop; pulled in automatically |

**Do NOT use:** `fancy_regex` for main grep (PCRE-style backtracking; removes linear-time guarantee). `pcre2` bindings require a C compiler and `libpcre2` — unacceptable for a self-contained Windows binary. Use `regex` exclusively; document known PCRE incompatibilities.

**sed-specific:** Implement the `s/pattern/replace/flags` command using `regex::Regex::replace` / `replace_all`. The `sedregex` crate provides a parser for sed substitution syntax but is minimally maintained — prefer implementing the small parser directly.

---

### Terminal Output & Colors

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `termcolor` | 1.4.1 | Colored output (`grep --color`, `ls --color`) | By BurntSushi; minimal dependency; explicitly designed for GNU-tool-style coloring; handles both ANSI escape sequences AND Windows Console API (Win7 fallback). Used in ripgrep |
| `crossterm` | 0.29.0 | Full terminal control (`less` pager, interactive UI) | Required for pager (`less`) which needs cursor movement, screen clearing, raw mode. Overkill for simple color output — use `termcolor` there instead |
| `terminal_size` | 0.4.0 | Detect terminal width (`ls` column layout) | Single-purpose; handles both UNIX `ioctl` and Windows `GetConsoleScreenBufferInfo` |

**Rule:** Use `termcolor` for simple color output (grep, ls, diff). Reserve `crossterm` for the `less` pager which needs interactive terminal control. Do not pull in `crossterm` for every utility — compile time and binary size cost is significant.

**Windows ANSI note:** Windows 10 1511+ enables ANSI virtual terminal processing via `SetConsoleMode` with `ENABLE_VIRTUAL_TERMINAL_PROCESSING`. Both `termcolor` and `crossterm` handle this automatically. GOW issue #85 (grep --color broken) is solved by using these crates rather than raw `\x1b[` escapes.

---

### Encoding & Unicode

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `encoding_rs` | 0.8.35 | Windows codepage ↔ UTF-8 conversion | Required for reading files in system codepage (CP932, CP1252, etc.) and for console I/O when `CHCP` is not 65001. Used by Firefox; high quality implementation |

**Strategy:** Default to UTF-8 everywhere. Use `encoding_rs` only at I/O boundaries when the user has a non-UTF-8 console or is reading legacy-encoded files. This directly resolves GOW issues #280 and #77 (UTF-8 corruption).

**Windows console:** Call `SetConsoleOutputCP(65001)` and `SetConsoleCP(65001)` at startup (via `windows-sys`) to set the console to UTF-8 mode before any output.

---

### Windows-Specific System APIs

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `windows-sys` | 0.61.2 | Raw Windows API bindings | Zero-overhead C-style bindings; raw-dylib in 0.61+ eliminates import lib download; used by uutils/coreutils; faster compile than `windows` crate |

**Use `windows-sys` not `windows-rs`** for this project. The higher-level `windows` crate (COM, WinRT, safe wrappers) is unnecessary for GNU utility work — you only need C-style Win32 APIs: `SetConsoleMode`, `SetConsoleCP`, `GetFileAttributes`, `CreateSymbolicLink`, `ReadDirectoryChangesW`. `windows-sys` compiles significantly faster.

**Specific APIs needed:**
- `SetConsoleOutputCP(65001)` — UTF-8 output mode (every binary)
- `GetFileAttributesW` — hidden file detection (`ls -a`)
- `CreateSymbolicLinkW` — symlink creation (`ln -s`)
- `GetConsoleScreenBufferInfo` — terminal width (via `terminal_size`)
- `ReadDirectoryChangesW` — file watching (via `notify`)

---

### Compression & Archive Utilities

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `flate2` | (check crates.io) | gzip/deflate (`gzip`, `gunzip`, `zcat`) | Standard Rust gzip; wraps zlib or pure-Rust miniz_oxide backend; use miniz feature for pure-Rust (no C dep) |
| `bzip2` | (check crates.io) | bzip2 compression | C binding to libbz2 — acceptable; no pure-Rust alternative at production quality |
| `tar` | (check crates.io) | tar archive creation/extraction | Pure Rust; used by cargo itself for `.crate` files |
| `xz2` | (check crates.io) | xz/lzma compression | C binding to liblzma; acceptable for completeness |

**Note:** Verify exact versions at build time — these were not checked against the live API in this research pass. Use `cargo add` to pull the latest stable versions. For Windows MSVC builds, ensure C dependencies compile — `bzip2` and `xz2` both require a C compiler but work fine with MSVC.

---

### HTTP/Network (curl replacement)

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `reqwest` | (check crates.io) | HTTP client (`curl` replacement) | De-facto standard Rust HTTP client; uses native TLS on Windows (`native-tls` feature) which uses Windows SChannel — no OpenSSL dependency |
| `native-tls` | (check crates.io) | TLS using Windows SChannel | Avoids bundling OpenSSL; uses OS-provided certificate store automatically; correct for a Windows-native tool |
| `tokio` | (check crates.io) | Async runtime (needed by reqwest) | Only required for the `curl` equivalent; do NOT add to other utilities |

**Scope note:** The `curl` reimplementation is explicitly in scope (PROJECT.md) but should be treated as a separate, later-phase utility given the async runtime dependency. Do not let `tokio` leak into the simpler coreutils implementations.

---

### Testing Framework

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `assert_cmd` | 2.2.1 | Integration test: spawn binary, assert output | The standard for CLI integration testing in Rust; `cargo_bin!` macro finds the debug binary; fluent assertion API |
| `predicates` | 3.1.4 | Assertion matchers for `assert_cmd` | Works with `assert_cmd` to write `contains`, `starts_with`, `matches_regex` assertions with rich diff output on failure |
| `snapbox` | 1.2.1 | Snapshot testing: compare output against `.txt` fixtures | Ideal for GNU compatibility tests — store expected output as `.txt` files next to tests; auto-update with `SNAPSHOTS=overwrite` |
| `tempfile` | 3.27.0 | Create temp dirs/files in tests | Needed for tests that write files; auto-cleanup on drop |

**Testing strategy:** Every utility needs two test layers:
1. **Unit tests** (in `#[cfg(test)]` modules): Pure logic — regex compilation, option parsing, exit code mapping
2. **Integration tests** (`tests/` dir in each utility crate): Spawn the actual binary with `assert_cmd`, verify stdout/stderr/exit code matches GNU behavior

Use `snapbox` for GNU compatibility snapshot tests — store the canonical expected output, update it when intentionally changing behavior. This creates a living GNU compatibility test suite.

**Do NOT use:** `rexpect` (interactive PTY testing) for most utilities — too heavy. Reserve it only if testing interactive prompts (currently none in scope).

---

### MSI Installer Tooling

| Tool | Version | Purpose | Why |
|------|---------|---------|-----|
| `cargo-wix` | 0.3.9 | Generate WiX XML and build MSI | Integrates with Cargo metadata; reads binary paths from Cargo workspace; handles PATH registration via WiX |
| WiX Toolset v3 (legacy) | 3.14.1 | MSI compiler/linker | GitHub Actions `windows-latest` image ships v3 by default (confirmed as of April 2026); use v3 to avoid CI setup complexity |

**Workflow:**
```bash
cargo install cargo-wix
cargo wix init --package gow-rust  # generates wix/main.wxs
# Edit main.wxs to add all utility binaries as Component entries
cargo wix --package gow-rust       # builds target/wix/gow-rust-*.msi
```

**Multi-binary MSI strategy:** The single MSI must install all utility `.exe` files. Use WiX `<Component>` entries for each binary within a single `<ComponentGroup>`. The installer registers one `<Environment>` element adding the install directory to `PATH` — this is how original GOW worked and what users expect.

**Signing:** `cargo-wix` supports `SignTool.exe` via `--sign`. For CI, use `signtool` with a code-signing certificate; for community builds, unsigned is acceptable.

---

### Project Structure (Workspace Layout)

```
gow-rust/
├── Cargo.toml              # workspace root
├── crates/
│   ├── ls/                 # uu_ls equivalent
│   ├── cat/
│   ├── grep/
│   ├── sed/
│   ├── find/
│   ├── ...
│   └── gow-common/         # shared utilities (encoding, exit codes, path conversion)
├── wix/
│   └── main.wxs            # WiX installer definition
├── tests/
│   └── integration/        # cross-utility integration tests
└── .github/
    └── workflows/
        └── ci.yml
```

**`gow-common` crate:** Shared library for cross-cutting concerns:
- UTF-8 console initialization (`SetConsoleOutputCP(65001)`)
- Unix ↔ Windows path conversion (`/c/Users/foo` ↔ `C:\Users\foo`)
- Common exit code constants (GNU convention: 0=success, 1=error, 2=misuse)
- GNU-style error message formatting (`<utility>: <message>`)

This avoids duplicating Windows initialization across 20+ binaries.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Arg parsing | `clap` 4 derive | `lexopt` | `lexopt` is low-level; no help generation; not appropriate for GNU-compatible tools that need `--help` |
| Arg parsing | `clap` 4 derive | `argh` | Google-internal lineage; less community; worse Windows support |
| Error handling | `thiserror` + `anyhow` | `eyre` | `eyre` adds color/context but is overkill for CLI exit-code-controlled programs; `anyhow` is simpler |
| Terminal colors | `termcolor` | `colored` crate | `colored` uses global state for color enable/disable; doesn't properly handle Windows Console API fallback; `termcolor` is purpose-built for this use case |
| Terminal colors | `termcolor` | `owo-colors` | `owo-colors` doesn't abstract Windows vs ANSI; requires manual detection |
| Regex | `regex` | `fancy_regex` | `fancy_regex` allows catastrophic backtracking via lookaheads; not safe for user-supplied patterns |
| Windows APIs | `windows-sys` | `windows` (full) | `windows` crate compiles COM/WinRT features we don't need; 3-5x slower compile times |
| Windows APIs | `windows-sys` | `winapi` | `winapi` is unmaintained (last release 2020); superseded by `windows-sys` |
| Installer | `cargo-wix` + WiX v3 | NSIS | NSIS requires separate scripting language; harder to automate from Cargo; WiX is the standard for MSI |
| Installer | `cargo-wix` + WiX v3 | WiX v4 | WiX v4 requires manual installation on CI (not in `windows-latest` image as of April 2026); use v3 to start, migrate later |
| File watching | `notify` | Manual `ReadDirectoryChangesW` | Manual Win32 watching is 200+ lines of unsafe code; `notify` wraps it correctly and is battle-tested |

---

## Minimum Rust Version

**MSRV: 1.85.0** — required by `clap` 4.6.1 and `snapbox` 1.2.1 (both verified against crates.io).

Set in workspace `Cargo.toml`:
```toml
[workspace.package]
rust-version = "1.85"
```

---

## Installation (Quick Reference)

```bash
# Build toolchain setup (Windows, one-time)
rustup target add x86_64-pc-windows-msvc
cargo install cargo-wix

# Add core dependencies to a utility crate (example: grep)
cargo add clap --features derive
cargo add thiserror
cargo add regex
cargo add termcolor
cargo add bstr

# Add to workspace root Cargo.toml [workspace.dependencies]
# (pin versions for reproducibility)
```

---

## Sources

- uutils/coreutils Cargo.toml (live): https://github.com/uutils/coreutils/blob/main/Cargo.toml
- clap 4.6.1 on crates.io: https://crates.io/crates/clap (verified 2026-04-20)
- crossterm 0.29.0 on crates.io: https://crates.io/crates/crossterm (verified 2026-04-20)
- regex 1.12.3 on crates.io: https://crates.io/crates/regex (verified 2026-04-20)
- anyhow 1.0.102 on crates.io: https://crates.io/crates/anyhow (verified 2026-04-20)
- thiserror 2.0.18 on crates.io: https://crates.io/crates/thiserror (verified 2026-04-20)
- windows-sys 0.61.2 on crates.io: https://crates.io/crates/windows-sys (verified 2026-04-20)
- assert_cmd 2.2.1 on crates.io: https://crates.io/crates/assert_cmd (verified 2026-04-20)
- predicates 3.1.4 on crates.io: https://crates.io/crates/predicates (verified 2026-04-20)
- snapbox 1.2.1 on crates.io: https://crates.io/crates/snapbox (verified 2026-04-20)
- cargo-wix 0.3.9 on crates.io: https://crates.io/crates/cargo-wix (verified 2026-04-20)
- notify 8.2.0 on crates.io: https://crates.io/crates/notify (verified 2026-04-20)
- windows-sys vs windows-rs official comparison: https://microsoft.github.io/windows-rs/book/rust-getting-started/windows-or-windows-sys.html
- cargo-wix README / WiX v3 vs v4 CI note: https://github.com/volks73/cargo-wix
- uutils extending blog post 2025: https://uutils.github.io/blog/2025-02-extending/
- BurntSushi/termcolor: https://github.com/BurntSushi/termcolor
- notify-rs/notify (ReadDirectoryChangesW): https://github.com/notify-rs/notify

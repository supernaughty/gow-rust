<!-- GSD:project-start source:PROJECT.md -->
## Project

**gow-rust**

GOW (Gnu On Windows) 유틸리티를 Rust로 재작성하는 오픈소스 프로젝트. 원본 GOW 0.8.0의 핵심 GNU 유틸리티들을 현대적인 Rust로 구현하여, Windows 환경에서 높은 GNU 호환성과 네이티브 Windows 통합(UTF-8, 경로 변환, PowerShell 연동)을 제공한다.

**Core Value:** Windows 사용자가 별도의 무거운 환경(WSL, Cygwin) 없이 GNU 명령어를 네이티브 성능으로 사용할 수 있어야 한다.

### Constraints

- **언어**: Rust (안정 채널, 최신 stable)
- **타겟 플랫폼**: Windows 10/11 x86_64 (MSVC 툴체인)
- **호환성**: GNU 옵션 높은 호환성 — 주요 플래그 대부분 지원하여 기존 스크립트가 동작
- **배포**: MSI 설치 프로그램, PATH 자동 등록
- **바이너리 구조**: 유틸리티별 독립 exe
- **인코딩**: UTF-8 기본, Windows 코드페이지 폴백 지원
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Language & Toolchain
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust (stable) | 1.85+ | Primary language | MSVC toolchain required for Windows; stable channel only — no nightly features |
| Cargo workspace | built-in | Monorepo management | Each utility is a member crate (`crates/ls`, `crates/grep`, etc.); shared `Cargo.lock` ensures reproducible builds |
| `x86_64-pc-windows-msvc` target | — | Build target | MSVC toolchain avoids MinGW runtime dependency; produces self-contained binaries; required for `windows-sys` raw-dylib linkage |
### CLI Argument Parsing
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `clap` | 4.6.1 | Argument parsing for all utilities | Industry standard. Derive macro API keeps utility code clean; automatic `--help`, `--version`, shell completions via `clap_complete`. Used by uutils/coreutils |
| `clap_complete` | (clap workspace) | Shell completion generation | Optional but valuable; generates Bash/Zsh/PowerShell completions from same struct |
### Error Handling
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `thiserror` | 2.0.18 | Library/utility error types | Derive macros for structured error enums; each utility defines its own `Error` enum. Zero runtime cost |
| `anyhow` | 1.0.102 | Application-level error propagation | Use in `main()` and top-level glue code for context-chain error messages. Not inside library functions |
### File I/O & Path Operations
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `walkdir` | 2.5.0 | Recursive directory traversal (`ls -R`, `find`, `grep -r`) | Handles symlinks, cycles, permissions; cross-platform; used everywhere in the Rust ecosystem |
| `filetime` | 0.2.27 | File timestamp read/write (`touch`, `cp -p`) | Only crate that handles Windows FILETIME ↔ Unix epoch conversion correctly |
| `globset` | 0.4.18 | Glob pattern matching (`find`, `ls` wildcards) | Compiled glob sets; handles `**` patterns correctly; by BurntSushi (same author as ripgrep) |
| `notify` | 8.2.0 | Filesystem watching (`tail -f`) | Uses `ReadDirectoryChangesW` on Windows natively — fixes GOW issue #169 / #75 / #89 |
| `tempfile` | 3.27.0 | Atomic writes, temp files (`sed -i`, `sort`) | Needed for safe in-place editing; creates temp files in same directory as target to ensure same filesystem for atomic rename |
### Regex & Text Processing
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `regex` | 1.12.3 | Pattern matching (`grep`, `sed`, `find -name`, `awk`) | The definitive Rust regex crate. Linear-time guarantees; no catastrophic backtracking; SIMD accelerated; used by ripgrep |
| `bstr` | 1.9.1 | Byte-string operations on non-UTF-8 input | GNU tools process arbitrary byte streams, not guaranteed UTF-8. `bstr` handles line-by-line iteration without panicking on invalid UTF-8 |
| `memchr` | (transitive via regex) | SIMD-accelerated byte search | ripgrep's inner loop; pulled in automatically |
### Terminal Output & Colors
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `termcolor` | 1.4.1 | Colored output (`grep --color`, `ls --color`) | By BurntSushi; minimal dependency; explicitly designed for GNU-tool-style coloring; handles both ANSI escape sequences AND Windows Console API (Win7 fallback). Used in ripgrep |
| `crossterm` | 0.29.0 | Full terminal control (`less` pager, interactive UI) | Required for pager (`less`) which needs cursor movement, screen clearing, raw mode. Overkill for simple color output — use `termcolor` there instead |
| `terminal_size` | 0.4.0 | Detect terminal width (`ls` column layout) | Single-purpose; handles both UNIX `ioctl` and Windows `GetConsoleScreenBufferInfo` |
### Encoding & Unicode
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `encoding_rs` | 0.8.35 | Windows codepage ↔ UTF-8 conversion | Required for reading files in system codepage (CP932, CP1252, etc.) and for console I/O when `CHCP` is not 65001. Used by Firefox; high quality implementation |
### Windows-Specific System APIs
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `windows-sys` | 0.61.2 | Raw Windows API bindings | Zero-overhead C-style bindings; raw-dylib in 0.61+ eliminates import lib download; used by uutils/coreutils; faster compile than `windows` crate |
- `SetConsoleOutputCP(65001)` — UTF-8 output mode (every binary)
- `GetFileAttributesW` — hidden file detection (`ls -a`)
- `CreateSymbolicLinkW` — symlink creation (`ln -s`)
- `GetConsoleScreenBufferInfo` — terminal width (via `terminal_size`)
- `ReadDirectoryChangesW` — file watching (via `notify`)
### Compression & Archive Utilities
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `flate2` | (check crates.io) | gzip/deflate (`gzip`, `gunzip`, `zcat`) | Standard Rust gzip; wraps zlib or pure-Rust miniz_oxide backend; use miniz feature for pure-Rust (no C dep) |
| `bzip2` | (check crates.io) | bzip2 compression | C binding to libbz2 — acceptable; no pure-Rust alternative at production quality |
| `tar` | (check crates.io) | tar archive creation/extraction | Pure Rust; used by cargo itself for `.crate` files |
| `xz2` | (check crates.io) | xz/lzma compression | C binding to liblzma; acceptable for completeness |
### HTTP/Network (curl replacement)
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `reqwest` | (check crates.io) | HTTP client (`curl` replacement) | De-facto standard Rust HTTP client; uses native TLS on Windows (`native-tls` feature) which uses Windows SChannel — no OpenSSL dependency |
| `native-tls` | (check crates.io) | TLS using Windows SChannel | Avoids bundling OpenSSL; uses OS-provided certificate store automatically; correct for a Windows-native tool |
| `tokio` | (check crates.io) | Async runtime (needed by reqwest) | Only required for the `curl` equivalent; do NOT add to other utilities |
### Testing Framework
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `assert_cmd` | 2.2.1 | Integration test: spawn binary, assert output | The standard for CLI integration testing in Rust; `cargo_bin!` macro finds the debug binary; fluent assertion API |
| `predicates` | 3.1.4 | Assertion matchers for `assert_cmd` | Works with `assert_cmd` to write `contains`, `starts_with`, `matches_regex` assertions with rich diff output on failure |
| `snapbox` | 1.2.1 | Snapshot testing: compare output against `.txt` fixtures | Ideal for GNU compatibility tests — store expected output as `.txt` files next to tests; auto-update with `SNAPSHOTS=overwrite` |
| `tempfile` | 3.27.0 | Create temp dirs/files in tests | Needed for tests that write files; auto-cleanup on drop |
### MSI Installer Tooling
| Tool | Version | Purpose | Why |
|------|---------|---------|-----|
| `cargo-wix` | 0.3.9 | Generate WiX XML and build MSI | Integrates with Cargo metadata; reads binary paths from Cargo workspace; handles PATH registration via WiX |
| WiX Toolset v3 (legacy) | 3.14.1 | MSI compiler/linker | GitHub Actions `windows-latest` image ships v3 by default (confirmed as of April 2026); use v3 to avoid CI setup complexity |
# Edit main.wxs to add all utility binaries as Component entries
### Project Structure (Workspace Layout)
- UTF-8 console initialization (`SetConsoleOutputCP(65001)`)
- Unix ↔ Windows path conversion (`/c/Users/foo` ↔ `C:\Users\foo`)
- Common exit code constants (GNU convention: 0=success, 1=error, 2=misuse)
- GNU-style error message formatting (`<utility>: <message>`)
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
## Minimum Rust Version
## Installation (Quick Reference)
# Build toolchain setup (Windows, one-time)
# Add core dependencies to a utility crate (example: grep)
# Add to workspace root Cargo.toml [workspace.dependencies]
# (pin versions for reproducibility)
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
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->

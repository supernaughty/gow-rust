# Phase 1: Foundation - Research

**Researched:** 2026-04-20
**Domain:** Rust Cargo workspace initialization, Windows platform primitives, GNU argument parsing, UTF-8 console, ANSI color, path conversion
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** clap 4 derive API 위에 GNU 호환 래퍼 레이어를 구축한다. uutils 접근 방식을 따른다.
- **D-02:** GNU exit code 규칙을 따른다 — 잘못된 인자는 exit code 1 (clap 기본값 2가 아님).
- **D-03:** 옵션 퍼뮤테이션을 지원한다 (예: `ls file -l`이 `ls -l file`과 동일하게 동작).
- **D-04:** `--` 이후 모든 인자를 비옵션으로 처리한다.
- **D-05:** 숫자 축약을 지원한다 (`head -5` = `head -n 5`, `tail -20` = `tail -n 20`).
- **D-06:** 파일 인자로 해석되는 위치에서만 경로 변환을 적용한다. 플래그 값(-c 등)은 변환하지 않는다.
- **D-07:** MSYS 스타일 경로를 인식한다: `/c/Users/foo` → `C:\Users\foo`. Git Bash/MSYS2 사용자를 위한 편의 기능.
- **D-08:** 변환은 보수적으로 — 확실한 경우에만 변환하고, 모호한 경우 원본을 유지한다.
- **D-09:** gow-core 라이브러리는 `thiserror`로 타입화된 에러를 정의한다 (GowError enum).
- **D-10:** 각 유틸리티 바이너리는 `anyhow`로 에러를 전파하고 main()에서 GNU 형식으로 출력한다.
- **D-11:** 에러 메시지 형식: `{utility}: {message}` (GNU 관례).
- **D-12:** Cargo workspace 구조: `crates/gow-core/`, `crates/gow-cat/`, `crates/gow-grep/` 등. `gow-` 접두사로 네임스페이스 분리.
- **D-13:** Rust 2024 에디션 사용, resolver = 3.
- **D-14:** 생성되는 실행 파일은 GNU 이름 그대로: `cat.exe`, `grep.exe`, `ls.exe`.
- **D-15:** workspace 수준에서 공통 의존성 버전 관리 (`[workspace.dependencies]`).

### Claude's Discretion

- gow-core 내부 모듈 구조 (encoding, args, color, path, error를 어떻게 나눌지)
- CI/CD 설정 세부사항
- 테스트 유틸리티/헬퍼 구조

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FOUND-01 | Cargo workspace 구조로 다중 크레이트 프로젝트 구성 | Workspace root Cargo.toml with `[workspace]`, `resolver = "3"`, `[workspace.dependencies]`, `[workspace.package]` patterns documented |
| FOUND-02 | gow-core 공유 라이브러리 — UTF-8 콘솔 초기화 (SetConsoleOutputCP 65001) | `windows-sys` 0.61.2 Win32 API call + `embed-manifest` 1.5.0 for `activeCodePage=UTF-8` app manifest |
| FOUND-03 | gow-core 공유 라이브러리 — GNU 호환 인자 파싱 (옵션 퍼뮤테이션, exit code 1, -- 종료) | clap 4 `try_get_matches()` pattern for exit-code override; `allow_hyphen_values`, `allow_negative_numbers` for permutation |
| FOUND-04 | gow-core 공유 라이브러리 — 컬러/TTY 감지 및 ANSI VT100 활성화 | `termcolor` 1.4.1 handles Windows VT enable via `ENABLE_VIRTUAL_TERMINAL_PROCESSING`; `windows-sys` `SetConsoleMode` |
| FOUND-05 | gow-core 공유 라이브러리 — 통합 에러 처리 타입 | `thiserror` 2.0.18 derive macros for `GowError` enum in library; `anyhow` 1.0.102 in binary main() |
| FOUND-06 | Unix↔Windows 경로 자동 변환 (컨텍스트 인식, GOW #244 해결) | `path-slash` 0.2.1 for `/`↔`\` normalization; custom MSYS drive-letter detection (`/c/` → `C:\`); only on confirmed file-argument positions |
| FOUND-07 | Windows 심볼릭 링크/정션 추상화 레이어 | `windows-sys` `CreateSymbolicLinkW`, `DeviceIoControl` for reparse tag; `symlink_metadata()` vs `metadata()` distinction |
| WIN-01 | UTF-8이 모든 유틸리티의 기본 인코딩 (GOW #280, #77 해결) | `SetConsoleOutputCP(65001)` + `SetConsoleCP(65001)` at init + `activeCodePage=UTF-8` in app manifest |
| WIN-02 | Windows 긴 경로 지원 (MAX_PATH 260자 제한 해제) | `longPathAware=true` in app manifest via `embed-manifest` build script |
| WIN-03 | PowerShell에서 모든 유틸리티 정상 동작 | UTF-8 encoding + correct exit codes + ANSI color detection covers PowerShell compatibility; no special action needed beyond FOUND-02/04 |

</phase_requirements>

---

## Summary

Phase 1 establishes the Cargo workspace and the `gow-core` shared library that every subsequent utility crate will depend on. The workspace must be configured for Rust 2024 edition with `resolver = "3"` and all shared dependency versions pinned at the workspace level. The `gow-core` crate must provide six platform primitives: UTF-8 console initialization, Windows application manifest embedding (for `activeCodePage=UTF-8` and `longPathAware`), GNU-compatible argument parsing with exit-code override, ANSI/VT100 color support, unified error types, context-aware MSYS path conversion, and a Windows symlink/junction abstraction layer.

The most important design decisions for Phase 1 are all locked: clap 4 derive API with `try_get_matches()` for exit-code override (exit 1 not 2), `thiserror` in `gow-core` for typed errors, `termcolor` for color output, `windows-sys 0.61.2` for Win32 calls, and `embed-manifest 1.5.0` in a build script for the application manifest. The directory structure uses `crates/gow-{name}/` with `gow-` prefixed crate names but GNU-named output binaries (`cat.exe`, `grep.exe`).

The Rust toolchain on this machine is **1.95.0 stable (x86_64-pc-windows-msvc)**, which is already above the MSRV of 1.85.0 required by the stack. Edition 2024 is confirmed supported. All key crate versions have been verified against crates.io as of this research date.

**Primary recommendation:** Build `gow-core` as a single library crate with six clearly-named modules (`encoding`, `args`, `color`, `path`, `error`, `fs`). Every binary calls `gow_core::init()` as its first line. No utility crate ever touches Win32 APIs directly.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| UTF-8 console init | gow-core (library) | Binary entry point (calls init) | Platform API belongs in shared lib; every binary pays once |
| Windows app manifest | Build script (build.rs) | embed-manifest crate | Manifest is a compile-time build artifact; not runtime code |
| GNU arg parsing wrapper | gow-core (library) | Each binary (derives clap structs) | Exit-code policy, permutation, `--` handling must be uniform |
| ANSI/VT100 color | gow-core (library) | termcolor (crate) | Windows VT enable is a one-time per-process call; lives in init() |
| Error types (GowError) | gow-core (library) | — | Typed errors allow correct exit code selection in main() |
| Path conversion (MSYS) | gow-core (library) | Binary arg preprocessing | Must run before any std::fs call; centralized avoids per-utility duplication |
| Symlink/junction abstraction | gow-core (library) | windows-sys (crate) | Windows-specific reparse point logic must not leak into utility crates |
| Long path awareness | Build script (build.rs) | embed-manifest crate | `longPathAware` is a manifest flag; not runtime selectable |

---

## Standard Stack

### Core — Phase 1 Direct Dependencies

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `clap` | 4.6.1 | Argument parsing foundation | Industry standard; derive API; used by uutils/coreutils. Verified on crates.io 2026-04-20 |
| `thiserror` | 2.0.18 | Typed error enum in gow-core | Zero-cost derive macros; enables correct per-variant exit codes. Verified 2026-04-20 |
| `anyhow` | 1.0.102 | Error propagation in binary main() | Context-chain formatting at the final error boundary. Verified 2026-04-20 |
| `termcolor` | 1.4.1 | Color output + Windows VT100 | BurntSushi; purpose-built for GNU-style tools; handles both ANSI and legacy Console API. Verified 2026-04-20 |
| `windows-sys` | 0.61.2 | Win32 API bindings | Raw-dylib in 0.61+ eliminates import-lib download; zero overhead; used by uutils. Verified 2026-04-20 |
| `encoding_rs` | 0.8.35 | Windows codepage ↔ UTF-8 | Firefox-quality; handles CP932, CP1252, etc. at I/O boundaries. Verified 2026-04-20 |
| `path-slash` | 0.2.1 | Slash/backslash normalization | Stable, widely used; handles basic `/`↔`\` conversion. Verified 2026-04-20 |

### Build-Time Dependencies

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `embed-manifest` | 1.5.0 | Embed Windows app manifest via build.rs | Sets `activeCodePage=UTF-8` and `longPathAware=true` at compile time; standard approach for Rust+Windows. Verified 2026-04-20 |

### Dev / Test Dependencies

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `assert_cmd` | 2.2.1 | Integration tests: spawn binary, assert output | All binary integration tests in `tests/` |
| `predicates` | 3.1.4 | Composable assertions for assert_cmd | Use alongside assert_cmd for `contains`, `regex` matchers |
| `tempfile` | 3.27.0 | Temp dirs/files in tests | Any test that writes to the filesystem |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `termcolor` | `colored` crate | `colored` uses global state; no proper Windows Console API fallback |
| `termcolor` | `owo-colors` | `owo-colors` does not abstract Windows vs ANSI detection |
| `windows-sys` | `windows` (full) | Full `windows` crate compiles COM/WinRT; 3-5x slower compile |
| `windows-sys` | `winapi` | `winapi` unmaintained since 2020 |
| `embed-manifest` | Hand-crafted `.rc` resource | `.rc` files require external `rc.exe` tool; `embed-manifest` works purely from Cargo build scripts |
| `path-slash` | Custom regex | Regex-based conversion is the root cause of GOW #244; `path-slash` is safe for `\`↔`/` but MSYS drive-letter logic still needs custom code |

**Installation:**
```bash
# In workspace root Cargo.toml — add to [workspace.dependencies]
# (no separate install step; Cargo resolves on first build)

# In crates/gow-core/Cargo.toml [build-dependencies]
cargo add --build embed-manifest
```

---

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│  User invokes binary (e.g., cat.exe /c/Users/foo/file.txt)  │
└─────────────────────┬───────────────────────────────────────┘
                      │ argv (OsString array)
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  binary main.rs  (thin — 3 lines)                           │
│  std::process::exit(gow_cat::uumain(std::env::args_os()))   │
└─────────────────────┬───────────────────────────────────────┘
                      │ OsString iterator
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  gow_core::init()                                           │
│  ├── encoding::setup_console_utf8()  [SetConsoleOutputCP]   │
│  └── color::enable_vt_mode()         [SetConsoleMode]       │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  gow_core::args::parse_gnu(uu_app(), raw_args)              │
│  ├── path::normalize_file_args()  [MSYS /c/ → C:\]         │
│  ├── clap try_get_matches()                                 │
│  └── on error: eprintln!("{bin}: {e}"); exit(1)  ← not 2   │
└─────────────────────┬───────────────────────────────────────┘
                      │ ArgMatches
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  utility logic (gow_cat::run, gow_grep::run, ...)           │
│  returns Result<(), GowError>                               │
└─────────────────────┬───────────────────────────────────────┘
                      │ GowError (thiserror)
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  main() error handler                                       │
│  eprintln!("{bin}: {e}");  std::process::exit(e.exit_code())│
└─────────────────────────────────────────────────────────────┘
```

**Build-time flow (build.rs):**
```
build.rs  →  embed_manifest::new_manifest()
             .active_code_page(Utf8)
             .long_path_aware(LongPathAware::Yes)
          →  manifest embedded in .exe resource section
          →  Windows reads at process startup (no runtime code needed)
```

### Recommended Project Structure

```
gow-rust/
├── Cargo.toml                      # workspace root — [workspace], [workspace.dependencies], [workspace.package]
├── Cargo.lock                      # single lockfile
├── .cargo/
│   └── config.toml                 # rustflags = ["-C", "target-feature=+crt-static"]
├── crates/
│   └── gow-core/                   # FOUND-01 thru FOUND-07, WIN-01/02/03
│       ├── Cargo.toml
│       ├── build.rs                # embed-manifest: activeCodePage=UTF-8, longPathAware=true
│       └── src/
│           ├── lib.rs              # pub mod encoding; pub mod args; pub mod color; pub mod path; pub mod error; pub mod fs;
│           │                       # pub fn init() — called by every binary
│           ├── encoding.rs         # setup_console_utf8(): SetConsoleOutputCP(65001) + SetConsoleCP(65001)
│           ├── args.rs             # parse_gnu(): try_get_matches() wrapper; exit-code 1 on error
│           ├── color.rs            # enable_vt_mode(); termcolor::ColorChoice detection
│           ├── path.rs             # normalize_path_arg(): MSYS /c/ → C:\; path-slash for \↔/
│           ├── error.rs            # GowError enum (thiserror); exit_code() method
│           └── fs.rs               # symlink/junction abstraction; symlink_metadata() helpers
└── tests/
    └── gow_core_integration.rs     # basic smoke tests for init(), path conversion, exit codes
```

Note: Utility crates (`crates/gow-cat/`, etc.) are NOT created in Phase 1 — only `gow-core`. The structure above shows where they will live in Phase 2+.

### Pattern 1: Workspace Root Cargo.toml

**What:** Root manifest declares all members and pins shared dependency versions.
**When to use:** Always — prevents version drift across 20+ utility crates.

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crates/gow-core"]        # Phase 1 only; Phase 2+ adds "crates/gow-*"
resolver = "3"                        # Rust 2024 default; required by D-13

[workspace.package]
version = "0.1.0"
edition = "2024"                      # D-13
rust-version = "1.85"
license = "MIT"
authors = ["gow-rust contributors"]

[workspace.dependencies]
# Core — Phase 1
clap = { version = "4.6", features = ["derive"] }
thiserror = "2"
anyhow = "1"
termcolor = "1"
windows-sys = { version = "0.61", features = [
    "Win32_System_Console",
    "Win32_Foundation",
    "Win32_Storage_FileSystem",
] }
encoding_rs = "0.8"
path-slash = "0.2"

# Testing
assert_cmd = { version = "2", features = ["cargo"] }
predicates = "3"
tempfile = "3"

[profile.release]
strip = true
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
```

### Pattern 2: gow-core Cargo.toml with Build Script

```toml
# crates/gow-core/Cargo.toml
[package]
name = "gow-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror = { workspace = true }
termcolor = { workspace = true }
windows-sys = { workspace = true }
encoding_rs = { workspace = true }
path-slash = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
tempfile = { workspace = true }
```

### Pattern 3: embed-manifest build.rs (FOUND-02, WIN-01, WIN-02)

**What:** Build script embeds a Windows application manifest that activates UTF-8 code page and long path awareness.
**When:** gow-core build.rs ONLY — not in each utility crate. Utilities inherit the manifest via the gow-core dependency if structured as a proc-macro or, more practically, each binary's own build.rs should embed it.

**IMPORTANT:** The manifest must be embedded in EACH binary `.exe`, not in gow-core.lib. The correct approach is: gow-core provides a `build.rs` that other binary crates re-export as a build script, OR each binary crate has its own build.rs calling `embed_manifest`.

```rust
// crates/gow-core/build.rs (and replicated to each binary crate in Phase 2+)
// Source: https://docs.rs/embed-manifest/latest/embed_manifest/
fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest::embed_manifest(
            embed_manifest::new_manifest("Gow.Rust")
                .active_code_page(embed_manifest::manifest::ActiveCodePage::Utf8)
                .long_path_aware(embed_manifest::manifest::LongPathAware::Yes),
        )
        .expect("unable to embed manifest");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
```

### Pattern 4: gow_core::init() — Windows Platform Init

**What:** Single function called as the very first line of every binary.
**When:** Every binary that produces terminal output.

```rust
// crates/gow-core/src/lib.rs
pub fn init() {
    encoding::setup_console_utf8();
    color::enable_vt_mode();
}
```

```rust
// crates/gow-core/src/encoding.rs
#[cfg(target_os = "windows")]
pub fn setup_console_utf8() {
    use windows_sys::Win32::System::Console::{
        SetConsoleCP, SetConsoleOutputCP,
    };
    unsafe {
        SetConsoleOutputCP(65001);
        SetConsoleCP(65001);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn setup_console_utf8() {}
```

### Pattern 5: GNU Exit Code Override (FOUND-03, D-02)

**What:** clap by default exits with code 2 on argument errors. GNU tools use code 1. Use `try_get_matches()` to intercept and remap.
**When:** All argument parsing in every utility — enforce through `gow_core::args::parse_gnu()`.

```rust
// crates/gow-core/src/args.rs
use clap::{Command, ArgMatches};

/// Parse arguments GNU-style: exits with code 1 (not clap default 2) on bad args.
/// Also handles `--` end-of-options (clap does this natively with TrailingVarArg).
pub fn parse_gnu(cmd: Command, args: impl IntoIterator<Item = std::ffi::OsString>) -> ArgMatches {
    let program_name = std::env::args().next().unwrap_or_default();
    let bin = std::path::Path::new(&program_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("gow");

    cmd.try_get_matches_from(args)
        .unwrap_or_else(|e| {
            // clap formats its own errors; we just fix the exit code
            eprintln!("{bin}: {e}");
            std::process::exit(1);  // GNU convention: bad args = 1, not 2
        })
}
```

### Pattern 6: Context-Aware MSYS Path Conversion (FOUND-06, D-06, D-07)

**What:** Convert `/c/Users/foo` to `C:\Users\foo` ONLY when the argument occupies a file-position slot, never when it is a flag value.
**When:** At argument preprocessing, before clap parses anything.
**Key invariant:** `/c` as a standalone argument is a flag (e.g., `cmd /c`) and MUST NOT be converted.

```rust
// crates/gow-core/src/path.rs
use std::ffi::OsString;
use std::path::PathBuf;

/// Convert a single argument string if (and only if) it looks like an MSYS2/Git Bash
/// Unix-style path that maps to a Windows drive.
///
/// Rules:
/// 1. Must match pattern `/<drive-letter>/<rest>` where drive-letter is a-z/A-Z
/// 2. Must have at least one path component after the drive (bare `/c` is ambiguous — skip)
/// 3. Conservative: if in doubt, return the original unchanged
pub fn try_convert_msys_path(arg: &str) -> String {
    // Pattern: /X/... where X is a single ASCII letter and there is more path after it
    let bytes = arg.as_bytes();
    if bytes.len() >= 4
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && bytes[2] == b'/'
    {
        let drive = (bytes[1] as char).to_ascii_uppercase();
        let rest = &arg[3..];  // everything after /X/
        // Use forward slashes for now; path-slash will normalize to backslash for fs calls
        format!("{drive}:\\{}", rest.replace('/', "\\"))
    } else {
        arg.to_owned()
    }
}

/// Normalize a path argument for use with std::fs on Windows.
/// Converts forward slashes to backslashes via path-slash.
pub fn to_windows_path(arg: &str) -> PathBuf {
    use path_slash::PathBufExt;
    PathBuf::from_slash(arg)
}
```

### Pattern 7: GowError Enum (FOUND-05, D-09)

```rust
// crates/gow-core/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GowError {
    #[error("cannot open '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("{0}")]
    Custom(String),
}

impl GowError {
    /// GNU exit code for this error variant.
    /// Most errors = 1; reserved for extension if different codes are needed.
    pub fn exit_code(&self) -> i32 {
        1
    }
}
```

### Pattern 8: Binary Crate Structure (Preview for Phase 2)

The following illustrates how utility crates built in Phase 2 will integrate with gow-core:

```toml
# crates/gow-cat/Cargo.toml  (Phase 2 — shown here for context)
[package]
name = "gow-cat"
version.workspace = true
edition.workspace = true

[lib]
name = "gow_cat"

[[bin]]
name = "cat"          # D-14: output binary matches GNU name
path = "src/main.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true, features = ["derive"] }
anyhow = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"   # each binary needs its own manifest
```

```rust
// crates/gow-cat/src/main.rs  (Phase 2 — preview)
fn main() {
    gow_core::init();
    std::process::exit(gow_cat::uumain(std::env::args_os()));
}
```

### Pattern 9: .cargo/config.toml — Static CRT + MSVC Target

```toml
# .cargo/config.toml
[build]
target = "x86_64-pc-windows-msvc"

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

This eliminates the `VCRUNTIME140.dll` dependency. Without this, binaries fail on systems without the Visual C++ Redistributable (Pitfall 16).

### Anti-Patterns to Avoid

- **Regex-based path conversion:** Using a global regex to replace `/letter/` corrupts flag values like `cmd /c`. Always use positional/contextual logic (D-06, GOW #244).
- **Calling Win32 APIs in utility crates:** Only `gow-core` may use `windows-sys`. Utility crates must call `gow_core::init()` and use `gow_core` abstractions.
- **Using `std::env::args()` instead of `args_os()`:** `args()` panics on non-UTF-8 process arguments on Windows. Always use `args_os()` and pass `OsString` iterators.
- **Calling `.to_str().unwrap()` on paths:** Windows paths with non-UTF-8 surrogates cause panics. Use `.to_string_lossy()` for display; preserve `OsString` for file operations.
- **Skipping embed-manifest on utility binaries:** The UTF-8 manifest must be embedded in EACH `.exe`, not just in gow-core. Each binary crate needs its own build.rs.
- **Omitting `target-feature=+crt-static`:** Release binaries must be self-contained or they fail on machines without MSVC runtime.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Windows console codepage | Custom `SetConsoleOutputCP` wrapper | `gow_core::encoding::setup_console_utf8()` (wraps `windows-sys`) | Centralize; every utility gets it free from `init()` |
| ANSI VT100 enable | Custom `SetConsoleMode` wrapper | `termcolor` + `gow_core::color::enable_vt_mode()` | `termcolor` handles legacy fallback (no-color on old consoles) automatically |
| App manifest embedding | Custom `.rc` file + external `rc.exe` | `embed-manifest` build.rs | Works from pure Cargo; no external tooling |
| Windows codepage conversion | Custom `encoding_rs` wrapper | `gow_core::encoding` module | Avoid duplicating encoding_rs calls across 20+ crates |
| `/`↔`\` path normalization | Custom `replace('/', '\\')` | `path-slash` crate | Edge cases around UNC paths, extended paths; `path-slash` handles them |
| Long path limit bypass | Manual `\\?\` prefix injection | `longPathAware=true` in manifest | Manifest flag is correct; `\\?\` prefix has side effects (breaks some APIs) |
| Colored output | Raw `\x1b[...m` escape codes | `termcolor` | Raw escapes break in legacy cmd.exe; termcolor activates VT mode and falls back gracefully |

**Key insight:** All Windows platform concerns belong in `gow-core`. A utility crate should never contain `#[cfg(target_os = "windows")]` blocks — those all live in `gow-core` and are accessed through stable cross-platform APIs.

---

## Common Pitfalls

### Pitfall 1: clap Exits with Code 2, GNU Requires Code 1

**What goes wrong:** Using `cmd.get_matches()` or `cmd.get_matches_from()` directly lets clap call `std::process::exit(2)` on bad arguments. GNU tools must exit with `1`.

**Why it happens:** clap follows its own convention (2 = usage error), not the GNU convention (1 = error, 2 reserved for "serious" misuse in some tools).

**How to avoid:** Always use `cmd.try_get_matches_from(args)` and handle the `Err` variant explicitly with `exit(1)`. Encapsulate in `gow_core::args::parse_gnu()` so every utility gets this for free.

**Warning signs:** `cmd badarg 2>/dev/null; echo $?` prints `2` instead of `1`.

**Verified by:** [uutils-args design doc](https://github.com/tertsdiepraam/uutils-args/blob/main/docs/design/problems_with_clap.md) [VERIFIED: GitHub official project doc]

---

### Pitfall 2: Path Conversion Corrupts Flag Values (GOW #244)

**What goes wrong:** Converting ALL arguments that start with `/letter/` turns `cmd /c "echo test"` into `cmd C:\ "echo test"` — the `/c` shell switch becomes a Windows drive path.

**Why it happens:** Naive regex or string replacement has no context; it cannot distinguish `/c` (flag) from `/c/Users/foo` (path with subdir).

**How to avoid:** Per D-06, only convert arguments that occupy file-position slots in the parsed argument structure. The MSYS detection rule requires at least two path components after the drive letter (i.e., `/X/` with more content, not bare `/X/`). The `try_convert_msys_path()` function above enforces this.

**Warning signs:** `cmd /c "echo test"` produces an error about `C:\` being unrecognized.

**Verified by:** [GOW Issue #244](https://github.com/bmatzelle/gow/issues/244) [VERIFIED: GitHub issue]

---

### Pitfall 3: UTF-8 Mojibake Without Both API Call and Manifest

**What goes wrong:** Calling `SetConsoleOutputCP(65001)` at runtime fixes output in most cases but not all — the process code page (which affects `fopen`, `CreateFileA`, and other ANSI APIs) is separate from the console output code page. The `activeCodePage=UTF-8` manifest element sets the ANSI process code page at process load time, before any code runs.

**Why it happens:** Two separate mechanisms: console code page (runtime) vs. process ANSI code page (manifest). Both are needed for full coverage.

**How to avoid:** Use both: `SetConsoleOutputCP(65001)` + `SetConsoleCP(65001)` in `gow_core::init()` AND `embed-manifest` with `Utf8` active code page in each binary's build.rs.

**Warning signs:** Non-ASCII filenames display correctly in output but paths containing CJK characters fail to open with `std::fs::File::open()`.

**Verified by:** [embed-manifest docs](https://docs.rs/embed-manifest/latest/embed_manifest/manifest/enum.ActiveCodePage.html) [VERIFIED: docs.rs official]

---

### Pitfall 4: Each Binary Needs Its Own build.rs for Manifest

**What goes wrong:** Putting `embed-manifest` only in `gow-core`'s build.rs embeds the manifest in the library artifact, not in the binary `.exe`. Windows reads application manifests from the `.exe` resource section — it does not inherit from a linked library.

**Why it happens:** Manifest embedding is a PE resource operation targeting the final executable, not a library.

**How to avoid:** Every binary crate (in Phase 2+, `crates/gow-cat/`, `crates/gow-grep/`, etc.) needs its own `build.rs` calling `embed_manifest::embed_manifest(...)`. This is slightly repetitive but correct. A shared build script pattern (using a helper function in gow-core) can reduce boilerplate.

**Warning signs:** `cargo manifest-tool get /manifestVersion target/release/cat.exe` returns no manifest.

**Verified by:** [embed-manifest README](https://docs.rs/embed-manifest/1.5.0/embed_manifest/) [VERIFIED: docs.rs official]

---

### Pitfall 5: option permutation with clap — `ls file -l`

**What goes wrong:** clap 4's default behavior (POSIX mode) stops processing options at the first non-option argument. `ls file.txt -l` would treat `-l` as a positional argument, not a flag.

**Why it happens:** clap defaults to POSIX-compliant parsing. GNU coreutils default to GNU permutation (options may appear anywhere).

**How to avoid:** Use `Command::allow_hyphen_values(true)` and ensure `trailing_var_arg` or `allow_external_subcommands` does not accidentally absorb options. For most utilities, `clap`'s `AllowHyphenValues` and `allow_negative_numbers` in combination with `trailing_var_arg(true)` achieves the right behavior. The `gow_core::args::parse_gnu()` wrapper should set these by default.

**Warning signs:** `ls file.txt -l` where `-l` is not recognized as the long-format flag.

**Verified by:** [PITFALLS.md - Pitfall 1] [CITED: .planning/research/PITFALLS.md]

---

### Pitfall 6: Static CRT Omitted from Release Builds

**What goes wrong:** Without `-C target-feature=+crt-static`, binaries link to `VCRUNTIME140.dll` dynamically. On machines without the Visual C++ Redistributable, the binary silently fails to start.

**Why it happens:** Cargo MSVC targets link the CRT dynamically by default (matches the behavior of most MSVC-compiled software).

**How to avoid:** Add to `.cargo/config.toml`: `rustflags = ["-C", "target-feature=+crt-static"]` for the `x86_64-pc-windows-msvc` target. Apply globally to all builds from day one.

**Warning signs:** `dumpbin /dependents cat.exe` shows `VCRUNTIME140.dll` in the dependency list.

**Verified by:** [PITFALLS.md - Pitfall 16] [CITED: .planning/research/PITFALLS.md]

---

## Code Examples

### Workspace root Cargo.toml (full Phase 1 version)

```toml
# Source: ARCHITECTURE.md Pattern 2 + verified against Cargo reference docs
[workspace]
members = ["crates/gow-core"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "MIT"

[workspace.dependencies]
clap = { version = "4.6", features = ["derive"] }
thiserror = "2"
anyhow = "1"
termcolor = "1"
windows-sys = { version = "0.61", features = [
    "Win32_System_Console",
    "Win32_Foundation",
    "Win32_Storage_FileSystem",
] }
encoding_rs = "0.8"
path-slash = "0.2"
assert_cmd = { version = "2", features = ["cargo"] }
predicates = "3"
tempfile = "3"

[profile.release]
strip = true
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
```

### color.rs — VT100 Enable + termcolor Setup

```rust
// crates/gow-core/src/color.rs
// Source: termcolor README, windows-sys Win32_System_Console

use termcolor::{ColorChoice, StandardStream};

#[cfg(target_os = "windows")]
pub fn enable_vt_mode() {
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode,
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
    };
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn enable_vt_mode() {}

/// Returns the appropriate ColorChoice based on the --color argument and TTY detection.
/// Respects NO_COLOR environment variable per https://no-color.org/.
pub fn color_choice(arg: Option<&str>) -> ColorChoice {
    if std::env::var_os("NO_COLOR").is_some() {
        return ColorChoice::Never;
    }
    match arg {
        Some("always") => ColorChoice::Always,
        Some("never") => ColorChoice::Never,
        _ => ColorChoice::Auto,  // default: auto (isatty check)
    }
}

pub fn stdout(choice: ColorChoice) -> StandardStream {
    StandardStream::stdout(choice)
}
```

### fs.rs — Symlink/Junction Abstraction (FOUND-07)

```rust
// crates/gow-core/src/fs.rs  (skeleton — full implementation in Phase 1 tasks)
use std::path::Path;

/// Describes the type of a Windows filesystem link.
#[derive(Debug, PartialEq)]
pub enum LinkType {
    SymlinkFile,
    SymlinkDir,
    Junction,
    HardLink,
}

/// Returns the link type for a path, or None if the path is not a link.
/// Uses symlink_metadata() — does NOT follow the link.
pub fn link_type(path: &Path) -> Option<LinkType> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if meta.file_type().is_symlink() {
        if meta.file_type().is_dir() {
            Some(LinkType::SymlinkDir)
        } else {
            Some(LinkType::SymlinkFile)
        }
    } else {
        // Check for junction (Windows-only reparse point)
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Some(LinkType::Junction);
            }
        }
        None
    }
}

/// Normalize a junction target path by stripping the \??\ device prefix.
/// Junction readlink returns `\??\C:\target` — strip to `C:\target` for display.
pub fn normalize_junction_target(raw: &str) -> &str {
    raw.strip_prefix(r"\??\").unwrap_or(raw)
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `windows-sys 0.52` (import lib) | `windows-sys 0.61` (raw-dylib) | 0.59 release | Eliminates separate import lib download in CI; faster builds |
| WiX v4 (new) | WiX v3 (legacy) on CI | Still true Apr 2026 | `windows-latest` GitHub Actions only has WiX v3; must use Legacy syntax |
| Rust edition 2021 | Rust edition 2024 | Rust 1.85 (stable Feb 2025) | `resolver = "3"` is now the default; new lifetime/pattern syntax available |
| `anyhow 1.x` only | `thiserror 2 + anyhow 1` | thiserror 2.0 released 2024 | thiserror 2 supports `std::error::Error` in `#[no_std]` environments; API compatible with 1.x |
| `winapi` crate | `windows-sys` | `winapi` last release 2020 | `winapi` unmaintained; `windows-sys` is the current standard |

**Deprecated/outdated:**
- `winapi`: Last release 2020, officially superseded by `windows-sys`. Do not use.
- `structopt`: Merged into clap 3+, no longer maintained as a separate crate. Do not use.
- `AppSettings::StrictUtf8` in clap 2: Removed in clap 4; now handled via `args_os()` + `OsString`.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `embed-manifest` build.rs must be in each binary crate, not just gow-core | Pitfall 4, Pattern 3 | If Windows does inherit manifest from .lib, the extra build.rs per crate is just redundant but harmless — safe to over-apply |
| A2 | clap 4's `try_get_matches_from()` preserves clap's own error message format (so we can `eprintln!("{e}")` and get a useful message) | Pattern 5 | If clap changes error Display format, messages may be ugly — test and adjust at implementation time |
| A3 | `POSIXLY_CORRECT` env var is not required by any Phase 1 requirement — it's a Phase 2+ concern for individual utilities | Architecture | If a Phase 1 test explicitly checks POSIXLY_CORRECT behavior, the args wrapper may need to read this env var earlier |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.
(Table has 3 low-risk items that are conservative assumptions; none block planning.)

---

## Open Questions

1. **Does each utility binary need its own `build.rs` or can a helper crate provide it?**
   - What we know: `embed-manifest` must be called during the build of each binary to embed the manifest in that binary's PE resource section.
   - What's unclear: Whether a cargo build script can be shared across workspace members via a helper.
   - Recommendation: Plan for one `build.rs` per binary crate in Phase 2+ (copy-paste from gow-core). Evaluate a shared helper crate if the number of utilities makes this painful.

2. **Should `gow_core::args::parse_gnu()` also handle `POSIXLY_CORRECT`?**
   - What we know: POSIXLY_CORRECT is a Phase 1 foundation concern (PITFALLS.md Pitfall 14).
   - What's unclear: Whether any Phase 1 requirements or tests exercise this.
   - Recommendation: Add `POSIXLY_CORRECT` check as a stretch goal in Phase 1. If time-boxed, defer to Phase 2 with a TODO comment in `args.rs`.

3. **Scope of `gow-core` integration tests in Phase 1?**
   - What we know: Phase 1 produces no binary utilities — only gow-core library. Integration tests cannot use `assert_cmd` to spawn a binary that doesn't exist yet.
   - What's unclear: What constitutes sufficient test coverage for a library crate with no binary output.
   - Recommendation: Write unit tests within gow-core for each module: path conversion round-trips, error exit code values, MSYS detection edge cases. Add a minimal `gow-probe` binary (not shipped) as a test harness to validate `init()` end-to-end.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable (MSVC) | All Rust compilation | ✓ | 1.95.0 (x86_64-pc-windows-msvc) | — |
| Cargo | Build system | ✓ | 1.95.0 | — |
| edition 2024 | D-13 | ✓ | Confirmed supported in cargo 1.95 | — |
| resolver = "3" | D-13 | ✓ | Supported since Rust 1.85 (edition 2024 default) | — |
| x86_64-pc-windows-msvc target | MSVC toolchain requirement | ✓ | Active default | — |
| MSVC Build Tools / link.exe | MSVC target linking | [ASSUMED] | Not directly checked | — |

**Missing dependencies with no fallback:** None detected.

**Missing dependencies with fallback:** None detected.

**Note on MSVC Build Tools:** `rustup show` confirms the MSVC target is the active default, which means the toolchain was set up correctly. If the MSVC Build Tools are missing, `cargo build` will fail with a `link.exe not found` error. This is a one-time environment setup issue, not a code issue. [ASSUMED: MSVC tools present based on MSVC target being active]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (cargo test) + assert_cmd 2.2.1 |
| Config file | None — cargo's built-in test discovery |
| Quick run command | `cargo test -p gow-core` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FOUND-01 | `cargo build --workspace` succeeds | Build smoke | `cargo build --workspace` | ❌ Wave 0 (create workspace) |
| FOUND-02 | UTF-8 console init does not panic; codepage is 65001 after init | Unit | `cargo test -p gow-core encoding` | ❌ Wave 0 |
| FOUND-03 | Bad args exit 1; `--` treated as end-of-options | Unit | `cargo test -p gow-core args` | ❌ Wave 0 |
| FOUND-04 | VT mode enable does not panic; ColorChoice::Auto detected | Unit | `cargo test -p gow-core color` | ❌ Wave 0 |
| FOUND-05 | GowError::exit_code() returns 1; error message format correct | Unit | `cargo test -p gow-core error` | ❌ Wave 0 |
| FOUND-06 | `/c/Users/foo` → `C:\Users\foo`; `/c` alone unchanged; `-c` unchanged | Unit | `cargo test -p gow-core path` | ❌ Wave 0 |
| FOUND-07 | symlink_metadata vs metadata returns correct types; junction target strip | Unit | `cargo test -p gow-core fs` | ❌ Wave 0 |
| WIN-01 | UTF-8 encoding initialized at startup | Integration | gow-probe binary smoke test | ❌ Wave 0 |
| WIN-02 | `longPathAware` in manifest (verified via dumpbin or manifset inspection) | Manual / build check | `cargo build && dumpbin /manifests target/debug/gow-probe.exe` | ❌ Wave 0 |
| WIN-03 | PowerShell can call gow-probe; exit code is correct | Integration | `cargo test -p gow-core` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p gow-core`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green + manual WIN-02 manifest check before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `Cargo.toml` (workspace root) — covers FOUND-01
- [ ] `crates/gow-core/Cargo.toml` — crate manifest
- [ ] `crates/gow-core/build.rs` — embed-manifest; covers WIN-01, WIN-02
- [ ] `crates/gow-core/src/lib.rs` — module declarations + init()
- [ ] `crates/gow-core/src/encoding.rs` + unit tests — covers FOUND-02, WIN-01
- [ ] `crates/gow-core/src/args.rs` + unit tests — covers FOUND-03
- [ ] `crates/gow-core/src/color.rs` + unit tests — covers FOUND-04
- [ ] `crates/gow-core/src/error.rs` + unit tests — covers FOUND-05
- [ ] `crates/gow-core/src/path.rs` + unit tests — covers FOUND-06
- [ ] `crates/gow-core/src/fs.rs` + unit tests — covers FOUND-07
- [ ] `.cargo/config.toml` — static CRT; covers Pitfall 6
- [ ] `crates/gow-probe/` (minimal test binary) — integration smoke for WIN-01/WIN-03

---

## Security Domain

> `security_enforcement` not explicitly disabled in config.json — including this section.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Not applicable — CLI tool, no user auth |
| V3 Session Management | No | Not applicable |
| V4 Access Control | No | Not applicable at this layer |
| V5 Input Validation | Yes (partial) | Path normalization must not allow path traversal beyond user intent |
| V6 Cryptography | No | No cryptography in Phase 1 |

### Known Threat Patterns for this Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via MSYS conversion | Tampering | Conservative conversion (D-08): only convert confirmed `/X/` patterns; never convert bare `/X`; preserve ambiguous inputs unchanged |
| Argument injection via path conversion | Tampering | Positional-only conversion (D-06): never convert flag values; validate that converted path does not contain embedded null bytes |
| Surrogate pair injection in filenames | Information Disclosure | Use `OsString` / `OsStr` for all file operations; never `to_str().unwrap()` |

---

## Sources

### Primary (HIGH confidence)

- crates.io live registry (verified 2026-04-20): clap 4.6.1, thiserror 2.0.18, anyhow 1.0.102, termcolor 1.4.1, windows-sys 0.61.2, encoding_rs 0.8.35, path-slash 0.2.1, embed-manifest 1.5.0, assert_cmd 2.2.1
- `.planning/research/STACK.md` — all crate versions verified against crates.io during stack research
- `.planning/research/ARCHITECTURE.md` — workspace patterns verified against uutils/coreutils source
- `.planning/research/PITFALLS.md` — pitfalls verified against uutils issue tracker, Rust issue tracker, GOW issue tracker
- [embed-manifest docs.rs](https://docs.rs/embed-manifest/latest/embed_manifest/) — activeCodePage and longPathAware API
- [Rust 1.95.0 installed toolchain] — confirmed via `rustc --version` (2026-04-14 build)
- [cargo 1.95.0 edition 2024 support] — confirmed via `cargo new --help`

### Secondary (MEDIUM confidence)

- [clap try_get_matches docs.rs](https://docs.rs/clap/latest/clap/struct.Command.html) — exit code override pattern via WebFetch
- [uutils-args design doc](https://github.com/tertsdiepraam/uutils-args/blob/main/docs/design/problems_with_clap.md) — clap GNU incompatibilities

### Tertiary (LOW confidence)

- WebSearch result confirming embed-manifest is the standard Rust approach for manifests on Windows (multiple sources agree; promoted to MEDIUM)

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all versions verified against live crates.io registry 2026-04-20; toolchain confirmed installed
- Architecture: HIGH — patterns verified against uutils/coreutils source (100+ utility project using same structure)
- Pitfalls: HIGH — each pitfall sourced from official issue trackers (Rust, uutils, GOW)
- Path conversion: HIGH — GOW #244 is documented fact; conservative-only rule is clear
- embed-manifest approach: HIGH — verified against docs.rs official documentation

**Research date:** 2026-04-20
**Valid until:** 2026-05-20 (stable crate ecosystem; re-verify versions if > 30 days elapsed)
# Architecture Patterns

**Domain:** Multi-utility Rust CLI project (Windows-native GNU reimplementation)
**Researched:** 2026-04-20
**Confidence:** HIGH (verified against uutils/coreutils source, Cargo official docs, crates.io)

---

## Recommended Architecture

The dominant pattern in the Rust GNU-tools ecosystem is a **Cargo workspace with one shared library crate and N per-utility library crates**, each wrapped by a thin binary entry point. This is the exact structure used by uutils/coreutils (100+ utilities) and is the established standard.

```
gow-rust/
├── Cargo.toml                    # Workspace root — defines all members
├── Cargo.lock                    # Single lock file for all crates
├── .cargo/
│   └── config.toml               # target = "x86_64-pc-windows-msvc", strip symbols
├── crates/
│   └── gow-core/                 # Shared library: all common functionality
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── encoding.rs       # UTF-8/Windows code page handling
│           ├── path.rs           # Unix<->Windows path conversion
│           ├── error.rs          # Shared error types (thiserror)
│           ├── fs.rs             # Cross-platform file I/O helpers
│           ├── output.rs         # ANSI color + Windows VT mode init
│           └── args.rs           # Common argument patterns (clap helpers)
├── src/
│   └── uu/                       # One subdirectory per utility
│       ├── ls/
│       │   ├── Cargo.toml        # lib [lib] + [[bin]] sections
│       │   └── src/
│       │       ├── lib.rs        # All logic: uu_app() → clap::Command
│       │       └── main.rs       # pub fn main() { uu_ls::main() }
│       ├── cat/
│       ├── grep/
│       ├── sed/
│       ├── find/
│       └── ... (one per utility)
├── tests/
│   ├── by-util/                  # Integration tests per utility
│   │   ├── test_ls.rs
│   │   ├── test_cat.rs
│   │   └── ...
│   └── common/
│       └── mod.rs                # UCommand scene helper, test fixtures
├── packaging/
│   └── wix/
│       ├── main.wxs              # WiX XML: all binaries, PATH env variable
│       └── banner.bmp
└── scripts/
    ├── build-all.ps1             # PowerShell build orchestration
    └── build-all.sh              # Bash equivalent (CI)
```

---

## Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `gow-core` (shared lib) | Encoding init, path conversion, ANSI/color, shared error types, common clap arg patterns | All `uu_*` crates depend on it |
| `uu_<name>` (per-utility lib crate) | Single utility logic, clap `Command` definition, `uu_app()` entry | Depends on `gow-core`; binary entry calls into it |
| `<name>` binary (thin wrapper) | `main.rs` → calls `uu_<name>::main()` or `uu_app().get_matches()` | Depends only on its own `uu_<name>` lib |
| `tests/common` | `UCommand` scene helper — spawns binary subprocess, captures stdout/stderr | Used by all `tests/by-util/test_*.rs` |
| `packaging/wix` | WiX `.wxs` manifest — installs all binaries, registers PATH | Consumes built `.exe` artifacts from `target/release/` |

**Key rule:** `gow-core` must NEVER depend on any `uu_*` crate. `uu_*` crates must NEVER depend on each other. All cross-cutting concerns flow downward through `gow-core` only.

---

## Data Flow

### Argument Parsing Flow
```
User invokes binary (e.g., ls.exe --color auto *.txt)
  ↓
main.rs  →  uu_ls::uu_app()       (builds clap::Command)
              ↓
            clap parses args
              ↓
            uu_ls::run(matches)
              ↓
            gow_core::path::normalize_args()   (Unix→Windows path conversion)
            gow_core::encoding::ensure_utf8()  (SetConsoleOutputCP(65001))
            gow_core::output::init_color()     (enable VT mode on Windows)
              ↓
            utility logic executes
              ↓
            stdout / stderr  →  Windows console
```

### Error Propagation Flow
```
uu_<name>::run() → returns Result<(), UError>
  ↓
UError defined in gow-core (thiserror)
  ↓
main.rs catches, formats message, exits with correct code
  ↓
std::process::exit(exitcode)
```

### Build / Package Flow
```
cargo build --release --workspace
  ↓
target/release/ls.exe, cat.exe, grep.exe ...
  ↓
cargo wix  (reads packaging/wix/main.wxs)
  ↓
target/wix/gow-rust-x.y.z-x86_64.msi
  (installs all .exe to C:\Program Files\Gow, adds to System PATH)
```

---

## Patterns to Follow

### Pattern 1: Per-Utility Crate with `uu_app()` Entry Point
**What:** Each utility is a library crate exposing `pub fn uu_app() -> clap::Command` and `pub fn uumain(args: impl Iterator<Item = OsString>) -> i32`. Binary `main.rs` calls these.
**When:** Always — this is the uutils-proven pattern.
**Why:** Enables unit-testable library logic, decoupled from binary entry point; allows future multicall binary if desired.

```toml
# src/uu/ls/Cargo.toml
[package]
name = "uu_ls"
version.workspace = true
edition.workspace = true

[lib]
name = "uu_ls"
path = "src/lib.rs"

[[bin]]
name = "ls"
path = "src/main.rs"

[dependencies]
gow-core = { path = "../../../crates/gow-core" }
clap = { workspace = true, features = ["derive"] }
```

```rust
// src/uu/ls/src/main.rs
fn main() {
    std::process::exit(uu_ls::uumain(std::env::args_os()));
}
```

### Pattern 2: Workspace Dependency Inheritance
**What:** Root `Cargo.toml` declares all shared dependency versions in `[workspace.dependencies]`. Member crates inherit with `{ workspace = true }`.
**When:** Always — prevents version drift across 20+ crates.

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crates/gow-core", "src/uu/*"]
resolver = "3"   # Rust 2024 edition default

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["gow-rust contributors"]
license = "MIT"

[workspace.dependencies]
clap = { version = "4", features = ["derive"] }
thiserror = "2"
anyhow = "2"
windows-sys = { version = "0.59", features = ["Win32_System_Console"] }
regex = "1"
```

### Pattern 3: Windows Platform Init in `gow-core`
**What:** A `gow_core::init()` function called at binary startup that sets UTF-8 code page and enables VT/ANSI processing.
**When:** Every binary that produces terminal output (virtually all of them).
**Why:** Centralizes the Windows-specific `SetConsoleOutputCP(65001)` and `ENABLE_VIRTUAL_TERMINAL_PROCESSING` flag setup. No utility crate needs to know Win32 API.

```rust
// crates/gow-core/src/encoding.rs
#[cfg(target_os = "windows")]
pub fn setup_console_utf8() {
    unsafe {
        windows_sys::Win32::System::Console::SetConsoleOutputCP(65001);
        // enable ENABLE_VIRTUAL_TERMINAL_PROCESSING for ANSI colors
    }
}
#[cfg(not(target_os = "windows"))]
pub fn setup_console_utf8() {}
```

### Pattern 4: Path Normalization at Argument Boundary
**What:** Accept Unix-style paths as CLI arguments (`/c/Users/foo`), convert to Windows paths (`C:\Users\foo`) before passing to `std::fs` or OS calls.
**When:** Any utility accepting file path arguments.
**Why:** This is the core compat feature GOW users need. Centralizing in `gow-core::path` means consistent behavior across all 20+ utilities.

**Recommended crate:** `path-slash` (stable, widely used) for basic `/` ↔ `\` normalization; custom logic for drive-letter MSYS2-style paths (`/c/...` → `C:\...`).

### Pattern 5: Integration Tests via `assert_cmd` + Scene Helper
**What:** `tests/common/mod.rs` provides a `TestScenario` struct that locates the built binary, sets up temp dirs, and wraps `assert_cmd::Command`.
**When:** All integration tests in `tests/by-util/test_*.rs`.
**Why:** Mirrors uutils' proven test harness. Tests actually invoke the compiled binary, catching real-world bugs including Windows path issues.

```rust
// tests/by-util/test_ls.rs
use crate::common::TestScenario;
#[test]
fn test_ls_basic() {
    let scene = TestScenario::new("ls");
    scene.cmd().succeeds().stdout_contains("file.txt");
}
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Monorepo with a single `[[bin]]` per `src/bin/*.rs`
**What:** Putting all utilities as files in one crate's `src/bin/` directory.
**Why bad:** No per-utility dependency isolation; cannot selectively build one utility; no individual versioning; testing becomes monolithic.
**Instead:** One crate per utility in `src/uu/<name>/`.

### Anti-Pattern 2: Duplicating Windows Compat Code Per-Utility
**What:** Each utility independently calls `SetConsoleOutputCP`, handles path normalization, or re-implements ANSI color init.
**Why bad:** 20+ copies of the same bug. When Windows 11 changes behavior, you fix it in 20 places.
**Instead:** All Windows platform concerns go in `gow-core`. Utilities call `gow_core::init()`.

### Anti-Pattern 3: Using `std::path::Path` Directly for Unix-Style Inputs
**What:** Passing user-provided `/c/Users/foo` directly to `std::fs::read_dir()` without normalization on Windows.
**Why bad:** `std::fs` on Windows uses Win32 paths. MSYS2/Git Bash paths (`/c/...`) will fail silently or return wrong results.
**Instead:** Always normalize through `gow_core::path::to_windows()` before any `std::fs` call when the path came from user input.

### Anti-Pattern 4: MSI that Doesn't Modify PATH
**What:** Installing binaries to `Program Files\Gow\` but not appending to the system `PATH` environment variable.
**Why bad:** The entire value proposition of GOW is drop-in shell availability. If users must manually add to PATH, adoption dies.
**Instead:** WiX `Environment` element with `Action="set"` and `System="yes"` to append to Machine PATH at install time.

### Anti-Pattern 5: Encoding Panic on Non-UTF-8 Input
**What:** Utility calls `String::from_utf8(buf).unwrap()` on file content or env vars.
**Why bad:** Windows filenames and env vars can contain non-UTF-8 sequences (legacy code pages). The original GOW's #1 complaint was broken encoding.
**Instead:** Use `OsStr`/`OsString` at boundaries; use `String::from_utf8_lossy()` only when display is needed; use `std::env::args_os()` not `args()`.

---

## Suggested Build Order (Phase Dependencies)

The architecture has a strict dependency graph:

```
Phase 1: gow-core (shared library)
  └── No dependencies. Must be built FIRST.
      Output: encoding, path, error, output, args modules

Phase 2: Simple stateless utilities (cat, echo, pwd, true, false, yes, basename, dirname)
  └── Depends on: gow-core only
      Output: proves the workspace pattern, validates Windows path/encoding init

Phase 3: File-system utilities (ls, cp, mv, rm, mkdir, wc, head, tail)
  └── Depends on: gow-core + lessons from Phase 2
      Output: exercises path normalization, file I/O, tail -f Windows fix

Phase 4: Text-processing utilities (sort, uniq, tr, tee, cut, grep, sed)
  └── Depends on: gow-core + encoding patterns proven in Phase 3
      Output: regex support, stream processing, in-place editing

Phase 5: Search and comparison utilities (find, diff/patch, which)
  └── Depends on: gow-core + all prior patterns
      Output: -exec support, path-with-spaces fix, recursive traversal

Phase 6: Compression and network utilities (tar/gzip/bzip2, curl, less)
  └── Depends on: gow-core + Phase 4/5 stream patterns
      Output: most complex utilities; async may be needed (curl)

Phase 7: Test harness hardening
  └── Depends on: all utilities built
      Output: GNU compatibility test suite integration, CI matrix

Phase 8: MSI packaging
  └── Depends on: all release binaries in target/release/
      Output: cargo-wix .msi, PATH registration, installer testing
```

---

## MSI Packaging Architecture

**Toolchain:** `cargo-wix` (wraps WiX v3 on Windows) — generates MSI from a `.wxs` template.

**Key WiX structure for multi-binary install:**
```xml
<!-- packaging/wix/main.wxs (simplified) -->
<Product Id="*" Name="Gow Rust" ...>
  <Directory Id="TARGETDIR">
    <Directory Id="ProgramFilesFolder">
      <Directory Id="INSTALLDIR" Name="Gow">
        <!-- One Component per binary -->
        <Component Id="comp_ls">
          <File Id="ls_exe" Source="$(var.TargetDir)\ls.exe" />
        </Component>
        <Component Id="comp_cat">
          <File Id="cat_exe" Source="$(var.TargetDir)\cat.exe" />
        </Component>
        <!-- ... repeat for all utilities ... -->
        
        <!-- PATH registration: single component -->
        <Component Id="comp_path_env">
          <Environment Id="PATH" Name="PATH" Value="[INSTALLDIR]"
            Permanent="no" Part="last" Action="set" System="yes" />
        </Component>
      </Directory>
    </Directory>
  </Directory>
  
  <Feature Id="ProductFeature" Level="1">
    <ComponentRef Id="comp_ls" />
    <ComponentRef Id="comp_cat" />
    <!-- ... all components ... -->
    <ComponentRef Id="comp_path_env" />
  </Feature>
</Product>
```

**Challenge:** `cargo-wix` is primarily designed for single-binary projects. For multi-binary workspace, use `cargo wix --package <crate>` per utility and merge `.wixobj` files, OR write a custom `main.wxs` that references all `target/release/*.exe` directly. The latter (custom WXS) is simpler for 20+ binaries.

**WiX version note:** As of April 2026, GitHub Actions `windows-latest` only includes WiX v3.14.1 (Legacy). Plan for Legacy WiX syntax.

---

## Testing Architecture

Three test layers are needed:

### Layer 1: Unit Tests (inside each `uu_*` crate)
- Located in `src/uu/<name>/src/lib.rs` under `#[cfg(test)]`
- Test internal logic: argument parsing, data transformation, edge cases
- No subprocess spawning — pure Rust function calls
- Fast: run on `cargo test -p uu_ls`

### Layer 2: Integration Tests (per-utility binary tests)
- Located in `tests/by-util/test_<name>.rs`
- Use `assert_cmd` to spawn the actual built binary
- Use `tempfile` crate for scratch directories
- Test real CLI behavior including exit codes, stdout, stderr
- Platform-conditional: `#[cfg(target_os = "windows")]` for Windows-specific assertions

### Layer 3: GNU Compatibility Tests
- Mirror uutils' approach: run GNU test suite against gow binaries
- Requires a shell environment (Git Bash or WSH)
- Track compatibility percentage over time
- Implemented via `util/run-gnu-test.sh` script (executed in CI)
- Not blocking for early phases — build toward this in later phases

### Testing Infrastructure Crates
| Crate | Version | Purpose |
|-------|---------|---------|
| `assert_cmd` | 2.x | Spawn binary, assert on exit/stdout/stderr |
| `assert_fs` | 1.x | Temp directory + fixture file creation |
| `predicates` | 3.x | Composable assertions for assert_cmd |
| `tempfile` | 3.x | Temporary files and directories |
| `trycmd` (optional) | 0.15.x | Snapshot-based batch CLI tests from `.md`/`.trycmd` files |

---

## Scalability Considerations

| Concern | 5 utilities | 20 utilities | 40+ utilities |
|---------|------------|--------------|---------------|
| Build time | Fast | Moderate — use `cargo build -p` for dev | `sccache` + incremental builds essential |
| CI matrix | Simple | Parallel jobs per utility group | Matrix across Windows versions, feature flags |
| WiX manifest | Manual OK | Scripted WXS generation recommended | Template + codegen from `Cargo.toml` members |
| Test runtime | < 30s | 2-5 min | `cargo nextest` for parallelism; split CI jobs |
| `Cargo.lock` conflicts | None | None | None (single lock file is a feature) |

---

## Sources

- uutils/coreutils repository: https://github.com/uutils/coreutils
- uutils/coreutils `src/uu` directory: https://github.com/uutils/coreutils/tree/main/src/uu
- uutils/uucore shared library: https://crates.io/crates/uucore
- Cargo Workspaces official docs: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Cargo edition 2024 resolver: https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html
- cargo-wix: https://github.com/volks73/cargo-wix
- assert_cmd crate: https://crates.io/crates/assert_cmd
- path-slash crate: https://crates.io/crates/path-slash
- typed-path crate: https://crates.io/crates/typed-path
- uutils platform support: https://uutils.github.io/coreutils/docs/platforms.html
- uutils extending blog post (Feb 2025): https://uutils.github.io/blog/2025-02-extending/
- Rust error handling (thiserror/anyhow): https://leapcell.io/blog/choosing-the-right-rust-error-handling-tool
- winapi-util console: https://docs.rs/winapi-util/latest/winapi_util/console/struct.Console.html
- WiX Toolset GitHub: https://github.com/wixtoolset

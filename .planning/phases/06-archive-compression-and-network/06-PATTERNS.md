# Phase 06: archive-compression-and-network — Pattern Map

**Mapped:** 2026-04-28
**Files analyzed:** 25 (5 crates × 5 files each: Cargo.toml, build.rs, src/main.rs, src/lib.rs, tests/integration.rs)
**Analogs found:** 25 / 25

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `Cargo.toml` | config | — | `Cargo.toml` (self) | exact |
| `crates/gow-gzip/Cargo.toml` | config | — | `crates/gow-grep/Cargo.toml` | exact |
| `crates/gow-gzip/build.rs` | config | — | `crates/gow-grep/build.rs` | exact |
| `crates/gow-gzip/src/main.rs` | utility | — | `crates/gow-grep/src/main.rs` | exact |
| `crates/gow-gzip/src/lib.rs` | utility | file-I/O, transform | `crates/gow-cat/src/lib.rs` | role-match |
| `crates/gow-gzip/tests/gzip_tests.rs` | test | — | `crates/gow-cat/tests/integration.rs` | exact |
| `crates/gow-bzip2/Cargo.toml` | config | — | `crates/gow-grep/Cargo.toml` | exact |
| `crates/gow-bzip2/build.rs` | config | — | `crates/gow-grep/build.rs` | exact |
| `crates/gow-bzip2/src/main.rs` | utility | — | `crates/gow-grep/src/main.rs` | exact |
| `crates/gow-bzip2/src/lib.rs` | utility | file-I/O, transform | `crates/gow-cat/src/lib.rs` | role-match |
| `crates/gow-bzip2/tests/bzip2_tests.rs` | test | — | `crates/gow-cat/tests/integration.rs` | exact |
| `crates/gow-xz/Cargo.toml` | config | — | `crates/gow-grep/Cargo.toml` | exact |
| `crates/gow-xz/build.rs` | config | — | `crates/gow-grep/build.rs` | exact |
| `crates/gow-xz/src/main.rs` | utility | — | `crates/gow-grep/src/main.rs` | exact |
| `crates/gow-xz/src/lib.rs` | utility | file-I/O, transform | `crates/gow-cat/src/lib.rs` | role-match |
| `crates/gow-xz/tests/xz_tests.rs` | test | — | `crates/gow-cat/tests/integration.rs` | exact |
| `crates/gow-tar/Cargo.toml` | config | — | `crates/gow-grep/Cargo.toml` | exact |
| `crates/gow-tar/build.rs` | config | — | `crates/gow-grep/build.rs` | exact |
| `crates/gow-tar/src/main.rs` | utility | — | `crates/gow-grep/src/main.rs` | exact |
| `crates/gow-tar/src/lib.rs` | utility | file-I/O, transform, batch | `crates/gow-grep/src/lib.rs` (multi-file iteration) | role-match |
| `crates/gow-tar/tests/tar_tests.rs` | test | — | `crates/gow-cat/tests/integration.rs` | exact |
| `crates/gow-curl/Cargo.toml` | config | — | `crates/gow-tail/Cargo.toml` | role-match |
| `crates/gow-curl/build.rs` | config | — | `crates/gow-grep/build.rs` | exact |
| `crates/gow-curl/src/main.rs` | utility | — | `crates/gow-grep/src/main.rs` | exact |
| `crates/gow-curl/src/lib.rs` | utility | request-response | `crates/gow-cat/src/lib.rs` (file-open → stdout) | partial-match |
| `crates/gow-curl/tests/curl_tests.rs` | test | — | `crates/gow-grep/tests/integration.rs` | role-match |

---

## Pattern Assignments

### Root `Cargo.toml` (config)

**Analog:** `Cargo.toml` (self — extend in-place)

**Members block addition** (after `"crates/gow-less",` at line 46):
```toml
    # Phase 6 — archive, compression, and network (S06)
    "crates/gow-gzip",
    "crates/gow-bzip2",
    "crates/gow-xz",
    "crates/gow-tar",
    "crates/gow-curl",
```

**workspace.dependencies addition** (after `# Phase 5 additions` block, before `[profile.release]`):
```toml
# Phase 6 additions (S06 archive, compression, and network)
flate2 = "1.1"                       # gzip/deflate (miniz_oxide pure-Rust backend) — gow-gzip, gow-tar
bzip2 = "0.6"                        # bzip2 (libbz2-rs-sys pure-Rust backend) — gow-bzip2, gow-tar
liblzma = { version = "0.4", features = ["static"] }  # xz/lzma (MSVC-safe fork of xz2) — gow-xz
tar = "0.4"                          # tar archive create/extract — gow-tar
reqwest = { version = "0.13", features = ["blocking", "native-tls"], default-features = false }
                                     # HTTP client (Windows SChannel TLS) — gow-curl only
```

---

### `crates/gow-gzip/Cargo.toml` (config)

**Analog:** `crates/gow-grep/Cargo.toml` (lines 1–36) — verbatim template, swap names and deps.

**Full file pattern:**
```toml
[package]
name = "gow-gzip"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU gzip/gunzip/zcat — Windows port."

[[bin]]
name = "gzip"
path = "src/main.rs"

[lib]
name = "uu_gzip"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
flate2 = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

Note: `bstr`, `walkdir`, `regex`, `termcolor`, `windows-sys` are NOT needed for gzip — this crate does pure byte-stream I/O through flate2.

---

### `crates/gow-bzip2/Cargo.toml` (config)

**Analog:** `crates/gow-grep/Cargo.toml`

**Full file pattern:**
```toml
[package]
name = "gow-bzip2"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU bzip2/bunzip2 — Windows port."

[[bin]]
name = "bzip2"
path = "src/main.rs"

[lib]
name = "uu_bzip2"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
bzip2 = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

---

### `crates/gow-xz/Cargo.toml` (config)

**Analog:** `crates/gow-grep/Cargo.toml`

**Full file pattern:**
```toml
[package]
name = "gow-xz"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU xz/unxz — Windows port."

[[bin]]
name = "xz"
path = "src/main.rs"

[lib]
name = "uu_xz"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
liblzma = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

---

### `crates/gow-tar/Cargo.toml` (config)

**Analog:** `crates/gow-grep/Cargo.toml`

**Full file pattern:**
```toml
[package]
name = "gow-tar"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU tar — Windows port."

[[bin]]
name = "tar"
path = "src/main.rs"

[lib]
name = "uu_tar"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
tar = { workspace = true }
flate2 = { workspace = true }
bzip2 = { workspace = true }
walkdir = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

Note: tar composes flate2 + bzip2 for `-z`/`-j` codec wrappers. walkdir is used for `append_dir_all` traversal in `-c` create mode.

---

### `crates/gow-curl/Cargo.toml` (config)

**Analog:** `crates/gow-tail/Cargo.toml` (external async dep pattern — closest existing network-adjacent crate with a third-party specialised dep)

**Full file pattern:**
```toml
[package]
name = "gow-curl"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "HTTP/HTTPS client (curl replacement) — Windows port."

[[bin]]
name = "curl"
path = "src/main.rs"

[lib]
name = "uu_curl"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
reqwest = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

Note: reqwest blocking spawns an internal tokio runtime. Do NOT add `tokio` as an explicit dep. Do NOT use `#[tokio::main]`.

---

### All five `build.rs` files (config)

**Analog:** `crates/gow-grep/build.rs` (lines 1–13) — **verbatim copy, no changes**.

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

---

### All five `src/main.rs` files (utility)

**Analog:** `crates/gow-grep/src/main.rs` (lines 1–3) — 3-line pattern, swap library name.

```rust
// gow-gzip/src/main.rs
fn main() {
    std::process::exit(uu_gzip::uumain(std::env::args_os()));
}

// gow-bzip2/src/main.rs
fn main() {
    std::process::exit(uu_bzip2::uumain(std::env::args_os()));
}

// gow-xz/src/main.rs
fn main() {
    std::process::exit(uu_xz::uumain(std::env::args_os()));
}

// gow-tar/src/main.rs
fn main() {
    std::process::exit(uu_tar::uumain(std::env::args_os()));
}

// gow-curl/src/main.rs
fn main() {
    std::process::exit(uu_curl::uumain(std::env::args_os()));
}
```

---

### `crates/gow-gzip/src/lib.rs` (utility, file-I/O, transform)

**Analog:** `crates/gow-cat/src/lib.rs` — closest match for "open file or stdin → transform → stdout" pipeline with multiple operands and per-file error handling.

**uumain entry point pattern** (from `crates/gow-cat/src/lib.rs` lines 208–258, adapted):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args_vec: Vec<OsString> = args.into_iter().collect();

    // argv[0] mode switching — detect invocation name
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

    let matches = gow_core::args::parse_gnu(uu_app(), args_vec);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    match run(cli, &invoked_as) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("gzip: {e}");
            1
        }
    }
}
```

**File-or-stdin loop pattern** (from `crates/gow-cat/src/lib.rs` lines 222–257):
```rust
// open each operand; fall back to stdin when operands is empty or operand == "-"
if operands.is_empty() {
    if let Err(e) = process_stream(io::stdin().lock(), &mut stdout) {
        eprintln!("gzip: stdin: {e}");
        exit_code = 1;
    }
    return exit_code;
}
for op in &operands {
    if op == "-" {
        if let Err(e) = process_stream(io::stdin().lock(), &mut stdout) {
            eprintln!("gzip: -: {e}");
            exit_code = 1;
        }
        continue;
    }
    let converted = gow_core::path::try_convert_msys_path(op);
    match File::open(&converted) {
        Ok(f) => {
            if let Err(e) = process_stream(f, &mut stdout) {
                eprintln!("gzip: {converted}: {e}");
                exit_code = 1;
            }
        }
        Err(e) => {
            eprintln!("gzip: {converted}: {e}");
            exit_code = 1;
        }
    }
}
```

**Imports pattern** (adapt from `crates/gow-cat/src/lib.rs` lines 10–14):
```rust
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use anyhow::Result;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use flate2::read::{GzDecoder, MultiGzDecoder};
use flate2::write::GzEncoder;
use flate2::Compression;
```

---

### `crates/gow-bzip2/src/lib.rs` (utility, file-I/O, transform)

**Analog:** `crates/gow-cat/src/lib.rs` — same file-or-stdin loop shape.

**Imports pattern:**
```rust
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};

use anyhow::Result;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use bzip2::read::{BzDecoder, MultiBzDecoder};
use bzip2::write::BzEncoder;
use bzip2::Compression;
```

**argv[0] mode detection** — identical shape to gow-gzip: check `invoked_as == "bunzip2"` to switch to decompress mode.

**Core transform pattern** (from RESEARCH.md Pattern 4):
```rust
// compress path
let mut encoder = BzEncoder::new(output, Compression::default());
io::copy(&mut input, &mut encoder)?;
encoder.finish()?;

// decompress path — use MultiBzDecoder for real-world multi-stream files
let mut decoder = MultiBzDecoder::new(input);
io::copy(&mut decoder, &mut output)?;
```

---

### `crates/gow-xz/src/lib.rs` (utility, file-I/O, transform)

**Analog:** `crates/gow-cat/src/lib.rs` — same file-or-stdin loop shape.

**Imports pattern:**
```rust
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};

use anyhow::Result;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use liblzma::read::XzDecoder;
use liblzma::write::XzEncoder;
```

**argv[0] mode detection** — check `invoked_as == "unxz"` to switch to decompress mode.

**Core transform pattern** (from RESEARCH.md Pattern 5):
```rust
// compress (level 0-9, default 6 matches xz CLI default)
let mut encoder = XzEncoder::new(output, 6);
io::copy(&mut input, &mut encoder)?;
encoder.finish()?;

// decompress
let mut decoder = XzDecoder::new(input);
io::copy(&mut decoder, &mut output)?;
```

---

### `crates/gow-tar/src/lib.rs` (utility, file-I/O, transform, batch)

**Analog:** `crates/gow-grep/src/lib.rs` — closest match for a utility that iterates over multiple file/directory paths, handles recursive traversal, and has distinct operation modes (-c/-x/-t).

**uumain + mode-dispatch pattern** (from `crates/gow-grep/src/lib.rs` lines 90–105, adapted):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("tar: {e}");
            2
        }
    }
}
```

**Imports pattern:**
```rust
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use bzip2::Compression as BzCompression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression as GzCompression;
use tar::{Archive, Builder};
```

**Multi-path iteration pattern** (from `crates/gow-grep/src/lib.rs` lines 133–189, adapted for create mode):
```rust
// -c create: iterate paths, append each to builder
for path in &cli.paths {
    let converted = gow_core::path::try_convert_msys_path(&path.to_string_lossy());
    let p = Path::new(&converted);
    if p.is_dir() {
        builder.append_dir_all(p.file_name().unwrap_or_default(), p)?;
    } else {
        let mut f = File::open(p)?;
        let mut header = tar::Header::new_gnu();
        header.set_metadata(&f.metadata()?);
        header.set_path(p.file_name().unwrap_or_default())?;
        header.set_cksum();
        builder.append(&header, &mut f)?;
    }
}
```

**IMPORTANT tar pitfall** (from RESEARCH.md):
```rust
// Always set follow_symlinks(false) — tar crate default is true (dereference),
// but GNU tar default is false (store symlink entry). Mismatch = wrong archives.
builder.follow_symlinks(false);
```

---

### `crates/gow-curl/src/lib.rs` (utility, request-response)

**Analog:** `crates/gow-cat/src/lib.rs` — closest shape: open source (URL vs file) → stream to stdout or `-o` output file. No direct analog for HTTP client exists in the codebase.

**uumain pattern** (from `crates/gow-cat/src/lib.rs` lines 208–220, adapted):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("curl: {e}");
            1
        }
    }
}
```

**Imports pattern:**
```rust
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};

use anyhow::Result;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use reqwest::blocking::ClientBuilder;
use reqwest::Proxy;
```

**Core request + output pattern** (from RESEARCH.md Pattern 6):
```rust
fn run(cli: Cli) -> Result<()> {
    let mut client_builder = ClientBuilder::new();
    if let Some(proxy_url) = &cli.proxy {
        // Use Proxy::all() to match curl -x behavior (proxy ALL protocols)
        client_builder = client_builder.proxy(Proxy::all(proxy_url)?);
    }
    if cli.insecure {
        // Only if -k/--insecure flag is explicitly passed
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    let client = client_builder.build()?;

    let mut response = client.get(&cli.url).send()?;
    let status = response.status();

    if let Some(output_path) = &cli.output {
        let mut file = File::create(output_path)?;
        io::copy(&mut response, &mut file)?;
    } else {
        let bytes = response.bytes()?;
        io::stdout().write_all(&bytes)?;
    }

    if !status.is_success() {
        eprintln!("curl: HTTP {status}");
        return Err(anyhow::anyhow!("HTTP error {status}"));
    }
    Ok(())
}
```

**IMPORTANT anti-pattern to avoid** (from RESEARCH.md Pitfall 3):
- Do NOT annotate `fn main()` with `#[tokio::main]`
- Do NOT add `tokio` to Cargo.toml
- `reqwest::blocking::Client` spawns an internal tokio runtime automatically

---

### Integration test files (test pattern)

**Analog:** `crates/gow-cat/tests/integration.rs` (lines 1–60) — round-trip tempfile pattern; `crates/gow-grep/tests/integration.rs` (lines 1–80) — assert_cmd + predicates pattern.

**Test file template pattern:**
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn gzip() -> Command {
    Command::cargo_bin("gzip").expect("gzip binary not found — run `cargo build -p gow-gzip` first")
}

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}
```

**Round-trip test pattern** (specific to compression crates):
```rust
#[test]
fn compress_decompress_roundtrip() {
    let tmp = tempdir().unwrap();
    let original = b"hello world\nline 2\n";
    let input_path = write_fixture(tmp.path(), "input.txt", original);
    let compressed_path = tmp.path().join("input.txt.gz");
    let output_path = tmp.path().join("output.txt");

    // compress
    gzip()
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();
    assert!(compressed_path.exists());

    // decompress
    Command::cargo_bin("gunzip").unwrap()
        .arg(compressed_path.to_str().unwrap())
        .assert()
        .success();

    let recovered = fs::read(&output_path).unwrap();
    assert_eq!(recovered, original);
}
```

**Network test pattern** (gow-curl — mark `#[ignore]` by default):
```rust
#[test]
#[ignore = "requires network access"]
fn get_httpbin_returns_200() {
    Command::cargo_bin("curl").unwrap()
        .arg("http://httpbin.org/get")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"url\""));
}
```

---

## Shared Patterns

### gow_core::init() — call as first line of every uumain
**Source:** `crates/gow-core/src/lib.rs` (lines 16–19)
**Apply to:** All five new crates — `gow-gzip`, `gow-bzip2`, `gow-xz`, `gow-tar`, `gow-curl`

```rust
pub fn init() {
    encoding::setup_console_utf8();  // SetConsoleOutputCP(65001)
    color::enable_vt_mode();         // ENABLE_VIRTUAL_TERMINAL_PROCESSING
}
// Call: gow_core::init(); — first line of every uumain
```

### gow_core::args::parse_gnu — clap argument parsing
**Source:** `crates/gow-grep/src/lib.rs` (lines 92–94)
**Apply to:** All five new crates

```rust
let matches = gow_core::args::parse_gnu(Cli::command(), args);
let cli = Cli::from_arg_matches(&matches).unwrap();
```

### gow_core::path::try_convert_msys_path — MSYS/Unix path conversion
**Source:** `crates/gow-cat/src/lib.rs` (line 242)
**Apply to:** `gow-gzip`, `gow-bzip2`, `gow-xz`, `gow-tar` — any crate that accepts file path arguments

```rust
let converted = gow_core::path::try_convert_msys_path(op);
let path = Path::new(&converted);
```

### Per-file error handling — print error and continue
**Source:** `crates/gow-cat/src/lib.rs` (lines 240–255) and `crates/gow-grep/src/lib.rs` (lines 155–188)
**Apply to:** All crates that accept multiple file operands (`gow-gzip`, `gow-bzip2`, `gow-xz`, `gow-tar`)

```rust
// Pattern: on per-file error, print to stderr and set exit_code = 1, then continue.
// Never early-return from the operand loop on a single file error.
Err(e) => {
    eprintln!("{utility}: {path}: {e}");
    exit_code = 1;
}
```

### GNU error message format
**Source:** `crates/gow-grep/src/lib.rs` (lines 102–104) and `crates/gow-cat/src/lib.rs` (line 228)
**Apply to:** All five new crates

```rust
// Format: "<utility>: <message>" to stderr
eprintln!("gzip: {e}");
// With path: "<utility>: <path>: <message>"
eprintln!("gzip: {converted}: {e}");
```

### Clap #[derive(Parser)] struct shape
**Source:** `crates/gow-grep/src/lib.rs` (lines 21–88)
**Apply to:** All five new crates (each defines its own `Cli` struct)

```rust
#[derive(Parser, Debug)]
#[command(
    name = "gzip",
    about = "GNU gzip — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,
    // ... utility-specific flags ...
}
```

### Workspace dependency inheritance
**Source:** `crates/gow-grep/Cargo.toml` (lines 2–7)
**Apply to:** All five new crates — every shared dep uses `{ workspace = true }`

```toml
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
```

---

## No Analog Found

All five crates have close structural analogs. However, two capabilities have no codebase precedent:

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `crates/gow-curl/src/lib.rs` (HTTP request logic) | utility | request-response | No HTTP client exists in codebase yet; use RESEARCH.md Pattern 6 (reqwest blocking) |
| `crates/gow-xz/src/lib.rs` (liblzma calls) | utility | file-I/O, transform | No xz/lzma code in codebase; use RESEARCH.md Pattern 5 (liblzma XzEncoder/XzDecoder) |

For both, the RESEARCH.md code examples are authoritative — they were verified against docs.rs and crates.io.

---

## Metadata

**Analog search scope:** `crates/gow-grep/`, `crates/gow-cat/`, `crates/gow-wc/`, `crates/gow-tail/`, `crates/gow-find/`, `crates/gow-core/`, root `Cargo.toml`
**Files scanned:** 15 source files + 2 integration test files + 4 Cargo.toml files
**Pattern extraction date:** 2026-04-28

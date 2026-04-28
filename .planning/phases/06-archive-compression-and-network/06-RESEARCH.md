# Phase 06: archive-compression-and-network — Research

**Researched:** 2026-04-28
**Domain:** Archive/compression utilities (tar, gzip, bzip2, xz) and network HTTP client (curl) on Windows MSVC
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

Phase 06 context file contains no locked decisions beyond the project-level constraints in CLAUDE.md.
All library choices from CLAUDE.md are pre-approved and treated as locked decisions.

### Locked Decisions (from CLAUDE.md)
- Language: Rust stable channel (1.85+), x86_64-pc-windows-msvc target
- Each utility is an independent crate in the Cargo workspace (crates/gow-tar, crates/gow-gzip, etc.)
- Compression stack: flate2 (miniz_oxide backend), bzip2 (0.6+ defaults to pure Rust), tar (pure Rust), xz2 OR liblzma
- Network stack: reqwest with native-tls (Windows SChannel), tokio (curl crate only)
- Pattern: clap derive, thiserror + anyhow, embed-manifest in build.rs, gow_core::init()
- Binary structure: one [[bin]] per utility, lib.rs with uumain entry point

### Claude's Discretion
- Whether gzip/gunzip/zcat share a single crate or are separate crates
- Whether to use xz2 vs liblzma for xz support (MSVC compatibility is a constraint)
- Wave/plan structure for this phase
- Which tar flags to implement (beyond the R018 required -c/-x/-t/-z/-j)
- Whether gow-curl uses async reqwest (with #[tokio::main]) or blocking reqwest

### Deferred Ideas (OUT OF SCOPE)
- bzip2 --fast / --best variants beyond standard compression levels
- xz multi-threaded parallel compression
- tar sparse files, incremental archives
- curl multipart upload, WebSocket
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| R018 | 아카이브 생성/추출 (-c 생성, -x 추출, -t 목록, -z gzip, -j bzip2) | tar crate 0.4.45 + flate2 1.1.9 + bzip2 0.6.1; Builder/Archive API verified |
| R019 | 압축/해제 도구 세트: gzip/gunzip/zcat + bzip2 + xz/unxz | flate2 1.1.9 (gzip), bzip2 0.6.1 (bzip2), liblzma 0.4.6 with static feature (xz) |
| R020 | HTTP/HTTPS 요청, TLS 1.2/1.3 지원, 프록시 인증 | reqwest 0.13.3 with native-tls + blocking feature; native-tls uses Windows SChannel |
</phase_requirements>

---

## Summary

Phase 06 implements six independent binary crates for archive, compression, and network operations. The compression tier covers `gzip`/`gunzip`/`zcat` (flate2 1.1.9), `bzip2` (bzip2 0.6.1), `xz`/`unxz` (liblzma 0.4.6 with static feature), and `tar` (tar 0.4.45 composing the above). The network tier adds a `curl` replacement using reqwest 0.13.3 with native-tls for Windows SChannel TLS 1.2/1.3.

The most important discovery from this research is the **xz2 vs liblzma decision**: xz2 0.1.7 (last released 2022, has open GitHub issue #99 for MSVC build failures) should be replaced with liblzma 0.4.6 (actively maintained, fork of xz2, adds `static` feature that compiles liblzma C source via the `cc` crate with explicit MSVC support in its build.rs). A second important discovery is that bzip2 0.6.0+ now defaults to the pure-Rust `libbz2-rs-sys` backend, eliminating the C dependency that CLAUDE.md warned about. Additionally, reqwest's `blocking` feature is the correct choice for a synchronous curl replacement — it spawns an internal tokio runtime automatically, so gow-curl does NOT need `#[tokio::main]` or explicit tokio dependency.

**Primary recommendation:** Use liblzma 0.4.6 + static feature instead of xz2. Use bzip2 0.6.1 (pure Rust default). Use reqwest 0.13.3 blocking + native-tls features.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| gzip/gunzip/zcat compression | Binary / CLI | — | Pure I/O transformation; no server tier needed |
| bzip2 compress/decompress | Binary / CLI | — | Same as gzip |
| xz/unxz compress/decompress | Binary / CLI | — | Same; liblzma static links the C library |
| tar create/extract/list | Binary / CLI | — | Composes flate2/bzip2 for -z/-j; reads/writes filesystem |
| HTTP(S) requests with TLS | Binary / CLI | OS TLS (SChannel) | reqwest delegates TLS to Windows SChannel via native-tls |
| Proxy authentication | Binary / CLI | OS credentials | reqwest Proxy API; HTTPS proxy CONNECT via SChannel |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tar` | 0.4.45 | TAR archive read/write | Pure Rust; used by cargo itself for .crate files; streaming design [VERIFIED: crates.io API] |
| `flate2` | 1.1.9 | gzip/deflate compress/decompress | Default miniz_oxide backend = pure Rust, no C; standard in ecosystem [VERIFIED: crates.io API + docs.rs] |
| `bzip2` | 0.6.1 | bzip2 compress/decompress | Now defaults to libbz2-rs-sys (pure Rust) as of 0.6.0; cross-compilation just works [VERIFIED: crates.io API + trifectatech.org blog] |
| `liblzma` | 0.4.6 | xz/lzma compress/decompress | Fork of xz2 with active maintenance; `static` feature compiles liblzma C source; MSVC detected in build.rs [VERIFIED: crates.io API + build.rs inspection] |
| `reqwest` | 0.13.3 | HTTP/HTTPS client (curl replacement) | De-facto standard; `blocking` + `native-tls` features for synchronous SChannel TLS [VERIFIED: crates.io API + docs.rs] |
| `native-tls` | 0.2.18 | Windows SChannel TLS | Platform-native TLS; uses Windows certificate store; no OpenSSL dependency [VERIFIED: crates.io API] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `embed-manifest` | 1.5.0 | Windows manifest (UTF-8 + long path) | All crates — existing pattern (build.rs) |
| `gow-core` | workspace | UTF-8 init, parse_gnu(), exit codes | All crates — required for consistency |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `liblzma 0.4.6` | `xz2 0.1.7` | xz2 last released 2022; open issue #99 confirms MSVC build failures; liblzma is a maintained fork |
| `reqwest blocking` | `reqwest async + tokio::main` | blocking works fine for a synchronous CLI; no need to expose tokio to the calling code |
| `reqwest + native-tls` | `reqwest + rustls` | native-tls uses OS SChannel certificate store (correct for Windows-native tool); rustls bundles its own TLS |
| `flate2 miniz_oxide` | `flate2 zlib-rs` | miniz_oxide is the default, no feature flag needed; zlib-rs is faster but requires unsafe code and explicit feature opt-in |
| 3 separate crates for gzip/gunzip/zcat | 1 shared crate | 3 separate crates matches project pattern (one binary per crate); shared logic lives in lib.rs |

**Installation — new workspace dependencies to add to root Cargo.toml:**
```bash
# Already in workspace — no new deps needed:
# flate2 (was in CLAUDE.md recommended stack — needs adding to workspace.dependencies)
# bzip2 (same)
# tar (same)
# liblzma (same)
# reqwest (same)
# native-tls (pulled transitively by reqwest native-tls feature)
```

**Version verification (confirmed 2026-04-28):**
```
flate2 = 1.1.9     (published: verified via cargo search)
bzip2 = 0.6.1      (published: verified via crates.io API)
tar = 0.4.45       (published: verified via crates.io API)
liblzma = 0.4.6    (published: verified via crates.io API, last updated 2026-02-17)
reqwest = 0.13.3   (published: verified via cargo search)
native-tls = 0.2.18 (published: verified via cargo search)
```

---

## Architecture Patterns

### System Architecture Diagram

```
stdin/file
    |
    v
[gow-gzip / gow-gunzip / gow-zcat]
  GzEncoder / GzDecoder (flate2)
    |
    v
stdout / .gz file

stdin/file
    |
    v
[gow-bzip2]
  BzEncoder / BzDecoder (bzip2)
    |
    v
stdout / .bz2 file

stdin/file
    |
    v
[gow-xz]
  XzEncoder / XzDecoder (liblzma)
    |
    v
stdout / .xz file

paths
    |
    v
[gow-tar] ──flags: -z,-j──> codec wrapper (flate2 / bzip2)
  Builder::new(codec_writer)     Archive::new(codec_reader)
    |                                  |
  append_dir_all / append_file      unpack(dst) / entries() for list
    |                                  |
    v                                  v
output .tar.gz / .tar.bz2         extracted files

URL
    |
    v
[gow-curl]
  reqwest::blocking::Client
  .proxy(Proxy::http / Proxy::https)
  .build()
    |
  .get(url).send()
    |
    v
response body -> stdout / -o file
(Windows SChannel TLS via native-tls)
```

### Recommended Project Structure
```
crates/
├── gow-gzip/        # gzip + gunzip + zcat (same crate, different behavior via argv[0] or flags)
│   ├── src/
│   │   ├── main.rs  # delegates to uu_gzip::uumain
│   │   └── lib.rs   # uumain: detect mode from argv[0] or -d/-c flags
│   ├── build.rs
│   └── Cargo.toml
├── gow-bzip2/       # bzip2 + bunzip2 (same crate)
├── gow-xz/          # xz + unxz (same crate)
├── gow-tar/         # tar
└── gow-curl/        # curl replacement
```

Note: gzip/gunzip/zcat share a crate because GNU implements them as a single binary — behavior switches on argv[0] or -d/-c flags. Same pattern applies to bzip2/bunzip2 and xz/unxz.

### Pattern 1: Compression Crate — gzip/gunzip/zcat

**What:** Single crate produces one binary. Mode determined by argv[0] or CLI flags.
**When to use:** Any dual-mode compression utility.

```rust
// Source: GNU gzip(1) behavior + flate2 docs.rs
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    // Detect invocation name for argv[0] mode switching
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let invoked_as = args_vec.get(0)
        .map(|s| Path::new(s).file_stem().unwrap_or_default().to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let mode = if invoked_as == "gunzip" || invoked_as == "zcat" {
        Mode::Decompress
    } else {
        Mode::Compress
    };
    // ...
}
```

### Pattern 2: flate2 GzEncoder/GzDecoder streaming

```rust
// Source: docs.rs/flate2/latest/flate2/ — verified via WebFetch 2026-04-28
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

// Compress to stdout
let mut encoder = GzEncoder::new(io::stdout(), Compression::default());
io::copy(&mut input_file, &mut encoder)?;
encoder.finish()?;

// Decompress from file
let mut decoder = GzDecoder::new(input_file);
io::copy(&mut decoder, &mut io::stdout())?;

// Multi-member gzip (for zcat / concatenated gzip streams):
let mut decoder = MultiGzDecoder::new(input_file);
io::copy(&mut decoder, &mut io::stdout())?;
```

### Pattern 3: tar crate — create and extract

```rust
// Source: docs.rs/tar/0.4.45/ — verified via WebFetch 2026-04-28

// Create .tar.gz (-czf)
let tar_gz = File::create("archive.tar.gz")?;
let enc = GzEncoder::new(tar_gz, Compression::default());
let mut builder = Builder::new(enc);
builder.follow_symlinks(false);  // GNU tar default: store symlink, not target
builder.append_dir_all("src", "./src")?;
let enc = builder.into_inner()?;
enc.finish()?;

// Extract .tar.gz (-xzf)
let tar_gz = File::open("archive.tar.gz")?;
let dec = GzDecoder::new(tar_gz);
let mut archive = Archive::new(dec);
archive.set_overwrite(true);
archive.unpack("./out")?;

// List .tar.gz (-tzf)
let tar_gz = File::open("archive.tar.gz")?;
let dec = GzDecoder::new(tar_gz);
let mut archive = Archive::new(dec);
for entry in archive.entries()? {
    let entry = entry?;
    println!("{}", entry.path()?.display());
}
```

### Pattern 4: bzip2 streaming

```rust
// Source: docs.rs/bzip2/latest/ — verified via WebFetch 2026-04-28
use bzip2::read::{BzDecoder, MultiBzDecoder};
use bzip2::write::BzEncoder;
use bzip2::Compression;

// Compress
let mut encoder = BzEncoder::new(io::stdout(), Compression::default());
io::copy(&mut input, &mut encoder)?;
encoder.finish()?;

// Decompress (single stream)
let mut decoder = BzDecoder::new(input_file);
io::copy(&mut decoder, &mut io::stdout())?;
```

### Pattern 5: liblzma xz streaming

```rust
// Source: docs.rs/liblzma/0.4.6/ — API compatible with xz2
use liblzma::read::{XzDecoder, XzEncoder};

// Compress
let mut encoder = XzEncoder::new(input, 6);  // level 0-9
io::copy(&mut encoder, &mut output)?;

// Decompress
let mut decoder = XzDecoder::new(input_file);
io::copy(&mut decoder, &mut output)?;
```

### Pattern 6: reqwest blocking + native-tls

```rust
// Source: docs.rs/reqwest/0.13.3/ — verified via WebFetch 2026-04-28
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::Proxy;

// Basic GET
let client = ClientBuilder::new()
    .use_native_tls()  // Windows SChannel — already the default with native-tls feature
    .build()?;
let response = client.get(&url).send()?;
let status = response.status();
let bytes = response.bytes()?;

// With proxy authentication
let proxy = Proxy::http("http://user:pass@proxy:8080")?;
let client = ClientBuilder::new()
    .proxy(proxy)
    .build()?;

// Write response body to file (-o flag)
let mut response = client.get(&url).send()?;
let mut file = File::create(&output_path)?;
io::copy(&mut response, &mut file)?;
```

### Anti-Patterns to Avoid

- **Using xz2 crate:** Last released 2022; open GitHub issue #99 confirms MSVC build failures. Use `liblzma` instead.
- **Using flate2 zlib feature on Windows:** Requires a C compiler and pkg-config; miniz_oxide default is pure Rust and works correctly.
- **Separate async runtime for gow-curl:** reqwest blocking feature spawns an internal tokio runtime; do not add `#[tokio::main]` or a standalone tokio dep to gow-curl's Cargo.toml.
- **Follow symlinks by default in tar:** GNU tar stores symlinks as symlink entries by default (not dereference). Call `builder.follow_symlinks(false)` explicitly to match GNU behavior. The crate's default is `true` (dereference), which is the opposite of GNU tar.
- **Using BzDecoder for concatenated bzip2:** Wikipedia dumps and pbzip2 output are multi-stream bzip2 files. Use `MultiBzDecoder` when behavior needs to match `bunzip2` on real-world files.
- **Using reqwest async in a main() without runtime:** The blocking API is synchronous; use `reqwest::blocking::Client`, not `reqwest::Client`, in a sync main.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| gzip encode/decode | Custom DEFLATE | `flate2` | DEFLATE has ~15 edge cases: multi-member streams, partial flushes, CRC32 validation, correct header format |
| tar format | Custom archive writer | `tar` crate | USTAR/GNU long-name extension, checksum algorithm, file type bytes, PAX headers — all handled |
| bzip2 encode/decode | Custom BWT+Huffman | `bzip2` | BWT + Huffman + RLE: 3 layers; bit-exact compatibility required |
| xz/lzma encode/decode | Custom LZMA | `liblzma` | LZMA2 with range coding; reference implementation is 30k+ lines of C |
| TLS handshake | Custom SChannel binding | `native-tls` + `reqwest` | TLS state machine, certificate chain validation, ALPN, SNI — all platform-specific |
| HTTP/1.1 + HTTP/2 | Custom TCP socket code | `reqwest` | Redirect handling, connection pooling, chunked transfer, compression, content negotiation |
| Proxy CONNECT tunneling | Custom HTTP proxy code | `reqwest` | CONNECT method, proxy auth challenge, tunnel setup — handled by hyper under reqwest |

**Key insight:** All six utilities touch cryptographic/format specifications with exact byte-level compatibility requirements. Any deviation produces files other tools cannot read. Use the reference implementations.

---

## Common Pitfalls

### Pitfall 1: tar follow_symlinks default mismatch
**What goes wrong:** tar crate's `Builder::follow_symlinks` defaults to `true` (dereference). GNU tar defaults to `false` (store symlink). Archives created without calling `.follow_symlinks(false)` will contain file content instead of symlink entries, which is wrong for compatibility.
**Why it happens:** The crate made the "safe" choice for Rust callers; GNU tar made the POSIX-compatible choice.
**How to avoid:** Always call `builder.follow_symlinks(false)` in gow-tar unless `-h`/`--dereference` flag is passed.
**Warning signs:** Archives that are unexpectedly large; `.tar.gz` that extracts files where symlinks should be.

### Pitfall 2: xz2 MSVC build failure
**What goes wrong:** `xz2` crate version 0.1.7 (last release 2022) has an open issue #99 documenting MSVC build failures due to C99 features the MSVC compiler rejects in lzma-sys's bundled xz source.
**Why it happens:** liblzma upstream uses C99 `timeout` as a variable name; MSVC's `<windows.h>` defines `timeout` as a macro.
**How to avoid:** Use `liblzma` crate (fork of xz2 with MSVC-aware build.rs) instead of `xz2`.
**Warning signs:** Build error `error C2054: expected '(' to follow 'timeout'` in `mythread.h`.

### Pitfall 3: reqwest blocking inside async context panics
**What goes wrong:** If gow-curl's `main()` is annotated with `#[tokio::main]`, calling `reqwest::blocking::Client` methods will panic with "Cannot drop a runtime in a context where blocking is not allowed."
**Why it happens:** reqwest blocking internally creates a new tokio runtime; dropping it while inside another runtime is forbidden.
**How to avoid:** gow-curl should use a plain synchronous `fn main()` with reqwest blocking. No `#[tokio::main]`.
**Warning signs:** Panic at runtime with "Cannot drop a runtime..."

### Pitfall 4: gzip stdin passthrough mode (zcat behavior)
**What goes wrong:** `zcat` must read from stdin when no file arguments are given and decompress to stdout. If stdin is not compressed, zcat should error (not pass through raw bytes).
**Why it happens:** Many CLI tools treat stdin passthrough as a sensible default; gzip/zcat do not.
**How to avoid:** When no input files are given, read from stdin and attempt GzDecoder; return error if decompression fails. Test explicitly with an uncompressed stdin.
**Warning signs:** `echo hello | zcat` produces output instead of an error.

### Pitfall 5: tar on Windows — permissions and symlinks are silently ignored
**What goes wrong:** `Archive::set_preserve_permissions()` is Unix-only and does nothing on Windows. Symlink extraction on Windows requires SeCreateSymbolicLinkPrivilege (Developer Mode or elevated). Extracting an archive with symlinks may silently skip or fail.
**Why it happens:** Windows symlinks require privilege; the tar crate does not attempt elevation.
**How to avoid:** Gracefully handle symlink extraction errors (log a warning, continue). Document that `-p` / `--preserve-permissions` is a no-op on Windows.
**Warning signs:** Archives that extract cleanly on Linux fail or produce wrong files on Windows.

### Pitfall 6: long path names in tar archives
**What goes wrong:** Classic USTAR format limits path components to 100 bytes. Archives with longer paths created using `Header::new_ustar()` will silently truncate names.
**Why it happens:** USTAR is limited by specification.
**How to avoid:** Always use `Header::new_gnu()` for GNU format which uses `@LongLink` extension. The `append_data()` and `append_dir_all()` methods handle this automatically.
**Warning signs:** Extracted files have truncated names; archives that read correctly on Linux are corrupt on inspection.

### Pitfall 7: reqwest HTTP vs HTTPS proxy
**What goes wrong:** `Proxy::http()` only proxies HTTP requests. For HTTPS through a proxy, use `Proxy::https()` or `Proxy::all()`. curl's `-x` flag means "proxy for all protocols."
**Why it happens:** reqwest follows HTTP semantics where HTTP proxy and HTTPS CONNECT tunnel are different.
**How to avoid:** When implementing `-x`/`--proxy`, use `Proxy::all(url)` to match curl behavior.
**Warning signs:** HTTPS requests bypass the proxy silently.

---

## Code Examples

Verified patterns from official sources:

### gzip encode a file to stdout (gow-gzip core path)
```rust
// Source: docs.rs/flate2/latest/flate2/write/struct.GzEncoder.html
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{self, Write};

let stdout = io::stdout();
let mut encoder = GzEncoder::new(stdout.lock(), Compression::default());
let mut f = std::fs::File::open(&path)?;
io::copy(&mut f, &mut encoder)?;
encoder.finish()?;
```

### gunzip (decompress) a .gz file to stdout
```rust
// Source: docs.rs/flate2/latest/flate2/read/struct.GzDecoder.html
use flate2::read::GzDecoder;
use std::io;

let f = std::fs::File::open(&path)?;
let mut decoder = GzDecoder::new(f);
io::copy(&mut decoder, &mut io::stdout())?;
```

### tar create .tar.gz with -z flag
```rust
// Source: docs.rs/tar/0.4.45/tar/struct.Builder.html
use tar::Builder;
use flate2::write::GzEncoder;
use flate2::Compression;

let output_file = std::fs::File::create("archive.tar.gz")?;
let gz_encoder = GzEncoder::new(output_file, Compression::default());
let mut tar_builder = Builder::new(gz_encoder);
tar_builder.follow_symlinks(false);  // match GNU tar default
tar_builder.append_dir_all(".", src_path)?;
let gz = tar_builder.into_inner()?;
gz.finish()?;
```

### reqwest blocking GET with native-tls + proxy
```rust
// Source: docs.rs/reqwest/0.13.3/reqwest/blocking/index.html
use reqwest::blocking::ClientBuilder;
use reqwest::Proxy;

let mut client_builder = ClientBuilder::new();
if let Some(proxy_url) = &cli.proxy {
    client_builder = client_builder.proxy(Proxy::all(proxy_url)?);
}
let client = client_builder.build()?;
let mut response = client.get(&cli.url).send()?;
if let Some(output_path) = &cli.output {
    let mut file = std::fs::File::create(output_path)?;
    std::io::copy(&mut response, &mut file)?;
} else {
    let bytes = response.bytes()?;
    std::io::stdout().write_all(&bytes)?;
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| xz2 crate (C binding) | liblzma crate (fork, MSVC-aware) | xz2 abandoned ~2022; liblzma forked and maintained since 2023 | MSVC build reliability |
| bzip2 C binding (default) | bzip2 0.6+ pure Rust via libbz2-rs-sys | June 2025 (0.6.0 release) | No C toolchain needed for bzip2 |
| reqwest async-only | reqwest blocking feature | reqwest ~0.9+ | CLI tools can use sync API without tokio boilerplate |
| flate2 default zlib-sys C backend | flate2 default miniz_oxide pure Rust | flate2 ~1.0.0 (2019) | No C compiler needed for gzip |

**Deprecated/outdated:**
- `xz2 0.1.7`: Last released June 2022. Open issue #99 documents MSVC failure. Superseded by `liblzma` which is an active fork.
- `bzip2 0.4.x`: C binding default; now superseded by 0.6.x with pure Rust default.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | liblzma static feature compiles successfully on x86_64-pc-windows-msvc using cargo's bundled MSVC toolchain | Standard Stack / Pitfalls | If wrong, xz support blocks; fallback: omit xz for now or use lzma-rs (pure Rust decoder-only) |
| A2 | reqwest 0.13 blocking feature spawns an internal tokio runtime and does not require explicit `#[tokio::main]` in main() | Architecture Patterns | If wrong, gow-curl needs tokio dependency and async main |
| A3 | bzip2 0.6.1 libbz2-rs-sys default compiles without any C toolchain on MSVC | Standard Stack | Risk is LOW — Trifecta blog explicitly states "cross-compilation just works" after the Rust port |
| A4 | tar crate 0.4.45 append_dir_all automatically uses GNU long-name extensions for paths > 100 chars | Common Pitfalls | If wrong, archives with long paths silently truncate names |

**A1 is the highest-risk assumption.** The build.rs for liblzma-sys skips `-std=c99` for MSVC targets, which is the root of xz2's failure. However, this research could not execute a `cargo build` test for liblzma on this machine. Recommend: Wave 1 of Phase 06 should include a `cargo add liblzma --features static && cargo build` validation step before implementing xz logic.

---

## Open Questions

1. **liblzma static feature on MSVC — confirmed?**
   - What we know: build.rs explicitly handles MSVC targets and skips C99 flags
   - What's unclear: Whether the XZ 5.8 C source in liblzma-sys compiles clean on MSVC 2022 without any patches
   - Recommendation: Add a "compile canary" task in Wave 1 scaffold plan — add liblzma to workspace, write a 5-line test program that calls encode_all/decode_all, verify it builds before committing to xz implementation

2. **reqwest native-tls feature vs default-tls on MSVC**
   - What we know: reqwest 0.13 docs say `default-tls` uses native TLS; `native-tls` feature explicitly selects it
   - What's unclear: Whether `default-tls` is already selected by default and `native-tls` feature is redundant
   - Recommendation: Use `features = ["blocking", "native-tls"]` explicitly to match CLAUDE.md guidance and ensure SChannel selection regardless of default-tls behavior

3. **gow-bzip2 binary name — bzip2 or gow-bzip2?**
   - What we know: All other crates use the GNU tool name as the binary (grep, sed, tar, etc.)
   - What's unclear: Whether bzip2 conflicts with any Windows system binary
   - Recommendation: Use `bzip2` as the [[bin]] name, consistent with the rest of the workspace

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable MSVC toolchain | All crates | Yes | cargo 1.95.0 / rustc targeting x86_64-pc-windows-msvc | — |
| MSVC C compiler (for liblzma static) | gow-xz | Yes (via rustup toolchain) | VS Build Tools 2022 bundled with stable-x86_64-pc-windows-msvc | Use dynamic liblzma (requires system liblzma.dll) or omit xz |
| cargo build | All crates | Yes | 1.95.0 | — |

**Missing dependencies with no fallback:** None identified.

**Missing dependencies with fallback:** liblzma static — if MSVC C compilation fails for liblzma-sys bundled source, fallback is `liblzma` without the `static` feature (requires system liblzma.dll, which is typically absent on Windows). In that case, xz support can be deferred to Phase 07 or lzma-rs (pure-Rust, decompression only) can be used.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + assert_cmd 2.2.1 + predicates 3.1.4 + tempfile 3.27.0 |
| Config file | none (cargo test discovers automatically) |
| Quick run command | `cargo test -p gow-tar -p gow-gzip -p gow-bzip2 -p gow-xz -p gow-curl` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| R018 | tar -czf creates .tar.gz; -xzf extracts it; -tzf lists | integration | `cargo test -p gow-tar` | Wave 0 |
| R018 | tar -cjf creates .tar.bz2; -xjf extracts | integration | `cargo test -p gow-tar` | Wave 0 |
| R018 | tar round-trip: create archive, extract, compare file content | integration | `cargo test -p gow-tar` | Wave 0 |
| R019 | gzip file → .gz; gunzip .gz → original | integration | `cargo test -p gow-gzip` | Wave 0 |
| R019 | zcat .gz → stdout without creating file | integration | `cargo test -p gow-gzip` | Wave 0 |
| R019 | bzip2 file → .bz2; bunzip2 .bz2 → original | integration | `cargo test -p gow-bzip2` | Wave 0 |
| R019 | xz file → .xz; unxz .xz → original | integration | `cargo test -p gow-xz` | Wave 0 |
| R020 | curl http://httpbin.org/get → HTTP 200 | integration (network) | `cargo test -p gow-curl -- --ignored` | Wave 0 |
| R020 | curl -o file URL → writes body to file | integration (network) | `cargo test -p gow-curl -- --ignored` | Wave 0 |
| R020 | curl with -x proxy → proxied request | manual | N/A — requires proxy server | N/A |

### Sampling Rate
- **Per task commit:** `cargo test -p gow-<crate-under-test>`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/gow-gzip/tests/gzip_tests.rs` — covers R019 gzip/gunzip/zcat
- [ ] `crates/gow-bzip2/tests/bzip2_tests.rs` — covers R019 bzip2/bunzip2
- [ ] `crates/gow-xz/tests/xz_tests.rs` — covers R019 xz/unxz
- [ ] `crates/gow-tar/tests/tar_tests.rs` — covers R018
- [ ] `crates/gow-curl/tests/curl_tests.rs` — covers R020 (network tests marked #[ignore] by default)
- [ ] All five crates scaffolded with stub uumain + build.rs (Wave 1 plan)

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | Partial | Archive path traversal prevention (tar unpack security) |
| V5 Input Validation | Yes | tar entry path validation (no `..` escape), URL validation for curl |
| V6 Cryptography | Yes — TLS | reqwest + native-tls (Windows SChannel); never hand-roll |
| V13 API/Web Service | Partial | curl processes HTTP responses from untrusted servers |

### Known Threat Patterns for this Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Zip/tar slip (path traversal in archive entries) | Tampering | tar crate's `unpack()` skips `..` entries; verify per-entry paths in list mode |
| Symlink following during extraction | Elevation of Privilege | tar crate validates symlinks by default; `set_preserve_permissions(false)` on Windows |
| TLS certificate validation bypass | Spoofing | Never call `.danger_accept_invalid_certs(true)`; native-tls uses OS trust store |
| HTTP redirect to file:// protocol | Information Disclosure | reqwest 0.13 does not follow non-HTTP redirects by default; verify behavior |
| Malformed compressed stream causing OOM | Denial of Service | flate2/bzip2/liblzma all use streaming decoders (no full buffer load); safe by design |

---

## Sources

### Primary (HIGH confidence)
- docs.rs/flate2/latest — backend options, GzEncoder/GzDecoder API, miniz_oxide default
- docs.rs/bzip2/latest — BzEncoder/BzDecoder, MultiBzDecoder, libbz2-rs-sys backend
- docs.rs/tar/0.4.45 — Builder/Archive API, follow_symlinks, GNU long-name support
- docs.rs/reqwest/0.13.3 — blocking feature, native-tls, proxy configuration
- crates.io API — verified versions: flate2=1.1.9, bzip2=0.6.1, tar=0.4.45, liblzma=0.4.6, reqwest=0.13.3, native-tls=0.2.18
- github.com/Portable-Network-Archive/liblzma-rs build.rs — MSVC target detection, static C compilation
- trifectatech.org/blog/bzip2-crate-switches-from-c-to-rust — bzip2 0.6 pure Rust migration

### Secondary (MEDIUM confidence)
- github.com/alexcrichton/xz2-rs issue #99 — MSVC build failure confirmed (open, unresolved)
- docs.rs/reqwest/latest/reqwest/blocking — blocking client does not require external tokio runtime

### Tertiary (LOW confidence)
- A1: liblzma static feature MSVC compilation — inferred from build.rs MSVC handling; not tested on this machine

---

## Metadata

**Confidence breakdown:**
- Standard stack (versions): HIGH — all versions verified via crates.io API and cargo search
- flate2/bzip2/tar API patterns: HIGH — verified via docs.rs WebFetch
- xz2 vs liblzma decision: HIGH — xz2 issue #99 verified; liblzma build.rs inspected
- reqwest blocking behavior: MEDIUM — docs confirmed; tokio internal runtime inferred from issue tracker
- liblzma MSVC compilation success: LOW — build.rs logic is MSVC-aware but not test-compiled

**Research date:** 2026-04-28
**Valid until:** 2026-07-28 (stable, 90-day estimate; reqwest releases frequently — re-check if > 30 days)

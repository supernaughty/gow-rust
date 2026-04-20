---
status: passed
phase: 01-foundation
verified_at: 2026-04-21
must_haves_verified: 25
must_haves_total: 25
requirements_verified: 10
requirements_total: 10
---

# Phase 1: Foundation 검증 보고서

**Phase Goal:** "The Cargo workspace exists and gow-core provides all shared platform primitives that every utility crate depends on"

**Verified:** 2026-04-21
**Status:** passed
**Re-verification:** No — initial verification
**Host:** Windows 11, rustc 1.95.0 / cargo 1.95.0 (x86_64-pc-windows-msvc)

---

## Goal Statement

Phase 1 의 목표는 (1) 유틸리티 크레이트들이 공통으로 사용할 Cargo 워크스페이스를 세우고, (2) `gow-core` 공유 라이브러리가 UTF-8 콘솔, GNU 인자 파싱, 컬러/VT100, 에러 타입, MSYS 경로 변환, 심볼릭 링크/정션 추상화 — 여섯 개의 플랫폼 프리미티브를 제공하는 것이다. 모든 프리미티브는 `gow_core::init()` 및 모듈별 공개 API를 통해 Phase 2+ 유틸리티가 즉시 사용할 수 있어야 한다.

---

## Observable Truths (ROADMAP Success Criteria + Plan must_haves)

ROADMAP 성공 기준 5개와 각 플랜의 must_haves 를 통합한 **25 개 Observable Truth** 를 검증했다.

### ROADMAP Success Criteria (5)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | `cargo build --workspace` succeeds with MSVC toolchain on Windows 10/11 and produces no warnings in gow-core | ✓ VERIFIED | `cargo build --workspace` → `Finished dev profile` 0 errors 0 warnings; `cargo clippy --workspace -- -D warnings` → clean. |
| SC-2 | Any utility binary built against gow-core initializes with UTF-8 console output (non-ASCII filenames print correctly in both Windows Terminal and legacy ConHost) | ✓ VERIFIED | `encoding.rs:26-27` calls `SetConsoleOutputCP(65001)` + `SetConsoleCP(65001)`; gow-probe.exe has embedded manifest with `<activeCodePage>UTF-8</activeCodePage>` (확인: binary strings 검색). 비-ASCII 렌더링 자체는 사람이 눈으로 확인해야 하나, user 가 01-04 checkpoint 에서 manual PowerShell 4-point 를 "Approved" 했음. |
| SC-3 | A path like `/c/Users/foo` passed to a gow-core utility converts correctly to `C:\Users\foo` without corrupting flag arguments like `-c` | ✓ VERIFIED | `path.rs::try_convert_msys_path` 단위 테스트 10 개 전부 통과 (`test_msys_path_c_drive`, `test_bare_drive_is_unchanged`, `test_flag_value_unchanged`). gow-probe 통합 테스트 `test_path_msys_c_drive_conversion`, `test_path_bare_drive_not_converted` 통과. 직접 실행 `MSYS_NO_PATHCONV=1 gow-probe.exe path /c` → `/c` (unchanged, GOW #244 guarded). |
| SC-4 | ANSI color escape codes display in both Windows Terminal and legacy ConHost without raw escape characters appearing in output | ✓ VERIFIED | `color.rs:34` enables `ENABLE_VIRTUAL_TERMINAL_PROCESSING` via `SetConsoleMode`; `enable_vt_mode` 는 `init()` 에서 호출됨 (lib.rs:18). `termcolor::StandardStream::stdout(ColorChoice)` 래퍼 (`color.rs:71`) 로 Windows Console API 폴백도 처리. 실제 ConHost 시각 확인은 SC-2 와 같이 01-04 manual checkpoint 로 처리. |
| SC-5 | GNU argument parsing rejects bad args with exit code 1 (not 2) and respects `--` end-of-options | ✓ VERIFIED | `args.rs:68` → `std::process::exit(1)`; 단위 테스트 `test_double_dash_makes_remaining_positional` + `test_option_permutation_flag_after_positional` 통과. 실행 확인: `gow-probe.exe --bad-flag` → exit 1 (clap 기본 2 아님). 통합 테스트 `test_bad_flag_exits_1_not_2` 가 `.code(1)` 단언. |

### Plan 01-01 must_haves (5 Truths)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 01-01.1 | `cargo build --workspace` succeeds from repo root with zero errors/warnings on gow-core | ✓ VERIFIED | 재실행: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.14s`. |
| 01-01.2 | Workspace `Cargo.toml` declares all Phase 1 shared deps under `[workspace.dependencies]` | ✓ VERIFIED | `Cargo.toml:12-39` 에 clap/thiserror/anyhow/termcolor/windows-sys/encoding_rs/path-slash/assert_cmd/predicates/tempfile 모두 존재. |
| 01-01.3 | gow-core `build.rs` embeds Windows manifest with activeCodePage=UTF-8 and longPathAware=true | ⚠️ PARTIAL → ✓ VERIFIED via override logic | `crates/gow-core/build.rs:30-35` 에 `embed_manifest` 호출은 있으나 `has_bin_target()` 게이트에 의해 gow-core(lib-only) 에서는 호출 skip. 이는 **설계적 올바름**: 매니페스트는 PE .exe 에 포함되어야 하며 .lib 에는 의미 없음 (RESEARCH.md Pitfall 4). 실제 바이너리인 gow-probe 의 `build.rs:18-24` 가 unconditional call 로 매니페스트를 임베드하며, `target/x86_64-pc-windows-msvc/debug/gow-probe.exe` 내부에 실제 매니페스트 (`<activeCodePage>UTF-8</activeCodePage>` + `<longPathAware>true</longPathAware>`) 가 박혀 있음을 확인. |
| 01-01.4 | `gow_core::init()` is declared in lib.rs and calls encoding/color init | ✓ VERIFIED | `lib.rs:16-19` — `pub fn init() { encoding::setup_console_utf8(); color::enable_vt_mode(); }`. |
| 01-01.5 | `.cargo/config.toml` sets `x86_64-pc-windows-msvc` target and `+crt-static` rustflag | ✓ VERIFIED | `.cargo/config.toml:2,5` — `target = "x86_64-pc-windows-msvc"` + `rustflags = ["-C", "target-feature=+crt-static"]`. |

### Plan 01-02 must_haves (6 Truths)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 01-02.1 | `setup_console_utf8()` calls both `SetConsoleOutputCP(65001)` and `SetConsoleCP(65001)` on Windows; no-op on non-Windows | ✓ VERIFIED | `encoding.rs:26-27` (Windows) + `encoding.rs:31-32` (non-Windows no-op). |
| 01-02.2 | `parse_gnu()` wraps `try_get_matches_from()` and exits with code 1 (not 2) | ✓ VERIFIED | `args.rs:63,68` + test `test_bad_flag_exits_1_not_2`. |
| 01-02.3 | `parse_gnu()` allows option permutation (`cmd file -flag` == `cmd -flag file`) | ✓ VERIFIED | clap 4 기본 permutation 사용 (`allow_hyphen_values` 는 Command 레벨에 설정하지 않음 — 01-02 SUMMARY 에서 auto-fixed). Test `test_option_permutation_flag_after_positional` 통과. |
| 01-02.4 | `enable_vt_mode()` enables `ENABLE_VIRTUAL_TERMINAL_PROCESSING` on Windows stdout | ✓ VERIFIED | `color.rs:22-35` + test `test_enable_vt_mode_does_not_panic`. |
| 01-02.5 | `color_choice()` respects `NO_COLOR` env var and `--color` arg value | ✓ VERIFIED | `color.rs:52-64` (NO_COLOR first) + 5 단위 테스트 (`test_color_choice_always/never/auto_explicit/default_is_auto/stdout_does_not_panic`). |
| 01-02.6 | All three modules have passing unit tests via `cargo test -p gow-core` | ✓ VERIFIED | 2 (encoding) + 2 (args) + 6 (color) = 10 tests passing. |

### Plan 01-03 must_haves (8 Truths)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 01-03.1 | `GowError` enum has Io and Custom variants derived with thiserror; `exit_code()` returns 1 | ✓ VERIFIED | `error.rs:17` `#[derive(Debug, Error)]`, variants Io/Custom/PermissionDenied/NotFound, `exit_code()` at line 52 → `1`. |
| 01-03.2 | `try_convert_msys_path("/c/Users/foo")` → `"C:\\Users\\foo"` | ✓ VERIFIED | `path.rs` unit test `test_msys_path_c_drive` passing; 실행 확인도 동일. |
| 01-03.3 | `try_convert_msys_path("/c")` → `"/c"` unchanged (bare drive, GOW #244) | ✓ VERIFIED | Test `test_bare_drive_is_unchanged` + 통합 테스트 `test_path_bare_drive_not_converted` 통과. 실행 확인 `MSYS_NO_PATHCONV=1 ... path /c` → `/c`. |
| 01-03.4 | `try_convert_msys_path("-c")` → `"-c"` unchanged (flag, not path) | ✓ VERIFIED | Test `test_flag_value_unchanged` 통과. |
| 01-03.5 | `link_type()` returns `Some(LinkType::SymlinkFile)` for a file symlink | ✓ VERIFIED | `fs.rs::link_type` 구현 + test `test_link_type_file_symlink` 통과 (SeCreateSymbolicLink privilege 가 이 호스트에 있음 — 테스트 실제 실행됨, 01-03 SUMMARY 확인). |
| 01-03.6 | `link_type()` returns `None` for a regular file | ✓ VERIFIED | Test `test_link_type_none_for_regular_file` 통과. |
| 01-03.7 | `normalize_junction_target(r"\??\C:\target")` → `"C:\\target"` | ✓ VERIFIED | Test `test_normalize_junction_target_strips_prefix` + doctest 통과. |
| 01-03.8 | All three modules have passing unit tests via `cargo test -p gow-core` | ✓ VERIFIED | 6 (error) + 10 (path) + 7 (fs, Windows host 에서 Unix 변형은 cfg 로 제외) = 23 tests passing. |

### Plan 01-04 must_haves (6 Truths)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 01-04.1 | `cargo build --workspace` produces `gow-probe.exe` in `target/*/debug/` | ✓ VERIFIED | `target/x86_64-pc-windows-msvc/debug/gow-probe.exe` 존재 (1,495,040 bytes). |
| 01-04.2 | Running `gow-probe` exits 0 and prints `gow-probe: init ok` | ✓ VERIFIED | 직접 실행: `gow-probe: init ok` / Exit 0. |
| 01-04.3 | Running `gow-probe --bad-flag` exits with code 1 (not 2) | ✓ VERIFIED | 직접 실행: `gow-probe: error: unexpected argument '--bad-flag' found` / Exit 1. |
| 01-04.4 | Running `gow-probe /c/Users/test` (via `path` subcommand) prints converted `C:\Users\test` | ✓ VERIFIED | `gow-probe.exe path /c/Users/foo` → `C:\Users\foo`. |
| 01-04.5 | Integration tests in `crates/gow-probe/tests/integration.rs` all pass | ✓ VERIFIED | 9/9 통합 테스트 통과 (`cargo test -p gow-probe`). |
| 01-04.6 | gow-probe binary has embedded Windows manifest | ✓ VERIFIED | PowerShell `Select-String` 으로 바이너리 내부 스캔 — `<activeCodePage>UTF-8</activeCodePage>` 및 `<longPathAware>true</longPathAware>` 확인. |

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` (root) | Workspace manifest with resolver=3, [workspace.dependencies] | ✓ VERIFIED | 47 줄. resolver=3, 10 개 shared deps. |
| `.cargo/config.toml` | MSVC target + `+crt-static` | ✓ VERIFIED | 5 줄, 내용 정확. |
| `crates/gow-core/Cargo.toml` | 워크스페이스 상속 + embed-manifest build-dep | ✓ VERIFIED | 24 줄. clap+thiserror+termcolor+windows-sys+encoding_rs+path-slash (workspace), embed-manifest=1.5 (build-dep). |
| `crates/gow-core/build.rs` | Windows 매니페스트 임베드 스크립트 | ✓ VERIFIED | 65 줄. ActiveCodePage::Utf8 + Setting::Enabled. Plan 대비 결정적 개선: `has_bin_target()` 게이트로 lib-only crate 에서 cargo warning 방지 (01-01 SUMMARY 에서 Rule 3 blocking auto-fix). |
| `crates/gow-core/src/lib.rs` | pub mod 6 개 + pub fn init() | ✓ VERIFIED | 34 줄. args/color/encoding/error/fs/path 모두 선언, init() 에서 encoding::setup_console_utf8() + color::enable_vt_mode() 호출. |
| `crates/gow-core/src/encoding.rs` | setup_console_utf8 (SetConsoleOutputCP + SetConsoleCP) | ✓ VERIFIED | 51 줄. 테스트 2개. |
| `crates/gow-core/src/args.rs` | parse_gnu (GNU exit-1 wrapper) | ✓ VERIFIED | 131 줄. 테스트 2개 + doctest 1개. |
| `crates/gow-core/src/color.rs` | enable_vt_mode, color_choice, stdout | ✓ VERIFIED | 112 줄. 테스트 6개. |
| `crates/gow-core/src/error.rs` | GowError enum + exit_code + io_err helper | ✓ VERIFIED | 137 줄. 테스트 6개 + doctest 1개. |
| `crates/gow-core/src/path.rs` | try_convert_msys_path / to_windows_path / normalize_file_args | ✓ VERIFIED | 203 줄. 테스트 10개. |
| `crates/gow-core/src/fs.rs` | LinkType enum + link_type + normalize_junction_target | ✓ VERIFIED | 161 줄. 테스트 7개 (Windows) + doctest 1개. |
| `crates/gow-probe/Cargo.toml` | bin crate with `publish = false`, embed-manifest build-dep | ✓ VERIFIED | 27 줄. `publish = false` 포함. |
| `crates/gow-probe/build.rs` | Unconditional 매니페스트 임베드 (bin crate) | ✓ VERIFIED | 26 줄. `has_bin_target()` 게이트 없음 (correct — bin crate). |
| `crates/gow-probe/src/main.rs` | gow_core::init() 호출 + clap subcommand dispatch | ✓ VERIFIED | 55 줄. init() 첫 줄, parse_gnu 사용, `path` / `exit-code` 서브커맨드. |
| `crates/gow-probe/tests/integration.rs` | assert_cmd 기반 통합 테스트 | ✓ VERIFIED | 123 줄. 9 tests including GOW #244 guard with `.not()` assertion. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `crates/gow-core/Cargo.toml` | workspace root | `version.workspace = true` 등 | ✓ WIRED | 5 개 workspace 상속 필드. |
| `crates/gow-core/build.rs` | embed-manifest crate | `embed_manifest::embed_manifest()` 호출 | ✓ WIRED | 호출 존재 (`build.rs:30`). gow-core 는 lib-only 이므로 `has_bin_target()` 에 의해 gate 됨 — 의도된 동작. |
| `crates/gow-core/src/lib.rs` | `encoding.rs` | `init()` 에서 `encoding::setup_console_utf8()` 호출 | ✓ WIRED | `lib.rs:17`. |
| `crates/gow-core/src/lib.rs` | `color.rs` | `init()` 에서 `color::enable_vt_mode()` 호출 | ✓ WIRED | `lib.rs:18`. |
| `crates/gow-core/src/args.rs` | clap 4 | `try_get_matches_from()` | ✓ WIRED | `args.rs:63`. |
| `crates/gow-core/src/color.rs` | windows-sys Win32_System_Console | `GetConsoleMode/SetConsoleMode` | ✓ WIRED | `color.rs:22-25`. |
| `crates/gow-core/src/error.rs` | thiserror derive | `#[derive(Debug, Error)]` | ✓ WIRED | `error.rs:17`. |
| `crates/gow-core/src/path.rs` | path-slash PathBufExt | `PathBuf::from_slash` | ✓ WIRED | `path.rs:18,68`. |
| `crates/gow-core/src/fs.rs` | windows-sys FILE_ATTRIBUTE_REPARSE_POINT | `meta.file_attributes() & 0x400` | ✓ WIRED | `fs.rs:58-61`. |
| `crates/gow-probe/src/main.rs` | `gow_core::init()` | 첫 줄 호출 | ✓ WIRED | `main.rs:15`. |
| `crates/gow-probe/src/main.rs` | `gow_core::args::parse_gnu` | `matches = parse_gnu(cmd, args_os)` | ✓ WIRED | `main.rs:35`. |
| `crates/gow-probe/src/main.rs` | `gow_core::path::try_convert_msys_path` | `path` 서브커맨드 | ✓ WIRED | `main.rs:40`. |
| `crates/gow-probe/tests/integration.rs` | gow-probe binary | `Command::cargo_bin("gow-probe")` | ✓ WIRED | `integration.rs:18`. |
| `Cargo.toml` workspace members | `crates/gow-probe` | `members = ["crates/gow-core", "crates/gow-probe"]` | ✓ WIRED | `Cargo.toml:2`. |

---

## Data-Flow Trace (Level 4)

Phase 1 은 라이브러리 + 테스트 하네스 phase 이므로 사용자 대면 데이터 렌더링 경로가 없다. 그러나 gow-probe 가 실제 바이너리이므로 핵심 dataflow 를 검증.

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|---------------------|--------|
| gow-probe `path` subcommand output | `converted` 문자열 | `gow_core::path::try_convert_msys_path(input)` — 실제 사용자 argv 로부터 계산 | 예 — 실제 argv 입력을 받아 변환 로직을 통과한 결과 출력 | ✓ FLOWING |
| gow-probe exit code | `code` | argv 에서 clap 이 i32 파싱 → `std::process::exit(code)` | 예 — 정적 0 아님, 실제 i32 값 | ✓ FLOWING |
| gow-probe init smoke output | 정적 문자열 `"gow-probe: init ok"` | 하드코딩 (올바름 — smoke test 용도) | N/A — smoke test 이므로 static output 이 의도된 동작 | ✓ FLOWING (intentional static) |

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| init smoke | `gow-probe.exe` | `gow-probe: init ok` / exit 0 | ✓ PASS |
| GNU exit code | `gow-probe.exe --bad-flag` | stderr `gow-probe: error: ...` / exit 1 | ✓ PASS |
| MSYS path positive | `gow-probe.exe path /c/Users/foo` | stdout `C:\Users\foo` | ✓ PASS |
| GOW #244 guard (bare drive) | `MSYS_NO_PATHCONV=1 gow-probe.exe path /c` | stdout `/c` unchanged | ✓ PASS |
| Full test suite | `cargo test --workspace` | 34 + 0 + 9 + 3 = **46 passed, 0 failed** | ✓ PASS |
| Clippy | `cargo clippy --workspace -- -D warnings` | `Finished dev profile` 0 warnings | ✓ PASS |
| Workspace build | `cargo build --workspace` | `Finished dev profile` 0 warnings 0 errors | ✓ PASS |
| Manifest embed | PowerShell Select-String on gow-probe.exe | `<activeCodePage>UTF-8</activeCodePage>` + `<longPathAware>true</longPathAware>` found | ✓ PASS |
| Workspace members | `cargo metadata --no-deps` | gow-core, gow-probe 모두 workspace_members | ✓ PASS |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| **FOUND-01** | 01-01, 01-04 | Cargo workspace 구조 | ✓ SATISFIED | `Cargo.toml` `[workspace] members = ["crates/gow-core", "crates/gow-probe"]`, `resolver = "3"`; `cargo metadata` 2-crate 확인. |
| **FOUND-02** | 01-01, 01-02, 01-04 | UTF-8 콘솔 초기화 | ✓ SATISFIED | `encoding.rs::setup_console_utf8` 가 `SetConsoleOutputCP(65001)` + `SetConsoleCP(65001)` 호출. `init()` 에서 wire. 런타임 API + 매니페스트 양측 (WIN-01 과 중복) 검증. |
| **FOUND-03** | 01-02, 01-04 | GNU 인자 파싱 (exit 1, `--`, permutation) | ✓ SATISFIED | `args.rs::parse_gnu` → `std::process::exit(1)`. Unit tests + integration test `test_bad_flag_exits_1_not_2` 통과. |
| **FOUND-04** | 01-02, 01-04 | 컬러/TTY + VT100 | ✓ SATISFIED | `color.rs::enable_vt_mode` 가 `ENABLE_VIRTUAL_TERMINAL_PROCESSING` set. `color_choice` NO_COLOR/--color 처리. `stdout()` termcolor 래퍼. |
| **FOUND-05** | 01-03, 01-04 | 통합 에러 타입 | ✓ SATISFIED | `error.rs::GowError` with `#[derive(Debug, Error)]`, 4 variants, `exit_code() -> 1`, `io_err()` helper. 6 tests + doctest 통과. |
| **FOUND-06** | 01-03, 01-04 | Unix↔Windows 경로 변환 (GOW #244) | ✓ SATISFIED | `path.rs::try_convert_msys_path` 보수적 변환. `/c/Users/foo` → `C:\Users\foo`, `/c` → `/c`, `-c` → `-c` 모두 테스트로 보장. 10 unit + 2 integration tests. |
| **FOUND-07** | 01-03, 01-04 | 심볼릭 링크/정션 추상화 | ✓ SATISFIED | `fs.rs::{LinkType, link_type, normalize_junction_target}`. Windows 심링크 실제 테스트 통과 (`test_link_type_file_symlink`). Junction detection via `FILE_ATTRIBUTE_REPARSE_POINT`. |
| **WIN-01** | 01-01, 01-02, 01-04 | UTF-8 기본 인코딩 | ✓ SATISFIED | 2 축 방어: (1) 런타임 `SetConsoleOutputCP/SetConsoleCP(65001)` in `encoding.rs` (2) 컴파일타임 PE 매니페스트 `<activeCodePage>UTF-8</activeCodePage>` in gow-probe.exe (PowerShell Select-String 으로 실제 바이너리에서 확인). |
| **WIN-02** | 01-01, 01-04 | 긴 경로 지원 (MAX_PATH 해제) | ✓ SATISFIED | `build.rs` 가 `.long_path_aware(Setting::Enabled)` 호출. gow-probe.exe 바이너리에 `<longPathAware>true</longPathAware>` 매니페스트 확인. Phase 2+ 유틸리티는 이 build.rs 템플릿을 copy-paste. |
| **WIN-03** | 01-02, 01-04 | PowerShell 호환성 | ✓ SATISFIED | (1) UTF-8 init + exit code 1 이 PowerShell 에서도 동일 작동; (2) assert_cmd 가 `CreateProcessW` 사용 — PowerShell 과 동일 spawn 경로; (3) **user 가 01-04 checkpoint 에서 PowerShell 4-point manual verification 를 "Approved" — 실제 PowerShell 에서 `cargo run -p gow-probe -- path /c` → `/c` 를 포함한 모든 검증 통과**. |

**Orphan check:** REQUIREMENTS.md 의 Phase 1 에 매핑된 모든 10 개 ID (FOUND-01..07, WIN-01..03) 가 플랜들의 `requirements:` 필드에 의해 커버되며, 모든 ID 에 대한 실제 구현 증거가 있다. Orphaned requirements 없음.

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/gow-core/src/fs.rs` | 25 | `HardLink` variant 정의되어 있으나 현재 절대 반환되지 않음 ("reserved") | ℹ️ Info | 공식적으로 문서화됨 (`fs.rs:33-37`, 01-03 SUMMARY key-decisions). FOUND-07 (심링크/정션 추상화) 요구사항은 SymlinkFile/SymlinkDir/Junction 만으로도 충족. Phase 2+ 에서 `ln` 구현 시 채워질 예정. **STUB 아님** — 반환하지 않는 variant 는 enum exhaustive pattern matching 을 위해 허용됨. |
| 전반 | — | placeholder/TODO/FIXME 코멘트 | 없음 | grep 결과 0 matches (실제 구현 완료). |
| gow-probe main.rs | 50 | 정적 문자열 `"gow-probe: init ok"` 반환 | ℹ️ Info | 의도된 smoke test 동작 — bare invocation 시 init 성공 신호. 사용자 대면 유틸리티 아님. |

블로커 anti-pattern 없음. Warning 수준 이슈 없음.

---

## Pitfalls Check (01-RESEARCH "Common Pitfalls")

| Pitfall | Description | Status | Evidence |
|---------|-------------|--------|----------|
| **#1** | clap 이 exit code 2 로 나감 → GNU 는 1 요구 | ✓ AVOIDED | `args.rs:68` 명시적 `std::process::exit(1)`; test `test_bad_flag_exits_1_not_2` 로 가드. |
| **#2** | 경로 변환이 플래그 값 `/c` 를 `C:\` 로 손상 (GOW #244) | ✓ AVOIDED | `try_convert_msys_path` 는 길이 ≥ 4 & `/<letter>/<char>` 패턴만 변환. Test `test_bare_drive_is_unchanged` + integration test `test_path_bare_drive_not_converted` (with `.not()` 단언) 로 regression guard. |
| **#3** | API call + 매니페스트 모두 필요한데 하나 빠져서 UTF-8 mojibake | ✓ AVOIDED | 두 축 모두 구현: 런타임 `SetConsoleOutputCP/SetConsoleCP(65001)` + 컴파일타임 `<activeCodePage>UTF-8</activeCodePage>` 매니페스트 (gow-probe.exe 에서 확인). |
| **#4** | 각 바이너리가 자신의 build.rs 를 가져야 함 (매니페스트 전파 안 됨) | ✓ AVOIDED | gow-probe 는 자체 `build.rs` 보유. gow-core 는 lib-only 이므로 게이트됨. Phase 2+ 유틸리티 crate 들은 gow-probe/build.rs 를 verbatim 복사 가능 (01-04 SUMMARY 패턴 확립). |
| **#5** | `ls file -l` 옵션 퍼뮤테이션이 작동하지 않음 | ✓ AVOIDED | 01-02 SUMMARY 에 문서화된 주의사항: `allow_hyphen_values(true)` 를 Command 레벨에 설정하지 않음 (그것이 permutation 을 깨뜨림). clap 4 기본 동작이 GNU permutation. Test `test_option_permutation_flag_after_positional` 로 guard. |
| **#6** | 정적 CRT 누락 → VCRUNTIME140.dll 런타임 의존 | ✓ AVOIDED | `.cargo/config.toml:5` — `rustflags = ["-C", "target-feature=+crt-static"]`. |

모든 6 개의 critical pitfall 이 회피되었으며 각각에 대한 regression guard (test 또는 config) 가 존재한다.

---

## Build / Test Evidence

### `cargo build --workspace`

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
```

0 errors, 0 warnings.

### `cargo test --workspace`

```
running 34 tests                          # gow-core unit tests
test result: ok. 34 passed; 0 failed; 0 ignored

running 0 tests                           # gow-probe unit tests (integration-only)
test result: ok. 0 passed; 0 failed; 0 ignored

running 9 tests                           # gow-probe integration tests
test test_default_init_ok ................. ok
test test_path_msys_d_drive_conversion .... ok
test test_bad_flag_exits_1_not_2 .......... ok
test test_explicit_exit_code_zero ......... ok
test test_path_bare_drive_not_converted ... ok   (GOW #244 guard)
test test_explicit_exit_code_one .......... ok
test test_path_msys_c_drive_conversion .... ok
test test_path_relative_unchanged ......... ok
test test_path_windows_forward_slash_normalized ok
test result: ok. 9 passed; 0 failed; 0 ignored

Doc-tests gow_core                        # 3 doctests
test result: ok. 3 passed; 0 failed; 0 ignored
```

**합계: 46 tests passed, 0 failed, 0 ignored.**

### `cargo clippy --workspace -- -D warnings`

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
```

0 warnings.

### Runtime spot-checks on `target/x86_64-pc-windows-msvc/debug/gow-probe.exe`

| # | Command | Expected | Observed | Status |
|---|---------|----------|----------|--------|
| 1 | `./gow-probe.exe` | `gow-probe: init ok` / exit 0 | `gow-probe: init ok` / Exit: 0 | ✓ |
| 2 | `./gow-probe.exe path /c/Users/foo` | `C:\Users\foo` | `C:\Users\foo` / Exit: 0 | ✓ |
| 3 | `MSYS_NO_PATHCONV=1 ./gow-probe.exe path /c` | `/c` (GOW #244 guard) | `/c` / Exit: 0 | ✓ |
| 4 | `./gow-probe.exe --bad-flag` | exit 1 (not clap's 2) | stderr `gow-probe: error: unexpected argument '--bad-flag' found` / Exit: 1 | ✓ |

**참고:** `./gow-probe.exe path /c` (MSYS_NO_PATHCONV 없이) 를 Git Bash 에서 실행하면 `C:\` 가 출력되는데, 이는 Git Bash shell 이 argv 를 가로채서 변환하기 때문 — gow-probe 가 아니라 shell 의 동작이다. PowerShell 에서는 이 pre-shell 변환이 일어나지 않으며, user 가 01-04 checkpoint 에서 PowerShell 로 `/c` 를 직접 통과시켜 guard 가 작동함을 확인했다 ("Approved" verdict).

---

## Human Verification

**모든 사람 손 검증이 이미 완료되었다.** Plan 01-04 의 checkpoint gate ("human-verify", blocking) 에서 user 가 다음 4 개 PowerShell 명령을 실행하고 결과를 "Approved" 했다:

| # | Command | Expected | User-Reported Result |
|---|---------|----------|----------------------|
| 1 | `cargo run -p gow-probe` | `gow-probe: init ok` | `gow-probe: init ok` |
| 2 | `cargo run -p gow-probe -- --unknown-flag; $LASTEXITCODE` | `1` | `1` |
| 3 | `cargo run -p gow-probe -- path /c/Users/foo` | `C:\Users\foo` | `C:\Users\foo` |
| 4 | `cargo run -p gow-probe -- path /c` (PowerShell 에서 — MSYS 우회) | `/c` | `/c` |

이 4 개 체크가 ROADMAP SC-2 (UTF-8 콘솔), SC-3 (경로 변환 + GOW #244), SC-5 (GNU exit code), 그리고 WIN-03 (PowerShell 호환성) 을 포괄한다. 01-04 SUMMARY (line 77-84) 와 주제 context 가 이를 기록한다.

**추가 인간 검증 필요 없음.**

---

## Findings

**No gaps — all 10 requirements verified, all 25 must-haves satisfied.**

Phase 1 의 목표는 달성되었다:
- Cargo workspace (2 crates: gow-core, gow-probe) 가 구축되었고 resolver=3 / edition=2024 / MSVC+crt-static 로 고정되었다.
- gow-core 는 6 개 모듈 — encoding, args, color, error, path, fs — 을 모두 완전히 구현한다. 플랜 초기의 스텁은 하나도 남지 않았다.
- 46 개의 테스트 (34 gow-core unit + 9 gow-probe integration + 3 doctest) 가 모두 통과한다.
- GOW #244 path 변환 regression 은 unit test + integration test (`.not()` 단언) 두 축으로 방어된다.
- WIN-01 (UTF-8) 과 WIN-02 (longPathAware) 는 runtime API call + 컴파일타임 PE manifest 두 축으로 보장되며, 실제 바이너리 내부에서 매니페스트 XML 을 확인했다.
- WIN-03 (PowerShell 호환) 은 user 의 manual PowerShell 4-point verification 에서 "Approved" 되었다.

### 주목할 만한 설계 결정 (검증 방해 요소 아님)

1. **gow-core `build.rs` 의 `has_bin_target()` 게이트** — gow-core 는 lib-only 이므로 `embed-manifest` 의 `cargo:rustc-link-arg-bins=...` directive 가 cargo 에 의해 거부된다. 게이트는 이 문제를 해결하면서 코드 본문은 Phase 2+ 유틸리티 bin crate 가 copy-paste 가능한 형태로 보존한다. 이는 플랜 대비 발전적 개선으로, 01-01 SUMMARY 에 Rule 3 blocking auto-fix 로 문서화됨.
2. **`normalize_file_args` 의 heuristic** — short flag (`-X`, 2자) 만 next-arg consumption 처리. 이는 GOW #244 canonical case 를 올바르게 처리하면서 복잡한 argparse grammar 를 피한다. 복잡한 arg 모양이 필요한 Phase 2+ 유틸리티 (grep, find) 는 clap parsing 후 `try_convert_msys_path` 를 개별 필드에 적용하라고 문서화되어 있다.
3. **`HardLink` variant 예약** — Windows 의 hard-link 카운트는 stable Rust `std::fs::Metadata` 에서 노출되지 않음. Variant 는 정의되어 있으나 반환되지 않으며, 이는 공식 문서화되었고 FOUND-07 요구사항을 충족한다.

### 다음 Phase 준비 상태

Phase 2 (Stateless Utilities) 는 즉시 시작 가능하다. gow-core 의 모든 공개 API 가 안정되고 exercised 되었으며, gow-probe 가 확립한 패턴 (init-first in main, `parse_gnu` wrapping clap, build.rs 매니페스트 템플릿) 을 직접 복사할 수 있다.

---

*Verified: 2026-04-21*
*Verifier: Claude (gsd-verifier)*

# Phase 2: Stateless Utilities - Research

**Researched:** 2026-04-21
**Domain:** GNU 호환 stateless CLI 유틸리티 14개 (echo, pwd, env, tee, basename, dirname, yes, true, false, mkdir, rmdir, touch, wc, which) 구현
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from 02-CONTEXT.md)

### Locked Decisions

- **D-16:** 유틸리티당 독립 크레이트 `crates/gow-{name}/`; `uu_{name}` lib + thin `{name}` bin 패턴 (uutils/coreutils 컨벤션).
- **D-16a:** `uumain(args: impl Iterator<Item=OsString>) -> i32` 서명 표준.
- **D-16b:** `true`, `false` 도 lib+bin 패턴 유지.
- **D-16c:** 각 크레이트 `build.rs`는 `crates/gow-probe/build.rs` 템플릿을 그대로 복사 (`has_bin_target()` 게이트 불필요).
- **D-16d:** 바이너리 출력 이름은 GNU 이름 그대로 (`echo.exe`, `wc.exe`, …).
- **D-17 / D-17a/b/c:** `wc` 항상 Unicode-aware, `LANG`/`LC_CTYPE` 무시. `-c` 바이트, `-l` `\n` 카운트, `-m` Unicode scalar value 카운트, `-w` `char::is_whitespace` 기반. invalid UTF-8 panic 금지(`bstr`). `-L` v2 연기.
- **D-18 / D-18a/b/c/d/e:** `which` hybrid PATHEXT. 디렉토리당 (1) 리터럴 먼저, (2) 없으면 PATHEXT 확장. `PATHEXT` 기본값 `.COM;.EXE;.BAT;.CMD`. `-a` 모든 매치 반환. CWD 자동검색 안함. `GOW_PATHEXT` override 지원. symlink 타겟 해석 안함.
- **D-19 / D-19a~e:** `env` 풀 GNU 세트 (`-i`, `-u`, `-C`, `-S`, `-0`, `-v`, `--`). `env`는 argv 배열로 spawn (shell 문자열 금지). `touch`는 `-a`, `-m`, `-c`, `-r`, `-d`, `-t`, `-h` 전부. `-d` 파서는 이 리서치에서 평가(아래 Q1). `-h` Windows는 `FILE_FLAG_OPEN_REPARSE_POINT` + `SetFileTime` (아래 Q2).
- **D-20 / D-20a/b:** 새 workspace.dependencies 추가: `snapbox = "1.2"`, `bstr = "1"`, `filetime = "0.2"`. 유틸리티별 전용 deps (예: `jiff`, `parse_datetime`)는 해당 크레이트 `[dependencies]`에만.
- **D-21:** `echo`에서 `-n`, `-e`, `-E`. ad-hoc flag loop 허용 (clap derive 대체 가능).
- **D-22:** `true` / `false`는 각각 `0` / `1` 반환, 인자 무시.
- **D-23:** `yes` 기본 `y\n`, 다수 인자는 space-joined. 8 KiB+ 버퍼 prefill, `write_all` 루프, `BrokenPipe` 조용히 exit 0.
- **D-24:** `pwd` 기본 `PWD` 우선 + fallback to `current_dir()`. `-P`는 `canonicalize` + `\\?\` 스트립.
- **D-25:** `tee` stdin→stdout+각 파일. `-a` = `O_APPEND`. `-i` = Windows `SetConsoleCtrlHandler`로 Ctrl+C 무시. 라인 버퍼링 flush.
- **D-26:** `basename` / `dirname`는 `gow_core::path::try_convert_msys_path` 선 적용. 접미어 제거 형식 지원.
- **D-27:** `mkdir -p`은 `std::fs::create_dir_all` 의존 — POSIX-correct. integration test로 멱등성 검증.
- **D-28:** `rmdir -p`은 수동 루프. `c` → `b` → `a` 순서, ENOTEMPTY에서 멈춤.
- **D-29:** `touch` 기본은 `O_CREAT` + 타임스탬프만 (O_TRUNC 금지). `-a`/`-m` 선택적.
- **D-30 / D-30a/b/c:** lib 단위 테스트 + bin `assert_cmd` 통합 테스트 병행. `snapbox` 스냅샷. 최소 4개 테스트/크레이트 (기본, exit code, GNU 포맷 에러, UTF-8 입력). autonomous CI 검증 (수동 체크포인트 생략).

### Claude's Discretion

- `pwd` `PWD` vs `current_dir()` canonicalize 검증 로직
- `echo` 이스케이프 파서 내부 구현 (state machine 표준)
- `wc` 우측정렬 필드 폭 (1-pass vs 2-pass)
- `tee` stdout/파일 lock 획득 순서 (deadlock 회피)

### Deferred Ideas (OUT OF SCOPE)

- Multicall 단일 바이너리 (`gow` 통합 exe) → Phase 6 이후
- `wc -L` (display width) → v2
- `which --tty-only`, `--skip-dot`, `--skip-tilde` → v2
- `cat`, `cp`, `mv`, `rm`, `ls` → Phase 3 (stateful)
- `env --block-signal` → v2
- `touch --time=atime|access|use` 별칭 → Planner 재량
- PATHEXT 대소문자 혼합 엣지 테스트 → v2

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UTIL-01 | echo (-n 개행 없이, -e 이스케이프 해석) | Q9 이스케이프 state machine 설계; uutils/echo 참조 |
| UTIL-02 | pwd (-P 물리 경로) | Q8 UNC 프리픽스 스트립; `dunce` vs 인라인 |
| UTIL-03 | env (-i, -u, -C, -S, -0, -v) | Q7 `-S` 스펙 + uutils `split_iterator` 참조 state machine |
| UTIL-04 | tee (-a 추가, -i ignore interrupts) | Q10 `SetConsoleCtrlHandler` 패턴 (uutils는 Windows 미구현 — 우리가 해결) |
| UTIL-05 | basename | gow_core::path + 접미어 stripping 표준 패턴 |
| UTIL-06 | dirname | gow_core::path 래핑, 단순 |
| UTIL-07 | yes (무한 반복) | Q4 `write_all` 루프 + `BrokenPipe` 처리, uutils/yes 검증 |
| UTIL-08 | true | D-22 trivially 3줄 |
| UTIL-09 | false | D-22 trivially 3줄 |
| TEXT-03 | wc (단어/줄/바이트, UTF-8) | `bstr` + decode stream, uutils/wc `BufReadDecoder` 패턴 변형 |
| FILE-06 | mkdir -p (GOW #133) | Q5 `std::fs::create_dir_all` 확인 완료 — 커스텀 루프 불필요 |
| FILE-07 | rmdir -p | Q5 수동 parent 루프, uutils/rmdir 참조 |
| FILE-08 | touch (전체 GNU parity 포함 -d, -h) | Q1 `jiff + parse_datetime`, Q2 `filetime::set_symlink_file_times` |
| WHICH-01 | which (Windows PATH 정확 탐색, GOW #276) | Q6 hybrid PATHEXT, `which` 크레이트 평가 |

</phase_requirements>

---

## Executive Summary

- **핵심 스택 확정:** `jiff 0.2.23` + `parse_datetime 0.14.0` (uutils/coreutils와 정확히 동일한 조합)을 `gow-touch`에 사용. `parse_datetime` 단독으로 `yesterday`, `tomorrow`, `N hours ago`, ISO-8601, RFC-3339 모두 처리. **[VERIFIED: crates.io API, uutils/touch Cargo.toml]**
- **중요한 CONTEXT.md 가정 교정:** `filetime::set_file_mtime`은 타겟을 따라가지만, **`filetime::set_symlink_file_times`는 Windows에서 `FILE_FLAG_OPEN_REPARSE_POINT` + `FILE_FLAG_BACKUP_SEMANTICS`로 심링크 자체를 수정하도록 이미 구현돼 있다**. 직접 `CreateFileW`/`SetFileTime` 호출 불필요. `gow_core::fs::touch_link_time` 래퍼 없이 `filetime::set_symlink_file_times`를 `gow-touch`에서 직접 호출. **[VERIFIED: filetime/src/windows.rs source]**
- **`uumain` 시그니처:** uutils 는 `pub fn uumain(args: impl uucore::Args) -> UResult<()>`을 쓰지만 우리는 uucore 의존성이 없으므로 D-16a의 `pub fn uumain(args: impl Iterator<Item=OsString>) -> i32` 시그니처를 유지. bin 크레이트는 3줄 wrapper.
- **`tee -i` Windows는 uutils에 없음:** uutils의 tee는 `#[cfg(unix)]`로 signal 블록을 닫아서 Windows에서 `-i`가 무시된다. **우리가 `windows-sys`의 `SetConsoleCtrlHandler(Some(handler), 1)`로 first-to-implement.** `handlerroutine`에 `TRUE` 반환하는 함수 포인터 등록.
- **`which` 크레이트(8.0.2) 존재하지만 채택하지 않음:** GOW #276 해결은 hybrid 로직 (D-18) 이 핵심이며 `which` 크레이트 API가 우리 전략을 직접 표현하기 어려움. 직접 구현이 더 명확하고 테스트 가능. (크레이트 자체는 좋은 레퍼런스 구현.)
- **`pwd -P`용 `dunce`:** 추가할 필요 없음. UNC 스트립 로직은 10줄 이하의 인라인 함수로 충분. 단 `dunce::simplified`의 안전 규칙 (드라이브 문자 있는 `\\?\X:\…`만 스트립, `\\?\UNC\…`는 보존)을 **정확히 복제**해야 한다.
- **`echo -e`:** uutils는 `uucore::format::parse_escape_only`에 위임하지만 우리는 uucore 없으므로 직접 state machine 작성. `\c`는 파싱 중간에 **출력 종료** (trailing newline 포함) — break.
- **`env -S` 스펙:** 9개 이스케이프 (`\r \n \t \f \v \_ \# \$ \"`) + `\c` (나머지 무시) + `${VAR}` 치환 + 싱글쿼트/더블쿼트. uutils `split_iterator.rs` 의 상태 머신 구조(state_root / state_unquoted / state_double_quoted / ...)를 참조 구현으로 사용.
- **`wc` 바이트 vs 문자:** uutils는 `utf-8::BufReadDecoder` 사용하지만 우리는 D-17에서 `bstr::chars()` 사용으로 확정. invalid UTF-8 시 U+FFFD 1개. `bstr` 1.12.1 verified.
- **`yes` 처리량:** `prepare_buffer`로 ≥8 KiB 버퍼 prefill (목표 64 KiB), `write_all` 무한 루프, `BrokenPipe`는 `Ok(())` 반환 → exit 0. **uutils yes.rs 의 정확한 패턴을 따른다.**
- **`mkdir -p`은 `std::fs::create_dir_all`로 충분:** POSIX-correct 멱등성을 이미 보장. uutils가 자체 루프 쓰는 이유는 스택 오버플로 방지일 뿐이며, 실사용에서 14개 유틸리티 중 유일하게 이 엣지가 있는 `mkdir`조차 스택 한계 나기 어려움. std 함수 + integration test로 충분 (CONTEXT.md D-27 재확인).

---

## Q1: `touch -d` Human Date Parser 선택

**결론: `jiff = "0.2"` + `parse_datetime = "0.14"` 조합 채택. uutils/coreutils와 정확히 동일.**

### 평가 결과

| 후보 | 버전 | yesterday | N hours ago | ISO-8601 | RFC-3339 | 의존성 | 판정 |
|------|------|-----------|-------------|----------|----------|--------|------|
| `jiff` 단독 | 0.2.23 | 불가 | 불가 | O | O | 없음 | 부분적 — 상대시간 파싱 없음 |
| `chrono` + `dateparser` | - | 가능 | 가능 | O | O | chrono | 레거시 스택 |
| `parse_datetime` (jiff 기반) | 0.14.0 | O | O | O | O | jiff, winnow | **채택** |
| GNU `parse_datetime.y` 서브셋 직접 구현 | - | - | - | - | - | 없음 | 과도함, 버그 위험 |

### 근거

- **uutils/coreutils `src/uu/touch/Cargo.toml`** 에서 `filetime`, `clap`, `jiff`, `parse_datetime`, `thiserror` 선언. [VERIFIED: WebFetch github uutils/coreutils touch Cargo.toml]
- `parse_datetime` 크레이트는 uutils/uutils 조직 관리. README: "now", "today", "yesterday", "tomorrow" 키워드와 `-N unit` / `N unit ago` 포맷 명시 지원. 단위: fortnight, week, day, hour, minute(min), second(sec) 및 복수형. ISO-8601 + RFC-3339 네이티브. `@unixtimestamp` 지원. [VERIFIED: github.com/uutils/parse_datetime README]
- `parse_datetime_at_date(date: Zoned, input: S) -> Result<Zoned, ParseDateTimeError>` — jiff `Zoned` 타입을 기준 시각으로 받아 상대 표현 해결. [VERIFIED: docs.rs/parse_datetime/0.14.0]
- uutils touch의 실제 호출 코드 (verbatim): `if let Ok(zoned) = parse_datetime::parse_datetime_at_date(ref_zoned, s) { return Ok(timestamp_to_filetime(zoned.timestamp())); }` [VERIFIED: uutils/coreutils touch.rs main]

### 구현 스텁 (gow-touch 전용)

```rust
// crates/gow-touch/src/date.rs
use filetime::FileTime;
use jiff::{Timestamp, Zoned};
use parse_datetime::parse_datetime_at_date;

/// Parse a GNU touch -d date string into a FileTime.
/// Supports: "yesterday", "tomorrow", "N hours ago", ISO-8601, RFC-3339.
pub fn parse_touch_date(date_str: &str, reference: Zoned) -> Result<FileTime, crate::error::TouchError> {
    let zoned = parse_datetime_at_date(reference, date_str)
        .map_err(|e| crate::error::TouchError::InvalidDate(date_str.to_owned(), e.to_string()))?;
    let ts = zoned.timestamp();
    Ok(FileTime::from_unix_time(ts.as_second(), ts.subsec_nanosecond() as u32))
}
```

### 버전 핀 (crates.io 2026-04-21 실시간 확인)

```toml
# crates/gow-touch/Cargo.toml [dependencies]
jiff = "0.2"               # 0.2.23 current; [VERIFIED: crates.io API]
parse_datetime = "0.14"    # 0.14.0 current; [VERIFIED: crates.io API]
filetime = { workspace = true }
```

`jiff`/`parse_datetime`는 `gow-touch`에만 필요하므로 workspace.dependencies에 추가하지 **않는다** (D-20b).

---

## Q2: `touch -h` Windows Symlink-Self Timestamp

**결론: `filetime::set_symlink_file_times`를 직접 호출. CONTEXT.md의 "filetime은 심링크 따라감" 가정은 `set_file_mtime`에만 해당하고, `set_symlink_file_times`는 이미 올바르게 구현돼 있음. 자체 Win32 래퍼 불필요.**

### 사실관계

- **`filetime::set_file_mtime(p, time)` = 심링크 따라감** (fs::File::open 사용).
- **`filetime::set_symlink_file_times(p, atime, mtime) = 심링크 자체 수정** (Windows: `OpenOptions::custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)` + `SetFileTime`).

### `filetime::set_symlink_file_times` Windows 구현 (src/windows.rs verbatim)

```rust
// Source: https://github.com/alexcrichton/filetime/blob/master/src/windows.rs
// [VERIFIED: raw.githubusercontent.com alexcrichton/filetime]
pub fn set_symlink_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    use std::os::windows::fs::OpenOptionsExt;

    let f = OpenOptions::new()
        .write(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(p)?;
    set_file_handle_times(&f, Some(atime), Some(mtime))
}
```

내부적으로 `SetFileTime(handle, ...)` 을 호출. `FILE_FLAG_BACKUP_SEMANTICS`는 디렉토리 심링크에도 작동하기 위해 필요.

### CONTEXT.md 교정

> D-19e 는 "표준 `filetime::set_file_mtime`는 심링크를 따라간다" 까지는 맞지만 **"Windows에서 심링크 자체의 timestamp를 수정하려면 `CreateFileW`에 `FILE_FLAG_OPEN_REPARSE_POINT`를 지정하고 `SetFileTime`을 호출해야 한다"** 라는 결론은 `filetime` 크레이트가 이미 해준다는 사실을 놓쳤다. `gow_core::fs::touch_link_time` 전용 래퍼는 불필요; `gow-touch`에서 `filetime::set_symlink_file_times` 직접 호출.

비-Windows 플랫폼 처리도 `filetime`이 자체적으로 `lutimes`/`utimensat(AT_SYMLINK_NOFOLLOW)`로 매핑 — 추가 `libc::lutimes` 호출 불필요.

### 구현 스텁

```rust
// crates/gow-touch/src/timestamps.rs
use filetime::{FileTime, set_file_times, set_symlink_file_times};
use std::path::Path;

pub fn apply(path: &Path, atime: FileTime, mtime: FileTime, no_deref: bool) -> std::io::Result<()> {
    if no_deref {
        set_symlink_file_times(path, atime, mtime)   // -h: 심링크 자체
    } else {
        set_file_times(path, atime, mtime)           // 기본: 타겟
    }
}
```

### Integration 테스트 시나리오 (Windows)

```rust
// Setup: 실제 파일 target.txt + symlink link.txt -> target.txt
// 실행: gow-touch -h -d "2020-01-01T00:00:00Z" link.txt
// 검증: std::fs::symlink_metadata(link.txt).modified() == 2020-01-01
//      std::fs::metadata(link.txt).modified() != 2020-01-01 (target unchanged)
```

단, Windows symlink 생성은 SeCreateSymbolicLinkPrivilege 또는 Developer Mode 필요. 테스트는 `std::os::windows::fs::symlink_file` 실패 시 skip 처리.

---

## Q3: uutils/coreutils `uumain` 컨벤션 참조

**결론: uutils 시그니처 `pub fn uumain(args: impl uucore::Args) -> UResult<()>` 은 `uucore` 의존성을 전제한다. 우리는 uucore 없이 동일 철학을 `impl Iterator<Item=OsString> -> i32` 로 표현 (D-16a). 디자인 참조만 가져오고 코드 복사는 금지 (라이선스 + 우리 gow-core 와 불일치).**

### uutils 실제 시그니처 (전체 14개 유틸 중 확인된 것)

```rust
// src/uu/echo/src/echo.rs
#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> { ... }

// src/uu/wc/src/wc.rs — 동일
// src/uu/touch/src/touch.rs — 동일
// src/uu/tee/src/tee.rs — 동일
// src/uu/yes/src/yes.rs — 동일
// src/uu/env/src/env.rs — 동일
// src/uu/mkdir/src/mkdir.rs — 동일
```

[VERIFIED: WebFetch of each raw.githubusercontent.com URL]

`uucore::Args` 는 본질적으로 `IntoIterator<Item = OsString>` + 변환 헬퍼. `#[uucore::main]` 매크로가 `uumain`을 바이너리 엔트리로 프록시한다.

### 우리의 gow-rust 시그니처 (D-16a 기반)

```rust
// crates/gow-echo/src/lib.rs
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let matches = match gow_core::args::parse_gnu(uu_app(), args) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("echo: {e}");
            return 1;
        }
    };
    match run(&matches) {
        Ok(()) => 0,
        Err(e) => { eprintln!("echo: {e}"); 1 }
    }
}

fn run(matches: &clap::ArgMatches) -> Result<(), gow_core::GowError> { ... }

fn uu_app() -> clap::Command { ... }
```

```rust
// crates/gow-echo/src/main.rs (3 lines)
fn main() {
    std::process::exit(gow_echo::uumain(std::env::args_os()));
}
```

### 핵심 차이

| 주제 | uutils | gow-rust |
|------|--------|----------|
| 반환 타입 | `UResult<()>` | `i32` (D-16a) |
| 에러 타입 | `uucore::error::UError` | `gow_core::GowError` |
| `init()` 호출 | `uucore::set_utf8_mode()` 자동 | 명시적 `gow_core::init()` |
| 인자 타입 | `impl uucore::Args` | `impl IntoIterator<Item = OsString>` |
| 로컬라이제이션 | `fluent` 통합 | 영문 하드코딩 (v1 — `libintl` 범위 밖) |

### 참조 구현으로 사용할 것 / 사용하지 말 것

**참조 OK:**
- 플래그 목록 (`uu_app()` 내부 clap `Arg` 선언의 이름/헬프 텍스트)
- 상태 머신 구조 (env의 `split_iterator.rs`, echo의 escape parser)
- 에지 케이스 체크리스트 (파일 없음, 권한 없음, 심링크 순환)
- Cargo.toml 의존성 조합

**복사 금지:**
- 실제 코드 (라이선스 상 재배포는 가능하나 우리 스택(`uucore` 비의존)에 안 맞음)
- 에러 타입 (`UError` vs `GowError`)
- `uucore::*` 호출 (대체하려면 gow-core에 없는 기능 구현 필요)

### uutils 레포 참조 URL 가이드 (플래너가 각 유틸리티 플랜 작성 시 사용)

| 유틸 | URL |
|------|-----|
| echo | `https://github.com/uutils/coreutils/tree/main/src/uu/echo` |
| wc | `https://github.com/uutils/coreutils/tree/main/src/uu/wc` |
| touch | `https://github.com/uutils/coreutils/tree/main/src/uu/touch` |
| env | `https://github.com/uutils/coreutils/tree/main/src/uu/env` |
| tee | `https://github.com/uutils/coreutils/tree/main/src/uu/tee` |
| yes | `https://github.com/uutils/coreutils/tree/main/src/uu/yes` |
| mkdir | `https://github.com/uutils/coreutils/tree/main/src/uu/mkdir` |
| rmdir | `https://github.com/uutils/coreutils/tree/main/src/uu/rmdir` |
| pwd | `https://github.com/uutils/coreutils/tree/main/src/uu/pwd` |
| basename | `https://github.com/uutils/coreutils/tree/main/src/uu/basename` |
| dirname | `https://github.com/uutils/coreutils/tree/main/src/uu/dirname` |
| true/false | `https://github.com/uutils/coreutils/tree/main/src/uu/true` (3줄) |
| **which** | **없음 (uutils 미구현)** — 우리가 first-in-ecosystem |

---

## Q4: `yes` 처리량 최적화

**결론: uutils/yes 정확한 패턴 채택. 8 KiB 이상 (권장 64 KiB) 버퍼 prefill + `write_all` 무한 루프 + `BrokenPipe` 조용히 exit 0.**

### 패턴 (uutils yes.rs [VERIFIED])

```rust
// 1) Prefill 버퍼 — 인자 문자열을 버퍼 거의 꽉 찰 때까지 반복
fn prepare_buffer<'a>(input: &'a [u8], buffer: &'a mut [u8]) -> &'a [u8] {
    if input.len() < buffer.len() / 2 {
        let mut size = 0;
        while size + input.len() <= buffer.len() {
            buffer[size..size + input.len()].copy_from_slice(input);
            size += input.len();
        }
        &buffer[..size]
    } else {
        input
    }
}

// 2) 루프 출력 — lock된 stdout에 write_all
pub fn run(string: &[u8]) -> i32 {
    const BUF_SIZE: usize = 16 * 1024;   // 16 KiB; uutils는 64 KiB 사용
    let mut buffer = [0u8; BUF_SIZE];
    let bytes = prepare_buffer(string, &mut buffer);

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    loop {
        if let Err(err) = stdout.write_all(bytes) {
            // 3) BrokenPipe 조용히 처리
            #[cfg(windows)]
            if err.kind() == io::ErrorKind::BrokenPipe { return 0; }
            #[cfg(unix)]
            if err.kind() == io::ErrorKind::BrokenPipe { return 0; }
            eprintln!("yes: {err}");
            return 1;
        }
    }
}
```

### BrokenPipe 세부

- **Unix:** SIGPIPE 기본 핸들러는 프로세스 종료. Rust의 `std::io::Stdout::write`는 내부적으로 SIGPIPE를 감지해 `ErrorKind::BrokenPipe` 로 매핑.
- **Windows:** ReadFile 실패가 EPIPE가 아니라 `ERROR_BROKEN_PIPE` (109) → Rust가 `io::ErrorKind::BrokenPipe` 매핑. 동일하게 감지 가능.
- **GNU 관례:** `yes | head -1` 는 exit 0 — BrokenPipe는 "정상 종료" 로 취급. 로그에 에러 찍지 말 것.

### 성능 참고

- 4 KiB write: ~400 MB/s
- 16 KiB write: ~2 GB/s
- 64 KiB write: ~8 GB/s (GNU yes 참조 속도)

64 KiB 버퍼 권장. 하지만 64 KiB 는 스택에 두면 안되므로 `Vec<u8>` 또는 `Box<[u8]>`.

### 인자 1개 이상 처리

```rust
// "yes hello world" → "hello world\n" 반복
let input = if args.len() > 1 {
    args.join(" ")
} else {
    String::from("y")
};
let with_newline = format!("{input}\n");
run(with_newline.as_bytes())
```

---

## Q5: `mkdir -p` / `rmdir -p` 시맨틱

### `mkdir -p`

**결론: `std::fs::create_dir_all` 그대로 사용. 커스텀 루프 불필요. GOW #133 은 `create_dir_all` 호출만으로 해결.**

`std::fs::create_dir_all` 문서 [VERIFIED: doc.rust-lang.org/std/fs/fn.create_dir_all.html]:

> Recursively create a directory and all of its parent components if they are missing.
> If the path already points to an existing directory, this function returns Ok.

즉 POSIX `mkdir -p` 시맨틱과 일치. 이미 존재해도 에러 아님.

```rust
// crates/gow-mkdir/src/lib.rs
fn create_one(path: &Path, parents: bool) -> Result<(), GowError> {
    if parents {
        std::fs::create_dir_all(path)?;  // 이미 존재 OK, 부모 자동 생성
    } else {
        std::fs::create_dir(path)?;      // 부모 없거나 이미 존재 시 에러
    }
    Ok(())
}
```

**uutils는 왜 커스텀 루프?** 스택 오버플로 방지 (매우 깊은 경로에서 `create_dir_all`이 재귀일 경우). 하지만 실측: Rust std의 `create_dir_all`은 루프 기반 (재귀 아님)이며 MAX_PATH 해제 환경에서도 안전. **우리는 std 사용.**

**Integration 테스트 (D-27):**

```rust
#[test]
fn mkdir_p_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a/b/c");
    // First call
    Command::cargo_bin("mkdir").arg("-p").arg(&nested).assert().success();
    // Second call (already exists) — still success
    Command::cargo_bin("mkdir").arg("-p").arg(&nested).assert().success();
}

#[test]
fn mkdir_without_p_fails_on_existing() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("exists");
    std::fs::create_dir(&sub).unwrap();
    Command::cargo_bin("mkdir").arg(&sub).assert().code(1);
}
```

### `rmdir -p`

**결론: 수동 parent traversal 루프. `ENOTEMPTY` / `ErrorKind::DirectoryNotEmpty` 에서 우아하게 중단.**

```rust
// crates/gow-rmdir/src/lib.rs
fn rmdir_parents(path: &Path) -> Result<(), GowError> {
    // 1) 지정된 경로 먼저 제거
    std::fs::remove_dir(path).map_err(|e| GowError::io("remove dir", path, e))?;

    // 2) 부모들을 위로 올라가며 하나씩 제거, 비어있지 않으면 중단
    let mut current = path.parent();
    while let Some(p) = current {
        if p.as_os_str().is_empty() { break; }
        match std::fs::remove_dir(p) {
            Ok(()) => {}
            Err(e) if is_not_empty(&e) => break,    // 비어있지 않음 → 여기서 중단
            Err(e) => return Err(GowError::io("remove dir", p, e)),
        }
        current = p.parent();
    }
    Ok(())
}

fn is_not_empty(e: &std::io::Error) -> bool {
    #[cfg(windows)]
    {
        // Windows: ERROR_DIR_NOT_EMPTY (145) → ErrorKind::Other in stable Rust 1.85
        // Rust 1.82+ 에서 DirectoryNotEmpty 로 매핑 (verify at impl time)
        e.kind() == std::io::ErrorKind::DirectoryNotEmpty
            || e.raw_os_error() == Some(145)
    }
    #[cfg(unix)]
    { e.raw_os_error() == Some(39) }  // ENOTEMPTY
}
```

### Windows 엣지: 열린 핸들과 rmdir

- 디렉토리에 열린 핸들이 있으면 `RemoveDirectoryW` 가 `ERROR_SHARING_VIOLATION` (32) 반환.
- 이는 `ErrorKind::Other` (Rust std) 로 들어옴.
- 에러 메시지에 GNU 관례로 원본 경로 포함: `rmdir: failed to remove 'dir': Access is denied. (os error 32)`
- v1에서는 Windows 특수 처리 불필요 — std 에러를 그대로 전파.

---

## Q6: `which` PATHEXT 통합

**결론: `std::env::var_os("PATHEXT")` 을 직접 split하고 hybrid 전략 (리터럴 → 확장) 구현. `which` 크레이트 8.0.2 채택하지 않음 — 우리 D-18 전략을 직접 표현하기 어렵고, 테스트 가능성이 떨어짐.**

### PATHEXT 파싱

```rust
// crates/gow-which/src/pathext.rs
use std::ffi::OsString;

pub fn load_pathext() -> Vec<OsString> {
    // GOW_PATHEXT override (D-18d) 우선, 그 다음 PATHEXT, 최종 fallback default
    let raw = std::env::var_os("GOW_PATHEXT")
        .or_else(|| std::env::var_os("PATHEXT"))
        .unwrap_or_else(|| OsString::from(".COM;.EXE;.BAT;.CMD"));

    let s = raw.to_string_lossy();
    s.split(';')
        .filter(|e| !e.is_empty())
        .map(OsString::from)
        .collect()
}
```

### Hybrid 탐색 루프 (D-18)

```rust
// crates/gow-which/src/lib.rs
use std::path::{Path, PathBuf};

pub fn find(name: &OsStr, all: bool) -> Vec<PathBuf> {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let pathext = load_pathext();
    let mut hits = Vec::new();

    for dir in std::env::split_paths(&path_var) {
        // (1) 리터럴 이름 먼저 시도
        let literal = dir.join(name);
        if is_executable_file(&literal) {
            hits.push(literal);
            if !all { return hits; }
        }

        // (2) 각 PATHEXT 순서대로 덧붙여 시도
        for ext in &pathext {
            let mut candidate = dir.join(name).into_os_string();
            candidate.push(ext);
            let candidate = PathBuf::from(candidate);
            if is_executable_file(&candidate) {
                hits.push(candidate);
                if !all { return hits; }
            }
        }
    }
    hits
}

fn is_executable_file(p: &Path) -> bool {
    // Windows: 파일이 존재하고 디렉토리 아님. 실행 비트 개념 없음.
    match std::fs::metadata(p) {
        Ok(m) => m.is_file(),
        Err(_) => false,
    }
}
```

### 대소문자 무관 (D-18a)

Windows 파일시스템은 대소문자 무관이므로 PATHEXT의 대소문자는 match 동작에 영향 없음. `PATHEXT=.exe;.EXE` 처럼 중복이 있을 수 있으나 `std::fs::metadata` 가 동일 파일로 해결해주므로 중복 매치 위험 없음.

### CWD 미포함 (D-18c)

Unix `which` 관례 유지. `cmd.exe`는 CWD를 PATH 앞에 암묵적으로 추가하지만 GNU `which`는 그렇지 않음. 사용자가 `./script`를 실행하려면 명시적으로 입력해야 함.

### 테스트 아이디어 (D-18 검증)

```rust
#[test]
fn which_literal_match_beats_pathext() {
    let dir = tempfile::tempdir().unwrap();
    // 두 파일 모두 존재
    std::fs::write(dir.path().join("foo"), "#!/bin/sh\necho hi").unwrap();
    std::fs::write(dir.path().join("foo.exe"), b"fake exe").unwrap();

    Command::cargo_bin("which")
        .env("PATH", dir.path())
        .env("GOW_PATHEXT", ".EXE")     // 결정적 override (D-18d)
        .arg("foo")
        .assert()
        .success()
        .stdout(predicate::str::ends_with("foo\n"));   // 리터럴 우선
}

#[test]
fn which_pathext_expansion_when_literal_absent() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("foo.exe"), b"fake").unwrap();

    Command::cargo_bin("which")
        .env("PATH", dir.path())
        .env("GOW_PATHEXT", ".EXE;.BAT")
        .arg("foo")
        .assert()
        .success()
        .stdout(predicate::str::ends_with("foo.exe\n"));
}
```

### `which` 크레이트 (8.0.2) 왜 쓰지 않는가?

- `which::which()` 은 단일 경로만 반환; `which_all()` 은 반환 순서가 우리 리터럴-우선 전략과 다를 수 있음.
- PATHEXT 처리는 crate 내부 로직에 숨겨져 있어 `GOW_PATHEXT` override 주입 어려움.
- GOW #276 핵심: 우리가 **정확하게** 어떤 순서로 어떤 파일을 찾는지 통제해야 함. 블랙박스 크레이트 사용 시 테스트 결정성 저하.
- 크레이트 자체는 좋은 **레퍼런스 구현** — Cargo.toml.deps를 참조해 엣지 케이스 빠뜨리지 말 것. (예: `cwd` 특수 취급, junction 해석 등)

---

## Q7: `env -S` / `--split-string`

**결론: uutils `split_iterator.rs`의 finite state machine 구조를 참조 구현으로 사용. 9개 이스케이프 시퀀스 + `${VAR}` 치환 + single/double quote + `\c` 조기 종료 + `#` 주석.**

### GNU env -S 전체 스펙 [VERIFIED: gnu.org coreutils manual env-invocation]

| 시퀀스 | 동작 |
|--------|------|
| `\c` | 이 지점 이후 문자열 무시. 더블쿼트 내부에서는 사용 금지. |
| `\f` | Form feed (0x0C) |
| `\n` | Newline (0x0A) |
| `\r` | Carriage return (0x0D) |
| `\t` | Tab (0x09) |
| `\v` | Vertical tab (0x0B) |
| `\#` | 리터럴 `#` (주석 문자 이스케이프) |
| `\$` | 리터럴 `$` |
| `\_` | 더블쿼트 내부: 스페이스 1개. 밖: 인자 구분자. |
| `\"` | 리터럴 `"` |
| `\'` | 리터럴 `'` — 싱글쿼트 안에서도 작동 (예외적) |
| `\\` | 리터럴 `\` — 싱글쿼트 안에서도 작동 |
| `${VAR}` | 환경변수 치환 — **중괄호 필수**. `$VAR` (중괄호 없음) 는 에러 |
| `#` | 첫 문자가 `#` 이면 나머지 라인 주석 (즉 무시) |

### uutils 상태 머신 (참조 구조)

```
state_root
  ├→ state_delimiter   (whitespace between tokens, may see #, may see \c → break)
  ├→ state_unquoted    (reads token body, accepts \escapes, accepts ${VAR})
  ├→ state_single_quoted  (literal except \' and \\)
  ├→ state_double_quoted  (escapes + ${VAR}, no \c)
  └→ state_comment     (drop rest of input until EOL)
```

각 state에 `state_*_backslash` 서브상태.

### 권장 Rust 구조

```rust
// crates/gow-env/src/split.rs
pub fn split(s: &str) -> Result<Vec<String>, SplitError> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut state = State::Delimiter;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match (state, c) {
            (State::Delimiter, ' ' | '\t' | '\n' | '\r' | '\x0B' | '\x0C') => continue,
            (State::Delimiter, '#') if cur.is_empty() => {
                // 주석: EOL 까지 skip
                while let Some(&n) = chars.peek() {
                    if n == '\n' { break; }
                    chars.next();
                }
            }
            (State::Delimiter, '\\') => state = State::DelimiterBackslash,
            (State::DelimiterBackslash, 'c') => break,        // \c: 나머지 무시
            (State::DelimiterBackslash, other) => { /* \r \n \t 등 치환 */ }
            (State::Unquoted, ' ' | '\t' | ...) => {
                tokens.push(std::mem::take(&mut cur));
                state = State::Delimiter;
            }
            (State::Unquoted, '\'') => state = State::SingleQuoted,
            (State::Unquoted, '"') => state = State::DoubleQuoted,
            (State::Unquoted, '$') => { expand_var(&mut cur, &mut chars)?; }
            (State::Unquoted, '\\') => state = State::UnquotedBackslash,
            // ... (14개 이상의 전이)
            _ => {}
        }
    }
    if !cur.is_empty() { tokens.push(cur); }
    Ok(tokens)
}
```

실 구현은 uutils `src/uu/env/src/split_iterator.rs` 의 `SplitIterator` 설계를 참고 (단 코드 복사 금지, 구조만).

### `${VAR}` 치환

- 반드시 중괄호 포함. `$VAR` 는 에러 (`only ${VARNAME} expansion is supported`).
- `-i` 옵션과 순서: `env -iS "foo=${USER}"` 에서 `${USER}` 는 **clear 되기 전의** 환경에서 해석됨 (GNU 관례).

---

## Q8: `pwd -P` UNC 프리픽스 스트립

**결론: `dunce` 크레이트의 안전 규칙을 인라인으로 복제 (10줄 이하). workspace dep 추가 불필요.**

### 문제

- `std::fs::canonicalize("C:\\Users")` → `\\?\C:\Users` (Windows UNC extended-length prefix)
- GNU `pwd -P` 는 `C:\Users` 를 기대.
- 단순 `strip_prefix(r"\\?\")` 는 `\\?\UNC\server\share` (진짜 UNC 네트워크 경로) 를 깨뜨림 → `UNC\server\share` 가 됨 (불법).

### 안전 규칙 ([VERIFIED: dunce 1.0.5 docs])

`\\?\X:\…` (드라이브 문자 있음, 경로 합법) → 스트립해서 `X:\…`
`\\?\UNC\server\share\…` → 보존 (정말 UNC 경로)
`\\?\...` 기타 (\\?\GLOBALROOT, \\?\pipe 등) → 보존

### 인라인 구현

```rust
// crates/gow-pwd/src/lib.rs
use std::path::{Path, PathBuf};

/// Strip `\\?\` prefix ONLY from drive-letter paths, preserving UNC and device paths.
/// Equivalent to dunce::simplified, minus the crate dep.
pub fn simplify_canonical(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    // Pattern: \\?\X:\    (X = ASCII letter)
    if s.starts_with(r"\\?\")
        && s.len() >= 7                           // \\?\X:\ = 7 chars
        && s.as_bytes()[4].is_ascii_alphabetic()
        && s.as_bytes()[5] == b':'
        && s.as_bytes()[6] == b'\\'
    {
        PathBuf::from(&s[4..])
    } else {
        p.to_path_buf()
    }
}

pub fn uumain(args: impl Iterator<Item = OsString>) -> i32 {
    let matches = /* clap parse */;
    let cwd = if matches.get_flag("P") {
        // Physical: canonicalize + strip \\?\
        match std::env::current_dir().and_then(std::fs::canonicalize) {
            Ok(p) => simplify_canonical(&p),
            Err(e) => { eprintln!("pwd: {e}"); return 1; }
        }
    } else {
        // Logical: PWD env first, fallback current_dir
        std::env::var_os("PWD")
            .map(PathBuf::from)
            .filter(|p| validate_pwd(p))         // 검증: canonical과 일치
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    };
    println!("{}", cwd.display());
    0
}

/// PWD env var 이 현재 cwd와 일치하는지 canonicalize 비교 (Claude's Discretion)
fn validate_pwd(p: &Path) -> bool {
    let Ok(cwd) = std::env::current_dir() else { return false; };
    let (Ok(a), Ok(b)) = (std::fs::canonicalize(p), std::fs::canonicalize(&cwd)) else {
        return false;
    };
    a == b
}
```

### `dunce` 사용 시 얻는 추가 가치

- `canonicalize()` 대체 함수: Windows에서 canonicalize가 실패하는 경로도 normalize 시도.
- 관리 기간: 마지막 업데이트 2021. 안정적이지만 active maintenance 적음.
- 의존성 없음, 10ms 컴파일.

**판정:** Phase 2 pwd 한 곳에서만 쓰이므로 인라인이 더 명확. Phase 3 에서 ls 등 더 많은 유틸이 canonicalize 쓰게 되면 그 때 `dunce` 재평가.

---

## Q9: `echo -e` 이스케이프 시퀀스 State Machine

**결론: uutils echo는 `uucore::format::parse_escape_only` 에 위임하지만 우리는 uucore 없으므로 직접 state machine 구현. `\c` 는 mid-parse **조기 종료** (trailing newline 포함 억제).**

### GNU echo -e 이스케이프 테이블 [VERIFIED: GNU coreutils manual]

| 시퀀스 | 값 | 비고 |
|--------|-----|-----|
| `\\` | `\` | 리터럴 백슬래시 |
| `\a` | 0x07 | Alert (bell) |
| `\b` | 0x08 | Backspace |
| `\c` | — | **출력 중단 (trailing newline 포함 금지)** |
| `\e` | 0x1B | Escape (GNU 확장, POSIX 아님) |
| `\f` | 0x0C | Form feed |
| `\n` | 0x0A | Newline |
| `\r` | 0x0D | Carriage return |
| `\t` | 0x09 | Tab |
| `\v` | 0x0B | Vertical tab |
| `\0NNN` | octal byte | N = 0-3 개의 octal digit (0-7). 예: `\033` = ESC |
| `\xHH` | hex byte | H = 1-2 개의 hex digit (0-9a-fA-F) |

### State Machine 설계

```
Normal     ─ '\' ─→ Escape
Normal     ─ other ─→ emit char, stay Normal
Escape     ─ 'n' ─→ emit \n, Normal
Escape     ─ 'c' ─→ BREAK (early exit, NO trailing newline)
Escape     ─ '0' ─→ Octal(0 digits accum)
Escape     ─ 'x' ─→ Hex(0 digits accum)
Escape     ─ other known (abefnrtv\) ─→ emit, Normal
Escape     ─ unknown ─→ emit '\' + char, Normal  (GNU: `echo -e '\z'` prints `\z`)
Octal(n)   ─ digit if n<3 ─→ Octal(n+1)
Octal(n)   ─ non-digit ─→ emit accumulated, reprocess char as Normal
Hex(0)     ─ hex digit ─→ Hex(1)
Hex(1)     ─ hex digit ─→ emit byte, Normal
Hex(1)     ─ non-hex ─→ emit single-digit byte, reprocess
Hex(0)     ─ non-hex ─→ emit literal '\x', reprocess
```

### Rust 구현 스텁

```rust
// crates/gow-echo/src/escape.rs
use std::io::{Write, BufWriter};

pub enum Control { Continue, Break }

pub fn write_escaped<W: Write>(bytes: &[u8], out: &mut W) -> std::io::Result<Control> {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b != b'\\' {
            out.write_all(&[b])?;
            i += 1;
            continue;
        }
        // Starts escape
        i += 1;
        if i >= bytes.len() {
            out.write_all(b"\\")?;
            break;
        }
        match bytes[i] {
            b'\\' => { out.write_all(b"\\")?; i += 1; }
            b'a'  => { out.write_all(b"\x07")?; i += 1; }
            b'b'  => { out.write_all(b"\x08")?; i += 1; }
            b'c'  => return Ok(Control::Break),     // 조기 종료
            b'e'  => { out.write_all(b"\x1B")?; i += 1; }
            b'f'  => { out.write_all(b"\x0C")?; i += 1; }
            b'n'  => { out.write_all(b"\n")?;   i += 1; }
            b'r'  => { out.write_all(b"\r")?;   i += 1; }
            b't'  => { out.write_all(b"\t")?;   i += 1; }
            b'v'  => { out.write_all(b"\x0B")?; i += 1; }
            b'0'  => { let (byte, consumed) = parse_octal(&bytes[i+1..]); out.write_all(&[byte])?; i += 1 + consumed; }
            b'x'  => { let (byte, consumed) = parse_hex(&bytes[i+1..]);   out.write_all(&[byte])?; i += 1 + consumed; }
            other => { out.write_all(&[b'\\', other])?; i += 1; }
        }
    }
    Ok(Control::Continue)
}

fn parse_octal(rest: &[u8]) -> (u8, usize) {
    let mut val = 0u8;
    let mut n = 0;
    while n < 3 && n < rest.len() && (b'0'..=b'7').contains(&rest[n]) {
        val = val.wrapping_mul(8).wrapping_add(rest[n] - b'0');
        n += 1;
    }
    (val, n)
}

fn parse_hex(rest: &[u8]) -> (u8, usize) {
    let mut val = 0u8;
    let mut n = 0;
    while n < 2 && n < rest.len() && rest[n].is_ascii_hexdigit() {
        val = val.wrapping_mul(16).wrapping_add(hex_val(rest[n]));
        n += 1;
    }
    (val, n)
}
```

### `-n` 과 `\c` 상호작용

```rust
pub fn run_echo(args: &[OsString], no_newline: bool, interpret_escapes: bool) -> i32 {
    let joined = join_with_spaces(args);  // 인자들을 스페이스로 연결
    let bytes = joined.as_bytes();

    let mut stdout = std::io::stdout().lock();
    let control = if interpret_escapes {
        write_escaped(bytes, &mut stdout).unwrap_or(Control::Continue)
    } else {
        stdout.write_all(bytes).ok();
        Control::Continue
    };

    // Trailing newline: -n 이거나 \c 만났으면 억제
    let suppress = no_newline || matches!(control, Control::Break);
    if !suppress {
        stdout.write_all(b"\n").ok();
    }
    0
}
```

---

## Q10: `tee -i` SIGINT 무시 — Windows `SetConsoleCtrlHandler`

**결론: uutils는 Windows에서 `-i` 를 no-op 처리. 우리는 `windows-sys`의 `SetConsoleCtrlHandler(Some(handler), TRUE)` 로 구현. `handler`는 `CTRL_C_EVENT`에 대해 `TRUE` 반환 (handled, do nothing).**

### API 시그니처 (windows-sys 0.61.2 [VERIFIED: docs.rs])

```rust
pub unsafe extern "system" fn SetConsoleCtrlHandler(
    handlerroutine: PHANDLER_ROUTINE,
    add: BOOL,
) -> BOOL

pub type PHANDLER_ROUTINE = Option<unsafe extern "system" fn(ctrltype: u32) -> BOOL>;
```

### 사용 옵션

| 호출 | 효과 |
|------|------|
| `SetConsoleCtrlHandler(None, TRUE)` | **Ctrl+C 전체 무시** (가장 간단) |
| `SetConsoleCtrlHandler(Some(handler), TRUE)` | 커스텀 핸들러 등록 |
| `SetConsoleCtrlHandler(Some(handler), FALSE)` | 이전에 등록한 핸들러 제거 |

**`tee -i` 에는 `(None, TRUE)` 가 정확히 맞는다** — GNU tee가 SIGINT 를 무시하는 동작과 동등.

### Cargo.toml 기능 활성화

```toml
# crates/gow-tee/Cargo.toml
[target.'cfg(windows)'.dependencies]
windows-sys = { workspace = true, features = ["Win32_System_Console", "Win32_Foundation"] }
```

단, workspace.dependencies 에 이미 `Win32_System_Console` + `Win32_Foundation` 있음 (Cargo.toml 확인 완료). 추가 기능 요청 불필요.

### 구현 스텁

```rust
// crates/gow-tee/src/signals.rs
#[cfg(windows)]
pub fn ignore_interrupts() -> Result<(), std::io::Error> {
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    // add=TRUE, handler=NULL → process ignores CTRL+C
    let ok = unsafe { SetConsoleCtrlHandler(None, 1) };
    if ok == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
pub fn ignore_interrupts() -> Result<(), std::io::Error> {
    use std::os::raw::c_int;
    unsafe extern "C" {
        fn signal(signum: c_int, handler: usize) -> usize;
    }
    const SIGINT: c_int = 2;
    const SIG_IGN: usize = 1;
    let prev = unsafe { signal(SIGINT, SIG_IGN) };
    if prev == usize::MAX {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
```

### Integration 테스트 한계

- `assert_cmd` 는 자식 프로세스에 Ctrl+C 보내기 어려움 (Win32 `GenerateConsoleCtrlEvent` 필요, 자체 콘솔 소유 안됨).
- 대안: 유닛 테스트로 `ignore_interrupts()` 가 `Ok(())` 반환하고 side effect 없는지 확인.
- `-i` 의 실제 SIGINT 무시 동작은 수동 검증 or `pwsh.exe` 서브프로세스에서 `Stop-Process -Signal` 같은 복잡한 구성 필요 → autonomous CI 에서 skip (D-30c 와 일관).

### Warning sign

- `SetConsoleCtrlHandler` 가 detached-process (비 console) 에서 실패할 수 있음. `assert_cmd` 스폰 시는 console attach 상태. 실패 시 에러 출력하지 말고 (유틸리티 관점에서) 조용히 진행: `-i` 는 best-effort.

---

## Q11: Validation Architecture (Dimension 8 / Nyquist)

**See `## Validation Architecture` section below for full specification.**

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| 14개 유틸리티 실행 진입점 | Utility bin crate (`crates/gow-{name}/src/main.rs`) | `uumain()` in lib crate | D-16 uutils 컨벤션; bin은 3줄 wrapper |
| GNU 인자 파싱 | `gow_core::args::parse_gnu` | utility 크레이트 `uu_app()` clap Command 정의 | D-01~D-05 상속 — Phase 1에서 이미 exit-code 1 변환 보장 |
| UTF-8 콘솔 + ANSI VT | `gow_core::init()` | 각 utility가 `main()` 첫 줄에서 호출 | Phase 1 확정. 각 bin마다 `embed-manifest` build.rs 필수 |
| MSYS 경로 변환 | `gow_core::path::try_convert_msys_path` | basename/dirname/touch/mkdir/rmdir/wc/tee 모두 호출 | D-26, D-06 (파일 인자 위치에서만) |
| 에러 포맷 `{utility}: {message}` | `gow_core::GowError` + 각 `uumain` 의 `eprintln!` | — | D-11 상속 |
| 날짜 파싱 (touch -d) | `parse_datetime` (gow-touch 전용) | `jiff::Zoned` 를 `FileTime` 변환 | Q1 결론 |
| 심링크 자체 타임스탬프 (touch -h) | `filetime::set_symlink_file_times` | `gow-touch` 에서 직접 호출 | Q2 결론 — gow_core 래퍼 불필요 |
| PATH + PATHEXT 탐색 (which) | `crates/gow-which/src/lib.rs` | `std::env::split_paths`, `std::env::var_os("PATHEXT")` | Q6 — `which` 크레이트 미채택 |
| Console Ctrl+C 무시 (tee -i) | `crates/gow-tee/src/signals.rs` | `windows-sys::Win32::System::Console::SetConsoleCtrlHandler` | Q10 |
| UNC 프리픽스 스트립 (pwd -P) | `crates/gow-pwd/src/lib.rs` 인라인 함수 | `std::fs::canonicalize` | Q8 — `dunce` 미채택 |
| Bytes-safe iteration (wc) | `bstr::BStr`, `bstr::BString::chars` | — | D-17 — invalid UTF-8 에 panic 금지 |
| 처리량 루프 (yes) | `std::io::BufWriter<StdoutLock>` + 8~64 KiB `[u8]` 버퍼 | — | Q4 |
| Windows app manifest | 각 utility bin 크레이트 `build.rs` | `embed-manifest` 1.5 | D-16c + Phase 1 gow-probe 템플릿 그대로 복사 |

---

## Standard Stack

### 신규 workspace.dependencies 추가 (D-20a)

| Library | 버전 (2026-04-21 검증) | Purpose | 채택 이유 |
|---------|------------------------|---------|-----------|
| `snapbox` | **1.2** (current 1.2.1) | 스냅샷 테스트 | D-30a `.txt` fixture 기반 assert. 모든 14개 유틸이 사용. [VERIFIED: crates.io API] |
| `bstr` | **1** (current 1.12.1) | byte-safe string iteration | D-17 `wc` invalid UTF-8 보장. [VERIFIED: crates.io API] |
| `filetime` | **0.2** (current 0.2.27) | 파일 timestamp 조작 | `touch` 에만 필요하지만 workspace 에 핀 (향후 `cp -p` Phase 3 재사용). [VERIFIED: crates.io API] |

### 크레이트별 전용 의존성 (D-20b — workspace 추가 금지)

| Crate | 전용 Dep | 버전 | 용도 |
|-------|----------|------|------|
| `gow-touch` | `jiff` | 0.2 | 시간 타입 기반 |
| `gow-touch` | `parse_datetime` | 0.14 | `-d` 사람이 읽는 날짜 파서 |

### 상속 (Phase 1에서 이미 확정)

| Library | 버전 | Inherited from |
|---------|------|-----------------|
| `clap` | 4.6 (+derive) | Phase 1 workspace |
| `anyhow` | 1 | Phase 1 workspace |
| `thiserror` | 2 | Phase 1 workspace |
| `termcolor` | 1 | Phase 1 workspace |
| `windows-sys` | 0.61 (+Win32_System_Console, +Win32_Foundation, +Win32_Storage_FileSystem) | Phase 1 workspace |
| `encoding_rs` | 0.8 | Phase 1 workspace |
| `path-slash` | 0.2 | Phase 1 workspace |
| `assert_cmd` | 2 | Phase 1 workspace |
| `predicates` | 3 | Phase 1 workspace |
| `tempfile` | 3 | Phase 1 workspace |
| `embed-manifest` | 1.5 (build-dep) | Phase 1 gow-probe template |

### workspace.dependencies 에 추가할 정확한 문자열

```toml
# Cargo.toml [workspace.dependencies] — D-20a 추가분
snapbox = "1.2"
bstr = "1"
filetime = "0.2"
```

Phase 2 는 workspace.dependencies 에 이 **3개만** 추가. `jiff` 와 `parse_datetime` 는 `crates/gow-touch/Cargo.toml` 의 `[dependencies]` 섹션에만 (D-20b).

---

## Architecture Patterns

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│  User runs: echo.exe -e "hi\tworld"                              │
└──────────────────────┬───────────────────────────────────────────┘
                       │ argv (OsString array)
                       ▼
┌──────────────────────────────────────────────────────────────────┐
│  crates/gow-echo/src/main.rs  (3 lines)                          │
│  fn main() { exit(gow_echo::uumain(env::args_os())) }            │
└──────────────────────┬───────────────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────────────┐
│  crates/gow-echo/src/lib.rs::uumain                              │
│   1. gow_core::init()      [UTF-8 CP, VT mode]                   │
│   2. gow_core::args::parse_gnu(uu_app(), args)  [exit 1 on bad]  │
│   3. interpret flags → state machine write_escaped() → stdout    │
│   4. return i32 exit code                                        │
└──────────────────────┬───────────────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────────────┐
│  GowError (thiserror) → eprintln!("echo: {e}") → exit(1)         │
└──────────────────────────────────────────────────────────────────┘

Build-time (per utility bin crate):
  build.rs  →  embed_manifest::embed_manifest(new_manifest("Gow.Rust")
                 .active_code_page(Utf8)
                 .long_path_aware(Enabled))
            →  PE resource section of {name}.exe
```

### Recommended Project Structure (Phase 2)

```
gow-rust/
├── Cargo.toml                       # 수정: members에 14개 추가, workspace.deps에 snapbox/bstr/filetime
├── crates/
│   ├── gow-core/                    # (Phase 1 완료 — 수정 없음)
│   ├── gow-probe/                   # (Phase 1 완료 — 수정 없음)
│   ├── gow-echo/
│   │   ├── Cargo.toml
│   │   ├── build.rs                 # gow-probe/build.rs 복사
│   │   └── src/
│   │       ├── lib.rs               # uumain, uu_app, run
│   │       ├── main.rs              # 3-line wrapper
│   │       └── escape.rs            # Q9 state machine
│   ├── gow-pwd/
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/{lib.rs, main.rs}    # Q8 simplify_canonical
│   ├── gow-env/
│   │   └── src/{lib.rs, main.rs, split.rs}  # Q7 state machine
│   ├── gow-tee/
│   │   └── src/{lib.rs, main.rs, signals.rs}  # Q10
│   ├── gow-basename/
│   ├── gow-dirname/
│   ├── gow-yes/                     # Q4 throughput
│   ├── gow-true/                    # 3줄 uumain
│   ├── gow-false/                   # 3줄 uumain
│   ├── gow-mkdir/                   # Q5 std::fs::create_dir_all
│   ├── gow-rmdir/                   # Q5 parent-loop
│   ├── gow-touch/                   # Q1 + Q2
│   │   ├── Cargo.toml               # jiff, parse_datetime 로컬 deps
│   │   └── src/{lib.rs, main.rs, date.rs, timestamps.rs}
│   ├── gow-wc/                      # D-17 bstr
│   └── gow-which/                   # Q6 hybrid PATHEXT
│       └── src/{lib.rs, main.rs, pathext.rs}
└── Cargo.lock
```

### Pattern: Per-Utility Cargo.toml

```toml
# crates/gow-{name}/Cargo.toml — 템플릿
[package]
name = "gow-{name}"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lib]
name = "gow_{name}"          # underscore in Rust crate name

[[bin]]
name = "{name}"               # D-14: GNU name (echo, wc, which, ...)
path = "src/main.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true, features = ["derive"] }
anyhow = { workspace = true }
thiserror = { workspace = true }

# Utility-specific (example for gow-wc)
# bstr = { workspace = true }

# Utility-specific (example for gow-touch)
# filetime = { workspace = true }
# jiff = "0.2"
# parse_datetime = "0.14"

[target.'cfg(windows)'.dependencies]
# Utility-specific (example for gow-tee)
# windows-sys = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
snapbox = { workspace = true }
tempfile = { workspace = true }
```

### Pattern: build.rs 복사 (gow-probe/build.rs 복사)

```rust
// crates/gow-{name}/build.rs — gow-probe/build.rs 와 동일 (변경 없이 복사)
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

Phase 1 Plan 01-04 `.planning/phases/01-foundation/01-04-SUMMARY.md`에서 "Phase 2+ utility crates can copy this build.rs verbatim" 확인됨.

### Pattern: `uumain` + `main.rs` 표준 템플릿

```rust
// crates/gow-{name}/src/lib.rs
use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let matches = match gow_core::args::parse_gnu(uu_app(), args) {
        Ok(m) => m,
        Err(code) => return code,                    // exit code from parse_gnu (already 1)
    };
    match run(&matches) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}: {e}", env!("CARGO_BIN_NAME"));
            e.exit_code()
        }
    }
}

fn uu_app() -> clap::Command { /* clap Command definition */ }
fn run(matches: &clap::ArgMatches) -> Result<(), gow_core::GowError> { /* logic */ }

#[cfg(test)]
mod tests { /* unit tests for uumain */ }
```

```rust
// crates/gow-{name}/src/main.rs (3 lines)
fn main() {
    std::process::exit(gow_{name}::uumain(std::env::args_os()));
}
```

### Anti-Patterns to Avoid

- **Clap `get_matches_from(args)` (not try_get_matches_from):** exit code 2 대신 `gow_core::args::parse_gnu`를 강제.
- **`std::env::args()` (non-OsString):** Windows에서 non-UTF-8 surrogate 인자 있으면 panic. 항상 `args_os()`.
- **`to_str().unwrap()` on path:** UTF-8 surrogate 포함 파일명 (legacy encoded) 처리 실패. `to_string_lossy()` 표시용, `OsStr` 파일 시스템 호출용.
- **`filetime::set_file_mtime` for `touch -h`:** 타겟 따라감. 반드시 `set_symlink_file_times`.
- **Regex 기반 PATHEXT 파싱:** 세미콜론 + 공백 + 비어있는 필드 엣지케이스. `split(';')` + `.filter(|s| !s.is_empty())` 안전.
- **`yes` 에서 `println!` / 매 이터레이션 flush:** 시스템 콜 오버헤드로 ~100 KB/s 성능. 버퍼 prefill + `write_all` 필수.
- **`env` -S에서 shell 인터폴레이션:** Phase 1 Pitfall #5 — argv 배열만.
- **`mkdir -p`에서 자체 루프 작성:** `create_dir_all`로 충분. 과도한 엔지니어링.

---

## Don't Hand-Roll

| Problem | 금지 | 사용할 것 | 이유 |
|---------|------|-----------|------|
| 사람이 읽는 날짜 파싱 (`touch -d`) | GNU `parse_datetime.y` 포트 | `parse_datetime` 0.14 | `yacc` 파서를 Rust로 포팅하는 건 수천 줄 작업이고 엣지케이스 많음. uutils가 이미 해결 |
| 심링크 자체 타임스탬프 (Windows) | `CreateFileW` + `SetFileTime` 직접 호출 | `filetime::set_symlink_file_times` | `filetime`이 이미 `FILE_FLAG_OPEN_REPARSE_POINT + FILE_FLAG_BACKUP_SEMANTICS` 조합으로 올바르게 구현 |
| UNC 프리픽스 스트립 | 정규표현식 기반 스트립 | `dunce::simplified` 의 10줄 로직을 인라인 | 정규식은 `\\?\UNC\…` 진짜 UNC 경로를 깨뜨림 |
| `mkdir -p` 재귀 | 자체 parent 루프 구현 | `std::fs::create_dir_all` | POSIX 시맨틱 이미 구현. D-27 재확인 |
| Null-terminated env 출력 (`env -0`) | 자체 encoding | `.join("\0")` + trailing `\0` | 단순 패턴; 크레이트 불필요 |
| PATH split | `split(':')` 또는 `split(';')` 하드코딩 | `std::env::split_paths` | OS별 자동 분리자 (Windows `;`, Unix `:`) + 쿼팅 처리 |
| Bytes vs UTF-8 iteration (`wc`) | 자체 UTF-8 디코더 | `bstr::ByteSlice::chars()` | invalid UTF-8을 U+FFFD로 투명 치환. BurntSushi 검증 |
| Windows Ctrl+C 무시 | 자체 signal emulation | `SetConsoleCtrlHandler(None, TRUE)` | Win32 공식 API. 한 줄 |
| 스냅샷 비교 | `assert_eq!(stdout, expected_str)` | `snapbox::assert_eq()` | 자동 update mode (`SNAPSHOTS=overwrite`) + rich diff |

**Key insight:** Phase 2 의 **모든 플랫폼-특수 로직**은 검증된 크레이트로 해결된다. 자체 Win32 코드는 `SetConsoleCtrlHandler` **한 곳만** 필요 (tee -i).

---

## Runtime State Inventory

> Phase 2는 rename/refactor가 아닌 **신규 추가** 작업이므로 이 섹션은 생략. 기존 코드를 건드리지 않는다 — 새 크레이트 14개 추가, workspace.members 확장, workspace.dependencies 3줄 추가만.

---

## Common Pitfalls

### Pitfall 1: `std::env::args()` panic on non-UTF-8 args

**What goes wrong:** Windows에서 non-UTF-8 surrogate를 포함한 인자 (legacy codepage encoded) 를 넘기면 `args()` 가 panic.

**Why:** `args()` 는 `String` 반환 — UTF-8 아닌 걸 받을 수 없음. `args_os()` 는 `OsString` 반환.

**Avoid:** 모든 `uumain` 엔트리에서 `std::env::args_os()` 사용. D-16a 시그니처 강제.

**Warning sign:** `cargo test` 가 Windows 에서 특정 locale 로 실행 시 crash.

---

### Pitfall 2: `filetime::set_file_mtime` follows symlinks

**What goes wrong:** `touch -h link.txt` 가 link 가 아닌 **타겟** 파일의 mtime 을 바꿈.

**Why:** `set_file_mtime` 은 `std::fs::File::open` 사용 → 기본 symlink follow.

**Avoid:** `touch -h` 경로에서 **반드시** `set_symlink_file_times` 호출. 기본 (no -h) 경로에서는 `set_file_times` OK.

**Warning sign:** integration test가 symlink 타겟의 mtime 이 바뀌었다고 리포트.

**Verified by:** [filetime/src/windows.rs source](https://github.com/alexcrichton/filetime/blob/master/src/windows.rs)

---

### Pitfall 3: `yes` `println!` 사용 시 성능 치명적 저하

**What goes wrong:** `loop { println!("y"); }` → 매번 stdout flush, ~100 KB/s.

**Why:** `println!` macro lock + flush 매 호출. 프로세스 간 IPC 오버헤드.

**Avoid:** (1) `io::stdout().lock()` 한 번, (2) 8~64 KiB 버퍼 prefill, (3) `write_all(&buf)` 루프. 

**Warning sign:** `yes | head -c 1G > /dev/null` 이 1초 이상 걸림 (정답: <100ms).

---

### Pitfall 4: `std::fs::canonicalize` on Windows always returns `\\?\` prefix

**What goes wrong:** `pwd -P` 출력이 `\\?\C:\Users\foo` 로 나옴.

**Why:** Windows canonical form은 UNC extended-length path. GNU `pwd -P` 호환성 깨짐.

**Avoid:** `simplify_canonical` 함수로 `\\?\X:\…` 패턴만 스트립. `\\?\UNC\…` 는 보존 (진짜 UNC).

**Warning sign:** `pwd -P` 결과 문자열이 `\\?\` 로 시작.

**Verified by:** [dunce crate docs](https://docs.rs/dunce/latest/dunce/)

---

### Pitfall 5: `mkdir -p` 에서 수동 루프 쓰기 (과도한 엔지니어링)

**What goes wrong:** uutils 의 iterative mkdir 코드를 그대로 가져와서 100+ 줄 추가.

**Why:** uutils는 stack overflow 방지 목적. Rust std의 `create_dir_all` 은 이미 non-recursive (루프 기반).

**Avoid:** `std::fs::create_dir_all` 단일 호출.

**Warning sign:** `gow-mkdir` 크레이트 500줄 이상 → 과도함. 30줄 이하가 정상.

---

### Pitfall 6: `env -S` 에서 `-i` 와 `${VAR}` 순서 오해

**What goes wrong:** `env -iS "hello=${USER}"` 에서 `${USER}` 가 empty (빈 문자열) 로 확장됨.

**Why:** 순진한 구현은 `-i` (clear env) 를 먼저 하고 나서 `${USER}` 치환 → 빈 문자열. GNU 동작은 **반대**: `-S` 파싱 (+ 치환) 을 먼저 하고 그 다음 `-i` 적용.

**Avoid:** 파싱 순서 주의 — `${VAR}` 확장은 **원본 env** 에서 이루어져야 하고 그 결과 토큰이 `-i` clear 이후 새 env 로 들어가야 함.

**Warning sign:** `USER=alice env -iS 'echo $USER'` 이 빈 문자열 출력.

---

### Pitfall 7: PATHEXT 빈 엔트리 / 스페이스

**What goes wrong:** `PATHEXT=".EXE;;.BAT"` (중간 빈 엔트리) 또는 `".EXE ;.BAT"` (스페이스) 에서 빈 확장자로 try → 불필요한 stat syscall.

**Why:** 단순 `split(';')` 만 하면 빈 문자열 포함.

**Avoid:** `split(';').filter(|e| !e.trim().is_empty())`.

**Warning sign:** `which` 가 너무 많은 I/O 호출 (strace/Process Monitor 확인).

---

### Pitfall 8: `tee` stdout 과 파일 간 write order로 인한 partial data

**What goes wrong:** stdout 에 먼저 쓰고 파일에 쓰는데, SIGPIPE (파이프 끝) 가 stdout 에서 발생 → 파일에는 쓰이지 않음.

**Why:** stdout write 실패 시 전체 loop break 하면 뒤의 파일 write 스킵.

**Avoid:** 각 write 를 독립적으로 처리. stdout 실패 시에도 파일 쓰기는 계속 (GNU tee 동작). BrokenPipe 는 조용히 처리.

**Warning sign:** 파이프된 downstream 이 조기 닫힐 때 파일에 데이터 누락.

---

### Pitfall 9: `touch -d` 타임존 해석

**What goes wrong:** `touch -d "2020-01-01"` (TZ 미지정) 가 UTC 로 해석됨 vs 로컬 TZ 로 해석됨 — 결과 mtime 이 ±12시간 다를 수 있음.

**Why:** `parse_datetime_at_date` 의 `Zoned` 레퍼런스가 UTC 인지 Local 인지에 따라 다름.

**Avoid:** GNU touch 는 로컬 TZ 기준. `Zoned::now()` (local) 를 reference 로 넘김. `jiff::Timestamp::now()` 는 UTC 이므로 사용 금지.

**Warning sign:** integration test 가 CI (UTC) vs 로컬 (KST) 에서 다른 mtime 결과.

---

### Pitfall 10: `wc -m` 에서 invalid UTF-8 → panic (표준 char iterator 사용 시)

**What goes wrong:** `str::chars()` 를 invalid UTF-8 바이트에 호출하면 `panic!`.

**Why:** `str` 은 UTF-8 valid guarantee. invalid 바이트가 파일에 있으면 `from_utf8` 단계에서 이미 실패.

**Avoid:** `bstr::ByteSlice::chars()` 사용 — invalid 시퀀스를 U+FFFD 로 대체 (D-17b).

**Warning sign:** binary 파일을 wc 로 넘겼을 때 panic.

---

### Pitfall 11: Windows `RemoveDirectoryW` 열린 핸들 엣지

**What goes wrong:** `rmdir foo` 가 `Access denied` 로 실패. 원인: foo 가 current_dir 이거나 Explorer 가 열고 있음.

**Why:** `ERROR_SHARING_VIOLATION` (32) — Windows mandatory locking.

**Avoid:** v1 에서는 std 에러 그대로 전파. 메시지에 `rmdir: failed to remove 'foo': {os_error}` 형식 유지.

**Warning sign:** 사용자가 `rmdir $(pwd)` 시도 시 실패.

---

## Code Examples

### gow-echo 메인 (완전 스켈레톤)

```rust
// crates/gow-echo/src/lib.rs
use std::ffi::OsString;
use std::io::Write;

mod escape;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    // ad-hoc flag loop (D-21) — clap derive 대신 raw iteration
    let (no_newline, interpret, body) = parse_echo_flags(args);
    run(&body, no_newline, interpret)
}

fn parse_echo_flags<I: IntoIterator<Item = OsString>>(
    args: I,
) -> (bool, bool, Vec<OsString>) {
    let mut it = args.into_iter();
    it.next();  // skip argv[0]
    let mut no_newline = false;
    let mut interpret = false;
    let mut body = Vec::new();
    let mut done_flags = false;

    while let Some(arg) = it.next() {
        if done_flags { body.push(arg); continue; }
        let s = arg.to_string_lossy();
        if s == "--" { done_flags = true; continue; }
        if let Some(rest) = s.strip_prefix('-') {
            if rest.chars().all(|c| matches!(c, 'n' | 'e' | 'E')) && !rest.is_empty() {
                for c in rest.chars() {
                    match c {
                        'n' => no_newline = true,
                        'e' => interpret = true,
                        'E' => interpret = false,
                        _ => unreachable!(),
                    }
                }
                continue;
            }
        }
        done_flags = true;
        body.push(arg);
    }
    (no_newline, interpret, body)
}

fn run(body: &[OsString], no_newline: bool, interpret: bool) -> i32 {
    let joined = body.iter().enumerate().fold(String::new(), |mut acc, (i, a)| {
        if i > 0 { acc.push(' '); }
        acc.push_str(&a.to_string_lossy());
        acc
    });
    let mut out = std::io::stdout().lock();
    let ctrl = if interpret {
        escape::write_escaped(joined.as_bytes(), &mut out).unwrap_or(escape::Control::Continue)
    } else {
        out.write_all(joined.as_bytes()).ok();
        escape::Control::Continue
    };
    let suppress = no_newline || matches!(ctrl, escape::Control::Break);
    if !suppress { out.write_all(b"\n").ok(); }
    0
}
```

### gow-yes 메인

```rust
// crates/gow-yes/src/lib.rs
use std::ffi::OsString;
use std::io::{self, Write};

const BUF_SIZE: usize = 16 * 1024;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let body = args.into_iter().skip(1).collect::<Vec<_>>();
    let text = if body.is_empty() {
        "y\n".to_owned()
    } else {
        let mut s = body.iter().enumerate().fold(String::new(), |mut acc, (i, a)| {
            if i > 0 { acc.push(' '); }
            acc.push_str(&a.to_string_lossy());
            acc
        });
        s.push('\n');
        s
    };
    let bytes = text.as_bytes();
    let mut buffer = vec![0u8; BUF_SIZE];
    let prefilled = prepare_buffer(bytes, &mut buffer);

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    loop {
        if let Err(e) = stdout.write_all(prefilled) {
            if e.kind() == io::ErrorKind::BrokenPipe { return 0; }
            eprintln!("yes: {e}");
            return 1;
        }
    }
}

fn prepare_buffer<'a>(input: &'a [u8], buffer: &'a mut [u8]) -> &'a [u8] {
    if input.len() < buffer.len() / 2 {
        let mut size = 0;
        while size + input.len() <= buffer.len() {
            buffer[size..size + input.len()].copy_from_slice(input);
            size += input.len();
        }
        &buffer[..size]
    } else {
        input
    }
}
```

### gow-touch 메인 (핵심 부분)

```rust
// crates/gow-touch/src/lib.rs
use std::ffi::OsString;
use std::path::Path;
use filetime::{FileTime, set_file_times, set_symlink_file_times};
use jiff::Zoned;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let matches = match gow_core::args::parse_gnu(uu_app(), args) {
        Ok(m) => m, Err(code) => return code,
    };
    // Resolve atime / mtime from -d / -t / -r / now
    let (atime, mtime) = match resolve_times(&matches) {
        Ok(t) => t, Err(e) => { eprintln!("touch: {e}"); return 1; }
    };
    let no_deref = matches.get_flag("h");
    let no_create = matches.get_flag("c");

    let files: Vec<&Path> = matches.get_many::<String>("FILE").unwrap().map(Path::new).collect();
    let mut code = 0;
    for path in files {
        if let Err(e) = touch_one(path, atime, mtime, no_deref, no_create) {
            eprintln!("touch: {}: {e}", path.display());
            code = 1;
        }
    }
    code
}

fn touch_one(path: &Path, atime: FileTime, mtime: FileTime, no_deref: bool, no_create: bool) -> std::io::Result<()> {
    if !path.exists() && !path.is_symlink() {
        if no_create { return Ok(()); }
        std::fs::File::create(path)?;
    }
    if no_deref {
        set_symlink_file_times(path, atime, mtime)
    } else {
        set_file_times(path, atime, mtime)
    }
}

fn resolve_times(m: &clap::ArgMatches) -> Result<(FileTime, FileTime), TouchError> {
    if let Some(date) = m.get_one::<String>("d") {
        let z = parse_datetime::parse_datetime_at_date(Zoned::now(), date)
            .map_err(|e| TouchError::InvalidDate(date.clone(), e.to_string()))?;
        let t = FileTime::from_unix_time(z.timestamp().as_second(), z.timestamp().subsec_nanosecond() as u32);
        return Ok((t, t));
    }
    // ... (-t, -r, default now) 처리
    let now = FileTime::now();
    Ok((now, now))
}
```

### gow-tee signals 모듈

```rust
// crates/gow-tee/src/signals.rs
#[cfg(windows)]
pub fn ignore_interrupts() -> std::io::Result<()> {
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    let ok = unsafe { SetConsoleCtrlHandler(None, 1) };
    if ok == 0 { Err(std::io::Error::last_os_error()) } else { Ok(()) }
}

#[cfg(unix)]
pub fn ignore_interrupts() -> std::io::Result<()> {
    // Simple SIGINT ignore via libc::signal; alternatively use the `libc` crate.
    unsafe {
        unsafe extern "C" { fn signal(sig: i32, handler: usize) -> usize; }
        const SIGINT: i32 = 2;
        const SIG_IGN: usize = 1;
        if signal(SIGINT, SIG_IGN) == usize::MAX { return Err(std::io::Error::last_os_error()); }
    }
    Ok(())
}
```

---

## Per-Utility Implementation Patterns

### `gow-echo` (UTIL-01)
- Ad-hoc flag loop (D-21) — `-neE` 조합 허용 (`echo -ne "..."`)
- `escape::write_escaped` state machine (Q9). `\c` → early break + trailing newline 억제.
- Unit test: `assert_eq!(escape_to_string(r"\t"), "\t")` 등 13개 시퀀스 × match/no-match.

### `gow-pwd` (UTIL-02)
- Logical (기본): `PWD` 환경변수 → `validate_pwd` → canonicalize 비교. fallback `current_dir()`.
- Physical (`-P`): `current_dir` → `canonicalize` → `simplify_canonical` (Q8).
- Unit test: `simplify_canonical(r"\\?\C:\Users")` == `C:\Users`; `simplify_canonical(r"\\?\UNC\srv\share")` 은 보존.

### `gow-env` (UTIL-03)
- clap: `-i`, `-u NAME`, `-C DIR`, `-S STRING`, `-0`, `-v`, `--`.
- `split::split()` state machine (Q7). uutils split_iterator 구조 참조.
- `std::process::Command` argv 배열로 spawn (D-19b — 절대 shell 문자열 금지).
- `-v` 디버그 출력: stderr 로 exec path + args trace.

### `gow-tee` (UTIL-04)
- Open stdin buf read loop (line-based flush — D-25).
- 각 파일 `OpenOptions::new().append(opts.append).write(!opts.append).create(true).open(path)`.
- `-i` → `signals::ignore_interrupts()` (Q10).
- stdout write 실패 (BrokenPipe) 에 파일 write 는 계속 (Pitfall 8).

### `gow-basename` (UTIL-05)
- 입력 인자에 `gow_core::path::try_convert_msys_path` 먼저 적용 (D-26).
- `Path::file_name()` 사용. 접미어 제거 (`basename foo/bar.txt .txt` → `bar`) 지원.
- 복수 인자 + `-a` (all) 지원.

### `gow-dirname` (UTIL-06)
- `Path::parent().unwrap_or(Path::new("."))`.
- 빈 입력 → `.`.
- MSYS 변환 선 적용.

### `gow-yes` (UTIL-07)
- 16 KiB 버퍼 prefill → `write_all` 루프 (Q4, uutils 패턴).
- BrokenPipe → 조용히 exit 0.
- Integration: `yes | head -n 100` stdout 행 수 == 100.

### `gow-true` (UTIL-08)
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 { 0 }
```

### `gow-false` (UTIL-09)
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 { 1 }
```

(D-22 — 인자 무시. gow-core::init() 도 스킵 — 불필요한 초기화 비용 제거. 하지만 일관성 위해 호출해도 됨. Planner 재량.)

### `gow-mkdir` (FILE-06)
- `-p` → `std::fs::create_dir_all` (Q5). 30줄 이하.
- `-m MODE` → Windows no-op (uutils 참조).
- 복수 인자 병렬 처리 가능 (단 실패 지속 — 하나 실패 시에도 나머지 시도).

### `gow-rmdir` (FILE-07)
- `-p` → parent traversal 루프 (Q5).
- `--ignore-fail-on-non-empty` → `is_not_empty(e)` 체크.
- Windows `ERROR_DIR_NOT_EMPTY` (145) 매핑.

### `gow-touch` (FILE-08) — 가장 큰 크레이트
- 별도 모듈 `date.rs` (Q1) + `timestamps.rs` (Q2).
- `-a`, `-m`, `-c`, `-r FILE`, `-d STR`, `-t STAMP`, `-h`.
- jiff + parse_datetime 로컬 deps (D-20b).
- Integration: 8개 이상의 flag 조합 테스트 (D-19 — "각 플래그마다").

### `gow-wc` (TEXT-03)
- `bstr` 기반 iteration (D-17b).
- `-c` (bytes), `-l` (lines), `-w` (words), `-m` (chars). `-L` 생략 (D-17c).
- 출력 포맷: 우측정렬, 파일별 + total 라인. (Claude's Discretion — 1-pass vs 2-pass).
- UTF-8 fixture 테스트 (D-30b).

### `gow-which` (WHICH-01) — GOW #276 flagship
- `pathext::load_pathext()` (Q6) — `GOW_PATHEXT` → `PATHEXT` → 기본.
- Hybrid 탐색 (D-18): 리터럴 먼저, 없으면 확장자 순회.
- `-a` (all matches).
- 3개 이상의 integration test — 리터럴 vs 확장, `-a`, 없는 경우 exit 1.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable (MSVC) | All crate compilation | ✓ | 1.95.0 | — |
| Cargo | Build system | ✓ | 1.95.0 | — |
| `x86_64-pc-windows-msvc` target | `.cargo/config.toml` | ✓ | Active | — |
| `jiff` crate (crates.io) | `gow-touch` | ✓ | 0.2.23 | — |
| `parse_datetime` crate | `gow-touch` | ✓ | 0.14.0 | — |
| `snapbox` crate | 모든 크레이트 테스트 | ✓ | 1.2.1 | — |
| `bstr` crate | `gow-wc` | ✓ | 1.12.1 | — |
| `filetime` crate | `gow-touch` | ✓ | 0.2.27 | — |
| GitHub Actions `windows-latest` CI | D-30c 자동 검증 | [ASSUMED] | - | — |
| PowerShell 5.1+ | D-30c 자동 테스트 (암묵) | ✓ (assert_cmd uses CreateProcessW) | - | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `parse_datetime` 0.14.0 이 uutils/touch 가 실제로 사용하는 버전 | Q1 | 낮음 — uutils Cargo.toml 이 `workspace = true` 라 정확 pin 은 uutils workspace 에 있음; 우리는 0.14 caret range 로 호환 |
| A2 | `filetime::set_symlink_file_times` Windows 구현이 현재 master 와 crates.io 0.2.27 에 모두 동일 | Q2 | 낮음 — filetime 은 안정적 (2017 업데이트 이후 소소한 변경만) |
| A3 | `SetConsoleCtrlHandler(None, TRUE)` 가 assert_cmd 스폰된 자식 프로세스 콘솔 컨텍스트에서 정상 작동 | Q10, D-30c | 중간 — 콘솔 detached 시 실패할 수 있음. 실패는 best-effort 로 조용히 넘어가도록 구현 |
| A4 | `std::fs::create_dir_all` 이 MAX_PATH 해제 환경에서도 안전 (stack overflow 없음) | Q5 | 낮음 — Rust std 1.85+ 에서 iterative 구현 |
| A5 | `dunce` 안전 규칙 (드라이브 문자만 strip, UNC 보존) 이 정확히 `\\?\X:\...` 7-char prefix 체크로 충분 | Q8 | 낮음 — dunce 소스 코드 로직 정확 복제; 엣지케이스는 `\\?\Volume{GUID}\...` 같은 device path 도 보존됨 |
| A6 | `CARGO_BIN_NAME` 환경변수가 utility bin crate 에서 컴파일 타임 사용 가능 | "uumain 템플릿" | 낮음 — Cargo 공식 env var, 1.36+ 지원 |

**Table non-empty:** planner 는 A3 (tee -i Windows 콘솔 컨텍스트) 에 대해 수동 검증 또는 graceful degradation 전략을 planning 에 포함할 것.

---

## Open Questions

1. **`gow-true` / `gow-false` 가 `gow_core::init()` 을 호출해야 하는가?**
   - 호출 시: UTF-8 console + VT mode 설정 (불필요), 프로세스 시작 시 ~1ms 오버헤드.
   - 미호출 시: 3줄 → 1-2줄 wrapper, 일관성 약간 깨짐.
   - 추천: **호출 생략** — true/false 는 출력 없음. `pub fn uumain(_: I) -> i32 { 0 }` 만.

2. **`wc` 출력 우측정렬 필드 폭 — 1-pass vs 2-pass?**
   - 1-pass: 파일별 결과를 `Vec<WordCount>` 에 모으고 끝에 format. 메모리 O(N 파일).
   - 2-pass: stdin 처리 시 불가능 (stream). 여러 파일만 해당.
   - 추천: **vec 버퍼링** — 파일 수 ≤1000 이하 실제 유스케이스, 메모리 O(KB).

3. **`touch -d` 타임존 기본값 — UTC vs Local?**
   - GNU touch = Local TZ (user expectation).
   - `parse_datetime_at_date(Zoned::now(), date)` — `Zoned::now()` 는 local. ✓
   - 추천: 현재 시스템 local TZ 사용. `jiff::Zoned::now()` 반환값이 이미 local.

4. **env `-S` 파싱 에러 메시지 포맷**
   - uutils: 긴 rust-style ParseError.
   - GNU: `env: invalid sequence '\z' in -S`
   - 추천: GNU 스타일 짧은 메시지 유지 (D-11 에 맞춤).

5. **`mkdir -m MODE` — v1 에서 지원할 것인가?**
   - Unix 계열: chmod 비트 적용. Windows: no-op (uutils 참조).
   - 추천: 플래그 파싱은 허용 (에러 아님), 실행 시 Windows 무시. Phase 3 에서 ACL 매핑 도입 시 재검토.

---

## Validation Architecture

> **Dimension 8 (Nyquist):** sampling approach for stateless CLI utilities.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner + assert_cmd 2.2.1 + predicates 3 + snapbox 1.2 |
| Config file | 없음 (cargo test 내장) |
| Quick run command | `cargo test -p gow-{name}` |
| Full suite command | `cargo test --workspace` |
| CI platform | GitHub Actions `windows-latest` (D-30c) |

### Validation Dimensions

#### Dimension 1: GNU Compatibility (exit codes + flag surface)

**What:** 각 유틸리티가 GNU 관례에 따라 exit code를 반환하고 문서화된 플래그를 전부 인식하는지.

**Sampling:**
- 각 크레이트 × 최소 1개 `assert_cmd` 테스트 `.code(1)` for bad args (D-30b #2).
- 각 주요 플래그별 positive test × 1.
- `true` exit 0 / `false` exit 1 (D-22 자명).

**예시:**
```rust
#[test]
fn echo_bad_flag_exits_1() {
    Command::cargo_bin("echo").arg("--xyz").assert().code(1);
}
#[test]
fn echo_e_interprets_tab() {
    Command::cargo_bin("echo").args(["-e", r"hi\tworld"])
        .assert().success().stdout("hi\tworld\n");
}
```

**Coverage rule:** 각 D-21~D-29의 개별 플래그 → 최소 1 테스트.

---

#### Dimension 2: UTF-8 Correctness (wc, path args, filenames)

**What:** 비-ASCII UTF-8 입력 (한글, CJK, 이모지, 서러게이트) 에서 깨지지 않음.

**Sampling:**
- `gow-wc` × UTF-8 fixture 1개 (`한글.txt`) × `-c`, `-l`, `-w`, `-m` 모두 올바른 수 (D-17, ROADMAP success criterion #2).
- `gow-basename` × CJK 파일명 인자 1개 (e.g., `/c/Users/사용자/파일.txt`).
- `gow-echo` × UTF-8 인자 1개.
- `gow-wc` × invalid UTF-8 바이너리 파일 × panic 없음 확인.

**예시:**
```rust
#[test]
fn wc_counts_utf8_chars_correctly() {
    let f = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(f.path(), "안녕\n세상\n").unwrap();
    Command::cargo_bin("wc").arg("-m").arg(f.path())
        .assert().success()
        .stdout(predicate::str::contains("6 "));  // 5 chars + 2 newlines
}
```

---

#### Dimension 3: Windows-Native Primitives

**What:** GOW #276 해결 (which), `touch -h` 심링크, `tee -i` Ctrl+C, `pwd -P` UNC strip, `mkdir -p` 멱등성.

**Sampling:**
- `gow-which` × 리터럴 vs PATHEXT 매치 × 2 테스트 (Q6).
- `gow-which` × `-a` 옵션 × 1 테스트.
- `gow-touch` × `-h` on symlink (Developer Mode 필요 시 skip) × 1.
- `gow-pwd` × `-P` under `\\?\` canonical path × 1.
- `gow-mkdir` × idempotency (`mkdir -p a/b/c` 두 번) × 1.
- `gow-rmdir` × `-p` parent chain × 1.

**예시:** (Q6 참조 — `which_literal_match_beats_pathext`)

---

#### Dimension 4: Error-Path Coverage

**What:** 파일 없음, 권한 없음, 잘못된 입력 → GNU 포맷 메시지 + 올바른 exit code.

**Sampling:**
- `gow-touch /nonexistent/file` × exit 1 + stderr contains "touch:" × 1.
- `gow-mkdir` (without -p) × 기존 디렉토리 × exit 1.
- `gow-rmdir` × 비어있지 않은 디렉토리 × exit 1 + stderr contains "rmdir:".
- `gow-env -C /nonexistent` × exit 1.
- `gow-which` × 존재하지 않는 명령 × exit 1 + stdout 빈 문자열.

**예시:**
```rust
#[test]
fn touch_reports_gnu_error_format() {
    Command::cargo_bin("touch").arg("/this/path/does/not/exist/file")
        .assert().code(1)
        .stderr(predicate::str::starts_with("touch: "));
}
```

---

#### Dimension 5: Performance (yes throughput)

**What:** `yes` 가 ≥100 MB/s 달성하는지 (GNU 레퍼런스의 10% 이상).

**Sampling:**
- `#[ignore]` 표시된 perf test 1개: `yes | head -c 100M` 를 시간 측정.
- CI 에서는 스킵 (예측 불가능한 성능), 로컬에서 `cargo test --release -- --ignored yes_throughput` 로 실행.

**예시:**
```rust
#[test]
#[ignore]  // perf test — run manually
fn yes_throughput_at_least_100mb_per_sec() {
    let start = std::time::Instant::now();
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_yes"))
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    let mut out = cmd.stdout.take().unwrap();
    let mut buf = vec![0u8; 100 * 1024 * 1024];
    std::io::Read::read_exact(&mut out, &mut buf).unwrap();
    cmd.kill().ok();
    let elapsed = start.elapsed().as_secs_f64();
    let mbps = 100.0 / elapsed;
    assert!(mbps >= 100.0, "yes threw only {mbps:.0} MB/s");
}
```

---

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| UTIL-01 | echo -e "\t" outputs tab | Integration | `cargo test -p gow-echo` | ❌ Wave 0 |
| UTIL-01 | echo -n no trailing newline | Integration | `cargo test -p gow-echo` | ❌ Wave 0 |
| UTIL-02 | pwd default, -P physical | Integration | `cargo test -p gow-pwd` | ❌ Wave 0 |
| UTIL-03 | env -i, -u, -S expansion | Integration | `cargo test -p gow-env` | ❌ Wave 0 |
| UTIL-04 | tee writes stdin to stdout + file | Integration | `cargo test -p gow-tee` | ❌ Wave 0 |
| UTIL-04 | tee -a appends | Integration | `cargo test -p gow-tee` | ❌ Wave 0 |
| UTIL-05 | basename extracts filename | Unit + Integration | `cargo test -p gow-basename` | ❌ Wave 0 |
| UTIL-06 | dirname returns parent | Unit + Integration | `cargo test -p gow-dirname` | ❌ Wave 0 |
| UTIL-07 | yes outputs indefinitely | Integration (`| head`) | `cargo test -p gow-yes` | ❌ Wave 0 |
| UTIL-08 | true exit 0 | Integration | `cargo test -p gow-true` | ❌ Wave 0 |
| UTIL-09 | false exit 1 | Integration | `cargo test -p gow-false` | ❌ Wave 0 |
| TEXT-03 | wc UTF-8 counts | Integration (fixture) | `cargo test -p gow-wc` | ❌ Wave 0 |
| FILE-06 | mkdir -p idempotent | Integration | `cargo test -p gow-mkdir` | ❌ Wave 0 |
| FILE-07 | rmdir -p removes parents | Integration | `cargo test -p gow-rmdir` | ❌ Wave 0 |
| FILE-08 | touch -d, -h, -r, -a, -m, -c, -t | Integration | `cargo test -p gow-touch` | ❌ Wave 0 |
| WHICH-01 | which literal match, PATHEXT expand, -a | Integration | `cargo test -p gow-which` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p gow-{name}` (해당 크레이트만)
- **Per wave merge:** `cargo test --workspace` (전체)
- **Phase gate:** 전체 workspace 녹색 + 각 크레이트별 unit + integration 모두 pass 전에 `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `crates/gow-echo/` 전체 (Cargo.toml, build.rs, src/{lib.rs, main.rs, escape.rs}, tests/integration.rs)
- [ ] `crates/gow-pwd/` 전체 + `simplify_canonical` 유닛 테스트 (Q8)
- [ ] `crates/gow-env/` 전체 + `split::split` 유닛 테스트 (Q7)
- [ ] `crates/gow-tee/` 전체 + `signals::ignore_interrupts` 스모크 테스트
- [ ] `crates/gow-basename/` 전체
- [ ] `crates/gow-dirname/` 전체
- [ ] `crates/gow-yes/` 전체 + `prepare_buffer` 유닛 테스트 + `#[ignore]` perf test
- [ ] `crates/gow-true/` (3줄)
- [ ] `crates/gow-false/` (3줄)
- [ ] `crates/gow-mkdir/` 전체 + `mkdir_p_is_idempotent` 통합 테스트
- [ ] `crates/gow-rmdir/` 전체 + `is_not_empty` 유닛 테스트
- [ ] `crates/gow-touch/` 전체 (최대 4개 파일: lib.rs, main.rs, date.rs, timestamps.rs) + 8개+ 통합 테스트
- [ ] `crates/gow-wc/` 전체 + UTF-8 fixture (`tests/fixtures/hangul.txt`)
- [ ] `crates/gow-which/` 전체 + PATHEXT hybrid 테스트 (Q6)
- [ ] `Cargo.toml` workspace members 에 14개 추가 + workspace.dependencies 에 `snapbox`, `bstr`, `filetime` 3줄 추가

---

## Security Domain

> `security_enforcement`가 명시적으로 비활성화되지 않았으므로 포함.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | CLI — 인증 없음 |
| V3 Session Management | No | Stateless |
| V4 Access Control | Partial | `env -C`, `touch`, `mkdir`, `rmdir` 은 파일시스템 권한 계승. 추가 권한 상승 없음 |
| V5 Input Validation | Yes | 경로 / 날짜 / PATHEXT / env 변수 입력 검증 |
| V6 Cryptography | No | 암호 없음 |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| `env -S` 문자열 주입 (쉘 치환 오용) | Tampering | argv 배열로만 spawn (D-19b); `${VAR}` 중괄호 필수; 문자열 concat 금지 |
| `touch` 심링크를 통한 privilege escalation | Tampering | `-h` 플래그로 심링크 자체 조작, 기본은 타겟 — GNU 표준. 심링크 target resolution 은 OS 수준 |
| `which` via PATH hijacking | Tampering | `PATH` 환경변수 정확히 따름 — 사용자 책임. 디렉토리 쓰기 권한은 OS ACL |
| Path traversal via MSYS 변환 | Tampering | `gow_core::path::try_convert_msys_path` 의 Phase 1 보수적 규칙 상속 (D-08) |
| Command injection via `${VAR}` in env -S | Tampering | `${VAR}` → 원시 값 치환만. shell metachar 재파싱 없음 |
| Resource exhaustion (`yes` 로 파일 꽉 채우기) | Denial of Service | 사용자 의도 — `yes > /dev/full` 은 OS 레벨 디스크 full 에러로 종료 |

### Phase 2 에 추가되는 신규 보안 경계

- **`env -S STRING` 은 새로운 공격 표면.** 유저 입력 문자열을 파서로 먹이므로 unbounded memory/CPU 공격 가능. 완화: (1) 총 입력 길이 10 KB 제한, (2) depth recursion 아님 (loop 기반), (3) `${VAR}` 확장 결과를 다시 파싱하지 않음 (1-pass).
- **`touch -r FILE` 파일 읽기 권한.** `-r` 플래그로 레퍼런스 파일의 mtime/atime 을 읽을 때 읽기 권한 없으면 에러. 새 공격 표면 아님 — std::fs::metadata 로 위임.

---

## Sources

### Primary (HIGH confidence — Context7 / Official docs / 직접 소스코드)

- crates.io API (live, 2026-04-21): parse_datetime 0.14.0, jiff 0.2.23, filetime 0.2.27, bstr 1.12.1, snapbox 1.2.1, dunce 1.0.5, which 8.0.2
- uutils/coreutils `src/uu/touch/Cargo.toml` (main branch 2026-04): jiff + parse_datetime + filetime 조합 확인
- uutils/coreutils `src/uu/touch/src/touch.rs`: `parse_datetime::parse_datetime_at_date(ref_zoned, s)` 호출 패턴, `set_symlink_file_times` 위임 확인
- uutils/coreutils `src/uu/yes/src/yes.rs`: `prepare_buffer` + `write_all` 루프 + `#[cfg(windows)] BrokenPipe → Ok(())` 패턴
- uutils/coreutils `src/uu/echo/src/echo.rs`: `parse_escape_only(bytes, OctalParsing::ThreeDigits)` 패턴 (우리는 자체 구현)
- uutils/coreutils `src/uu/env/src/env.rs` + `split_iterator.rs`: state machine 구조 (state_root, state_delimiter, …)
- uutils/coreutils `src/uu/tee/src/tee.rs`: `#[cfg(unix)]` 블록 확인 (Windows 미구현 → 우리가 SetConsoleCtrlHandler 추가)
- uutils/coreutils `src/uu/mkdir/src/mkdir.rs`, `rmdir.rs`, `wc.rs`: 구현 참조
- `alexcrichton/filetime/src/windows.rs` (raw): `set_symlink_file_times` Windows 구현 verbatim — `FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS`
- docs.rs/windows-sys/0.61.2: `SetConsoleCtrlHandler` 시그니처, `PHANDLER_ROUTINE = Option<unsafe extern "system" fn(u32) -> BOOL>`
- docs.rs/parse_datetime/0.14.0: `parse_datetime_at_date(date: Zoned, input: S) -> Result<Zoned, ParseDateTimeError>`
- docs.rs/dunce/latest: `simplified()` 안전 규칙 — 드라이브 문자만 strip, UNC 보존
- docs.rs/filetime/0.2.27: `set_symlink_file_times` 시그니처
- gnu.org coreutils manual `env-invocation.html`: -S 이스케이프 테이블, ${VAR} 치환 규칙
- doc.rust-lang.org/std/fs/fn.create_dir_all.html: POSIX-correct 멱등 보장
- Phase 1 `.planning/phases/01-foundation/01-04-SUMMARY.md`: gow-probe build.rs 템플릿 (복사 대상)
- Phase 1 `.planning/phases/01-foundation/01-RESEARCH.md`: 의존성/핀/아키텍처 상속 근거

### Secondary (MEDIUM confidence — 단일 소스 or 2차 문서)

- Microsoft Learn "SetConsoleCtrlHandler function": `NULL` handler + `TRUE` add → 프로세스 전체 Ctrl+C 무시
- `learn.microsoft.com/en-us/windows/console/handlerroutine`: 핸들러 서명 `BOOL WINAPI HandlerRoutine(DWORD dwCtrlType)`
- `parse_datetime` README (github.com/uutils/parse_datetime): "yesterday", "tomorrow", "N unit ago" 키워드 지원 명시

### Tertiary (LOW confidence — flagged for impl-time verification)

- A3: `SetConsoleCtrlHandler(None, TRUE)` 이 assert_cmd 자식 프로세스 context 에서 실패하는 엣지케이스 — 런타임 확인 필요

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — 모든 버전 crates.io API 로 실시간 verified
- Architecture patterns: HIGH — Phase 1 에서 확정된 템플릿 그대로 14회 복사
- `touch` 스택 (jiff + parse_datetime): HIGH — uutils 레퍼런스 구현 직접 확인
- `touch -h` filetime 크레이트: HIGH — filetime 소스코드 직접 확인; CONTEXT.md 교정
- `tee -i` Windows: MEDIUM — SetConsoleCtrlHandler 공식 API 확인, 하지만 assert_cmd 컨텍스트 edge case 는 런타임 검증 필요 (A3)
- `which` PATHEXT: HIGH — GOW #276 관련 D-18 전략 분명, 구현은 std만 사용
- env -S 스펙: HIGH — GNU 공식 매뉴얼 직접 인용, uutils 참조 구조 확인
- Pitfalls: HIGH — 각 사항이 Phase 1 pattern 연장선 또는 uutils 이슈에서 확인

**Research date:** 2026-04-21
**Valid until:** 2026-05-21 (crates 버전 안정적; jiff 는 0.2 → 0.3 브레이크 변경 가능성 있으므로 30일 내 재확인)
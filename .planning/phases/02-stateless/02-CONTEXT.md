# Phase 2: Stateless Utilities - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 2는 14개의 단순 stateless GNU 유틸리티를 Rust로 구현한다: `echo`, `pwd`, `env`, `tee`, `basename`, `dirname`, `yes`, `true`, `false`, `mkdir`, `rmdir`, `touch`, `wc`, `which`. 각 유틸리티는 gow-core 위에 얇게 래핑되며, 공통 파이프라인은 parse args → do work → print → exit 로 고정된다. 파일 시스템 감시, 비동기, 스트리밍 상태는 Phase 2 범위 밖 (Phase 3 이후).

각 유틸리티 바이너리는 Windows에서 GNU 호환 동작을 관찰할 수 있어야 하고, 성공 기준은 ROADMAP.md의 5개 항목에 고정된다: `echo -e "\t"`, `echo -n`, `wc` UTF-8 정확성, `which` Windows PATH, `mkdir -p` 중첩, `tee -a` 추가.

**커버 요구사항:** UTIL-01..09, TEXT-03, FILE-06..08, WHICH-01 — 총 14개.

</domain>

<decisions>
## Implementation Decisions

### 크레이트 레이아웃 — 유틸리티별 lib + thin bin 분리
- **D-16:** 유틸리티당 독립 크레이트를 만든다: `crates/gow-echo/`, `crates/gow-wc/`, … 총 14개. uutils/coreutils 컨벤션을 따라 각 크레이트는 `uu_echo` lib 크레이트 + thin `echo` bin 크레이트 패턴을 사용한다. bin 크레이트는 `main() { std::process::exit(uu_echo::uumain(std::env::args_os())) }` 형태의 wrapper만 포함한다.
- **D-16a:** `uumain(args: impl Iterator<Item=OsString>) -> i32` 서명을 표준으로 한다. 단위 테스트는 lib 크레이트 레벨에서 `uumain`을 직접 호출하고, integration 테스트는 `assert_cmd` + bin 크레이트로 작성한다.
- **D-16b:** `true`, `false` 같이 실제로 3줄인 유틸리티도 lib+bin 패턴을 유지한다 — 일관성 + Phase 6에서 multicall binary로 합칠 때 수정이 거의 필요 없다.
- **D-16c:** 각 `crates/gow-{name}/build.rs`는 Plan 01-04에서 확정된 gow-probe build.rs 템플릿을 복사한다 (`has_bin_target()` 게이트 없이 Windows에서 항상 `embed_manifest()` 호출 — bin 크레이트이므로).
- **D-16d:** 바이너리 출력 이름은 GNU 이름 그대로 (`echo.exe`, `wc.exe`, …). D-14 재확인.

### 의존성 관리
- **D-20:** Phase 2에 새로 추가되는 공통 의존성은 `[workspace.dependencies]`에만 추가하고 각 크레이트는 `workspace = true`로 상속한다 (D-15 재확인).
- **D-20a:** 추가가 필요한 공통 의존성:
  - `snapbox = "1.2"` — CLAUDE.md 기술 스택에 명시되어 있지만 Phase 1에서 아직 workspace에 추가되지 않았다. Phase 2 스냅샷 테스트에 필요.
  - `bstr = "1"` — CLAUDE.md 기술 스택에 명시. `wc -w` / `-m`에서 invalid UTF-8 바이트 스트림을 panic 없이 반복하기 위해 필수 (D-17 참조).
  - `filetime = "0.2"` — `touch` 크레이트에만 필요 (workspace에 추가하되 `gow-touch` 크레이트에서만 의존).
- **D-20b:** 유틸리티별 전용 의존성은 해당 크레이트의 `[dependencies]`에만 선언한다 (예: `filetime`은 gow-touch만, `jiff`/date 파서는 gow-touch만).

### `wc` Unicode 정책 — 항상 Unicode-aware
- **D-17:** `wc`는 locale을 무시하고 항상 Unicode-aware로 동작한다. `LANG` / `LC_CTYPE` 환경변수를 읽지 않는다.
- **D-17a:** `-c` = 바이트 수 (`input.len()`), `-l` = `b'\n'` 카운트, `-m` = Unicode scalar value 카운트 (UTF-8 디코딩, invalid 시퀀스는 `bstr::chars`가 U+FFFD 1개로 처리), `-w` = `char::is_whitespace` 기반 word boundary.
- **D-17b:** invalid UTF-8이 나타나도 panic하지 않는다 — `bstr` 사용. `-c` / `-l`은 raw 바이트에서 동작하므로 invalid UTF-8 파일도 문제없이 처리.
- **D-17c:** `-L` (최장 라인 길이, display width)는 v1에 포함하지 않는다 — Unicode width 계산(`unicode-width`)이 터미널 폭과 일치하지 않는 엣지 케이스가 많다. v2 백로그.

### `which` PATHEXT 전략 — hybrid (리터럴 → 확장)
- **D-18:** 각 PATH 디렉토리에서 다음 순서로 시도: (1) 리터럴 이름 그대로, (2) 없으면 `%PATHEXT%`의 각 확장자를 순서대로 덧붙여 시도.
- **D-18a:** `PATHEXT` 환경변수를 시작 시 한 번 읽는다. 없거나 비어있으면 `.COM;.EXE;.BAT;.CMD`로 기본값. 세미콜론 구분, 대소문자 무관.
- **D-18b:** `-a` (all) 옵션은 리터럴 매치와 확장자 확장 매치 모두 반환한다.
- **D-18c:** 현재 디렉토리를 자동으로 먼저 검색하지 않는다 — `cmd.exe`의 CWD-first 동작이 아니라 Unix 관례 유지. 사용자가 `./script`를 실행하려면 명시적으로 `./` 경로를 줘야 한다.
- **D-18d:** `GOW_PATHEXT` 환경변수를 제공해 테스트에서 PATHEXT를 결정적으로 override할 수 있게 한다. 실제 사용자 환경의 PATHEXT에 의존하는 테스트는 flaky하다.
- **D-18e:** symlink/junction 타겟 해석은 하지 않는다 — `which`는 발견한 경로의 절대 경로만 출력한다. `gow_core::fs::link_type`는 이 유틸리티에서 사용하지 않는다.

### `env` / `touch` 플래그 커버리지 — Full GNU parity
- **D-19:** `env`와 `touch`는 v1에서 GNU 풀 기능 세트를 목표로 한다. 각 플래그마다 integration 테스트를 작성한다.
- **D-19a `env`:** `-i` / `--ignore-environment` (빈 env로 시작), `-u NAME` / `--unset=NAME` (반복 가능), `-C DIR` / `--chdir=DIR` (exec 전에 cwd 변경), `-S STRING` / `--split-string=STRING` (shebang `#!/usr/bin/env -S …` 지원), `-0` / `--null` (NUL 구분 env 출력), `-v` / `--debug` (exec 경로 trace), `--` (옵션 종료).
- **D-19b `env` 하위 프로세스 spawn:** 인자는 Rust `std::process::Command` argv 배열로 전달한다 — shell 문자열 인터폴레이션 금지. Phase 1 Pitfall #5 ("find -exec arguments must be passed as argv array, never through a shell string")를 env에 그대로 적용. Windows에서도 `Command` API가 자동으로 argv quoting을 처리한다.
- **D-19c `touch`:** `-a` (atime only), `-m` (mtime only), `-c` / `--no-create` (없으면 만들지 않음), `-r FILE` / `--reference=FILE` (다른 파일의 타임스탬프 복사), `-d STRING` / `--date=STRING` (사람이 읽는 날짜 문자열), `-t STAMP` (`[[CC]YY]MMDDhhmm[.ss]` 고정 포맷), `-h` (심링크 자체 수정 — 타겟이 아니라).
- **D-19d `touch -d` 파서:** 사람이 읽는 날짜 ("yesterday", "2 hours ago", "2020-01-01 15:00 UTC")를 파싱해야 한다. 연구자는 세 옵션을 평가해야 한다: `jiff` crate (2024~ 현대적, Jiff의 `civil::DateTime::parse` + relative-time 지원 여부 확인), `chrono` + `dateparser` crate 조합, 또는 GNU의 `parse_datetime.y` 서브셋을 직접 구현. 목표: `yesterday`, `N {days|hours|minutes} ago`, ISO-8601, RFC-3339 최소 지원.
- **D-19e `touch -h` (Windows 심링크 자체 수정):** 표준 `filetime::set_file_mtime`는 심링크를 따라간다. Windows에서 심링크 자체의 timestamp를 수정하려면 `CreateFileW`에 `FILE_FLAG_OPEN_REPARSE_POINT`를 지정하고 `SetFileTime`을 호출해야 한다. `gow_core::fs`에 이 기능을 추가하고 (`touch_link_time(path, mtime, atime)`), `gow-touch`가 호출한다. 비-Windows 플랫폼은 `libc::lutimes` 사용.

### 유틸리티별 구현 세부 (수렴된 항목)
- **D-21 `echo`:** `-n` (개행 억제), `-e` (이스케이프 해석: `\t \n \r \\ \0 \a \b \f \v \xHH \NNN`), `-E` (이스케이프 비활성, 기본). `gow_core::args::parse_gnu`를 사용하지만 clap의 derive 스타일보다 raw argv 반복이 깔끔하다 — ad-hoc flag loop 허용.
- **D-22 `true` / `false`:** `uumain`은 각각 `0` / `1`을 반환. 인자 무시 (GNU 호환). 플래그 파싱 없음.
- **D-23 `yes`:** 기본값 "y\n" 반복. 인자 1개 이상이면 space-separated로 조합해서 반복. 큰 버퍼(8 KiB 이상)에 미리 채워두고 반복 `write_all`로 throughput 최적화 — GNU yes가 수 GB/s 인 이유. `BrokenPipe` 에러는 조용히 exit 0 (파이프 끝난 것은 정상 상황).
- **D-24 `pwd`:** 기본은 `PWD` 환경변수 우선, 없으면 `std::env::current_dir()`. `-P` (physical)는 `std::fs::canonicalize` + UNC 프리픽스(`\\?\`) 스트립. `-L` (logical, 기본) 와 `-P` 모두 지원.
- **D-25 `tee`:** stdin을 stdout + 각 파일에 동시 기록. `-a` / `--append`는 파일을 `O_APPEND`로 연다. `-i` / `--ignore-interrupts` (SIGINT 무시 — Windows에서는 `SetConsoleCtrlHandler`로 Ctrl+C 무시). 라인 버퍼링 — 매 라인마다 flush (interactive pipe 사용 케이스에서 필수).
- **D-26 `basename` / `dirname`:** 입력 문자열에 대해 먼저 `gow_core::path::try_convert_msys_path`를 적용한 뒤 표준 path 조작을 수행한다. `basename foo/bar.txt .txt` 처럼 접미어 제거 형식도 지원. MSYS 경로 편의성 유지.
- **D-27 `mkdir -p` (GOW #133):** Rust의 `std::fs::create_dir_all`은 이미 POSIX-correct 시맨틱 (디렉토리가 이미 존재하면 성공) 을 구현한다. `create_dir_all`의 의도를 문서 코멘트로 명시하고 integration 테스트로 `mkdir -p a/b/c && mkdir -p a/b/c`가 둘 다 exit 0인지 확인. 별도 Win32 로직 필요 없음.
- **D-28 `rmdir -p`:** GNU `rmdir -p a/b/c`는 `a/b/c` → `a/b` → `a` 순서로 비어있는 한 모두 삭제. Rust 표준 라이브러리에 대응 함수가 없으므로 수동 루프. `c`가 비어있지 않으면 exit 1, 이미 `b` 수준은 건드리지 않음.
- **D-29 `touch` 기본 동작:** 인자 파일이 없으면 `O_CREAT | O_TRUNC` 아니라 `O_CREAT` + 타임스탬프만 변경. 이미 있으면 타임스탬프만 업데이트. `-a` / `-m`이 둘 다 없으면 둘 다 업데이트, 하나만 있으면 그것만.

### 테스트 전략
- **D-30:** 각 유틸리티는 lib 크레이트 레벨의 `uumain` 단위 테스트 + bin 크레이트 레벨의 `assert_cmd` integration 테스트 모두 작성한다.
- **D-30a:** 스냅샷 테스트는 `snapbox`를 사용한다 (workspace.dependencies에 추가 — D-20a). 기대값은 hand-written (실제 GNU 유틸 실행해서 생성 X — CI에서 GNU 유틸 runner 세팅 복잡도 피함). 엣지 케이스가 애매하면 실제 Linux에서 한 번 돌려보고 주석에 commit-time 증거 기록.
- **D-30b:** 각 크레이트는 최소한 다음 테스트를 포함한다: 기본 동작 1개, exit code 확인 1개 (`--bad-flag` → 1), GNU 포맷 에러 메시지 1개 (`{util}: {msg}` 형식), UTF-8 입력/파일명 1개.
- **D-30c:** PowerShell/Windows Terminal specific 동작은 GitHub Actions `windows-latest` runner에서 `cargo test --workspace`로 검증한다. Phase 1의 gow-probe 수동 4-체크 경험에서, 대부분의 문제는 programmatic 테스트로 재현 가능하므로 per-utility 수동 체크포인트는 생략 (autonomous: true).

### Claude's Discretion
- **`pwd`의 `PWD` 환경변수 vs `current_dir()` 우선순위 세부:** `PWD`가 현재 프로세스의 cwd와 일치하는지 `canonicalize`로 검증하고 불일치 시 fallback. 표준 동작이므로 planner가 결정.
- **`echo` 이스케이프 시퀀스 파서 내부 구현:** state machine vs regex vs char iterator — 취향 문제. state machine이 표준.
- **`wc` 출력 포맷 정렬 (right-aligned 필드 폭):** GNU wc는 파일 개수에 따라 필드 폭을 동적으로 결정. 구현은 2-pass (먼저 최대값 구한 뒤 포맷) vs 1-pass with vec 버퍼링 — 아무거나.
- **`tee`의 lock contention 회피:** 여러 파일에 동시 쓰기 시 stdout lock과 파일 lock 획득 순서. 데드락 없는 순서로 planner가 결정.

</decisions>

<specifics>
## Specific Ideas

- **GOW #276 (`which`) 해결은 Phase 2의 flagship 이슈.** 사용자가 기존 GOW에서 겪은 가장 큰 불편이 "which foo" 가 foo.exe 를 못 찾는 것이었다. hybrid 전략은 이 문제를 고치면서도 GNU 스크립트(리터럴 이름 매치 기대)를 깨지 않는다.
- **GOW #133 (`mkdir -p`) 은 실제로는 거의 이슈가 아니다.** Rust std lib이 이미 올바르게 구현하므로 integration 테스트만 추가하면 된다. plan 1태스크로 충분.
- **`touch` 풀 GNU parity는 Phase 2의 가장 큰 단일 태스크.** `-d` 파서와 `-h` Windows 심링크 처리에 각각 별도 플랜 태스크가 필요할 수 있다. 연구자가 `jiff` 의 현황을 반드시 확인해야 한다.
- **uutils/coreutils 레포를 reference 구현으로 사용할 것.** 공개 MIT 라이선스. Rust 표준 coreutils 구현이며 이 프로젝트의 이전 결정(D-01..D-05, D-16)과 일치. 코드 복사는 하지 않되 (라이선스 + 우리 gow-core 래퍼와 안 맞음) 동작 참조는 적극적으로 활용.
- **Phase 1의 gow-probe build.rs가 Phase 2 템플릿.** 14개 build.rs 모두 `crates/gow-probe/build.rs`를 거의 그대로 복사한다 (`has_bin_target()` 게이트 제거). 연구자는 별도 embed-manifest 패턴을 다시 조사할 필요 없음 — Phase 1에서 확정됨.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-local
- `.planning/PROJECT.md` — 프로젝트 핵심 가치, 제약, v1 스코프
- `.planning/REQUIREMENTS.md` — Phase 2 요구사항 (UTIL-01..09, TEXT-03, FILE-06..08, WHICH-01)
- `.planning/ROADMAP.md` — Phase 2 goal + success criteria (ROADMAP.md:82~96)
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 decisions D-01..D-15 (이 중 D-01..D-05 arg 파싱, D-06..D-08 path 변환, D-09..D-11 error 포맷, D-12 크레이트 네이밍, D-14 바이너리 이름, D-15 workspace deps 는 Phase 2가 그대로 상속)
- `.planning/phases/01-foundation/01-RESEARCH.md` — Phase 1 research (embed-manifest, Cargo 1.95 특이사항, crt-static, SetConsoleOutputCP 패턴)
- `.planning/phases/01-foundation/01-01-SUMMARY.md` — workspace 레이아웃 실제 구현
- `.planning/phases/01-foundation/01-02-SUMMARY.md` — `gow_core::args::parse_gnu` 실제 구현 (exit-code 1 처리, allow_hyphen_values drop 이유)
- `.planning/phases/01-foundation/01-03-SUMMARY.md` — `gow_core::path::try_convert_msys_path` 실제 구현 (GOW #244 음성 케이스)
- `.planning/phases/01-foundation/01-04-SUMMARY.md` — `crates/gow-probe/build.rs` canonical embed-manifest template + 9-test assert_cmd 패턴
- `./CLAUDE.md` — Technology Stack 전체 (snapbox 1.2.1, bstr 1.9.1, filetime 0.2.27, globset 0.4.18 — Phase 2의 workspace deps 추가 근거)

### External
- uutils/coreutils repo — https://github.com/uutils/coreutils (각 유틸리티의 reference Rust 구현; `src/uu/echo/src/echo.rs` 등)
- GNU coreutils manual — https://www.gnu.org/software/coreutils/manual/ (각 유틸리티의 공식 동작 스펙)
- GOW 이슈 #276 (`which`) / #133 (`mkdir`) / #244 (path conversion, Phase 1에서 이미 해결) — 원본 GOW 리포지토리의 해당 이슈 스레드
- Rust `std::fs::create_dir_all` 문서 — https://doc.rust-lang.org/std/fs/fn.create_dir_all.html (mkdir -p 의 레퍼런스)
- `filetime` crate docs — https://docs.rs/filetime (touch -r 시 사용)
- `jiff` crate docs — https://docs.rs/jiff (touch -d 날짜 파서 후보)
- `snapbox` crate docs — https://docs.rs/snapbox (스냅샷 테스트 패턴)
- `bstr` crate docs — https://docs.rs/bstr (wc 의 byte-safe 반복)

</canonical_refs>

<deferred>
## Deferred Ideas

이번 Phase의 스코프를 넘어서는 아이디어들. 이슈로 잃지 않도록 여기 기록.

- **Multicall 단일 바이너리 (`gow` coreutils-style):** Phase 6 이후 Defender 스캔 페널티가 문제되면 재검토. 지금은 14개 독립 exe 유지.
- **`wc -L` (최장 라인 display width):** Unicode width(`unicode-width`)가 터미널 폭과 일치하지 않는 엣지 케이스 다수. v2.
- **`which --tty-only`, `--skip-dot`, `--skip-tilde`:** GNU which extensions. 실제 사용 빈도 낮음. v2.
- **`cat` / `cp` / `mv` / `rm` / `ls`:** Phase 3 (Filesystem Utilities)로 이동 — stateless가 아님 (파일 내용 처리, 재귀, -i 대화형 등).
- **`env --block-signal` (최신 GNU):** 희귀 플래그. v2 backlog.
- **`touch --time=atime|access|use` (별칭):** `-a`, `-m`의 긴 형태 별칭. 플래그 스코프 확장 여지 있으면 Planner 재량으로 포함 가능, 필수 아님.
- **PATHEXT 대소문자 혼합 테스트 (`.Exe`, `.EXE`):** Windows는 대소문자 무관하지만 GOW_PATHEXT override 시 엣지. v2 테스트 보강.

</deferred>

---

*Context gathered: 2026-04-21*
*Next: `/gsd-plan-phase 2` to create detailed execution plans.*

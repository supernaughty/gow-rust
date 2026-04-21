---
status: passed
phase: 02-stateless
verified_at: 2026-04-21
must_haves_verified: 19
must_haves_total: 19
requirements_verified: 14
requirements_total: 14
tests_passed: 265
tests_failed: 0
---

# Phase 2: Stateless Utilities — Verification Report

**Phase Goal:** Users can run the complete set of simple, stateless GNU utilities and observe correct GNU-compatible behavior on Windows
**Verified:** 2026-04-21
**Status:** passed
**Re-verification:** No — initial verification

## Goal Statement

Phase 2는 14개의 stateless GNU 유틸리티 (`echo`, `pwd`, `env`, `tee`, `basename`, `dirname`, `yes`, `true`, `false`, `mkdir`, `rmdir`, `touch`, `wc`, `which`) 를 Rust 네이티브로 구현하여 Windows 10/11 에서 GNU 호환 동작을 보장하는 것을 목표로 한다. 각 유틸리티는 `uu_{name}` 라이브러리 크레이트와 GNU 이름의 thin 바이너리 크레이트로 구성되며, Windows 전용 프리미티브(PATHEXT, `SetConsoleCtrlHandler`, UNC 스트립, symlink-self timestamp) 를 네이티브로 처리한다.

모든 관찰 가능한 진술, 모든 요구사항 ID, 모든 ROADMAP 성공 기준을 코드베이스에 대해 직접 검증하였다.

---

## Requirement Verification

REQUIREMENTS.md 의 Phase 2 ID 14개 전부 검증. 각 ID마다 Plan 출처, 실제 바이너리 실행 증거를 포함한다.

| 요구사항 | 플랜 | 설명 | 상태 | 증거 |
| -------- | ---- | ---- | ---- | ---- |
| UTIL-01 | 02-03 | echo (-n, -e 이스케이프) | SATISFIED | `cargo run -p gow-echo -- -e '\t' \| od`: `09 0a` (TAB+LF). `echo -n hello`: `68 65 6c 6c 6f` (trailing newline 없음). `pub fn uumain` @ `crates/gow-echo/src/lib.rs:54`. `crates/gow-echo/src/escape.rs` state machine 존재. |
| UTIL-02 | 02-04 | pwd (-P 물리 경로) | SATISFIED | `pwd -P` returns `D:\workspace\gow-rust` (UNC prefix 없음). `simplify_canonical` @ `crates/gow-pwd/src/canonical.rs:19` + 8 unit tests passing. |
| UTIL-03 | 02-09 | env (-i, -u, -C, -S, -0, -v) | SATISFIED | `env -i` 출력 0 라인 (빈 환경). `env FOO=bar echo hi` 자식 프로세스 정상 exec. `crates/gow-env/src/split.rs` state machine. `StdCommand::new(command_name)` 직접 호출 — shell string 없음 (D-19b). |
| UTIL-04 | 02-07 | tee (-a 추가) | SATISFIED | `echo hi \| tee out.txt`: stdout + 파일 둘 다 `hi\n`. `tee -a`로 `bye` 추가 후 파일은 `hi\nbye\n`. `crates/gow-tee/src/signals.rs`가 `SetConsoleCtrlHandler(None, 1)` 호출. |
| UTIL-05 | 02-05 | basename | SATISFIED | `basename foo/bar.txt` → `bar.txt`. MSYS 경로 pre-convert (`try_convert_msys_path`) 연결 검증 @ `crates/gow-basename/src/lib.rs`. |
| UTIL-06 | 02-05 | dirname | SATISFIED | `dirname foo/bar.txt` → `foo`. MSYS pre-convert 연결 검증 @ `crates/gow-dirname/src/lib.rs`. |
| UTIL-07 | 02-02 | yes (무한 반복) | SATISFIED | `yes \| head -3` 출력 `y\ny\ny`. `yes hello world` 출력 `hello world\n...`. BrokenPipe 처리 @ `crates/gow-yes/src/lib.rs:61`. |
| UTIL-08 | 02-02 | true (exit 0) | SATISFIED | `cargo run -p gow-true` exit=0. `uu_true::uumain` returns 0 무조건 @ `crates/gow-true/src/lib.rs:5`. |
| UTIL-09 | 02-02 | false (exit 1) | SATISFIED | `cargo run -p gow-false` exit=1. `uu_false::uumain` returns 1 무조건 @ `crates/gow-false/src/lib.rs:5`. |
| TEXT-03 | 02-08 | wc (단어/줄/바이트) | SATISFIED | `안녕 세상\n` fixture (14 bytes) 에 대해 `wc -l`=1, `wc -w`=2, `wc -c`=14, `wc -m`=6 (4 Korean scalars + space + newline). `use bstr::ByteSlice` @ `crates/gow-wc/src/lib.rs:20`. |
| FILE-06 | 02-06 | mkdir -p | SATISFIED | `mkdir -p a/b/c` 첫 호출 exit=0, 동일 명령 재호출 exit=0 (idempotent). `create_dir_all` 사용 @ `crates/gow-mkdir/src/lib.rs`. |
| FILE-07 | 02-06 | rmdir -p | SATISFIED | `rmdir -p a/b/c` exit=0, 모든 부모 디렉토리 제거 확인. 수동 parent-walk loop @ `crates/gow-rmdir/src/lib.rs`. |
| FILE-08 | 02-10 | touch | SATISFIED | `touch t.txt` 신규 파일 생성. `touch -t 202001010000 f.txt` → `Jan 1 2020` 타임스탬프. `touch -c nope.txt` exit=0 파일 없음. `set_symlink_file_times` 직접 호출 @ `crates/gow-touch/src/timestamps.rs:19` (D-19e 교정 반영). |
| WHICH-01 | 02-11 | which (Windows PATH, GOW #276) | SATISFIED | `which cargo` → `C:\Users\노명훈\.cargo\bin\cargo.EXE` (PATHEXT 확장). `which nonexistent_xyzzy_9999` exit=1 + GNU 포맷 에러. `GOW_PATHEXT` 지원 @ `crates/gow-which/src/pathext.rs:25`. Literal-first 전략 구현 (lib.rs:62). |

**Coverage: 14/14 requirements satisfied. Zero orphans.** REQUIREMENTS.md 가 Phase 2로 매핑한 14개 ID 전부 플랜 frontmatter에서 claim됐고 전부 실제 코드로 검증.

---

## Success Criteria Check (ROADMAP)

ROADMAP.md의 5개 Success Criteria 각각에 대해 실제 명령을 실행하고 출력을 관찰.

### SC #1 — `echo -e "\t"` outputs real TAB; `echo -n` suppresses trailing newline

- **Command A:** `cargo run -q -p gow-echo -- -e '\t' | od -An -tx1`
- **Observed A:** `09 0a` — 리터럴 TAB(0x09) + trailing LF(0x0a) ✓
- **Command B:** `cargo run -q -p gow-echo -- -n hello | od -An -tx1`
- **Observed B:** `68 65 6c 6c 6f` — "hello" only, no trailing 0x0a ✓
- **Status:** ✓ VERIFIED

### SC #2 — `wc -l`, `wc -w`, `wc -c` on non-ASCII UTF-8 content

- **Fixture:** `printf '안녕 세상\n' > $TMP` (hex: `ec 95 88 eb 85 95 20 ec 84 b8 ec 83 81 0a`, 14 bytes)
- **Command:** `wc -l $TMP` → `1`; `wc -w $TMP` → `2`; `wc -c $TMP` → `14`; `wc -m $TMP` → `6`
- **Observed:** 모두 정확. `-l` = 1 newline, `-w` = 2 whitespace-separated tokens, `-c` = 14 bytes, `-m` = 6 Unicode scalar values (4 Korean chars + space + newline)
- **Bonus:** 파일명이 `/tmp/...` → `C:\Users\노명훈\AppData\...` MSYS pre-convert 로 자동 변환돼 출력됨
- **Status:** ✓ VERIFIED

### SC #3 — `which` locates executables on Windows PATH including `.exe`/`.cmd`

- **Command A:** `cargo run -q -p gow-which -- cargo`
- **Observed A:** `C:\Users\노명훈\.cargo\bin\cargo.EXE` (literal `cargo` not present, PATHEXT fallback to `.EXE` succeeds) ✓
- **Command B:** `which nonexistent_xyzzy_9999`
- **Observed B:** exit=1 + GNU 포맷 메시지 `which: no nonexistent_xyzzy_9999 in (...)` ✓
- **Status:** ✓ VERIFIED

### SC #4 — `mkdir -p a/b/c` creates nested dirs idempotently

- **Command 1:** `mkdir -p a/b/c` in tempdir
- **Observed 1:** exit=0, `a/b/c` 생성됨 ✓
- **Command 2:** 동일 `mkdir -p a/b/c` 재실행
- **Observed 2:** exit=0, 에러 없음 (idempotent) ✓
- **Bonus:** `rmdir -p a/b/c` exit=0, 전체 체인 제거 확인
- **Status:** ✓ VERIFIED

### SC #5 — `tee file.txt` writes stdin to both file and stdout; `tee -a` appends

- **Command 1:** `echo hi | tee out.txt`
- **Observed 1:** stdout 에 `hi`; `cat out.txt` → `hi` ✓
- **Command 2:** `echo bye | tee -a out.txt`
- **Observed 2:** stdout 에 `bye`; `cat out.txt` → `hi\nbye` (append 정상) ✓
- **Status:** ✓ VERIFIED

**5/5 success criteria 전부 실제 실행으로 검증 완료.**

---

## CONTEXT.md Fidelity (D-16..D-30 Spot-Check)

각 핵심 결정 사항이 실제 코드에서 관찰되는지 확인.

| Decision | 검증 방법 | 결과 |
| -------- | ------- | ---- |
| D-14 / D-16d — 바이너리 이름 GNU (echo, pwd, …) | 14 `[[bin]] name =` 엔트리 scan | PASS — `echo`, `pwd`, `env`, `tee`, `basename`, `dirname`, `yes`, `true`, `false`, `mkdir`, `rmdir`, `touch`, `wc`, `which` 모두 GNU 이름 |
| D-16 — uu_{name} lib + thin bin 분리 | `grep "name = \"uu_"` in Cargo.toml | PASS — 14 `[lib] name = "uu_{name}"` 엔트리 확인 |
| D-16a — `uumain<I: IntoIterator<Item = OsString>> -> i32` 서명 | `grep "pub fn uumain"` in lib.rs | PASS — 14/14 크레이트 정확한 서명 |
| D-16b — true/false 도 lib+bin 패턴 유지 | crates/gow-{true,false}/ 구조 | PASS — 두 크레이트 모두 lib.rs + main.rs + Cargo.toml |
| D-16c — 모든 bin 크레이트 build.rs에서 `embed_manifest` 무조건 호출 (no `has_bin_target`) | `grep "has_bin_target" in Phase 2 crates` | PASS — Phase 2 14개 중 `has_bin_target` 를 사용하는 크레이트 없음. `gow-core` (lib-only)만 해당 gate 유지 |
| D-17 — wc Unicode-aware via bstr | `grep "bstr" in gow-wc/src/lib.rs` | PASS — `use bstr::ByteSlice` @ line 20, `bstr::chars()` 사용 |
| D-18 — hybrid PATHEXT (literal → extension) | `gow-which/src/lib.rs:62 "Phase 1: literal match"` | PASS — 리터럴 이름 시도 후 PATHEXT 확장 시도 순서 |
| D-18d — GOW_PATHEXT 테스트 override 지원 | `grep "GOW_PATHEXT"` | PASS — `pathext.rs:25` `var_os("GOW_PATHEXT")` + integration test 10+ 건 |
| D-19b — env는 shell 통해 spawn 금지 | `grep 'Command::new("(sh|bash|cmd)")' in gow-env` | PASS — 매치 0건. `StdCommand::new(command_name)` 직접 호출 @ `lib.rs:158` |
| D-19e 교정 — gow_core::fs 에 `touch_link_time` 없음 | `grep "touch_link_time"` repo-wide | PASS — 함수 정의 0건 (문서 메모만 존재). `gow-touch/src/timestamps.rs:19` 가 `filetime::set_symlink_file_times` 직접 호출 |
| D-20a — snapbox 1.2, bstr 1, filetime 0.2 workspace deps | Cargo.toml:60-62 | PASS — 세 의존성 pin 확인 |
| D-22 — true/false는 argv 무시 | `uu_true::uumain` returns 0 무조건 | PASS — `lib.rs:5 _args: I` underscore-prefixed |
| D-23 / Q4 — yes BrokenPipe 조용히 exit 0 | `grep "BrokenPipe"` gow-yes | PASS — `io::ErrorKind::BrokenPipe => return 0` @ `lib.rs:61` |
| D-25 / Q10 — tee -i `SetConsoleCtrlHandler` | `grep "SetConsoleCtrlHandler"` gow-tee | PASS — `signals.rs:19` `SetConsoleCtrlHandler(None, 1)` |
| D-26 — MSYS path pre-convert | `grep "try_convert_msys_path"` Phase 2 crates | PASS — 7 Phase 2 크레이트에서 사용 (touch, wc, tee, mkdir, rmdir, basename, dirname) |
| D-27 — mkdir -p delegates to `create_dir_all` | `grep "create_dir_all"` gow-mkdir | PASS — 직접 위임 |
| D-28 — rmdir -p 수동 parent-walk | `grep "remove_dir"` gow-rmdir | PASS — 루프 존재 |
| D-30c — autonomous PowerShell 검증 | 모든 플랜 frontmatter `autonomous: true` | PASS — Phase 2 11개 플랜 전부 autonomous, 수동 체크포인트 없음 |

**18/18 spot-checks passed.**

---

## Build / Test Evidence

```
$ cargo build --workspace
   Compiling gow-core ... gow-probe
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.07s
```

경고 0건, 에러 0건.

```
$ cargo test --workspace
Total tests passed: 265, failed: 0
```

테스트 분포 (sum of non-zero bins):
- gow-core: 34 unit + 3 doctest (Phase 1 regression intact)
- gow-wc: 8 unit + 14 integration = 22
- gow-echo: 9 unit + 13 integration = 22
- gow-env: 16 unit + 8 integration = 24
- gow-pwd: 8 unit + 9 integration = 17
- gow-which: 5 unit + 12 integration = 17
- gow-touch: 9 unit + 11 integration = 20
- gow-tee: 6 unit + 13 integration = 19
- gow-basename: 17
- gow-dirname: 8 integration
- gow-mkdir: 9 integration
- gow-rmdir: 9 integration
- gow-yes: 6 integration
- gow-true: 3 integration
- gow-false: 4 integration
- gow-probe: 6 integration
- plus 2 lib unit + 5 unit elsewhere

```
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
```

Lint 경고 0건.

### 바이너리 존재 확인

`target/x86_64-pc-windows-msvc/debug/` 디렉토리에서 기대되는 15개 바이너리 전부 생성됨:

```
basename.exe, dirname.exe, echo.exe, env.exe, false.exe,
gow-probe.exe, mkdir.exe, pwd.exe, rmdir.exe, tee.exe,
touch.exe, true.exe, wc.exe, which.exe, yes.exe
```

---

## Pitfall Check

CONTEXT.md와 RESEARCH.md에서 식별된 주요 pitfall 각각의 mitigation이 코드에 관찰되는지.

| Pitfall | Mitigation 위치 | 상태 |
| ------- | -------------- | ---- |
| `filetime::set_file_mtime` 이 심링크를 따라가는 문제 (Q2) | `gow-touch/src/timestamps.rs:19` `set_symlink_file_times` 직접 호출 | ✓ |
| env가 shell 문자열로 spawn 하여 quoting 깨짐 (D-19b, Phase 1 Pitfall #5) | `gow-env/src/lib.rs:158` `StdCommand::new(command_name)` | ✓ |
| `\\?\` prefix가 UNC 경로에서 스트립되면 깨지는 문제 (Q8) | `gow-pwd/src/canonical.rs:19` `simplify_canonical` + 8 unit tests | ✓ |
| PATHEXT 미탐색으로 `which foo` 실패 (GOW #276, D-18) | `gow-which/src/lib.rs:62` literal-first + PATHEXT 확장 fallback | ✓ |
| `yes \| head` 가 BrokenPipe panic (D-23, Q4) | `gow-yes/src/lib.rs:61` `ErrorKind::BrokenPipe => return 0` | ✓ |
| `tee` 중 Ctrl+C 수신 시 stdout 손상 (D-25, Q10) | `gow-tee/src/signals.rs:19` `SetConsoleCtrlHandler(None, 1)` | ✓ |
| `wc` 가 invalid UTF-8 바이트 스트림에서 panic (D-17b) | `gow-wc` 는 `bstr::chars` 사용 — U+FFFD 로 복구 | ✓ |
| MSYS `/c/Users/...` 경로 처리 | Path 받는 모든 유틸리티에서 `try_convert_msys_path` 호출 (7 crates) | ✓ |

8/8 pitfall 전부 코드 레벨에서 관찰 가능.

---

## Cross-Phase Regression

Phase 1 (Foundation) 회귀 체크:

| 테스트 | 기대 | 관찰 |
| ----- | ---- | ---- |
| `cargo test -p gow-core` unit | 34 passed | 34 passed ✓ |
| `cargo test -p gow-core --doc` | 3 passed | 3 passed ✓ |
| `gow-probe` integration | 6 passed | 6 passed ✓ |
| `cargo build --workspace` (MSVC) | 0 warnings | 0 warnings ✓ |
| Phase 1 `[[bin]]` manifest longPathAware | 유지 | `gow-probe/build.rs` 변경 없음 확인 |

Phase 1 산출물 모두 그대로 유지. 회귀 0건.

---

## Anti-Patterns Scan

`crates/**/*.rs` 전체에서 다음 패턴 조회:

| 패턴 | 결과 |
| ---- | ---- |
| `TODO`, `FIXME`, `XXX`, `HACK`, `PLACEHOLDER` | 0건 |
| `unimplemented!`, `todo!` 매크로 | 0건 |
| `return \[\]`, `return \{\}` 하드코드된 빈 값 | 코드 경로에 해당 없음 |
| Console.log-only 구현 | 해당 없음 (Rust 프로젝트) |
| Placeholder UI 문자열 ("coming soon") | 해당 없음 (CLI 유틸리티) |

Anti-pattern 0건 발견.

---

## Human Verification Required

**None.** CONTEXT.md 의 D-30c 및 Phase 2의 모든 11 플랜 frontmatter `autonomous: true` 에 따라 수동 체크포인트는 의도적으로 생략됐다. 모든 Windows-specific 동작은 programmatic 테스트(assert_cmd + `SetConsoleOutputCP` initialization)로 재현되도록 설계됐고, 실제로 Windows 11 호스트(rustc 1.95.0 / MSVC) 에서 `cargo test --workspace` 265 tests 전부 green 을 관찰했다.

Phase 2 는 stateless utilities 만 다루며, 실시간 파일 시스템 감시(`tail -f` — Phase 3), 대화형 UI(`less` — Phase 5), 네트워크 동작 (`curl` — Phase 6) 과 같은 인간 검증이 필요한 항목은 해당 Phase 에서 다룬다.

---

## Findings

**No gaps — all requirements verified.**

### 관찰된 특이사항 (Informational, not gaps)

1. **`wc: /path: 지정된 파일을 찾을 수 없습니다. (os error 2)`** — `wc` 가 missing file 에러를 표시할 때 메시지 본문은 Windows OS 로캘에 따라 localized 됨 (한국어 호스트에서는 한글). GNU 포맷 `<utility>: <path>: <message>` 구조와 exit code(=1)는 준수되며, 이는 Rust `io::Error` 의 Windows `FormatMessage` 반환값을 그대로 propagate 한 결과. GNU 호환성 기준에서는 허용 범위이고, 필요 시 v2 에서 메시지 영문화 백로그로 둘 수 있음.

2. **Phase 2 workspace members 순서** — `Cargo.toml` 에 Phase 2 크레이트가 comment `# Phase 2 — stateless utilities (D-16)` 아래 14개 전부 나열돼 있어 향후 phase 추가 시 구조적으로 확장 가능.

3. **필드 폭 정렬** — `wc` 의 기본 출력 `" 1  2 14 {file}"` 은 GNU 관례에 따른 right-aligned 컬럼 폭으로 정렬됨. 복수 파일 입력에서 total 라인이 동적 필드 폭에 맞춰 출력되는 동작은 integration 테스트로 검증됨.

---

## Gaps Summary

**해당 없음.** 14/14 요구사항 SATISFIED, 5/5 ROADMAP Success Criteria 실행 증거로 VERIFIED, 18/18 CONTEXT decision spot-check PASS, 8/8 pitfall mitigation 코드 관찰 가능, Phase 1 회귀 0건, clippy/build 경고 0건, 265 tests all green.

Phase 2 목표 "Users can run the complete set of simple, stateless GNU utilities and observe correct GNU-compatible behavior on Windows" 은 달성되었다. Phase 3 진입 준비 완료.

---

*Verified: 2026-04-21*
*Verifier: Claude (gsd-verifier)*

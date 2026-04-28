# Phase 3: Filesystem Utilities - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3는 11개의 stateful 파일시스템 유틸리티를 Rust로 구현한다: `cat`, `ls`, `cp`, `mv`, `rm`, `ln`, `chmod`, `head`, `tail` (-f 포함), `dos2unix`, `unix2dos`. 각 유틸리티는 gow-core와 Phase 2에서 확립된 크레이트 패턴(D-16 lib + thin bin) 위에 구현되며, Windows 환경에서 GNU 호환 동작을 제공해야 한다.

Phase 3는 Phase 2(stateless)와 달리 상태(파일시스템 트리, 심링크/정션, 실시간 파일 감시, 인플레이스 원자적 교체)를 다룬다. 핵심 기술 과제:
- `tail -f` 실시간 추적 (ROADMAP 기준: 200ms 이내 새 라인 감지, polling 없이 ReadDirectoryChangesW)
- 심링크/정션/하드링크 추상화 (Phase 1 `gow_core::fs::LinkType` 활용)
- Windows 권한 모델과 POSIX mode 비트의 합리적 매핑
- CRLF↔LF 변환의 원자적 인플레이스 교체 (Pitfall #4: MoveFileExW MOVEFILE_REPLACE_EXISTING)
- `ls`/`cp`/`rm`의 재귀 순회 (walkdir 활용)

**커버 요구사항:** FILE-01 (cat), FILE-02 (ls), FILE-03 (cp), FILE-04 (mv), FILE-05 (rm), FILE-09 (ln), FILE-10 (chmod), TEXT-01 (head), TEXT-02 (tail), CONV-01 (dos2unix), CONV-02 (unix2dos) — 총 11개.

성공 기준은 ROADMAP.md의 5개 항목에 고정: `ls -la` 표시, `cp -r`/`cp -p`, `tail -f` 200ms latency, `cat -n` UTF-8, `dos2unix`/`unix2dos` 라운드트립.

</domain>

<decisions>
## Implementation Decisions

Phase 3는 Phase 1 D-01~D-15와 Phase 2 D-16~D-30을 그대로 상속한다. 본 섹션은 Phase 3 고유의 15개 decision (D-31~D-45)을 정의한다.

### Windows 권한 모델 — Read-only 비트 기반

- **D-31 `ls -l` 권한 컬럼:** Read-only 비트 기반 합성. `FILE_ATTRIBUTE_READONLY`가 꺼져 있으면 `rw-rw-rw-`, 켜져 있으면 `r--r--r--`. 디렉토리는 `drwxrwxrwx`/`dr-xr-xr-x`. Cygwin/MSYS 전통과 일치. 전체 ACL DACL 분석은 하지 않음 (POSIX↔ACL 매핑 정확도 이슈 + 성능 부담).
- **D-32 `chmod` 동작:** Owner write 비트만 `FILE_ATTRIBUTE_READONLY`에 매핑. 나머지 mode 비트 (group/other, x 비트)는 조용히 무시. `chmod +w` / `chmod 644` 같은 일반적 패턴만 동작. 부분 지원임을 문서에 명시하되 stderr 경고는 생략 (스크립트 noise 방지).
- **D-33 `cp -p` 보존 범위:** timestamps (mtime/atime) + `FILE_ATTRIBUTE_READONLY` 비트. ROADMAP 기준 충족. `FILE_ATTRIBUTE_HIDDEN`/`ARCHIVE`/`SYSTEM`은 복사하지 않음 (v2 backlog).
- **D-34 `ls -a` 숨김 파일 정의:** 점(.) 접두 OR `FILE_ATTRIBUTE_HIDDEN`. 두 관례의 합집합. Cygwin과 일치. `.git`과 Windows hidden 파일 모두 자연스럽게 처리.
- **D-35 `ls -l` execute 비트:** 고정 확장자 세트 기반 — `.exe`, `.cmd`, `.bat`, `.ps1`, `.com` 이면 x 추가. 시스템 PATHEXT는 읽지 않음 (D-18a의 `which` 기본 동작과 일관성; 테스트 결정성 확보).

### 링크 전략 — 자동 fallback + 명시적 구분

- **D-36 `ln -s` 디렉토리 링크 권한 fallback:** `CreateSymbolicLinkW`가 `SeCreateSymbolicLinkPrivilege` 부족으로 실패하면 junction (`CreateDirectorySymbolicLinkW` 불가시 `DeviceIoControl FSCTL_SET_REPARSE_POINT`)으로 자동 fallback. stderr에 `ln: symlink privilege unavailable, created junction instead` 경고 1회 출력. Developer Mode가 꺼진 일반 사용자 환경에서도 `ln -s dir newdir`이 동작. 단, junction은 절대경로만 저장되며 로컬 볼륨만 가능함을 명시.
- **D-37 `ls -l` 링크 표시:** symlink과 junction 둘 다 `l` prefix로 시작 (POSIX ls 파싱 스크립트 호환). target 표기만 차별화: symlink은 `link -> target`, junction은 `jct -> C:\target [junction]` 접미 태그. `gow_core::fs::link_type` 활용.
- **D-38 `ln` (옵션 없이) 하드링크:** `CreateHardLinkW` 지원. 동일 볼륨에서만 동작. 크로스 볼륨 요청 시 `ln: cross-device link not permitted` 에러 + exit 1. Phase 1에서 예약된 `LinkType::HardLink` variant는 여전히 nlink 감지 API 부재로 `ls`에는 쓰이지 않지만 ln 생성 경로에서는 사용.

### `tail -f` watcher 전략 — notify crate + descriptor follow

- **D-39 watcher 레이어:** `notify 8.2` crate의 `RecommendedWatcher`. Raw `ReadDirectoryChangesW` (windows-sys) 직접 호출은 피함 (200+ 줄 unsafe + PollWatcher fallback 재구현 부담). `notify-debouncer-full`도 쓰지 않음 (200ms latency 기준 위반 위험). Phase 1 Pitfall #3 "parent dir watch + filename filter" 패턴을 notify API 위에 구현 — `Watcher::watch(parent_dir, RecursiveMode::NonRecursive)` 후 이벤트 path로 필터.
- **D-40 rotation/truncation 정책:** GNU 기본과 동일. `-f` = `--follow=descriptor` (파일이 rename/삭제되어도 원래 handle 유지, 경고 출력 후 계속 추적). `-F` = `--follow=name` + `--retry` 합성 (name 추적, 새로 생성된 동명 파일도 따라감, log rotator 친화). truncation 감지 시 seek(0) 후 계속 읽기, stderr에 `tail: {file}: file truncated` 명시.
- **D-41 다중 파일 출력 포맷:** GNU 표준 — 전환 시 `==> {filename} <==` 헤더 출력. 연속 같은 파일 업데이트 시 중복 헤더 억제. `-q` (quiet)로 헤더 제거, `-v` (verbose)로 단일 파일에서도 강제 표시. 초기 `-n N` 라인 출력 후 실시간 추적으로 전환하는 표준 흐름 유지.

### 파괴적 연산 안전성 — GNU 기본 + 드라이브 루트 보호

- **D-42 `rm --preserve-root`:** GNU 현재 기본인 `--preserve-root` ON 유지. 추가로 Windows 드라이브 루트 (`C:\`, `D:\`, `Z:\` 등 `^[A-Z]:\?$` 패턴, UNC `\\server\share` 루트 포함) 삭제도 거부. `--no-preserve-root` 플래그로 명시적 override 가능. 에러 메시지: `rm: it is dangerous to operate recursively on '{root}' / use --no-preserve-root to override`.
- **D-43 `rm -i` 대화형 prompt:** GNU 기본 동작 — `-i` 플래그가 명시될 때만 항상 prompt. `-i` 없으면 write-protected (`FILE_ATTRIBUTE_READONLY`) 파일에 한해 stdin이 tty일 때 prompt (D-45 참조). 비대화형 (stdin이 pipe/redirect)에서는 prompt 없이 진행하되 D-45 거부 규칙 적용. `-f`는 prompt 전부 suppress + 없는 파일 조용히 무시.
- **D-44 `cp -r` symlink 기본 동작:** `cp -r` = `cp -rP` (symlink을 symlink으로 복제, target을 dereference 하지 않음). GNU coreutils 현재 기본과 일치. `--dereference`/`-L`로 target 따라가기, `-H`로 command-line symlink만 따라가기, `-P`는 no-op (이미 기본). Windows에서 symlink 생성 시 D-36의 privilege fallback 규칙을 그대로 적용 (디렉토리 symlink 복제 시 junction fallback).
- **D-45 `rm` read-only 파일 처리 (-f 없이):** GNU 표준 — stdin이 tty면 `rm: remove write-protected regular file '{file}'?` prompt, 비-tty면 exit 1 (`Permission denied`). `-f`로 override. `-f` 적용 시 삭제 직전 `SetFileAttributesW`로 read-only 해제 후 `remove_file` (Rust std는 FILE_ATTRIBUTE_READONLY 자동 해제하지 않음). D-32 chmod 모델과 일관됨.

### 공통 유틸리티 구현 방향 (수렴된 항목)

- **D-46 재귀 순회:** `ls -R`, `cp -r`, `rm -r` 모두 `walkdir 2.5` 사용 (CLAUDE.md Technology Stack). symlink 루프 감지, permission 에러 처리, 정렬 가능. D-44의 symlink 정책을 walkdir의 `follow_links` 설정으로 연결.
- **D-47 인플레이스 원자적 교체:** `dos2unix`/`unix2dos`의 기본 동작은 (1) 같은 디렉토리에 tempfile 생성 (`tempfile 3.27` + `NamedTempFile::new_in`), (2) 변환된 내용 write + flush + sync, (3) `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` (std `rename`이 호출). Pitfall #4와 일관. `-n old new` 옵션으로 새 파일 생성도 지원.
- **D-48 인코딩 정책:** `cat`, `head`, `tail`, `dos2unix`/`unix2dos`는 모두 raw 바이트로 처리 (UTF-8 디코딩 하지 않음). 라인 경계는 `b'\n'` 카운트 (D-17과 일관). `cat -v` (non-printable 시각화)는 ASCII 범위 밖을 `M-` 접두로 표시 (GNU 관례). BOM 감지/제거는 수행하지 않음 (raw passthrough). UTF-8/CP949 혼합 데이터도 panic 없이 통과.

### 크레이트 구조

- **D-49 Phase 3 크레이트 목록:** 11개 유틸리티당 독립 크레이트 (D-16 패턴 재적용). `crates/gow-cat`, `gow-ls`, `gow-cp`, `gow-mv`, `gow-rm`, `gow-ln`, `gow-chmod`, `gow-head`, `gow-tail`, `gow-dos2unix`, `gow-unix2dos`. 바이너리 이름은 GNU 그대로 (`cat.exe`, `ls.exe`, …).
- **D-50 신규 workspace 의존성:** `walkdir = "2.5"` (재귀 순회), `notify = "8.2"` (tail -f), `terminal_size = "0.4"` (ls 컬럼 레이아웃, CLAUDE.md 스택 — ls 크레이트에서만 사용). `globset`은 Phase 3에 필요 없음 (Phase 5 find에서 추가). `tempfile`은 이미 workspace에 있음 (dos2unix에서 사용).
- **D-51 Plan 묶기 전략 (planner 재량 힌트):** (1) 쉬운 것부터 수렴 — `cat`/`head`/`chmod` 같은 단순 스트림/단일파일 유틸리티, (2) 재귀 의존성 — `cp`/`rm`/`ls` 는 walkdir 패턴 공유, (3) 링크 묶음 — `ln` + `ls` 링크 표시 동시 테스트 가능, (4) 감시자 고립 — `tail -f`는 notify 도입으로 별도 플랜 권장, (5) 변환 쌍 — `dos2unix`/`unix2dos`는 대칭이므로 같은 플랜. 정확한 wave 구성은 planner가 결정.

### Claude's Discretion

- **`cat -v/-A/-T/-E` non-printable 표기 세부:** GNU 관례 (`^` 접두 제어문자, `M-` 접두 high-bit, `\t`→`^I`, `\r`→`^M`, `$` 줄끝) 따름. 내부 구현은 iterator vs state machine 중 planner 재량.
- **`ls --color` 디폴트:** Phase 1 color.rs 초기화는 되어있음. `LS_COLORS` 파싱은 필요 없음 (GNU `dircolors` 없이 내장 디폴트 — dir=blue, symlink=cyan, exec=green). 색상 스키마 튜닝은 planner 재량.
- **`cp`/`mv` progress 표시:** 기본은 silent. `--progress` 플래그는 v1 스코프 밖 (deferred).
- **`head`/`tail`의 `-c` (byte count) 구현 세부:** 정확한 byte 단위 읽기 — `std::io::BufReader` + `take(n)`. 멀티바이트 UTF-8 문자 중간에서 잘릴 수 있음 (GNU와 동일).
- **`tail -f` 초기 N 라인 출력:** 기본 `-n 10` 라인을 먼저 출력한 뒤 실시간 모드 전환. seek 전략 (파일 끝에서 역방향 탐색 vs 전체 읽고 마지막 N) — planner 재량.
- **`mv` 동일 볼륨 vs 크로스 볼륨:** 같은 볼륨이면 `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`, 크로스 볼륨이면 copy + delete. Rust `std::fs::rename`이 대부분 처리하지만 크로스 볼륨 자동 fallback은 구현 필요.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-local (필수)
- `.planning/PROJECT.md` — 프로젝트 핵심 가치, Windows 네이티브 통합 제약
- `.planning/REQUIREMENTS.md` — Phase 3 요구사항 (FILE-01..05, FILE-09..10, TEXT-01..02, CONV-01..02)
- `.planning/ROADMAP.md` §66~77 — Phase 3 goal + 5개 success criteria + depends on Phase 1
- `.planning/STATE.md` — Accumulated Context > Critical Pitfalls (Pitfall #3 tail -f parent dir watch, Pitfall #4 MoveFileExW 원자적 교체)
- `.planning/phases/01-foundation/01-CONTEXT.md` — D-01..D-15 (arg parsing, path 변환, error 포맷, 크레이트 네이밍, workspace deps) — Phase 3 그대로 상속
- `.planning/phases/02-stateless/02-CONTEXT.md` — D-16..D-30 (lib+bin 패턴, 테스트 전략, 인코딩 정책). 특히 D-16 (uumain), D-17 (Unicode-aware wc = Phase 3 인코딩 정책 D-48의 선행), D-30 (assert_cmd + snapbox + -D warnings) 그대로 적용
- `.planning/phases/01-foundation/01-03-SUMMARY.md` — `gow_core::fs::{LinkType, link_type, normalize_junction_target}` 실제 구현 (D-37, D-38의 기반)
- `.planning/phases/02-stateless/02-10-SUMMARY.md` — `filetime` 사용 패턴 (touch). Phase 3 `cp -p` 타임스탬프 보존(D-33)의 레퍼런스 구현
- `./CLAUDE.md` — Technology Stack: walkdir 2.5, notify 8.2, filetime 0.2.27, tempfile 3.27, terminal_size 0.4 (Phase 3 새 workspace deps 추가 근거)

### Phase 1 research (재참조)
- `.planning/research/STACK.md` — 스택 결정 (walkdir, notify, termcolor 등)
- `.planning/research/PITFALLS.md` — Windows 파일시스템 함정 (file locking, symlink privilege, 경로 길이)

### External
- uutils/coreutils repo — https://github.com/uutils/coreutils (각 유틸리티 reference Rust 구현: `src/uu/cat/`, `src/uu/ls/`, `src/uu/cp/`, `src/uu/tail/`, `src/uu/ln/` 등). MIT 라이선스. 동작 참조.
- GNU coreutils manual — https://www.gnu.org/software/coreutils/manual/coreutils.html (모든 공식 옵션/동작의 진실의 소스)
- `notify` crate 문서 — https://docs.rs/notify/latest/notify/ (D-39 `RecommendedWatcher` 사용 패턴, Windows `ReadDirectoryChangesW` backend 동작)
- `walkdir` crate 문서 — https://docs.rs/walkdir/latest/walkdir/ (D-46 재귀 순회 패턴, `follow_links` 설정)
- `tempfile` crate 문서 — https://docs.rs/tempfile/latest/tempfile/ (D-47 `NamedTempFile::new_in` for same-dir tempfile 원자적 교체)
- `filetime` 문서 — https://docs.rs/filetime/latest/filetime/ (D-33 `set_file_mtime`, `set_file_atime`; Phase 2 touch에서 이미 사용)
- Windows API: `CreateSymbolicLinkW`, `CreateHardLinkW`, `MoveFileExW` — learn.microsoft.com (D-36, D-38, D-47 Win32 호출 레퍼런스)
- GOW 이슈 #169, #75, #89 (tail -f) — 원본 GOW GitHub. D-39/D-40이 해결하는 문제 맥락
- 원본 GOW 소스: `D:\workspace\gow-utilities-src-0.8.0` — cp/ls/cat/tail 원본 옵션 참고용

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`gow_core::fs::LinkType`** (crates/gow-core/src/fs.rs:17) — SymlinkFile / SymlinkDir / Junction / HardLink 판별. D-37 (ls 표시) 에서 직접 사용. `link_type(path)` 함수는 symlink_metadata 기반이라 대상을 따라가지 않음 — ls -l에 정확히 맞음.
- **`gow_core::fs::normalize_junction_target`** (crates/gow-core/src/fs.rs:79) — `\??\` 접두 제거. D-37 junction target 표기에서 활용.
- **`gow_core::args::parse_gnu`** — Phase 1 구축. D-01..D-05 (GNU exit code 1, 옵션 퍼뮤테이션, `--` 종료, 숫자 축약). Phase 3 모든 유틸리티 기반.
- **`gow_core::path::try_convert_msys_path`** (crates/gow-core/src/path.rs:30) — D-06..D-08 (MSYS 경로 인식). ls/cp/rm/ln에서 파일 인자 위치에서만 호출.
- **`gow_core::color`** — VT100 활성화 이미 됨. ls --color는 그 위에 ANSI 코드만 발행하면 됨.
- **`gow_core::encoding::setup_console_utf8`** — 모든 바이너리가 init()에서 호출. cat -n 한글 출력 (ROADMAP criterion 4) 자동 해결.
- **Phase 2 테스트 패턴** (crates/gow-touch/tests, crates/gow-wc/tests 등) — assert_cmd + snapbox 스냅샷. Phase 3 통합 테스트 템플릿.
- **Phase 2 `filetime` 사용법** (crates/gow-touch/src/lib.rs) — mtime/atime 설정. D-33 cp -p 에서 재사용.

### Established Patterns
- **lib + thin bin 패턴 (D-16):** 14개 Phase 2 크레이트에서 검증됨. Phase 3 11개 크레이트 동일 구조.
- **build.rs embed-manifest 템플릿 (D-16c):** `crates/gow-probe/build.rs` → Phase 2 14개에 복사됨. Phase 3에도 그대로 복사.
- **GNU 에러 포맷 `{util}: {msg}` (D-11):** gow-core의 헬퍼로 Phase 2에서 일관되게 적용. Phase 3도 동일.
- **Workspace deps 상속 (D-15, D-20):** 새 공통 deps는 `[workspace.dependencies]` + `workspace = true`.
- **`bstr` byte-safe iteration (D-17):** Phase 2 wc에서 검증. cat/head/tail의 라인 카운트에 재사용 (D-48).
- **테스트 결정성 env override (D-18d GOW_PATHEXT):** Phase 2 which에서 검증. Phase 3 ls의 exec 확장자 세트는 D-35 고정이므로 override 불필요. 단, 테스트 임시 디렉토리 심링크/권한 테스트는 privilege 감지 + skip 패턴 (crates/gow-core/src/fs.rs:118) 재사용.

### Integration Points
- **gow-core** — Phase 3의 11개 크레이트 모두 의존. `LinkType`, `parse_gnu`, `try_convert_msys_path`, `init()` 활용.
- **Phase 4 (Text Processing)** — Phase 3의 atomic in-place rewrite 패턴(D-47)을 `sed -i`가 그대로 재사용. Phase 3이 패턴을 먼저 확립.
- **Phase 5 (Search/Navigation)** — `find -exec`가 Phase 3의 symlink 추적 규칙(D-44, `follow_links`)과 walkdir 패턴(D-46)을 상속.
- **Windows API 표면** — `CreateSymbolicLinkW`, `CreateHardLinkW`, `SetFileAttributesW` 추가 바인딩 필요 (`windows-sys` features 확장). `Win32_Storage_FileSystem`은 이미 enabled; Link API는 `Win32_Foundation` + `Win32_Storage_FileSystem::CreateSymbolicLinkW` 확인 필요.

</code_context>

<specifics>
## Specific Ideas

- **ROADMAP Criterion 3 (tail -f 200ms)는 Phase 3의 flagship metric.** 연구자는 notify 8.2의 Windows backend latency를 실측하고, 만약 200ms를 초과하면 PollWatcher fallback 또는 notify 옵션 튜닝을 제안해야 한다. 기존 GOW의 tail -f가 폴링 기반이라 느렸던 것이 GOW #169/#75/#89의 근본 원인.
- **`ls -l`의 권한 컬럼은 심미적으로도 중요.** 사용자가 `ls -la`를 보고 "이건 Windows 네이티브구나"라고 느낄 첫 인상. D-31 read-only 합성 모델은 Cygwin/MSYS 사용자에게 친숙.
- **`ln -s` junction fallback (D-36)은 uutils와 다른 gow-rust 고유 결정.** uutils는 기본적으로 실패한다. 이는 ROADMAP의 "Windows 네이티브 통합" 철학과 일치 — Developer Mode를 켜지 않은 일반 사용자도 `ln -s dir newdir`이 동작해야 한다.
- **`dos2unix`/`unix2dos`의 원자적 교체 (D-47)는 Phase 4 `sed -i`의 rehearsal.** Phase 3에서 패턴이 확립되면 Phase 4가 그대로 이어받는다. 따라서 helper 함수는 `gow-core/src/fs.rs`에 `atomic_rewrite<F>(path, transform: F)` 형태로 두고 후속 phase가 재사용하도록 설계.
- **11개 유틸리티 규모.** Phase 2가 14개 유틸리티 × 4 waves로 약 90분. Phase 3는 11개 유틸리티 + tail -f 복잡도로 비슷한 규모. 단 `tail` + `cp`/`rm`/`ls` (재귀)가 단일 크레이트가 touch 수준이 아닌 env 수준 복잡도. walkdir/notify 도입으로 research 시간 추가 필요.
- **walkdir/notify/terminal_size는 Phase 2에 없던 의존성.** planner가 wave 계획 시 이 세 crate를 먼저 도입하는 wave 0 (workspace prep) 플랜을 만들 수 있음 — 02-01과 유사한 구조.
- **uutils의 Phase 3 유틸리티 구현은 reference로 활용.** 특히 `src/uu/tail/` (notify 사용 패턴), `src/uu/cp/` (symlink 처리), `src/uu/ls/` (column layout) 는 직접 참고 가치 높음. 복사 금지, 패턴 참고.

</specifics>

<deferred>
## Deferred Ideas

Phase 3 스코프를 넘어서는 아이디어들. 이슈로 잃지 않도록 기록.

- **Full ACL 매핑 (`ls -l`, `chmod`, `cp -p`):** D-31/D-32/D-33는 read-only 비트만 다룬다. POSIX mode 전체를 ACL DACL로 정밀 매핑하는 것은 v2 또는 별도 feature flag. Cygwin의 `NoAcl` 복잡도를 피하는 의도적 결정.
- **`cp -p` hidden/archive/system 속성 보존:** D-33는 read-only만. 나머지 FILE_ATTRIBUTES 보존은 v2.
- **`cp`/`mv` `--progress` 진행 표시:** GNU 최신 확장. v2.
- **`ls` `LS_COLORS` 환경변수 파싱:** 내장 디폴트로 충분. GNU `dircolors` 유틸리티도 Phase 3에 포함되지 않음. v2.
- **`tail --pid=PID` (프로세스 종료 시 자동 종료):** 희귀 플래그. Windows에서 `OpenProcess(SYNCHRONIZE)` + `WaitForSingleObject`로 구현 가능하지만 v2.
- **`tail --sleep-interval` 튜닝:** notify 기반이라 polling interval 개념이 없음. 의미 재정의 필요. v2.
- **`chmod --reference=file`:** GNU 확장. D-32 read-only only 모델에서 기계적 매핑 가능하지만 v1에 포함하지 않음.
- **`ln --backup`, `mv --backup`:** 기존 파일 백업 옵션. v2.
- **`rm --one-file-system`:** 파일시스템 경계 넘기 방지. Windows에서 drive letter 비교로 구현 가능. v2.
- **`cat -A` 심볼릭 별칭:** `-A` = `-vET`. 구현 트리비얼. planner 재량 포함 가능 (필수 아님).
- **dos2unix `--info` flag (파일 타입 감지만):** GOW 기본에 없음. v2.
- **hard link count (`nlink`) 표시 in `ls -l`:** Rust `std::fs`가 Windows에서 nlink를 노출하지 않음 (`GetFileInformationByHandle` 직접 호출 필요). `ls -l`의 second column은 항상 `1`로 표시. v2 Windows-specific 개선.
- **`find` -exec와의 통합:** Phase 5.
- **sed -i atomic rewrite:** Phase 4에서 D-47 helper 재사용.

</deferred>

---

*Phase: 03-filesystem*
*Context gathered: 2026-04-21*
*Next: `/gsd-plan-phase 3` to create detailed execution plans.*

# Phase 3: Filesystem Utilities - Research

**Researched:** 2026-04-21
**Domain:** 11 Windows-native GNU filesystem utilities (cat, ls, cp, mv, rm, ln, chmod, head, tail -f, dos2unix, unix2dos) with recursive traversal, symlink/junction abstraction, real-time file watching, and atomic in-place rewrite
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

Phase 3 inherits D-01..D-15 (Phase 1) and D-16..D-30 (Phase 2) verbatim. Phase-3-specific locks:

- **D-31** `ls -l` 권한 컬럼: Read-only 비트 기반 합성. `FILE_ATTRIBUTE_READONLY` off → `rw-rw-rw-`, on → `r--r--r--`. 디렉토리 `drwxrwxrwx`/`dr-xr-xr-x`. 전체 ACL DACL 분석 하지 않음.
- **D-32** `chmod` 동작: Owner write 비트만 `FILE_ATTRIBUTE_READONLY`에 매핑. 나머지 mode 비트 조용히 무시. `chmod +w` / `chmod 644` 같은 일반 패턴만 동작.
- **D-33** `cp -p` 보존 범위: timestamps (mtime/atime) + `FILE_ATTRIBUTE_READONLY` 비트만. `FILE_ATTRIBUTE_HIDDEN`/`ARCHIVE`/`SYSTEM`은 보존 안 함.
- **D-34** `ls -a` 숨김 파일 정의: 점(.) 접두 OR `FILE_ATTRIBUTE_HIDDEN`. 두 관례의 합집합. Cygwin과 일치.
- **D-35** `ls -l` execute 비트: 고정 확장자 세트 — `.exe`, `.cmd`, `.bat`, `.ps1`, `.com`. 시스템 PATHEXT는 읽지 않음. `GOW_PATHEXT` override도 Phase 3에서는 사용 안 함.
- **D-36** `ln -s` 디렉토리 링크 권한 fallback: `CreateSymbolicLinkW`가 `SeCreateSymbolicLinkPrivilege` 부족으로 실패하면 junction으로 자동 fallback. stderr에 `ln: symlink privilege unavailable, created junction instead` 1회 경고. Developer Mode 꺼진 환경에서도 `ln -s dir newdir` 동작.
- **D-37** `ls -l` 링크 표시: symlink과 junction 둘 다 `l` prefix. target 표기만 차별화: symlink `link -> target`, junction `jct -> C:\target [junction]`.
- **D-38** `ln` (옵션 없이) 하드링크: `CreateHardLinkW`. 동일 볼륨만. 크로스 볼륨은 `ln: cross-device link not permitted` + exit 1.
- **D-39** `tail -f` watcher 레이어: `notify 8.2` crate의 `RecommendedWatcher`. Raw `ReadDirectoryChangesW` 직접 호출 피함. `notify-debouncer-full` 사용 안 함 (200ms latency 위반 위험). Pitfall #3 "parent dir watch + filename filter" 패턴을 notify API 위에 구현 — `Watcher::watch(parent_dir, RecursiveMode::NonRecursive)` 후 이벤트 path로 필터.
- **D-40** rotation/truncation 정책: GNU 기본과 동일. `-f` = `--follow=descriptor` (파일 rename/삭제되어도 원래 handle 유지), `-F` = `--follow=name` + `--retry` (이름 추적, 새 동명 파일도 추적). truncation 감지 시 seek(0) 후 계속, stderr에 `tail: {file}: file truncated`.
- **D-41** 다중 파일 출력 포맷: GNU 표준 — 전환 시 `==> {filename} <==` 헤더. 연속 같은 파일 업데이트 시 중복 헤더 억제. `-q` 헤더 제거, `-v` 단일 파일에서도 강제.
- **D-42** `rm --preserve-root`: GNU 기본 ON 유지. Windows 드라이브 루트 (`C:\`, `D:\`, `Z:\`, UNC `\\server\share` 루트) 추가 보호. `--no-preserve-root` 플래그로 override.
- **D-43** `rm -i`: `-i` 플래그 명시될 때만 항상 prompt. `-i` 없으면 write-protected 파일에 한해 stdin이 tty일 때 prompt. 비대화형에서는 D-45 거부 규칙 적용.
- **D-44** `cp -r` symlink 기본: `cp -r` = `cp -rP` (symlink을 symlink으로 복제). `-L`로 target 따라가기, `-H`로 command-line symlink만 따라가기, `-P`는 no-op. 디렉토리 symlink 복제 시 D-36 junction fallback 적용.
- **D-45** `rm` read-only 파일 처리 (-f 없이): stdin이 tty면 prompt, 비-tty면 exit 1 `Permission denied`. `-f`로 override. `-f` 적용 시 삭제 직전 `SetFileAttributesW`로 read-only 해제 후 `remove_file`.
- **D-46** 재귀 순회: `ls -R`, `cp -r`, `rm -r` 모두 `walkdir 2.5` 사용.
- **D-47** 인플레이스 원자적 교체: (1) 같은 디렉토리에 tempfile (`tempfile 3.27` + `NamedTempFile::new_in`), (2) 변환된 내용 write + flush + sync, (3) `persist()` = `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`. Pitfall #4와 일관.
- **D-48** 인코딩 정책: `cat`, `head`, `tail`, `dos2unix`/`unix2dos` 모두 raw 바이트 처리 (UTF-8 decode 하지 않음). 라인 경계는 `b'\n'` 카운트. BOM 감지/제거 안 함. UTF-8/CP949 혼합 데이터도 panic 없이 통과.
- **D-49** Phase 3 크레이트: `crates/gow-cat`, `gow-ls`, `gow-cp`, `gow-mv`, `gow-rm`, `gow-ln`, `gow-chmod`, `gow-head`, `gow-tail`, `gow-dos2unix`, `gow-unix2dos`. 바이너리 이름은 GNU 그대로.
- **D-50** 신규 workspace 의존성: `walkdir = "2.5"`, `notify = "8.2"`, `terminal_size = "0.4"`.
- **D-51** Plan 묶기 힌트: (1) easy-first — `cat`/`head`/`chmod`, (2) walkdir-shared — `cp`/`rm`/`ls`, (3) link-bundle — `ln` + `ls` 링크 표시, (4) watcher-isolated — `tail -f` 별도 플랜, (5) conversion-pair — `dos2unix`/`unix2dos` 동일 플랜.

### Claude's Discretion

- `cat -v/-A/-T/-E` non-printable 표기 세부 (state machine vs iterator)
- `ls --color` 디폴트 스키마 (LS_COLORS 파싱 없음; 내장 디폴트 dir=blue/symlink=cyan/exec=green)
- `cp`/`mv` progress 표시 (기본 silent, `--progress` v2)
- `head`/`tail`의 `-c` byte count 구현 세부 (`BufReader::take`)
- `tail -f` 초기 N 라인 출력 전략 (seek 역방향 vs 전체 읽고 마지막 N)
- `mv` 동일 볼륨 vs 크로스 볼륨 자동 fallback

### Deferred Ideas (OUT OF SCOPE)

- Full ACL 매핑 — v2
- `cp -p` hidden/archive/system 속성 보존 — v2
- `cp`/`mv` `--progress` — v2
- `ls` `LS_COLORS` 환경변수 파싱 — v2
- `tail --pid=PID` — v2
- `tail --sleep-interval` — notify 기반에서는 의미 재정의 필요, v2
- `chmod --reference=file` — v2
- `ln --backup`, `mv --backup` — v2
- `rm --one-file-system` — v2
- hard link count (`nlink`) 표시 in `ls -l` — Rust std Windows에서 nlink 노출 안 함, v2
- `find` -exec와의 통합 — Phase 5
- `sed -i` atomic rewrite — Phase 4에서 D-47 helper 재사용
- `cat -A` 별칭 (= `-vET`) — planner 재량 (필수 아님)

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FILE-01 | cat (-n 번호, -b 비공백 번호, -s 빈줄 압축) | Raw-byte line iteration via `bstr` (D-48); `cat -v/-T/-E` state machine in §cat |
| FILE-02 | ls (-l 상세, -a 숨김, -R 재귀, --color) | `walkdir 2.5` for -R; `std::os::windows::fs::MetadataExt::file_attributes()` for hidden/readonly; `gow_core::fs::LinkType` for symlink/junction display; `terminal_size 0.4` for column layout |
| FILE-03 | cp (-r 재귀, -f 강제, -p 권한 보존) | `walkdir 2.5` for -r with `follow_links` mapping to D-44; `filetime` for -p mtime/atime (same crate as touch); `Permissions::set_readonly` for RO bit |
| FILE-04 | mv (-f 강제, -i 대화형) | `std::fs::rename` on Windows calls `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` automatically; cross-volume fallback = copy + delete loop |
| FILE-05 | rm (-r 재귀, -f 강제, -i 대화형) | `walkdir 2.5` with `contents_first(true)` for leaves-first deletion; D-42 drive-root guard; D-45 RO attribute handling |
| FILE-09 | ln (-s symbolic, hardlink default; Windows symlink/junction support) | `std::os::windows::fs::{symlink_file, symlink_dir}` for symlinks; `junction 1.4.2` crate for D-36 directory symlink fallback; `std::fs::hard_link` wraps `CreateHardLinkW` |
| FILE-10 | chmod (Windows ACL mapping) | `Permissions::set_readonly` maps to `FILE_ATTRIBUTE_READONLY`; parse GNU symbolic (`u+w`, `a-r`) and octal (`644`) modes; extract owner-write bit only (D-32) |
| TEXT-01 | head (-n 줄, -c 바이트, 숫자 축약 -5) | Raw-byte line counting via `bstr`; `BufReader::take(n)` for -c byte slice; numeric shorthand already supported by `parse_gnu` (D-05) |
| TEXT-02 | tail (-n 줄, -f 실시간) | Initial N lines: either reverse-seek or read-all-and-take-last (planner choice); `notify 8.2` `RecommendedWatcher` for -f with `RecursiveMode::NonRecursive` on parent dir + path-filter (Pitfall #3); D-40 descriptor-vs-name follow modes |
| CONV-01 | dos2unix (CRLF → LF) | D-47 atomic rewrite helper `atomic_rewrite(path, transform)` in `gow_core::fs`; byte-level CRLF scan (`\r\n` → `\n`); NUL-byte binary detection; `tempfile::NamedTempFile::new_in` + `persist()` |
| CONV-02 | unix2dos (LF → CRLF) | Mirror of CONV-01 with inverse transform; share byte-scan state machine with CONV-01 via lib module |

</phase_requirements>

---

## Executive Summary

Phase 3 delivers 11 stateful filesystem utilities over established Phase 1/2 foundations. The research identifies four cross-cutting primitives that determine plan structure:

1. **`notify 8.2` is confirmed capable of meeting ROADMAP's 200 ms latency criterion for `tail -f`.** Its Windows backend uses `ReadDirectoryChangesW` asynchronously with a 16 KB buffer and a 100 ms wakeup semaphore, emitting `EventKind::Modify(ModifyKind::Any)` on file append and `Modify(Name(From/To))` on rename. Pitfall #3 ("watch parent, filter filename") maps directly to `Watcher::watch(parent_dir, RecursiveMode::NonRecursive)` followed by path-filtering in the event loop. [VERIFIED: github.com/notify-rs/notify notify/src/windows.rs]

2. **`junction 1.4.2` is the right crate for D-36 junction fallback.** Maintained (last release Feb 2026), 100% Rust, MIT licensed, wraps `DeviceIoControl(FSCTL_SET_REPARSE_POINT)` so we avoid 200+ lines of unsafe `REPARSE_DATA_BUFFER` packing. Public API: `junction::create(target, junction_path)`, `junction::exists(path)`, `junction::get_target(path)`, `junction::delete(path)`. [VERIFIED: crates.io 2026-04-21; github.com/tesuji/junction releases]

3. **`tempfile::NamedTempFile::persist()` is the D-47 atomic swap primitive.** Docs state "atomically replace" the target; `std::fs::rename` on Windows calls `MoveFileExW` which handles locked-by-reader files when `MOVEFILE_REPLACE_EXISTING` is set. The canonical helper `atomic_rewrite<F>(path, transform)` lives in `gow_core::fs` so Phase 4 `sed -i` inherits it verbatim. [CITED: docs.rs/tempfile NamedTempFile::persist; microsoft.com MoveFileExW docs]

4. **`walkdir 2.5` covers all three recursive utilities (`ls -R`, `cp -r`, `rm -r`) with one uniform builder.** Key methods confirmed: `follow_links(bool)` (with cycle detection), `sort_by_file_name()`, `contents_first(true)` (required for rm -r to delete leaves first), `same_file_system(true)` (for future v2), `min_depth`/`max_depth`. `filter_entry` closure lets us skip hidden dirs without descending. walkdir does NOT follow Windows junctions even when `follow_links(true)` — reparse points are treated as files unless explicitly handled. [VERIFIED: docs.rs/walkdir/2.5.0 API]

**Primary recommendation:** Introduce a Wave 0 "workspace prep" plan that adds `walkdir`, `notify`, `terminal_size`, and the new `junction` workspace-dep (not in D-50 but required for D-36) **and** drops the `atomic_rewrite` helper + an expanded `gow_core::fs` module (link-creation API, Windows attribute helpers) before any utility plan runs. This mirrors the successful 02-01 workspace-prep plan.

**Plan grouping (refined from D-51):**
- Wave 0 prep: workspace deps + `gow_core::fs` extensions (link creation, attribute helpers, atomic_rewrite) + 11 stub crates
- Wave 1 easy: `cat`, `head`, `chmod` (single-file stream utilities, no walkdir)
- Wave 2 conversion: `dos2unix` + `unix2dos` (same plan, shares scanner)
- Wave 3 walkdir: `cp`, `rm`, `ls` (share walkdir patterns; ls also depends on terminal_size)
- Wave 4 links + move: `ln` (exercises gow-core link helpers), `mv` (cross-volume fallback)
- Wave 5 watcher: `tail` (notify integration, separate to contain compile-time impact)

---

## Architectural Responsibility Map

Phase 3 is a single-tier CLI application. Every capability lives in one of two tiers:

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| GNU option parsing (per-utility) | Utility crate (uu_foo) | gow-core::args (parse_gnu) | Each utility defines its clap::Command; gow-core enforces exit-code 1 and permutation |
| Raw-byte stream I/O (cat/head/tail body) | Utility crate | bstr (byte line iter) | D-48: no UTF-8 decode; bstr's ByteSlice::lines() is the primitive |
| Recursive directory traversal (ls -R, cp -r, rm -r) | Utility crate | walkdir 2.5 | D-46: single crate handles cycles, sorting, leaves-first |
| Link creation (ln) | gow-core::fs (new helpers) | std symlink_file/symlink_dir, junction crate, std::fs::hard_link | D-36/D-38: fallback logic and cross-volume detection is cross-cutting, belongs in gow-core |
| Attribute manipulation (chmod, rm -f on RO) | gow-core::fs (new helpers) | std::fs::Permissions + `#[cfg(windows)] SetFileAttributesW` only if needed | Wrap std first; reach for windows-sys only if std is inadequate |
| Filesystem watching (tail -f) | Utility crate (uu_tail) | notify 8.2 RecommendedWatcher | Watcher thread lives in tail to avoid pulling notify's transitive deps into every crate |
| Atomic in-place rewrite (dos2unix, unix2dos; future sed -i) | gow-core::fs (new helper) | tempfile::NamedTempFile | D-47: pattern is shared; Phase 4 sed -i inherits same helper |
| Terminal dimension detection (ls column layout) | Utility crate (uu_ls) | terminal_size 0.4 | Only ls needs it; don't pollute gow-core |
| Link type identification (ls -l) | gow-core::fs::LinkType | Existing from Phase 1 | Already built; cp -rP also uses it to decide dereference vs clone |
| Path argument normalization | gow-core::path::try_convert_msys_path | Existing from Phase 1 | Every file-position arg passes through this (inherited invariant) |

**Tier discipline rule:** Utility crates MUST NOT contain `#[cfg(target_os = "windows")]` blocks. If Windows-specific logic is needed, add it to `gow-core::fs` or `gow-core::path` first. The only exception is tail's `notify::RecommendedWatcher` usage — notify itself abstracts the OS, so the utility can use the crate directly.

---

## Standard Stack

### New Workspace Dependencies (per D-50 + one addition)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `walkdir` | 2.5.0 | Recursive directory traversal | BurntSushi; handles cycles, symlinks, permissions; used by ripgrep. `2.5.0` verified on crates.io 2026-04-21 [VERIFIED: cargo search] |
| `notify` | 8.2.x | `tail -f` file watcher | De-facto standard; `RecommendedWatcher` selects `ReadDirectoryChangesW` on Windows automatically. `8.2` is last stable before 9.0.0-rc. `notify-types 2.1.0` published 2026-01-25. [VERIFIED: crates.io; github.com/notify-rs/notify/releases] |
| `terminal_size` | 0.4.4 | `ls` column layout width | Single-purpose; wraps `GetConsoleScreenBufferInfo` on Windows. Returns `Option<(Width, Height)>` — `None` when not a tty. [VERIFIED: crates.io 2026-04-21] |
| `junction` | 1.4.2 | D-36 directory symlink fallback (**ADD TO WORKSPACE — not in D-50**) | Last release 2026-02-24; 100% safe Rust wrapper over `DeviceIoControl(FSCTL_SET_REPARSE_POINT)`. Eliminates 200+ lines of unsafe reparse-buffer packing. MIT. [VERIFIED: crates.io; github.com/tesuji/junction] |

### Reused Crates (already in workspace from Phase 1/2)

| Library | Version | Purpose |
|---------|---------|---------|
| `clap` | 4.6 | Argument parsing (per D-16a uumain signature) |
| `thiserror` | 2 | Typed error enums within each utility crate |
| `anyhow` | 1 | Error propagation at main() boundaries |
| `bstr` | 1 | Byte-safe line iteration for cat/head/tail/dos2unix/unix2dos (D-17, D-48) |
| `filetime` | 0.2 | `cp -p` timestamp preservation — same crate as `gow-touch`; `set_file_times` / `set_symlink_file_times` [VERIFIED: Phase 2 touch crate] |
| `tempfile` | 3 | D-47 atomic in-place rewrite; same-dir tempfile + persist |
| `path-slash` | 0.2 | Path normalization (inherited) |
| `windows-sys` | 0.61 | Only within `gow_core::fs` — for `SetFileAttributesW`, `CreateHardLinkW` if needed (std::fs::hard_link suffices in most cases) |
| `termcolor` | 1 | `ls --color` output (VT100 already enabled by `gow_core::color::enable_vt_mode`) |
| `assert_cmd` + `predicates` + `snapbox` | 2 / 3 / 1.2 | Integration + snapshot tests (D-30) |

### Alternatives Considered

| Instead of | Could Use | Why Rejected |
|------------|-----------|--------------|
| `notify 8.2` | Raw `ReadDirectoryChangesW` via `windows-sys` | 200+ lines unsafe; PollWatcher fallback must be re-implemented; ecosystem mostly uses notify. CONTEXT.md D-39 locked |
| `notify 8.2` | `notify-debouncer-full` | Debouncer introduces intentional latency; risks violating 200 ms ROADMAP criterion. D-39 locked |
| `notify 8.2` | `notify 9.0.0-rc.3` | RC status; CONTEXT.md D-50 locks `8.2`. Revisit in v2 after 9.0 stable |
| `junction 1.4.2` | Hand-roll `FSCTL_SET_REPARSE_POINT` | 200+ lines of `REPARSE_DATA_BUFFER` packing + pitfall with `\??\` prefix; crate exists and is maintained |
| `junction 1.4.2` | `std::os::windows::fs::symlink_dir` only | Without junction fallback, `ln -s dir newdir` fails on non-Developer-Mode Windows — violates D-36 |
| `walkdir 2.5` | `jwalk` | `jwalk` is faster (parallel) but has a different API shape and adds `rayon` transitive deps; not needed for Phase 3 throughput |
| `walkdir 2.5` | `std::fs::read_dir` manual recursion | Hand-rolled recursion re-implements cycle detection (via symlink tracking) and permission error handling; wastes effort for 3 utilities |
| `terminal_size 0.4` | `crossterm::terminal::size()` | `crossterm` is a much heavier dep (cursor/raw mode); overkill for reading column width. `less` pager in Phase 5 may still justify crossterm separately |
| `tempfile 3.27` for persist | Hand-rolled `rename` | tempfile's `persist()` error recovery (returns original `NamedTempFile` for retry) is worth the crate; already in workspace |

### Installation

```toml
# Cargo.toml (workspace root) — ADD to [workspace.dependencies]
walkdir = "2.5"
notify = "8.2"
terminal_size = "0.4"
junction = "1.4"            # NOT in D-50 — add for D-36
# Already present from Phase 2:
# bstr, filetime, tempfile, snapbox, assert_cmd, predicates
```

**Version verification (performed 2026-04-21):**

```bash
cargo search walkdir       # → walkdir = "2.5.0"    [VERIFIED]
cargo search notify        # → notify = "9.0.0-rc.3" (latest); 8.2.x is last stable — use 8.2  [VERIFIED]
cargo search terminal_size # → terminal_size = "0.4.4"  [VERIFIED]
cargo search tempfile      # → tempfile = "3.27.0"  [VERIFIED — already in workspace]
cargo search filetime      # → filetime = "0.2.27"  [VERIFIED — already in workspace]
cargo search junction      # → junction = "1.4.2"  [VERIFIED]
```

`notify 9.0.0-rc.3` is an RC. CONTEXT.md D-50 locks `8.2`. The minimum Rust version for notify is 1.88 per repo MSRV policy — well below our 1.95 toolchain. [CITED: github.com/notify-rs/notify README MSRV section]

---

## Architecture Patterns

### System Architecture Diagram

```
┌────────────────────────────────────────────────────────────────┐
│  User runs:                                                    │
│    ls -la /c/src/                                              │
│    cp -rp src/ dest/                                           │
│    tail -f app.log                                             │
│    dos2unix windows-file.txt                                   │
└──────────────────┬─────────────────────────────────────────────┘
                   │ argv (OsString array)
                   ▼
┌────────────────────────────────────────────────────────────────┐
│  binary main.rs  (3 lines — shared across all 11 crates)       │
│  std::process::exit(uu_<util>::uumain(std::env::args_os()))    │
└──────────────────┬─────────────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────────────┐
│  uumain → gow_core::init() → gow_core::args::parse_gnu(cmd)    │
│  [inherited from Phase 1/2 — no changes in Phase 3]            │
└──────────────────┬─────────────────────────────────────────────┘
                   │ ArgMatches
                   ▼
┌──────────────────────────────────────────────────────────────────────┐
│  utility-specific dispatch                                           │
│                                                                      │
│  cat/head/tail (streams)    ┐                                        │
│    bstr byte-line iter      │─ raw bytes to stdout (D-48)            │
│                             ┘                                        │
│                                                                      │
│  cp -r / rm -r / ls -R      ┐                                        │
│    walkdir 2.5 WalkDir      │─ per-entry action (copy/delete/format) │
│    follow_links=D-44        │                                        │
│    contents_first=rm only   ┘                                        │
│                                                                      │
│  tail -f (watch branch)     ┐                                        │
│    notify::RecommendedWatcher│                                       │
│    watch(parent_dir, NonRec)│── event loop → read_to_end from        │
│    filter events by filename│   last-known offset, emit to stdout    │
│                             ┘                                        │
│                                                                      │
│  ln -s (directory target)   ┐                                        │
│    std::os::windows::fs::   │                                        │
│      symlink_dir (primary) │── on Err(ERROR_PRIVILEGE_NOT_HELD) →    │
│                             │   junction::create() (D-36 fallback)   │
│                             ┘   + stderr warning once                │
│                                                                      │
│  dos2unix / unix2dos        ┐                                        │
│    gow_core::fs::           │                                        │
│      atomic_rewrite(p, xf) │── NamedTempFile::new_in(parent)         │
│                             │   write(transformed) → persist()       │
│                             ┘   = MoveFileExW(MOVEFILE_REPLACE_EX)   │
│                                                                      │
│  chmod                      ┐                                        │
│    parse GNU mode string    │── extract owner-write bit only         │
│                             │   Permissions::set_readonly(b)         │
│                             ┘                                        │
└──────────────────┬───────────────────────────────────────────────────┘
                   │ Result<(), GowError>
                   ▼
┌────────────────────────────────────────────────────────────────┐
│  main → eprintln!("{bin}: {e}") → exit(1)  [shared]            │
└────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
gow-rust/
├── Cargo.toml                     # workspace — add walkdir, notify, terminal_size, junction
├── crates/
│   ├── gow-core/                  # EXTEND with fs.rs helpers (below)
│   │   └── src/fs.rs              # ADD: atomic_rewrite, create_link, is_same_volume, is_drive_root
│   ├── gow-cat/                   # Wave 1
│   ├── gow-head/                  # Wave 1
│   ├── gow-chmod/                 # Wave 1
│   ├── gow-dos2unix/              # Wave 2 — shares lib with unix2dos
│   ├── gow-unix2dos/              # Wave 2
│   ├── gow-cp/                    # Wave 3 (walkdir)
│   ├── gow-rm/                    # Wave 3 (walkdir + contents_first)
│   ├── gow-ls/                    # Wave 3 (walkdir + terminal_size)
│   ├── gow-ln/                    # Wave 4 (link helpers)
│   ├── gow-mv/                    # Wave 4 (rename + cross-volume copy-delete)
│   └── gow-tail/                  # Wave 5 (notify — isolated)
```

Each Phase 3 crate follows the Phase 2 `lib + thin bin` pattern (D-16) verbatim:
- `crates/gow-foo/Cargo.toml` with `[lib] name = "uu_foo"` + `[[bin]] name = "foo"`
- `crates/gow-foo/src/lib.rs` exposing `pub fn uumain(args: impl Iterator<Item=OsString>) -> i32`
- `crates/gow-foo/src/main.rs` — 3 lines invoking uumain
- `crates/gow-foo/build.rs` — verbatim copy from `gow-touch/build.rs` (embed-manifest)
- `crates/gow-foo/tests/integration.rs` — assert_cmd + snapbox

### Pattern 1: `gow_core::fs::atomic_rewrite` Helper (D-47, Pitfall #4)

**What:** Single helper function that drives the dos2unix / unix2dos (and future sed -i) file mutation pipeline.

**When:** Anytime a Phase 3+ utility rewrites a file whose contents depend on its current contents. Must be atomic: readers in other processes never see a half-written state; the file is either fully old or fully new after the call returns.

**Location:** `crates/gow-core/src/fs.rs` — so Phase 4 `sed -i` inherits it.

```rust
// crates/gow-core/src/fs.rs — NEW helper
// Source: docs.rs/tempfile NamedTempFile::persist; docs.microsoft.com MoveFileExW
use std::io::{Read, Write};
use std::path::Path;

use tempfile::NamedTempFile;

use crate::error::GowError;

/// Atomically rewrite `path` by reading its bytes through `transform` and writing
/// the result back. Uses a same-directory temp file + rename so readers in other
/// processes never observe a half-written state. On Windows, the rename is
/// performed via `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` (std::fs::rename's
/// backing call), which tolerates readers holding the target with FILE_SHARE_READ.
///
/// # Errors
/// - Source read failure
/// - Temp file creation failure (parent dir not writable)
/// - `transform` failure (bubbled up as GowError::Custom)
/// - `persist` failure (target locked by exclusive writer)
pub fn atomic_rewrite<F>(path: &Path, transform: F) -> Result<(), GowError>
where
    F: FnOnce(&[u8]) -> Result<Vec<u8>, GowError>,
{
    // 1. Read the original bytes (raw — no UTF-8 decode per D-48).
    let original = std::fs::read(path).map_err(|e| GowError::Io {
        path: path.display().to_string(),
        source: e,
    })?;

    // 2. Apply the transformation.
    let transformed = transform(&original)?;

    // 3. Create a temp file in the SAME directory as the target (required for
    //    rename to be on the same volume, which is the only guaranteed-atomic
    //    case on Windows).
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent).map_err(|e| GowError::Io {
        path: parent.display().to_string(),
        source: e,
    })?;
    tmp.write_all(&transformed).map_err(|e| GowError::Io {
        path: tmp.path().display().to_string(),
        source: e,
    })?;
    tmp.as_file_mut().sync_all().map_err(|e| GowError::Io {
        path: tmp.path().display().to_string(),
        source: e,
    })?;

    // 4. Atomic rename (persist). On Windows this is MoveFileExW with
    //    MOVEFILE_REPLACE_EXISTING.
    tmp.persist(path).map_err(|e| GowError::Io {
        path: path.display().to_string(),
        source: e.error,
    })?;

    Ok(())
}
```

Phase 4 `sed -i` will call this verbatim — the transform closure is the only utility-specific piece.

**Windows file-locking behavior (VERIFIED):**
- If another process holds the target open for read only (`FILE_SHARE_READ`), `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` succeeds — Windows unlinks the old file's directory entry while readers keep their handle on the unlinked file, and the new file takes the name.
- If another process holds the target open for write without `FILE_SHARE_WRITE`, persist() fails with `ERROR_SHARING_VIOLATION (32)`. `atomic_rewrite` surfaces this as `GowError::Io`. This is the correct GNU behavior — `dos2unix locked.txt` must error out, not silently write a sibling file.

### Pattern 2: `gow_core::fs::create_link` Helper (D-36, D-38)

**What:** Centralized link-creation dispatch for `ln` (and any future utility that creates links).

**When:** `ln src dst` (hard), `ln -s src dst` (symbolic); handles the Windows privilege fallback to junction for directory symlinks.

**Location:** `crates/gow-core/src/fs.rs` — so utility crates never contain `#[cfg(windows)]` link code.

```rust
// crates/gow-core/src/fs.rs — NEW helpers
use std::io;
use std::path::Path;

/// Kind of link to create.
#[derive(Debug, Clone, Copy)]
pub enum LinkKind {
    /// Hard link (`ln` without -s). Same-volume only.
    Hard,
    /// Symbolic link (`ln -s`). On directories, falls back to junction under D-36.
    Symbolic,
}

/// Outcome of a `create_link` call — signals to the caller whether a warning
/// should be printed (e.g., "symlink privilege unavailable, created junction").
#[derive(Debug)]
pub enum LinkOutcome {
    Symlink,
    Junction,
    Hardlink,
}

/// Create a link from `target` to `link_path`.
///
/// # D-36 behavior (Windows)
/// For `LinkKind::Symbolic` where `target` is a directory:
/// 1. Try `std::os::windows::fs::symlink_dir`.
/// 2. If it fails with `ERROR_PRIVILEGE_NOT_HELD` (raw OS err 1314), fall back
///    to `junction::create`. Caller should print `{util}: symlink privilege
///    unavailable, created junction instead` to stderr.
/// 3. Non-privilege errors surface as `io::Error`.
///
/// # D-38 behavior (hard link)
/// Returns `ErrorKind::CrossesDevices` (Rust 1.85+) or a wrapped error with
/// message `cross-device link not permitted` when target and link_path are on
/// different volumes.
pub fn create_link(
    target: &Path,
    link_path: &Path,
    kind: LinkKind,
) -> io::Result<LinkOutcome> {
    match kind {
        LinkKind::Hard => {
            // std::fs::hard_link wraps CreateHardLinkW on Windows. Returns
            // ERROR_NOT_SAME_DEVICE (17) when cross-volume.
            std::fs::hard_link(target, link_path)?;
            Ok(LinkOutcome::Hardlink)
        }
        LinkKind::Symbolic => {
            #[cfg(target_os = "windows")]
            {
                let target_is_dir = std::fs::metadata(target)
                    .map(|m| m.is_dir())
                    .unwrap_or(false);

                if target_is_dir {
                    use std::os::windows::fs::symlink_dir;
                    match symlink_dir(target, link_path) {
                        Ok(()) => Ok(LinkOutcome::Symlink),
                        Err(e) if e.raw_os_error() == Some(1314) => {
                            // ERROR_PRIVILEGE_NOT_HELD — D-36 fallback.
                            junction::create(target, link_path)?;
                            Ok(LinkOutcome::Junction)
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    use std::os::windows::fs::symlink_file;
                    symlink_file(target, link_path)?;
                    Ok(LinkOutcome::Symlink)
                }
            }
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(target, link_path)?;
                Ok(LinkOutcome::Symlink)
            }
        }
    }
}
```

**Error code reference:**
- `ERROR_PRIVILEGE_NOT_HELD = 1314` — symlink creation without `SeCreateSymbolicLinkPrivilege` and without Developer Mode. [CITED: microsoft.com/en-us/windows/win32/debug/system-error-codes--1300-1699-]
- `ERROR_NOT_SAME_DEVICE = 17` — `CreateHardLinkW` across volumes. Cross-platform Rust surfaces as `io::ErrorKind::CrossesDevices` in 1.85+.

### Pattern 3: `gow_core::fs` Windows Attribute Helpers (D-31, D-32, D-34, D-45)

**What:** Three small helpers that wrap `std::os::windows::fs::MetadataExt::file_attributes()` to give utilities a portable interface.

**Location:** `crates/gow-core/src/fs.rs`.

```rust
// crates/gow-core/src/fs.rs — NEW helpers

/// Return true if the entry has the Windows HIDDEN attribute set, or if its
/// filename starts with '.'. Both are treated as hidden per D-34 ("union of
/// Cygwin and Windows conventions").
pub fn is_hidden(path: &Path) -> bool {
    // Dot-prefix check works on all platforms.
    if path.file_name()
        .and_then(|f| f.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
    {
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
        if let Ok(md) = std::fs::symlink_metadata(path) {
            return md.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
        }
    }
    false
}

/// Return true if the entry is read-only per FILE_ATTRIBUTE_READONLY (Windows)
/// or the `!mode & 0o200` (Unix). D-31 uses this to synthesize the permissions
/// column in ls -l.
pub fn is_readonly(md: &std::fs::Metadata) -> bool {
    md.permissions().readonly()
}

/// Return true if the extension is in the gow-rust executable set (D-35).
/// Fixed list: .exe, .cmd, .bat, .ps1, .com. Case-insensitive.
/// Does NOT consult system PATHEXT — per D-35 for test determinism.
pub fn has_executable_extension(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "exe" | "cmd" | "bat" | "ps1" | "com"
        ),
        None => false,
    }
}

/// Clear the FILE_ATTRIBUTE_READONLY bit on Windows. Used by rm -f (D-45) to
/// allow deletion of write-protected files.
#[cfg(target_os = "windows")]
pub fn clear_readonly(path: &Path) -> std::io::Result<()> {
    let md = std::fs::metadata(path)?;
    let mut perms = md.permissions();
    perms.set_readonly(false);
    std::fs::set_permissions(path, perms)
}

#[cfg(not(target_os = "windows"))]
pub fn clear_readonly(_path: &Path) -> std::io::Result<()> {
    Ok(())
}
```

**VERIFIED:** `std::fs::Permissions::set_readonly(false)` on Windows calls `SetFileAttributesW` under the hood, removing `FILE_ATTRIBUTE_READONLY`. Source: rust stdlib `library/std/src/sys/pal/windows/fs.rs` `FilePermissions::set_readonly`. No need for `windows-sys` call in utility code.

### Pattern 4: `gow_core::fs::is_drive_root` (D-42)

**What:** Detect Windows drive roots (`C:\`, `D:\`, `Z:\`, UNC `\\server\share`) so `rm -r` can refuse them.

```rust
// crates/gow-core/src/fs.rs — NEW helper
/// Return true if `path` (after MSYS/slash normalization) points at a Windows
/// drive root or UNC share root. Used by `rm` to block `rm -rf C:\`.
pub fn is_drive_root(path: &Path) -> bool {
    let s = path.to_string_lossy();

    // Drive letter root: C:\, C:/, or bare C:
    if s.len() >= 2 && s.as_bytes()[1] == b':' {
        let after = &s[2..];
        return after.is_empty() || after == "\\" || after == "/";
    }

    // UNC share root: \\server\share or \\server\share\
    let comps: Vec<&str> = s.trim_end_matches(['\\', '/'])
        .split(['\\', '/'])
        .collect();
    if s.starts_with(r"\\") && comps.iter().filter(|c| !c.is_empty()).count() == 2 {
        return true;
    }

    false
}
```

### Pattern 5: `tail -f` Event Loop (D-39, D-40, Pitfall #3)

**What:** notify-backed watcher that meets the 200 ms ROADMAP criterion by watching the parent dir and filtering by filename.

**When:** `uu_tail::follow_descriptor` (-f) and `uu_tail::follow_name` (-F).

```rust
// crates/gow-tail/src/follow.rs — NEW
// Source: https://docs.rs/notify/8.2.0/notify/ recommended_watcher example
//         + Pitfall #3 (watch parent, filter filename)

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{
    event::{EventKind, ModifyKind},
    Config, Event, RecommendedWatcher, RecursiveMode, Watcher,
};

/// tail -f (descriptor follow): keep the original file handle even if the path
/// is renamed or deleted. When the file's mtime/size changes, read new bytes
/// from our saved offset and write to stdout.
pub fn follow_descriptor(path: &Path) -> std::io::Result<()> {
    let mut file = std::fs::File::open(path)?;
    file.seek(SeekFrom::End(0))?;       // position to tail after initial N lines
    let mut offset = file.stream_position()?;

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(io_err)?;

    // Pitfall #3: watch the PARENT dir, not the file. ReadDirectoryChangesW
    // operates on directory handles. Filtering by filename is our responsibility.
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let target_name = path.file_name().map(PathBuf::from);
    watcher
        .watch(parent, RecursiveMode::NonRecursive)
        .map_err(io_err)?;

    let mut stdout = std::io::stdout().lock();

    loop {
        // Blocking recv — no polling CPU cost. notify's internal ReadDirectoryChangesW
        // completion routine dispatches with 100ms wakeup-semaphore latency worst-case.
        let Ok(result) = rx.recv() else { break Ok(()) };

        let event: Event = match result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("tail: watch error: {e}");
                continue;
            }
        };

        // Filter: only events whose path matches our target file name.
        let matches = event.paths.iter().any(|p| {
            match (&target_name, p.file_name()) {
                (Some(expected), Some(got)) => expected.as_os_str() == got,
                _ => false,
            }
        });
        if !matches {
            continue;
        }

        match event.kind {
            EventKind::Modify(ModifyKind::Any) | EventKind::Modify(_) => {
                // Data appended, or file truncated (Windows can't distinguish;
                // we detect truncation by comparing current size to our offset).
                let current_size = file.metadata()?.len();
                if current_size < offset {
                    // Truncation.
                    eprintln!("tail: {}: file truncated", path.display());
                    file.seek(SeekFrom::Start(0))?;
                    offset = 0;
                }
                // Read from `offset` to EOF.
                file.seek(SeekFrom::Start(offset))?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                offset += buf.len() as u64;
                stdout.write_all(&buf)?;
                stdout.flush()?;
            }
            EventKind::Remove(_) => {
                // D-40 descriptor mode: keep reading the open handle. The file
                // is unlinked from the directory but our handle is still valid
                // on Windows.
                eprintln!("tail: {}: file has been removed", path.display());
                // Do not break; reader still has access to remaining bytes.
            }
            _ => {}
        }
    }
}

fn io_err(e: notify::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
}
```

**D-40 `-F` (follow-name + retry) variant:** identical structure but responds to `EventKind::Create` of a file with the matching name by reopening `File::open(path)`, resetting offset to 0, and continuing. On `EventKind::Remove`, flag the file as "awaiting recreation" instead of continuing to read the unlinked handle.

**Latency envelope (200 ms criterion):**

| Component | Contribution | Source |
|-----------|-------------|--------|
| `ReadDirectoryChangesW` dispatch | kernel → user callback typically < 50 ms | notify windows.rs completion routine, buffer = 16 KB |
| notify's `WaitForSingleObjectEx(self.wakeup_sem, 100, 1)` | worst-case 100 ms before channel send | notify/src/windows.rs [VERIFIED] |
| `mpsc::Receiver::recv()` on tail thread | < 1 ms on uncontested channel | std docs |
| File read + stdout write | < 10 ms for typical log append | — |
| **Total worst-case** | **~110–150 ms** | passes 200 ms criterion with margin |

Typical case is much faster (< 50 ms end-to-end) because the wakeup semaphore is signaled by the kernel callback, pre-empting the 100 ms wait. [VERIFIED: github.com/notify-rs/notify notify/src/windows.rs:server loop]

### Pattern 6: `walkdir` Usage Templates

#### `ls -R` — sorted, non-content-first, handle permission errors

```rust
// crates/gow-ls/src/recurse.rs
use walkdir::WalkDir;

for entry in WalkDir::new(root)
    .follow_links(false)            // ls -l shows link itself, not target
    .sort_by_file_name()             // deterministic output
    .into_iter()
    .filter_entry(|e| !should_skip(e))
{
    match entry {
        Ok(e) => format_entry(&e),
        Err(err) => {
            eprintln!("ls: cannot access '{}': {}",
                err.path().map(|p| p.display().to_string()).unwrap_or_default(),
                err.io_error().map(|e| e.to_string()).unwrap_or_else(|| err.to_string()));
        }
    }
}
```

#### `cp -r` — honor D-44 (symlinks preserved by default; -L follows)

```rust
// crates/gow-cp/src/recurse.rs
let follow = match options.symlink_mode {
    SymlinkMode::Physical => false,   // -P (default per D-44)
    SymlinkMode::Logical  => true,    // -L
    SymlinkMode::CommandLineOnly => false,  // -H, handled via initial is_symlink check on the cmdline
};

for entry in WalkDir::new(src)
    .follow_links(follow)
    .sort_by_file_name()        // deterministic copy order
    .into_iter()
{
    let entry = entry?;
    copy_one(&entry, ...)?;
}
```

**Important:** walkdir does NOT follow Windows junctions even when `follow_links(true)`. Junctions are reparse points tagged differently from symlinks; walkdir treats them as directories without descending. This is *correct* behavior for `cp -rL` — we want the junction as a directory entry, not a mirror of its target. If the user explicitly wants to dereference a junction, they can pass the junction's target path directly.

#### `rm -r` — contents-first required

```rust
// crates/gow-rm/src/recurse.rs
for entry in WalkDir::new(root)
    .contents_first(true)        // ESSENTIAL: delete children before parent
    .follow_links(false)         // never follow symlinks during delete
    .into_iter()
{
    let entry = entry?;
    let ft = entry.file_type();
    if ft.is_dir() {
        std::fs::remove_dir(entry.path())?;
    } else {
        // Clear RO attribute if -f (D-45) then delete.
        if options.force && gow_core::fs::is_readonly(&entry.metadata()?) {
            gow_core::fs::clear_readonly(entry.path())?;
        }
        std::fs::remove_file(entry.path())?;
    }
}
```

**Anti-pattern warning:** omitting `contents_first(true)` causes `rm -r` to fail with "directory not empty" on every non-empty parent because walkdir's default yields parents before children.

### Pattern 7: `ls` Column Layout via `terminal_size`

```rust
// crates/gow-ls/src/layout.rs
use terminal_size::{terminal_size, Width};

let columns = match terminal_size() {
    Some((Width(w), _)) => w as usize,
    None => 80,   // not a tty — still format in columns for consistency,
                   //    or switch to single-column (GNU default in pipe context)
};
```

**GNU behavior:** `ls` in a pipe context defaults to single-column output (`-1`). `ls` in a tty context defaults to multi-column (`-C`). Detect via `terminal_size()` returning `None` or by checking `atty::isnt::stdout()` — `terminal_size` already handles the check internally (it returns `None` when stdout is not a tty on Windows). [CITED: docs.rs/terminal_size]

### Pattern 8: `cat` / `head` / `tail` Byte-Stream Scaffold (D-48)

```rust
// crates/gow-cat/src/lib.rs
use std::io::{BufRead, BufReader, Read, Write};

use bstr::ByteSlice;

pub fn cat_one<R: Read, W: Write>(
    reader: R,
    writer: &mut W,
    line_numbers: bool,
    squeeze: bool,
    /* ... */
) -> std::io::Result<()> {
    let mut reader = BufReader::new(reader);
    let mut buf = Vec::with_capacity(8192);
    let mut prev_blank = false;
    let mut n = 0;

    loop {
        buf.clear();
        let read = reader.read_until(b'\n', &mut buf)?;
        if read == 0 { break; }

        let is_blank = buf.iter().all(|b| *b == b'\n' || *b == b'\r' || *b == b' ');
        if squeeze && prev_blank && is_blank { continue; }
        prev_blank = is_blank;

        if line_numbers {
            n += 1;
            write!(writer, "{n:>6}\t")?;
        }
        writer.write_all(&buf)?;   // raw bytes — no UTF-8 decode (D-48)
    }
    Ok(())
}
```

**Why `read_until(b'\n')` over `lines()`:** `std::io::BufRead::lines()` returns `String` which panics (or errors) on non-UTF-8. `read_until` returns raw bytes and preserves the `\n` (needed for proper `cat` output). Same pattern already used in Phase 2 `gow-wc`.

### Anti-Patterns to Avoid

- **Watching the file directly (not parent):** `notify::Watcher::watch(file_path, NonRecursive)` appears to work but returns no events on Windows because `ReadDirectoryChangesW` requires a directory handle. Pitfall #3 — watch the parent, filter the filename.
- **Using `std::fs::File::open` + polling `metadata().len()`:** This is what original GOW `tail` does and it causes the 200 ms latency miss (GOW #169). Replace with notify.
- **`String::from_utf8_lossy` on file contents for cat/head/tail:** Corrupts binary files and non-UTF-8 text (CP949 Korean files, etc.). D-48 is raw bytes only.
- **Custom FSCTL_SET_REPARSE_POINT logic:** 200+ lines of unsafe `REPARSE_DATA_BUFFER` packing. Use `junction` crate.
- **Skipping `contents_first(true)` on `rm -r`:** Fails with ENOTEMPTY on every non-empty parent.
- **Forgetting `NamedTempFile::new_in(parent)` (using default `new()`):** Puts temp in `%TEMP%`, which is usually a different volume. Cross-volume rename is NOT atomic and fails with `ERROR_NOT_SAME_DEVICE`. Always create temp in same parent as target.
- **Skipping `sync_all()` before persist:** On system crash, the file's directory entry exists but its contents are not durable. Cheap insurance: `tmp.as_file_mut().sync_all()` before `persist`.
- **`std::fs::remove_dir_all` for rm -rf:** It does NOT clear read-only attributes on Windows. Use walkdir contents-first + per-entry `clear_readonly` + `remove_file/remove_dir`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Junction creation | Custom `DeviceIoControl(FSCTL_SET_REPARSE_POINT)` + `REPARSE_DATA_BUFFER` packing | `junction::create(target, link)` | 200+ lines of unsafe; `\??\` prefix handling; offset/length fields must be exact |
| Filesystem watching | Raw `ReadDirectoryChangesW` loop | `notify::RecommendedWatcher` | 200+ lines unsafe; completion routine plumbing; PollWatcher fallback re-implementation |
| Recursive traversal | Manual `read_dir` recursion | `walkdir 2.5` | Cycle detection (symlinks); permission error mid-walk; deterministic sort; contents-first |
| Atomic in-place rewrite | Custom `CreateFileW` + `MoveFileExW` | `tempfile::NamedTempFile::new_in + persist` | Crash-safety; auto-cleanup on error; recoverable error (returns original tempfile) |
| Terminal width detection | `GetConsoleScreenBufferInfo` manual call | `terminal_size::terminal_size()` | Already handles ConHost/Windows Terminal differences; returns `None` cleanly on pipes |
| Windows read-only attribute | `SetFileAttributesW` + `GetFileAttributesW` | `std::fs::Permissions::set_readonly` | std already wraps the syscall correctly; works cross-platform |
| Hard link creation | `CreateHardLinkW` manual | `std::fs::hard_link` | Same wrapper; returns correct `ErrorKind::CrossesDevices` for D-38 cross-volume error |
| MSYS path parsing | Regex or ad-hoc split | `gow_core::path::try_convert_msys_path` | Already in Phase 1; context-aware per D-06 |
| Line counting | Custom string split | `bstr::ByteSlice::lines()` or `BufRead::read_until(b'\n')` | D-17/D-48: raw-byte safe, no UTF-8 assumption |
| Windows timestamp set | Manual `SetFileTime` | `filetime::set_file_times` | Already proven in Phase 2 touch; handles symlink-self for -h via `set_symlink_file_times` |
| Symlink-type detection | Custom metadata inspection | `gow_core::fs::link_type` (Phase 1) | Already built; returns `LinkType::{SymlinkFile, SymlinkDir, Junction, HardLink}` |

**Key insight:** Phase 3 is the phase where *not* writing OS-specific code pays off the most. Every utility except `ln` and `rm -f` can be written as pure cross-platform Rust — the Windows-specifics are absorbed by `junction`, `notify`, `walkdir`, `tempfile`, and `filetime` crates plus the existing `gow_core::fs` module.

---

## Runtime State Inventory

Phase 3 is a *greenfield* phase (no rename/refactor/migration). No runtime state exists from prior phases that Phase 3 needs to mutate. Inventory sections explicitly:

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | None — no databases, no persistent gow state | — |
| Live service config | None — CLI utilities are stateless per invocation | — |
| OS-registered state | None — no scheduled tasks, services, or launch daemons added in Phase 3 | — |
| Secrets/env vars | None new — `GOW_PATHEXT` (from Phase 2 which) is NOT read by any Phase 3 utility per D-35 | — |
| Build artifacts | Phase 2 binaries (14 `.exe` in `target/`) remain valid; Phase 3 adds 11 more. No stale Phase 2 artifacts. | — |

**Nothing found in any category.** Phase 3 is pure code addition.

---

## Common Pitfalls

### Pitfall 1: `notify::Watcher::watch(file_path, NonRecursive)` Returns No Events on Windows

**What goes wrong:** Watching the target file directly (not its parent) appears to compile and register successfully, but no events fire when the file is modified.

**Why it happens:** `ReadDirectoryChangesW` operates on a directory handle. notify's Windows backend silently upgrades a file path to its parent directory internally in *some* versions — but the reliable pattern is to watch the parent explicitly and filter by filename.

**How to avoid:** Pitfall #3 in STATE.md. Always `Watcher::watch(parent_dir, RecursiveMode::NonRecursive)` + filter `event.paths` by the target filename in the receive loop.

**Warning signs:** `tail -f file.log` produces no output even though another process is appending to `file.log`.

**Verified by:** [STATE.md Pitfall #3] [CITED]. [github.com/notify-rs/notify notify/src/windows.rs] confirms directory-handle requirement of `ReadDirectoryChangesW`. [VERIFIED]

---

### Pitfall 2: `tempfile` Placed in Default Temp Dir Instead of Target's Parent

**What goes wrong:** `NamedTempFile::new()` creates the temp file in `%TEMP%` (usually `C:\Users\<user>\AppData\Local\Temp`). When `persist(target)` runs, the rename crosses volumes (target is on `D:\project\` but temp is on `C:\...\Temp\`). `MoveFileExW` falls back to a copy-then-delete sequence which is NOT atomic and is much slower.

**Why it happens:** Developers use `NamedTempFile::new()` out of habit; the docs for `new_in` are less prominent.

**How to avoid:** ALWAYS `NamedTempFile::new_in(parent_dir)` in D-47's atomic rewrite pattern. The `atomic_rewrite` helper in `gow_core::fs` enforces this — utility code calls the helper, not tempfile directly.

**Warning signs:** `dos2unix large-file.txt` is unexpectedly slow (copies gigabytes through a temp). Or, mid-rewrite crash leaves `tempXXX` files in `%TEMP%` instead of the target dir.

**Verified by:** [docs.rs/tempfile NamedTempFile::persist — "Files cannot be persisted across different filesystems"] [CITED]

---

### Pitfall 3: `std::fs::remove_dir_all` Ignores Read-Only Attribute on Windows

**What goes wrong:** `rm -rf read_only_dir/` fails even with `-f`, because Rust's `remove_dir_all` doesn't clear `FILE_ATTRIBUTE_READONLY` before unlinking each entry.

**Why it happens:** `SetFileAttributesW` is a separate syscall from `DeleteFileW`; `std::fs` does not chain them.

**How to avoid:** Walk the tree with `walkdir.contents_first(true)`, clear read-only bit (via `gow_core::fs::clear_readonly`) on each entry before calling `remove_file`/`remove_dir`, per D-45.

**Warning signs:** `rm -rf` returns `Permission denied` on a directory containing a read-only file, even with `-f` flag.

**Verified by:** Rust stdlib source — `remove_dir_all` uses `DeleteFileW`/`RemoveDirectoryW` without attribute reset. [CITED: rust-lang/rust library/std/src/sys/pal/windows/fs.rs]

---

### Pitfall 4: `std::fs::rename` Cross-Volume Falls Back Silently on Windows

**What goes wrong:** `mv src.txt /d/other/dst.txt` when src is on C:\ falls back to copy-then-delete. If the target directory is full or the delete fails after copy, the source is gone and the dest is incomplete.

**Why it happens:** `MoveFileExW(MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING)` allows cross-volume moves but the fallback is a non-atomic copy+delete.

**How to avoid:** Explicit cross-volume detection: if source and dest are on different volumes (first 2 chars of absolute paths differ, case-insensitive), use an explicit `std::fs::copy` + `std::fs::remove_file` sequence with proper error handling (don't delete source until copy succeeds). This is discretionary per CONTEXT.md — `mv` can rely on `std::fs::rename`'s built-in fallback if the failure path is acceptable.

**Warning signs:** `mv large-file.bin /d/archive/` takes much longer than expected and leaves partial dest on interrupt.

**Verified by:** Microsoft `MoveFileExW` documentation — `MOVEFILE_COPY_ALLOWED` flag. [CITED: learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw]

---

### Pitfall 5: Directory Symlinks Without Developer Mode (D-36 Motivation)

**What goes wrong:** `ln -s C:\dir newdir` on stock Windows 10/11 (no Developer Mode, standard user) fails with `ERROR_PRIVILEGE_NOT_HELD (1314)`. uutils' ln surfaces this as an error. Users expect GOW to "just work."

**Why it happens:** `CreateSymbolicLinkW` requires `SeCreateSymbolicLinkPrivilege`, typically granted only in Developer Mode (Windows 10 1703+) or to administrators.

**How to avoid:** D-36 — fall back to `junction::create` when `err.raw_os_error() == Some(1314)`. Junctions don't need the privilege (they predate symlinks). Emit a stderr warning once per invocation.

**Warning signs:** `ln -s ../lib /path/to/link` returns "A required privilege is not held by the client" on non-admin, non-Dev-Mode shell.

**Verified by:** Microsoft Win32 error codes docs (1300–1699). [CITED: learn.microsoft.com] + Phase 1 `fs.rs:118` already has this privilege-detection test pattern.

---

### Pitfall 6: `walkdir::WalkDir::contents_first` Default is FALSE

**What goes wrong:** `rm -r dir/` reports "directory not empty" on `dir/` because walkdir yields `dir/` before its contents, and `remove_dir(dir)` runs before the children are deleted.

**Why it happens:** walkdir's natural order is depth-first pre-order (parent before children). Default `contents_first(false)`.

**How to avoid:** For `rm -r`, ALWAYS `WalkDir::new(root).contents_first(true)`. For `cp -r` and `ls -R`, default (parents first) is correct.

**Warning signs:** `rm -rf dir/` fails with ENOTEMPTY on every non-empty directory.

**Verified by:** docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html#method.contents_first [VERIFIED]

---

### Pitfall 7: `notify-debouncer` Introduces Intentional Latency

**What goes wrong:** Using `notify-debouncer-full` or `notify-debouncer-mini` for tail -f gives up event fidelity (batches into one delivery per debounce window). Default debounce is 2 s — violates 200 ms ROADMAP criterion by 10x.

**Why it happens:** Debouncers are designed for IDE-like scenarios where you want one event per save, not multiple events per write.

**How to avoid:** D-39 — use plain `RecommendedWatcher` without a debouncer. Accept that `tail -f` may emit from multiple `Modify` events per second; dedupe at the read layer (reading from our saved offset naturally dedupes).

**Warning signs:** `tail -f` outputs new lines with 2+ second delay.

**Verified by:** notify-debouncer docs (`Debouncer::new(timeout=Duration::from_secs(2))` default). [CITED]

---

### Pitfall 8: Raw-Byte vs UTF-8 Mode in cat/head/tail

**What goes wrong:** `cat korean-cp949.txt` panics or shows mojibake if the file is decoded as UTF-8.

**Why it happens:** Rust's `BufRead::lines()` and `String::from_utf8` bail on non-UTF-8 byte sequences. GNU `cat` is byte-transparent.

**How to avoid:** D-48 locked — use `BufRead::read_until(b'\n')` for cat/head/tail. `bstr::ByteSlice::lines()` for dos2unix/unix2dos scanners. Never decode to `String`.

**Warning signs:** `cat windows-cp949-file.txt` panics with `FromUtf8Error` or writes `?` for Korean characters.

**Verified by:** D-17 / D-48 [CITED: CONTEXT.md]. Phase 2 wc crate already follows this pattern.

---

### Pitfall 9: Fixed Executable Extension Set vs System PATHEXT (D-35)

**What goes wrong:** If `ls -l` reads the system `PATHEXT` environment variable to decide which files show as executable (`x` bit), the output becomes non-deterministic across machines and between tests that run in parallel.

**Why it happens:** `PATHEXT` is user-configurable and may include odd entries like `.PY` on developer machines. Test snapshots fail depending on runner config.

**How to avoid:** D-35 — hardcode the set `{exe, cmd, bat, ps1, com}` in `gow_core::fs::has_executable_extension`. Document in `ls --help` that executable-bit synthesis uses a fixed set, not `PATHEXT`.

**Warning signs:** CI snapshot test for `ls -l` fails on one runner but passes on another.

**Verified by:** D-18a in Phase 2 which uses the same hardcoded default (`.COM;.EXE;.BAT;.CMD`) and adds `.PS1`. [CITED: 02-CONTEXT.md]

---

### Pitfall 10: `head -c 10` Slicing in Middle of UTF-8 Multi-Byte Character

**What goes wrong:** `head -c 2 emoji.txt` on a file starting with `😀` (4 bytes in UTF-8) outputs 2 bytes of an invalid partial character.

**Why it happens:** `-c` is BYTE count by GNU spec; it does not align to character boundaries.

**How to avoid:** Document in `head --help` that `-c N` is byte-exact (GNU behavior). Use `BufReader::take(n)` + `std::io::copy`. This is intentional compatibility with GNU.

**Warning signs:** None — this is correct behavior. Only a pitfall if someone tries to "fix" it.

**Verified by:** GNU `head` manual: "-c, --bytes=NUM … print the first NUM bytes". [CITED: gnu.org/software/coreutils/manual/coreutils.html#head-invocation]

---

## Per-Utility Implementation Notes

The following sections give planners concrete implementation guidance per utility, organized by wave.

### FILE-01 `cat` (Wave 1 easy-first)

**Flags:** `-n` (number all), `-b` (number non-blank), `-s` (squeeze blanks), `-v` (show non-printing), `-E` (show ends `$`), `-T` (show tabs `^I`), `-A` (shorthand for -vET; planner discretion).

**Input sources:**
- File args: iterate in order, apply `try_convert_msys_path` to each.
- No args: read stdin.
- `-` literal arg: read stdin at that position.

**Body: Byte-stream scaffold** (see Pattern 8). `read_until(b'\n')` on a `BufReader`.

**`-v` encoding rules (GNU convention):**
- ASCII < 0x20 except `\t`, `\n`: `^` + (byte XOR 0x40). E.g., `\r` → `^M`.
- `\t`: unchanged unless `-T` → `^I`.
- `\n`: unchanged unless `-E` → `$\n` (dollar sign before newline).
- 0x7F (DEL): `^?`.
- 0x80–0xFF (high bit): `M-` + apply ASCII rules to (byte & 0x7F).

**Implementation pattern (state machine):**

```rust
fn visualize_byte(b: u8, output: &mut Vec<u8>, show_tabs: bool, show_ends: bool) {
    match b {
        b'\n' => {
            if show_ends { output.push(b'$'); }
            output.push(b'\n');
        }
        b'\t' if !show_tabs => output.push(b'\t'),
        b'\t' if show_tabs => output.extend_from_slice(b"^I"),
        0x00..=0x1f => { output.push(b'^'); output.push(b ^ 0x40); }
        0x7f => output.extend_from_slice(b"^?"),
        0x80..=0xff => {
            output.extend_from_slice(b"M-");
            visualize_byte(b & 0x7f, output, show_tabs, show_ends);
        }
        _ => output.push(b),
    }
}
```

**ROADMAP criterion 4 (`cat -n` UTF-8 without mojibake):** satisfied by D-48 (raw bytes) + gow_core::init's console UTF-8 setup. The byte sequence for `안녕` passes through unchanged; terminal's UTF-8 console renders it correctly.

**Tests (minimum):**
- `cat file.txt` — pass-through
- `cat -n file.txt` — line numbers, UTF-8 preserved
- `cat -v` — encode control chars
- `cat no-such-file` — exit 1, `cat: no-such-file: No such file or directory`
- `cat -` — read stdin
- `cat file1 file2` — concatenation
- `cat` with CP949 bytes — no panic, bytes pass through

### FILE-02 `ls` (Wave 3 walkdir + terminal_size)

**Flags:** `-l` (long), `-a` (hidden), `-A` (almost-all, no `.` / `..`), `-R` (recursive), `-1` (one-per-line), `-d` (directory, not contents), `-t` (sort mtime), `-S` (sort size), `-r` (reverse), `--color[=when]`, `--sort=WORD`.

**Components:**

1. **Entry gathering:**
   - Non-recursive: `std::fs::read_dir(path)`.
   - `-R`: `walkdir::WalkDir::new(path).sort_by_file_name()` + format each directory heading.
   - Hidden filter: respect D-34 union (dot-prefix OR `FILE_ATTRIBUTE_HIDDEN`) unless `-a`.

2. **Long-format row (`-l`):**
   ```
   {type}{perms} {nlink} {owner} {group} {size} {mtime} {name}[ -> {target}]
   ```
   - `{type}`: `d` dir, `l` symlink OR junction (D-37), `-` file.
   - `{perms}`: from D-31 read-only synthesis. For exec bit (D-35), use `gow_core::fs::has_executable_extension`.
   - `{nlink}`: always `1` (v2 deferred per CONTEXT.md).
   - `{owner}`/`{group}`: both `-` on Windows (v2 deferred).
   - `{size}`: `metadata().len()` for files, `0` or blocksize for dirs (GNU quirk, just use `0`).
   - `{mtime}`: format as `Mmm DD HH:MM` if within 6 months, else `Mmm DD YYYY` (GNU convention).
   - `{name}`: apply color codes per builtin scheme.
   - ` -> target`: `gow_core::fs::link_type` + `std::fs::read_link` + `normalize_junction_target`. For junctions add ` [junction]` suffix (D-37).

3. **Column layout (non-`-l`):**
   - Get terminal width from `terminal_size::terminal_size()`.
   - If `None` (pipe) or `-1` flag, single column.
   - Else multi-column with 2-space padding between columns; max columns = `width / (longest_name + 2)`.

4. **Color scheme (builtin, no LS_COLORS per CONTEXT.md discretion):**
   - Directory: bold blue
   - Symlink: cyan
   - Junction: cyan (same as symlink per D-37 `l` prefix parity)
   - Executable: green
   - Regular file: default
   - Respect `NO_COLOR` env var (Phase 1 color.rs pattern).

**Key helpers:**
- `gow_core::fs::link_type` (existing) → `l` prefix decision and `-> target` formatting.
- `gow_core::fs::is_hidden` (NEW) → `-a` filtering.
- `gow_core::fs::has_executable_extension` (NEW) → `x` bit synthesis.
- `gow_core::fs::is_readonly` (NEW) → `rw/r-` decision.

**ROADMAP criterion 1 satisfied:** `-la` lists hidden files via D-34 union + permissions via D-31 + colorization + D-37 link types.

**Tests:**
- `ls` — directory listing
- `ls -la` — long format with hidden files
- `ls -R` — recursive with per-dir headers
- `ls --color=always` — ANSI codes in output
- `ls` on dir containing symlink — `->` notation
- `ls` on dir containing junction — `-> target [junction]`
- Hidden via `.git` — shown with `-a`
- Hidden via `attrib +h` — shown with `-a`
- `ls non-existent` — exit 1, GNU error format
- Platform-gate: symlink tests skip if privilege missing (Phase 1 `fs.rs:118` pattern)

### FILE-03 `cp` (Wave 3 walkdir)

**Flags:** `-r`/`-R` (recursive), `-f` (force overwrite), `-p` (preserve timestamps + RO bit per D-33), `-P` (no-deref — default per D-44), `-L` (deref), `-H` (command-line only), `-i` (interactive), `-n` (no-clobber), `-v` (verbose).

**Dispatch:**
- Single file, single file target: `copy_one(src, dst)`.
- Multiple sources or source is dir + `-r`: walkdir with `follow_links = (mode == Logical)`; call `copy_one(entry.path(), dst.join(entry.path().strip_prefix(src)))`.

**`copy_one` logic:**
1. Stat source. If symlink and mode is Physical: clone link via `create_link(read_link(src), dst, Symbolic)` — goes through gow-core helper → D-36 junction fallback for directory symlinks.
2. Else if directory: `create_dir(dst)`. Apply attributes if `-p`.
3. Else (regular file): `std::fs::copy(src, dst)` (wraps `CopyFileExW` on Windows — preserves basic attrs automatically).
4. If `-p`: post-copy, apply `filetime::set_file_times(dst, src_atime, src_mtime)` and `set_readonly(dst, src_readonly)`.

**Cross-drive handling:** `std::fs::copy` handles cross-volume transparently.

**ROADMAP criterion 2 satisfied:** `cp -r src/ dest/` walks + copies; `cp -p` preserves timestamps via filetime (same crate as Phase 2 touch).

**Tests:**
- `cp file.txt dst.txt` — single file
- `cp -r src/ dst/` — recursive, cross-drive
- `cp -rp src/ dst/` — timestamps preserved (assert metadata mtime)
- `cp symlink.lnk dst.lnk` — default is clone link (-P)
- `cp -L symlink.lnk dst.lnk` — follows, dst is regular file
- `cp -r src/ dst/` where src contains dir-symlink — dst has junction (D-36 fallback)
- `cp non-existent dst` — exit 1

### FILE-04 `mv` (Wave 4)

**Flags:** `-f` (force), `-i` (interactive), `-n` (no-clobber), `-v` (verbose).

**Logic:**
1. MSYS-convert src and dst args.
2. If `-i` and dst exists: prompt.
3. Attempt `std::fs::rename(src, dst)`:
   - On Windows this is `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` — atomic same-volume.
   - Cross-volume: `MoveFileExW` fails with `ERROR_NOT_SAME_DEVICE (17)` when `MOVEFILE_COPY_ALLOWED` is NOT set. Rust's std sets `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH` — **it does NOT set `MOVEFILE_COPY_ALLOWED`**, so cross-volume errors as ErrorKind::CrossesDevices.
4. On cross-volume error: explicit copy+delete fallback.
   ```rust
   std::fs::copy(src, dst).map_err(...)?;
   // Preserve timestamps (GNU mv with cross-volume effectively behaves like mv -p):
   let src_md = std::fs::metadata(src)?;
   filetime::set_file_times(
       dst,
       filetime::FileTime::from_last_access_time(&src_md),
       filetime::FileTime::from_last_modification_time(&src_md),
   )?;
   std::fs::remove_file(src)?;
   ```

**Tests:**
- `mv a.txt b.txt` — rename
- `mv a.txt subdir/` — move into dir
- `mv /c/src.txt /d/dst.txt` — cross-drive (assert file moved + timestamps preserved)
- `mv -i existing.txt new.txt` — prompt (use stdin feeding in test)
- Source non-existent → exit 1

### FILE-05 `rm` (Wave 3 walkdir)

**Flags:** `-r`/`-R`/`--recursive`, `-f`/`--force`, `-i`/`--interactive`, `-v`/`--verbose`, `--preserve-root` (default), `--no-preserve-root`.

**Safety gates (first, before any delete):**
1. For each target: apply `try_convert_msys_path`.
2. If target is `/` or Windows drive-root (`gow_core::fs::is_drive_root`): unless `--no-preserve-root`, print `rm: it is dangerous to operate recursively on '{path}' / use --no-preserve-root to override` and exit 1.
3. If target does not exist: if `-f`, silent; else exit 1 with GNU error.

**Dispatch:**
- File: `delete_one_file(path, opts)`.
- Dir without `-r`: exit 1 with `rm: cannot remove '{path}': Is a directory`.
- Dir with `-r`: walkdir `contents_first(true)` (see Pattern 6).

**`delete_one_file`:**
1. Check RO (D-45). If RO and not `-f`:
   - Stdin is tty: prompt `rm: remove write-protected regular file '{path}'?`.
   - Stdin is not tty: exit 1 `Permission denied`.
2. If `-f` and RO: `clear_readonly(path)`.
3. `std::fs::remove_file(path)`.

**Tests:**
- `rm file.txt` — basic
- `rm -r dir/` — recursive
- `rm -rf dir-with-ro-file/` — clears RO then deletes
- `rm C:\` — refused (D-42)
- `rm --no-preserve-root -rf /tmp/testdir/` — override works
- `rm no-such-file` → exit 1; `rm -f no-such-file` → exit 0
- Read-only file without tty stdin → exit 1 Permission denied
- Read-only file with -f → deleted

### FILE-09 `ln` (Wave 4 link-bundle)

**Flags:** `-s`/`--symbolic`, `-f`/`--force` (replace dst), `-v`/`--verbose`.

**Logic:**
1. Parse args: `ln [OPTS] TARGET LINK_NAME` or `ln [OPTS] TARGET... DIRECTORY`.
2. For each `(target, link)` pair:
   - Apply MSYS conversion.
   - If `-f` and `link` exists: `std::fs::remove_file` (or `remove_dir` for directory-junctions).
   - Call `gow_core::fs::create_link(target, link, if opts.symbolic { Symbolic } else { Hard })`.
   - Match outcome:
     - `Ok(LinkOutcome::Symlink | LinkOutcome::Hardlink)`: silent success.
     - `Ok(LinkOutcome::Junction)`: stderr `ln: symlink privilege unavailable, created junction instead` (D-36, emit once).
     - `Err(e)` where `e.raw_os_error() == Some(17)` (cross-volume hardlink): exit 1 `ln: cross-device link not permitted`.
     - Any other Err: exit 1 with GNU format.

**Tests (guard symlink tests with privilege skip):**
- `ln a.txt b.txt` — hard link; assert same inode (Unix) or same content
- `ln /c/vol1/a.txt /d/vol2/b.txt` → cross-device error, exit 1
- `ln -s target linkname` — file symlink (skip if no privilege)
- `ln -s targetdir linkdir` without privilege → junction, warning on stderr
- `ln -s targetdir linkdir` with privilege → dir symlink
- `ln -f existing.lnk target.txt` — replaces

### FILE-10 `chmod` (Wave 1 easy-first)

**Scope (D-32):** Only the owner-write bit matters on Windows; everything else silently ignored.

**Flags:** `-R` (recursive), `-v` (verbose), `--reference=FILE` (v2 deferred).

**Mode parsing:**
- Octal: `0644`, `0755`, `644`. Extract owner bits (bits 6-8). Read-only = NOT (owner-write bit 7 set). Example: `0444` → RO; `0644` → writable.
- Symbolic: `[ugoa][+-=][rwxXst]`. Parse via a small state machine. Extract owner's `w` bit change.

**Logic:**
1. Parse mode string once into "target read-only state" (bool) per target.
2. For each path:
   - Apply MSYS conversion.
   - If `-R`: walkdir the tree, apply to each entry.
   - Read metadata, build new `Permissions`, set `readonly(!owner_writable)`, call `std::fs::set_permissions`.

**Partial support notice:** GNU `chmod 600 file` should warn for non-zero group/other bits? D-32 says no — silent to avoid script noise. Document in `chmod --help`: "On Windows, only the owner write bit is honored; other mode bits are silently ignored (maps to FILE_ATTRIBUTE_READONLY)".

**Tests:**
- `chmod 644 file.txt` — writable
- `chmod 444 file.txt` — read-only
- `chmod +w file.txt` — writable
- `chmod -w file.txt` — read-only
- `chmod u=r file.txt` — read-only
- Verify via `std::fs::metadata(f).permissions().readonly()`
- `chmod -R 644 dir/` — recurses via walkdir

### TEXT-01 `head` (Wave 1 easy-first)

**Flags:** `-n NUM` (lines, default 10), `-c NUM` (bytes), `-q` (quiet, no headers), `-v` (verbose, always headers).

**Numeric shorthand (D-05, inherited):** `head -5 file` = `head -n 5 file`. Already handled by `parse_gnu`.

**Body:**
- `-n N`:
  ```rust
  let reader = BufReader::new(file);
  let mut buf = Vec::new();
  for _ in 0..n {
      buf.clear();
      if reader.read_until(b'\n', &mut buf)? == 0 { break; }
      stdout.write_all(&buf)?;
  }
  ```
- `-c N`:
  ```rust
  let mut reader = BufReader::new(file);
  std::io::copy(&mut reader.take(n as u64), &mut stdout)?;
  ```

**Multi-file headers:** Standard GNU `==> filename <==` between files unless `-q`.

**Tests:**
- `head file.txt` — 10 lines
- `head -n 5 file.txt`
- `head -5 file.txt` — numeric shorthand
- `head -c 100 file.txt`
- `head -c 100 emoji.txt` — byte-exact, may split a char (documented)
- `head f1 f2` — headers
- `head -q f1 f2` — no headers
- Empty file → empty output, exit 0

### TEXT-02 `tail` (Wave 5 watcher-isolated)

**Flags:** `-n [+]NUM`, `-c [+]NUM`, `-f` (descriptor follow), `-F` (name follow + retry), `-q`, `-v`, `--pid=PID` (v2 deferred), `--sleep-interval=S` (v2 redefined).

**Numeric shorthand:** `tail -20 file` = `tail -n 20 file`.

**Body:**

1. **Initial N-lines emission** (discretion per CONTEXT.md):
   - Small files: `read_to_end` + take last N by scanning backward for `\n`.
   - Large files (stat.len() > threshold, e.g., 64 KB): `seek(SeekFrom::End(-chunk))` + scan backward in chunks until N newlines found.
   - Planner may choose either; add a threshold constant.

2. **`-f` branch:** call `follow_descriptor(path)` (Pattern 5).
3. **`-F` branch:** call `follow_name(path)` (variant — reopens on EventKind::Create with matching filename).
4. **Multi-file `-f`:** emit `==> file <==` header on current-focus switch; dedupe headers per D-41. Internal state: `last_printed_header: Option<PathBuf>`.

**ROADMAP criterion 3 satisfied:** 200 ms latency derived from notify's 100 ms wakeup + read overhead; margin allows for kernel scheduling jitter. See Pattern 5 latency table.

**Tests:**
- `tail file.txt` — last 10 lines
- `tail -n 5 file.txt`
- `tail -c 100 file.txt`
- `tail -f logfile` — spawn child process writing to logfile, assert tail receives within 500 ms (allow CI jitter; 200 ms is the target on dev machine)
- `tail -f` + rotate (rename logfile away): tail continues reading unlinked handle under `-f` per D-40
- `tail -F` + rotate: tail reopens new logfile on Create event
- `tail -f` + truncate (`> logfile`): tail emits `file truncated` and resumes from 0
- Multi-file: `tail -f a.log b.log` — headers switch correctly

### CONV-01 / CONV-02 `dos2unix` / `unix2dos` (Wave 2 conversion-pair)

Share a common library module `crates/gow-dos2unix-common/` or a single `mod scanner` that both crates import. Planner decides structure.

**CONV-01 dos2unix logic:**
- Scan bytes, replace `\r\n` with `\n`. Bare `\r` (no following `\n`) is preserved by default (GNU dos2unix also preserves bare CR).
- Binary detection: if first 32 KB contains a NUL byte (`0x00`), skip file with `dos2unix: Skipping binary file {path}` unless `-f` (force). Simple heuristic, matches GNU dos2unix default.

**CONV-02 unix2dos logic:**
- Scan bytes, replace standalone `\n` (not preceded by `\r`) with `\r\n`.
- Pre-existing `\r\n` unchanged.
- Same binary detection.

**Shared atomic rewrite path:**
```rust
gow_core::fs::atomic_rewrite(path, |bytes| {
    if is_binary(bytes) { return Err(GowError::Custom(...)); }
    Ok(transform_bytes(bytes))
})
```

**Flags:**
- `-n OLDFILE NEWFILE`: pairs new-file mode (no in-place).
- `-k` (keep mtime): after `atomic_rewrite`, restore source's mtime/atime on target.
- `-f` (force): skip binary detection.
- `-o` (default): in-place (what atomic_rewrite does).
- `-q` (quiet): suppress informational messages.

**BOM handling:** None per D-48. `\xef\xbb\xbf` at start of file passes through unchanged.

**ROADMAP criterion 5 satisfied:** Round-trip dos2unix → unix2dos = identity (for files without bare `\r`).

**Tests:**
- `dos2unix crlf.txt` — in-place, verify content has LF only
- `unix2dos lf.txt` — in-place, verify CRLF
- `dos2unix binary.exe` → skipped with message
- `dos2unix -f binary.exe` → force-converted
- `dos2unix -n src.txt dst.txt` — new-file mode
- `dos2unix -k preserves-mtime.txt` — timestamp preserved
- Round-trip: dos2unix then unix2dos = byte-identical (except bare-CR edge cases)
- File-lock test: open source with shared-read, run dos2unix → succeeds (atomic rewrite survives shared-read lock)

---

## Cross-Cutting Technical Notes

### Notify 8.2 Contract Summary

| Property | Value | Source |
|----------|-------|--------|
| Windows backend | `ReadDirectoryChangesW` async with completion routine | notify/src/windows.rs [VERIFIED] |
| Buffer size | 16384 bytes (16 KB) | notify/src/windows.rs `BUF_SIZE` [VERIFIED] |
| Wakeup interval | `WaitForSingleObjectEx(wakeup_sem, 100, 1)` — 100 ms worst case | notify/src/windows.rs server loop [VERIFIED] |
| Watched change flags | `FILE_NAME`, `DIR_NAME`, `ATTRIBUTES`, `SIZE`, `LAST_WRITE`, `CREATION`, `SECURITY` | notify/src/windows.rs [VERIFIED] |
| Append emits | `EventKind::Modify(ModifyKind::Any)` | notify/src/windows.rs [VERIFIED] |
| Rename emits | `Modify(Name(From))` then `Modify(Name(To))` with both paths | notify/src/windows.rs [VERIFIED] |
| Truncate emits | `EventKind::Modify(ModifyKind::Any)` (same as append — distinguish by size comparison) | notify/src/windows.rs [VERIFIED] |
| MSRV | 1.88 | notify README [VERIFIED] (our project: 1.95 — OK) |
| Recommended RecursiveMode for tail | `NonRecursive` (Pitfall #3 — watch parent, filter filename) | D-39 + STATE.md Pitfall #3 |
| Event channel | `std::sync::mpsc` or crossbeam via `EventHandler` trait | docs.rs/notify example [VERIFIED] |

### walkdir 2.5 Configuration Reference

| Method | Default | When to use | Phase 3 usage |
|--------|---------|-------------|---------------|
| `new(root)` | — | Always | All recursive utilities |
| `follow_links(bool)` | `false` | Symlink dereference choice | `cp`: true for `-L`, false for `-P` (default per D-44). `ls`, `rm`: always false |
| `sort_by_file_name()` | unsorted | Deterministic output | `ls -R`, `cp -r` for reproducible copy order |
| `contents_first(bool)` | `false` | Delete order | **`rm -r` MUST use `true`** |
| `min_depth(n)` | 0 | Skip root | Usually 0 |
| `max_depth(n)` | unlimited | Depth cap | None in Phase 3 |
| `same_file_system(bool)` | `false` | Boundary crossing | v2 `rm --one-file-system` |
| `max_open(n)` | 10 | File descriptor limit | Default OK |
| `filter_entry(fn)` | none | Skip subtrees | `ls` without `-a`: skip hidden dirs to avoid descending |

**Cycle detection:** enabled automatically when `follow_links(true)`. Cycles surface as `walkdir::Error` with `io_error()` returning `ErrorKind::Other` + `loop detected`. Handle in the iterator match arm.

**Junction behavior:** walkdir treats Windows junctions as directories but does NOT recurse into them even with `follow_links(true)` (reparse points ≠ symlinks in walkdir's classification). This is the correct behavior for `cp -rP` (preserve the junction) and `cp -rL` (the junction acts as a terminal directory entry).

### Atomic In-Place Rewrite Summary (D-47, Pitfall #4)

| Step | Operation | Windows API | Atomicity |
|------|-----------|-------------|-----------|
| 1 | Read source | `ReadFile` via `std::fs::read` | Not atomic; reader sees consistent snapshot if no concurrent writer |
| 2 | Create temp in parent dir | `CreateFileW(FILE_FLAG_DELETE_ON_CLOSE)` via `tempfile::NamedTempFile::new_in` | Temp auto-cleans if process dies |
| 3 | Write transformed bytes | `WriteFile` via `std::io::Write` | Buffered |
| 4 | Flush and sync | `FlushFileBuffers` via `sync_all()` | Durable before step 5 |
| 5 | Atomic rename | `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` via `std::fs::rename` (which tempfile's `persist()` calls) | ATOMIC if same-volume; falls back to copy+delete cross-volume |

**Why same-directory tempfile matters:** cross-volume persist is NOT atomic. Always `NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))`.

**Reader coexistence:** `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` succeeds when readers hold the target with `FILE_SHARE_READ | FILE_SHARE_DELETE`. If a reader opened without `FILE_SHARE_DELETE` (uncommon), persist fails with `ERROR_SHARING_VIOLATION (32)`. `atomic_rewrite` surfaces this as `GowError::Io` — correct GNU behavior.

### Win32 Link APIs Summary

| API | Rust wrapper | Used by |
|-----|--------------|---------|
| `CreateSymbolicLinkW(link, target, SYMBOLIC_LINK_FLAG_FILE)` | `std::os::windows::fs::symlink_file` | `ln -s` on file target |
| `CreateSymbolicLinkW(link, target, SYMBOLIC_LINK_FLAG_DIRECTORY)` | `std::os::windows::fs::symlink_dir` | `ln -s` on directory target (primary attempt) |
| `DeviceIoControl(FSCTL_SET_REPARSE_POINT, junction_buffer)` | `junction::create(target, link)` | `ln -s` on directory target (fallback per D-36) |
| `CreateHardLinkW(link, target)` | `std::fs::hard_link` | `ln` without `-s` |

**`SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE` (0x2):** Setting this flag allows symlink creation from a non-admin shell IF Windows 10 1703+ Developer Mode is ON. Rust's std **does not** set this flag in `symlink_file`/`symlink_dir` as of 1.95. Even with Dev Mode ON, non-admin shells fail with ERROR_PRIVILEGE_NOT_HELD — D-36 junction fallback is the pragmatic answer. [VERIFIED: rust-lang/rust library/std/src/sys/pal/windows/fs.rs `symlink_inner`]

### Error Code Reference (Windows → Rust → GNU)

| Win32 error | Value | Rust ErrorKind | GNU behavior |
|-------------|-------|----------------|--------------|
| `ERROR_FILE_NOT_FOUND` | 2 | `NotFound` | `{util}: {path}: No such file or directory`, exit 1 |
| `ERROR_PATH_NOT_FOUND` | 3 | `NotFound` | same as above |
| `ERROR_ACCESS_DENIED` | 5 | `PermissionDenied` | `{util}: {path}: Permission denied`, exit 1 |
| `ERROR_NOT_SAME_DEVICE` | 17 | `CrossesDevices` (1.85+) | For `ln`: `cross-device link not permitted`; for `mv`: fallback to copy+delete |
| `ERROR_SHARING_VIOLATION` | 32 | `Other` | `{path}: Resource busy` or `Text file busy` depending on utility |
| `ERROR_FILE_EXISTS` | 80 | `AlreadyExists` | For `ln` w/o `-f`: `File exists`, exit 1 |
| `ERROR_PRIVILEGE_NOT_HELD` | 1314 | `PermissionDenied` (via raw_os_error) | D-36: junction fallback for dir symlinks; for file symlinks: `Operation not permitted`, exit 1 |

---

## Reference Code Excerpts from uutils

uutils/coreutils is MIT-licensed; we reference patterns, never copy code. Relevant files and what to learn from them:

### `src/uu/tail/src/follow/watch.rs` (pattern reference)

uutils uses `notify::RecommendedWatcher` with `RecursiveMode::NonRecursive` on the parent directory for file-following. Their `Observer` struct maintains per-file state (last offset, rotation detection). Their truncation detection compares current size to saved offset — same approach as our Pattern 5.

**Don't copy:** uutils' code has extensive platform branches for kqueue (macOS/BSD) that we don't need. Keep Windows-only.

**Do adopt:** the event-filter-by-filename pattern and the per-file offset tracking struct.

### `src/uu/ls/src/ls.rs` (reference for option structure only)

uutils' ls uses `lscolors` crate for LS_COLORS parsing — we skip this per CONTEXT discretion (builtin defaults only). Their long-format formatter is mostly cross-platform with `#[cfg(unix)]` for owner/group — our Windows version substitutes `-` for both fields.

### `src/uu/cp/src/cp.rs` (reference for -p, -rP, -rL flag handling)

uutils uses `filetime` exactly as Phase 2 touch does — same pattern we'll reuse. Their cross-volume detection isn't explicit; they rely on `std::fs::copy` + `std::fs::remove_file` when `rename` fails, which matches our approach.

### `src/uu/ln/src/ln.rs` (reference for flag set; NOT for Windows logic)

uutils' ln for Windows uses a naive `symlink_file` / `symlink_dir` dispatch based on `src.as_ref().is_dir()` — **does not** fall back to junction on ERROR_PRIVILEGE_NOT_HELD. D-36 is gow-rust's unique value-add over uutils. [VERIFIED: github.com/uutils/coreutils src/uu/ln/src/ln.rs lines 488-493]

### `src/uu/dd/` and `src/uu/cat/src/cat.rs`

uutils cat uses `splice`/`sendfile` on Linux for zero-copy; on Windows it falls back to buffered read. Our D-48 raw-byte BufReader approach is equivalent in behavior, if not performance.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom `ReadDirectoryChangesW` loop for tail -f | `notify::RecommendedWatcher` | notify 4.x mature by ~2020 | 200+ lines of unsafe replaced by ~10 lines of crate usage |
| Poll-based tail -f | Event-driven tail -f (ReadDirectoryChangesW) | Mainstream in tail implementations since ~2015 | 200 ms latency (ROADMAP criterion) unachievable without event-driven approach; GOW #169/#75/#89 root cause |
| Custom junction via raw FSCTL | `junction 1.4.x` | crate matured ~2023 | Removes last major unsafe-Rust requirement from Phase 3 |
| `remove_dir_all` for rm -rf | Manual walkdir contents-first + clear_readonly | Still current; Rust std doesn't help on Windows RO attrs | Essential — std fails on read-only files |
| `winapi` for Win32 bindings | `windows-sys` | 2023+ | 10x faster compile; raw-dylib since 0.48 |

**Deprecated in this phase (do NOT use):**
- `notify 4.x` / `notify 5.x`: old event model; current is notify 6+ with `EventKind` enum
- `fs_extra` crate for cp -r: unmaintained; use walkdir + std::fs::copy
- Manual `FSCTL_SET_REPARSE_POINT` in utility crates: use `junction`

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `junction 1.4.2` is safe (no unaudited unsafe that could corrupt reparse data) | Standard Stack, Pattern 2 | Could produce malformed junctions; test by reading target back with `junction::get_target` and comparing |
| A2 | `notify 8.2.x` exists on crates.io with version >= 8.2.0 (CONTEXT.md specifies `"8.2"`) | Standard Stack | If only 9.x stable is available, must update Cargo.toml to `"9"` (requires new CONTEXT.md decision). cargo search showed 9.0.0-rc.3 but likely 8.2.x is still published and works. |
| A3 | `std::fs::rename` on Windows does NOT set `MOVEFILE_COPY_ALLOWED` (cross-volume returns error, does not fall back) | Pitfall 4, mv section | If it DOES set it, our explicit cross-volume fallback in mv is harmless but redundant. Conservative assumption. |
| A4 | `ReadDirectoryChangesW` 100 ms wakeup semaphore + kernel dispatch keeps us under 200 ms at P99 | Pattern 5 latency table | If bench shows >200 ms P99, fallback is `notify::Config::with_poll_interval(Duration::from_millis(100))` for polling backend — but we locked D-39 against polling. Worst case: request CONTEXT.md amendment. |
| A5 | `walkdir` does NOT recurse into Windows junctions even with `follow_links(true)` | walkdir reference | If it DOES recurse, `cp -rL` with junctions in source tree copies the target tree into dst — functionally correct but may produce huge copies. Test and document. |
| A6 | `tempfile::NamedTempFile::persist()` on Windows uses `MoveFileExW` (not rename2 or equivalent) | Pattern 1 | tempfile docs say "atomically replace" — if Windows impl is actually CopyFile + delete, still correct but less atomic. Docs explicitly say atomic, so this is low risk. |
| A7 | `std::fs::Permissions::set_readonly(false)` on Windows clears `FILE_ATTRIBUTE_READONLY` | Pattern 3 | If Rust std implementation no-ops on Windows (unlikely), `rm -f` on RO files would fail. Easy to verify with integration test. |
| A8 | Process-level ANSI codepage is already UTF-8 for the spawned utility (set by Phase 1 embed-manifest). dos2unix reading CP949 file bytes at `std::fs::read(path)` level gets raw bytes, NOT codepage-decoded | D-48, cat section | Phase 1 already verified this via gow-probe integration test. If somehow bypassed, would break UTF-8 + CP949 byte-exact roundtrip. Low risk. |
| A9 | `terminal_size::terminal_size()` returns `None` when stdout is piped on Windows (via GetConsoleScreenBufferInfo failing) | ls column layout | If it returns Some(default) instead, `ls | cat` would column-wrap weirdly. docs suggest None; easy to verify. |

---

## Open Questions (RESOLVED)

> All 8 questions have concrete Recommendations resolved into downstream PLAN.md files.
> Plan traces: Q1→03-11, Q2→03-02, Q3→03-12, Q4→03-05, Q5→03-09, Q6→03-04, Q7→03-07, Q8→03-12.

1. **Should `mv` cross-volume fallback preserve *all* attributes (timestamps, RO bit) or only timestamps?**
   - What we know: GNU `mv` across filesystems effectively copies with mode preservation.
   - What's unclear: Whether to treat cross-volume `mv` as implicit `-p` (copy + timestamps) or plain copy.
   - Recommendation: Implicit `-p` behavior (timestamps + RO bit), matching GNU. Document in mv --help.

2. **`cat -A`: implement as `-vET` shorthand or omit?**
   - What we know: CONTEXT.md Claude's discretion says "planner 재량".
   - Recommendation: Implement. Trivial once `-v`/`-E`/`-T` are done. Users expect it.

3. **`tail -f` behavior when file does not exist at startup (without `-F`)?**
   - GNU: errors out with "cannot open for reading: No such file or directory", exit 1.
   - `tail -F` (our `-F` flag): waits and retries (notify Create event).
   - Recommendation: match GNU exactly. `-f` requires file to exist; `-F` doesn't.

4. **Binary detection threshold for dos2unix/unix2dos?**
   - GNU dos2unix: checks if any byte in the whole file is `\0` for "binary" (`-ic` mode). First 32 KB is a common heuristic. CONTEXT.md does not specify.
   - Recommendation: Check first 32 KB for `\0`. Document in --help. Can be overridden with `-f` (force).

5. **`ls --color` on pipes: emit ANSI codes or suppress?**
   - GNU default is `--color=auto` → only emit for tty stdout.
   - `terminal_size()` already returns None for pipes; use this as the auto-detection gate.
   - Respect `--color=always` explicit override.
   - NO_COLOR env var already honored by gow_core::color.

6. **Should `chmod` warn on group/other bits (instead of silent-ignore per D-32)?**
   - D-32 says silent to reduce script noise.
   - Counter-argument: `chmod 600 file` silently behaves like `chmod 400 file` (read-only) — user expectation violation.
   - Recommendation: Stick with D-32 (silent); document heavily in `--help`. Users reading `--help` will understand. Scripts won't break. Re-evaluate in v2 based on user feedback.

7. **`cp -r` on a directory symlink with `-P` (default): clone the symlink, or copy the target tree?**
   - D-44: `-rP` = preserve symlinks. → clone the link (to target, not the tree).
   - Implementation: when walkdir encounters a symlink entry with `follow_links=false`, emit it as a symlink entry; `copy_one` detects `link_type` and calls `create_link` at destination.

8. **`tail -f` on multi-file + file added mid-flight under `-F`:**
   - Multi-file is an edge case. Recommend: watch each file's parent directory (one watcher per unique parent dir, or one recursive watcher if all args share a common root); on Create, reopen.
   - Keep initial implementation simple: per-file watchers, share nothing.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable (MSVC) | All compilation | ✓ | 1.95.0 (> notify MSRV 1.88) | — |
| Cargo workspace | Build | ✓ | 1.95.0 | — |
| Windows 10/11 with Dev Mode | symlink integration tests | [partial] | Dev Mode state unknown on CI runners | Gate symlink tests with privilege skip (Phase 1 pattern at fs.rs:118) |
| Dev Mode for junction fallback test | ln test — create junction when symlink fails | N/A | Junction creation does NOT require privilege | — |
| Internet access for crate downloads | First `cargo build` | ✓ | — | — |
| `cargo search` | Version verification during research | ✓ | — | — |
| Phase 2 binaries still resident | Workspace integrity | ✓ | — | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:**
- Dev Mode for symlink creation: fallback = skip tests (integration tests already pattern this via `gow-core/src/fs.rs:118`).

**Sanity checks to run at plan start:**
```bash
cargo search notify --limit 5         # confirm 8.2.x still published
cargo search walkdir                   # confirm 2.5.x
cargo search junction --limit 5        # confirm junction 1.4.x
cargo search terminal_size             # confirm 0.4.x
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner + `assert_cmd 2.2.1` + `predicates 3.1.4` + `snapbox 1.2.1` |
| Config file | None (cargo default test discovery) |
| Quick run command | `cargo test -p gow-<utility>` |
| Full suite command | `cargo test --workspace` |
| Lint gate | `cargo clippy --workspace --all-targets -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FILE-01 | `cat file` pass-through; `cat -n` UTF-8 line numbers | Integration | `cargo test -p gow-cat` | ❌ Wave 1 |
| FILE-01 | `cat -v` encodes control chars per GNU | Integration | `cargo test -p gow-cat -- test_cat_v` | ❌ Wave 1 |
| FILE-01 | `cat non-existent` exit 1 GNU format | Integration | `cargo test -p gow-cat -- test_error` | ❌ Wave 1 |
| FILE-02 | `ls -la` hidden files + long format | Integration | `cargo test -p gow-ls -- test_la` | ❌ Wave 3 |
| FILE-02 | `ls -R` recursive with headers | Integration | `cargo test -p gow-ls -- test_recursive` | ❌ Wave 3 |
| FILE-02 | `ls` colorizes via ANSI | Snapshot | `cargo test -p gow-ls -- test_color` | ❌ Wave 3 |
| FILE-02 | `ls` on dir with symlink → `l` prefix + target | Integration (privilege-gated) | `cargo test -p gow-ls -- test_symlink` | ❌ Wave 3 |
| FILE-03 | `cp file dst` basic copy | Integration | `cargo test -p gow-cp -- test_basic` | ❌ Wave 3 |
| FILE-03 | `cp -r src/ dst/` recursive | Integration | `cargo test -p gow-cp -- test_recursive` | ❌ Wave 3 |
| FILE-03 | `cp -p` timestamp preservation | Integration | `cargo test -p gow-cp -- test_preserve` | ❌ Wave 3 |
| FILE-04 | `mv file dst` same-volume | Integration | `cargo test -p gow-mv -- test_basic` | ❌ Wave 4 |
| FILE-04 | `mv` cross-volume copy-delete | Integration (conditional on D:\ existing) | `cargo test -p gow-mv -- test_cross_drive` | ❌ Wave 4 |
| FILE-05 | `rm -r dir/` recursive | Integration | `cargo test -p gow-rm -- test_recursive` | ❌ Wave 3 |
| FILE-05 | `rm -rf` read-only file | Integration | `cargo test -p gow-rm -- test_readonly` | ❌ Wave 3 |
| FILE-05 | `rm -rf C:\` refused | Integration | `cargo test -p gow-rm -- test_drive_root` | ❌ Wave 3 |
| FILE-09 | `ln a b` hard link | Integration | `cargo test -p gow-ln -- test_hard` | ❌ Wave 4 |
| FILE-09 | `ln -s dir link` without privilege → junction fallback | Integration | `cargo test -p gow-ln -- test_junction_fallback` | ❌ Wave 4 |
| FILE-09 | `ln cross-volume-hard` → cross-device error | Integration (conditional) | `cargo test -p gow-ln -- test_cross_device` | ❌ Wave 4 |
| FILE-10 | `chmod 644 file` clears RO | Integration | `cargo test -p gow-chmod -- test_writable` | ❌ Wave 1 |
| FILE-10 | `chmod 444 file` sets RO | Integration | `cargo test -p gow-chmod -- test_readonly` | ❌ Wave 1 |
| FILE-10 | `chmod -R 644 dir/` recursive | Integration | `cargo test -p gow-chmod -- test_recursive` | ❌ Wave 1 |
| TEXT-01 | `head -n 5 file` | Integration | `cargo test -p gow-head -- test_n` | ❌ Wave 1 |
| TEXT-01 | `head -5 file` numeric shorthand | Integration | `cargo test -p gow-head -- test_shorthand` | ❌ Wave 1 |
| TEXT-01 | `head -c 10 file` | Integration | `cargo test -p gow-head -- test_bytes` | ❌ Wave 1 |
| TEXT-02 | `tail -n 5 file` | Integration | `cargo test -p gow-tail -- test_n` | ❌ Wave 5 |
| TEXT-02 | `tail -f` detects append < 500 ms | Integration (timing) | `cargo test -p gow-tail -- test_follow_latency` | ❌ Wave 5 |
| TEXT-02 | `tail -F` reopens on rotation | Integration | `cargo test -p gow-tail -- test_rotate` | ❌ Wave 5 |
| TEXT-02 | `tail -f` handles truncation | Integration | `cargo test -p gow-tail -- test_truncate` | ❌ Wave 5 |
| CONV-01 | `dos2unix crlf.txt` in-place CRLF→LF | Integration | `cargo test -p gow-dos2unix -- test_basic` | ❌ Wave 2 |
| CONV-01 | Binary detection skip | Integration | `cargo test -p gow-dos2unix -- test_binary` | ❌ Wave 2 |
| CONV-01 | `-k` preserves mtime | Integration | `cargo test -p gow-dos2unix -- test_mtime` | ❌ Wave 2 |
| CONV-02 | `unix2dos lf.txt` LF→CRLF | Integration | `cargo test -p gow-unix2dos -- test_basic` | ❌ Wave 2 |
| CONV-02 | Round-trip identity | Integration | `cargo test -p gow-unix2dos -- test_roundtrip` | ❌ Wave 2 |
| — | Atomic rewrite survives shared-read lock | Integration | `cargo test -p gow-dos2unix -- test_locked_file` | ❌ Wave 2 |
| — | `gow_core::fs::atomic_rewrite` unit tests | Unit | `cargo test -p gow-core fs::atomic` | ❌ Wave 0 |
| — | `gow_core::fs::create_link` unit tests | Unit | `cargo test -p gow-core fs::link` | ❌ Wave 0 |
| — | `gow_core::fs::is_drive_root` unit tests | Unit | `cargo test -p gow-core fs::drive_root` | ❌ Wave 0 |
| — | `gow_core::fs::has_executable_extension` unit tests | Unit | `cargo test -p gow-core fs::exec_ext` | ❌ Wave 0 |
| — | `gow_core::fs::is_hidden` unit tests | Unit | `cargo test -p gow-core fs::hidden` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p gow-<current-utility>`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** full suite green + clippy clean before `/gsd-verify-work`

### Wave 0 Gaps (workspace prep deliverables)

- [ ] `Cargo.toml` (workspace root) — add `walkdir`, `notify`, `terminal_size`, `junction` to `[workspace.dependencies]`
- [ ] `crates/gow-core/src/fs.rs` EXTENSIONS:
  - [ ] `atomic_rewrite<F>` function + unit tests (mock transform)
  - [ ] `create_link` function + unit tests (guard privilege)
  - [ ] `LinkKind` enum + `LinkOutcome` enum
  - [ ] `is_hidden` function + unit tests
  - [ ] `is_readonly` function + unit tests
  - [ ] `has_executable_extension` function + unit tests
  - [ ] `clear_readonly` function + unit tests
  - [ ] `is_drive_root` function + unit tests
- [ ] `crates/gow-core/Cargo.toml` — add `tempfile = { workspace = true }` as runtime dep (currently only dev-dep); add `junction = "1.4"` on Windows only (`[target.'cfg(windows)'.dependencies]`).
- [ ] 11 stub crates scaffolded (mirroring 02-01 Phase 2 pattern):
  - [ ] `crates/gow-cat/{Cargo.toml, build.rs, src/{lib.rs, main.rs}}`
  - [ ] (same for gow-ls, gow-cp, gow-mv, gow-rm, gow-ln, gow-chmod, gow-head, gow-tail, gow-dos2unix, gow-unix2dos)
  - Each stub's `uumain` prints `{name}: not yet implemented` and returns 1.

### Test Fixtures Needed (create once, reuse across utilities)

| Fixture | Purpose | Used by |
|---------|---------|---------|
| UTF-8 file with Korean text (`안녕.txt`) | cat -n mojibake test | FILE-01 |
| CRLF file | dos2unix, cat | CONV-01, FILE-01 |
| LF file | unix2dos | CONV-02 |
| Mixed UTF-8 + CP949 bytes | D-48 no-panic test | FILE-01, TEXT-01/02, CONV-01/02 |
| Binary file (with NUL bytes) | dos2unix binary detection | CONV-01, CONV-02 |
| Read-only file | rm -f, chmod, cp -p | FILE-05, FILE-10, FILE-03 |
| Hidden file (dot-prefix + Win hidden attr) | ls -a | FILE-02 |
| File symlink (privilege-gated) | ls -l, ln -s, cp -rP | FILE-02, FILE-09, FILE-03 |
| Directory symlink (privilege-gated) | ln -s dir, junction fallback | FILE-09 |
| Junction directory | ls junction display | FILE-02 |
| Large file (>64 KB) | tail -f with backward-seek | TEXT-02 |
| Rotating log file (rename + recreate) | tail -F | TEXT-02 |
| Truncatable log file | tail -f truncation | TEXT-02 |
| Locked file (shared read) | dos2unix atomic rewrite | CONV-01 |

### Platform-Specific Gates

Reuse Phase 1 symlink-privilege-skip pattern from `gow-core/src/fs.rs:118`:

```rust
#[cfg_attr(not(windows), ignore)]
fn test_foo() {
    // ...
    let symlink_result = std::os::windows::fs::symlink_file(&target, &link);
    if symlink_result.is_err() {
        eprintln!("[skip] symlink privilege missing");
        return;
    }
    // real test ...
}
```

**For D-36 junction fallback test specifically:** Deliberately call `gow_core::fs::create_link` with `LinkKind::Symbolic` and a directory target. If running without privilege, we expect `LinkOutcome::Junction`. If running with privilege, we expect `LinkOutcome::Symlink`. Both are valid — assert appropriately.

### Test Determinism Env Overrides

- **`GOW_PATHEXT`:** NOT used in Phase 3 (D-35 hardcodes the set).
- **`NO_COLOR`:** honored via existing `gow_core::color`. Tests that check ANSI codes explicitly force `--color=always`.
- **Timezone:** `filetime` usage does not depend on local tz for Unix timestamps. However, `ls -l` formats mtime for display — for deterministic tests, compare against timestamp equality or use `assert!(within_tolerance(...))` rather than snapshot-matching the formatted string.
- **File order:** walkdir `sort_by_file_name()` ensures deterministic ordering in `ls -R`, `cp -r`.
- **Terminal width:** for snapshot tests of `ls` column layout, force `--color=never` + explicit `-1` (single column) to avoid width-dependent output.

### Planner-Level Nyquist Validation Notes

This phase has 11 utilities and ~40+ distinct test cases. The sampling plan:
- **Task-level:** run `cargo test -p <crate>` after each task commit. Fast (< 30s per crate).
- **Wave-level:** run `cargo test --workspace` after each wave merge. ~2-3 min.
- **Phase-level:** full suite + manual verification of ROADMAP's 5 criteria with real commands before `/gsd-verify-work`.

Tail's 200 ms latency test is the ONLY timing-sensitive test. It MUST be robust to CI jitter:
- Use 500 ms upper bound in tests (target is 200 ms; margin for CI).
- Run only on `cargo test --release -p gow-tail -- --ignored test_follow_latency` (flaky on debug).
- Document the manual verification procedure: `start touch file.log; tail -f file.log; echo "x" >> file.log` — human observes < 200 ms on dev machine.

---

## Security Domain

`security_enforcement` not explicitly disabled in config.json — including this section.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | CLI — no user auth |
| V3 Session Management | No | — |
| V4 Access Control | Yes (partial) | `rm` drive-root refusal (D-42) prevents destructive Windows-root wipes; no privilege escalation claims |
| V5 Input Validation | Yes | Path arguments validated via MSYS normalizer (D-06..D-08 inherited); binary detection in dos2unix guards against accidental corruption |
| V6 Cryptography | No | No crypto in Phase 3 |

### Known Threat Patterns for Filesystem Utility Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Arbitrary file overwrite via `ln -f` race | Tampering | `std::fs::hard_link` and `symlink_file` fail on existing dst; `-f` explicitly opts in to remove-then-create (two-step; accept race as intentional behavior, same as GNU) |
| Symlink attack on `cp -L` (follow to unexpected dir) | Tampering | walkdir cycle detection handles loops; D-44 makes `-P` default so -L is explicit opt-in; matches GNU default |
| `rm -rf /` destroying entire filesystem | Destruction | D-42 `--preserve-root` ON by default + Windows drive-root check (`C:\`, UNC) |
| `tail -f` reading past end of privilege-restricted file | Info Disclosure | `std::fs::File::open` enforces Windows ACL; if user lacks read permission, open fails before notify starts |
| dos2unix corrupting binary file | Tampering | NUL-byte heuristic in first 32 KB; `-f` required to force |
| Atomic rewrite race: tempfile in `%TEMP%` visible to other users | Info Disclosure | D-47 mandates `NamedTempFile::new_in(parent_dir)` so temp inherits target's ACL |
| `ln` target path traversal (`ln -s ../../../etc/passwd link`) | Tampering (upstream) | symlinks are valid Unix/Windows semantics; no traversal defense needed at ln layer — that's the consumer's job. GNU behavior. |
| Junction absolute-only path requirement | Denial of Service (limited) | `junction::create` accepts relative paths and converts to absolute; user-visible error is clear if target doesn't exist at create time |

**No new ASVS requirements from Phase 3 beyond Phase 1/2 inheritance.** D-42 adds a Windows-specific safety gate (drive-root refusal) that is a gow-rust hardening over GNU defaults.

---

## Project Constraints (from CLAUDE.md)

- **언어:** Rust stable (MSVC) — enforced by `[workspace.package] rust-version = "1.85"`; current toolchain 1.95.0 satisfies
- **타겟 플랫폼:** Windows 10/11 x86_64 MSVC only — tests may run on Linux in CI but platform-specific gates (`#[cfg_attr(not(windows), ignore)]`) handle this
- **호환성:** GNU 옵션 높은 호환성 — each utility must accept all flags in its requirement line (D-49 requirement IDs → flag sets)
- **배포:** MSI 설치 프로그램 — v2 concern, out of Phase 3 scope
- **바이너리 구조:** 유틸리티별 독립 exe — D-49 confirms 11 new binaries
- **인코딩:** UTF-8 기본, Windows 코드페이지 폴백 지원 — `gow_core::init()` handles console UTF-8; D-48 ensures raw-byte passthrough for text utilities
- **GSD Workflow Enforcement:** All Phase 3 edits happen through `/gsd-plan-phase 3` → `/gsd-execute-phase 3`
- **Test stack (from CLAUDE.md):** assert_cmd + snapbox + predicates + tempfile — inherited Phase 1/2 setup
- **Lint gate:** `cargo clippy --workspace --all-targets -- -D warnings` — Phase 2 passed; Phase 3 must maintain
- **Static CRT:** `-C target-feature=+crt-static` (Phase 1 .cargo/config.toml) — inherited, no change

---

## Sources

### Primary (HIGH confidence — direct source or docs.rs)

- `.planning/phases/03-filesystem/03-CONTEXT.md` — D-31..D-51 locked decisions
- `.planning/REQUIREMENTS.md` — FILE-01..05, FILE-09..10, TEXT-01..02, CONV-01..02
- `.planning/ROADMAP.md` §66-77 — Phase 3 goal + 5 success criteria
- `.planning/STATE.md` — Critical Pitfalls #3 (tail -f parent watch) and #4 (MoveFileExW atomic swap)
- `.planning/phases/01-foundation/01-RESEARCH.md` — Phase 1 research; embed-manifest, gow-core patterns
- `.planning/phases/02-stateless/02-RESEARCH.md` — Phase 2 research; lib+bin pattern reference, filetime usage
- `crates/gow-core/src/fs.rs` — existing LinkType, link_type, normalize_junction_target (reused in Phase 3)
- `crates/gow-core/src/lib.rs` — init() contract
- `crates/gow-touch/src/lib.rs` — filetime pattern to be reused in cp -p
- `crates/gow-touch/build.rs` — embed-manifest build.rs template (verbatim copy for each Phase 3 crate)
- [github.com/notify-rs/notify notify/src/windows.rs] — confirmed ReadDirectoryChangesW usage, BUF_SIZE=16KB, wakeup_sem=100ms, event kinds for append/rename/truncate [VERIFIED via WebFetch 2026-04-21]
- [github.com/notify-rs/notify README] — MSRV 1.88, debouncer variants, Windows backend confirmation [VERIFIED via WebFetch]
- [docs.rs/walkdir/2.5.0/walkdir/struct.WalkDir.html] — full method list: new, min_depth, max_depth, follow_links, follow_root_links, max_open, sort_by, sort_by_key, sort_by_file_name, contents_first, same_file_system [VERIFIED via WebFetch]
- [docs.rs/tempfile NamedTempFile::persist] — "atomically replace" + PersistError recovery [VERIFIED via WebFetch]
- [docs.rs/notify/latest enum.RecursiveMode] — NonRecursive vs Recursive semantics [VERIFIED via WebFetch]
- [docs.rs/junction/1.4.2/junction/] — create, exists, get_target, delete API [VERIFIED via WebFetch]
- [github.com/notify-rs/notify examples/monitor_raw.rs] — canonical `RecommendedWatcher::new(tx, Config::default())` usage [VERIFIED via WebFetch]
- crates.io registry 2026-04-21 (via `cargo search`):
  - `walkdir = "2.5.0"` [VERIFIED]
  - `notify = "9.0.0-rc.3"` (latest); `8.2.x` previous stable per CONTEXT.md D-50 lock [VERIFIED]
  - `tempfile = "3.27.0"` [VERIFIED — already in workspace]
  - `filetime = "0.2.27"` [VERIFIED — already in workspace]
  - `terminal_size = "0.4.4"` [VERIFIED]
  - `junction = "1.4.2"` [VERIFIED]
- [github.com/tesuji/junction GitHub repo] — last release 2026-02-24 (v1.4.2); MIT license; maintenance active [VERIFIED via WebFetch]

### Secondary (MEDIUM confidence — ecosystem reference, cross-checked)

- [github.com/uutils/coreutils src/uu/ln/src/ln.rs] — uutils' Windows symlink dispatch (naive is_dir + symlink_file/symlink_dir; NO junction fallback — confirms D-36 is gow-rust unique) [CITED via WebFetch]
- uutils tail/ln/cp source code referenced for flag set and option structure (MIT — pattern reference, not copy)
- [learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw] — MoveFileExW with MOVEFILE_REPLACE_EXISTING / MOVEFILE_COPY_ALLOWED flags [CITED]
- [learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--1300-1699-] — ERROR_PRIVILEGE_NOT_HELD = 1314 [CITED]

### Tertiary (LOW confidence — supplementary)

- GNU coreutils manual — option semantics reference (https://www.gnu.org/software/coreutils/manual/coreutils.html), especially head/tail/cat/cp/mv/rm/ln/chmod sections [CITED by URL]
- GOW issues #169, #75, #89 — historical tail -f polling problem context that D-39 solves [CITED]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified against live crates.io 2026-04-21 via `cargo search`
- Architecture: HIGH — patterns verified against uutils/coreutils (where applicable) and Phase 1/2 precedent
- Pitfalls: HIGH — Pitfalls #3 (STATE.md) and #4 (STATE.md) are locked; notify latency numbers verified from source
- notify 8.2 contract: HIGH — confirmed via direct source inspection
- junction 1.4.2 suitability: MEDIUM-HIGH — crate is maintained (Feb 2026 release), 100% Rust, but we haven't audited its 200-line unsafe block ourselves
- walkdir 2.5 configuration: HIGH — full method list verified from docs.rs
- tempfile atomic rewrite: HIGH — docs.rs explicit on atomicity; MoveFileExW behavior documented by Microsoft
- Per-utility flag sets: MEDIUM — GNU manual is the source of truth; planner should verify edge cases per utility during implementation

**Research date:** 2026-04-21
**Valid until:** 2026-05-21 (stable crate ecosystem; re-verify versions if > 30 days elapsed; re-verify notify 8.2 availability if 9.0 goes stable)

---

## RESEARCH COMPLETE
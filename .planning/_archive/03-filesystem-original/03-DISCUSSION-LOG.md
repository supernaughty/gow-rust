# Phase 3: Filesystem Utilities - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 03-filesystem
**Areas discussed:** Windows 권한 모델, 링크 전략, tail -f watcher 전략, 파괴적 연산 안전성

---

## Area Selection

**Question:** Phase 3은 11개 유틸리티로 구성되어 있습니다. 어떤 gray area를 논의할까요? (여러 개 선택 가능)

| Option | Description | Selected |
|--------|-------------|----------|
| Windows 권한 모델 (ls -l, chmod, cp -p) | POSIX mode bits가 Windows에 존재하지 않음. ls -l 권한 컬럼 표시 방식, chmod 실제 동작, cp -p 보존 범위 결정. | ✓ |
| 링크 전략 (ln, ls symlink/junction 표시) | Developer Mode 없는 환경에서 ln -s 동작, junction fallback, symlink/junction 표시 구분, 하드링크 지원 여부. | ✓ |
| tail -f watcher 전략 | ROADMAP 200ms 기준, notify crate vs raw ReadDirectoryChangesW, rotation/truncation 처리, 다중 파일 출력 포맷. | ✓ |
| 파괴적 연산 안전성 (rm, cp, mv) | rm --preserve-root + 드라이브 루트 보호, -i 기본 동작, cp -r symlink 처리 (-L/-P/-H), read-only 파일 처리. | ✓ |

**User's choice:** 네 가지 모두 선택.

---

## Windows 권한 모델

### Q1: ls -l의 권한 컬럼(9자)에 무엇을 표시할까요?

| Option | Description | Selected |
|--------|-------------|----------|
| Read-only 비트 기반 합성 (추천) | FILE_ATTRIBUTE_READONLY 기반 rw-rw-rw- / r--r--r--. 실행 비트는 .exe/.cmd/.bat 확장자에서 x. Cygwin/MSYS 전통. | ✓ |
| 고정 '-rw-rw-rw-' 플레이스홀더 | 모든 파일 동일 문자열. uutils 기본. 정보 없음. | |
| 전체 ACL DACL 분석 | GetSecurityInfo로 DACL 해석. 정확하지만 POSIX 매핑 괴리 + 성능 부담. | |

**User's choice:** Read-only 비트 기반 합성 (추천)
**Lock:** D-31

### Q2: chmod가 Windows에서 실제로 무엇을 변경할까요?

| Option | Description | Selected |
|--------|-------------|----------|
| Read-only 비트만 매핑 (추천) | Owner write 비트 → FILE_ATTRIBUTE_READONLY. 나머지 무시. | ✓ |
| 경고 정보 + Read-only 매핑 | 무시되는 비트마다 stderr 경고 출력. | |
| 완전 ACL 매핑 | SetSecurityInfo로 DACL 조작. 위험함 (파일 잠금 위험). | |

**User's choice:** Read-only 비트만 매핑 (추천)
**Lock:** D-32

### Q3: cp -p가 보존하는 메타데이터 범위는?

| Option | Description | Selected |
|--------|-------------|----------|
| Timestamps + Read-only 비트 (추천) | mtime/atime + FILE_ATTRIBUTE_READONLY 복사. ROADMAP 기준 충족. | ✓ |
| Timestamps만 | mtime/atime만. 가장 단순. | |
| Timestamps + 모든 FILE_ATTRIBUTES | read-only + hidden + archive + system 전부 복사. | |

**User's choice:** Timestamps + Read-only 비트 (추천)
**Lock:** D-33

### Q4: ls -a의 '숨김 파일' 정의는?

| Option | Description | Selected |
|--------|-------------|----------|
| 점(.) 접두 OR FILE_ATTRIBUTE_HIDDEN (추천) | 두 관례의 합집합. Cygwin과 일치. | ✓ |
| 점(.) 접두만 | 순수 Unix 시맨틱. Windows hidden 속성 파일 노출. | |
| FILE_ATTRIBUTE_HIDDEN만 | Windows 시맨틱. dotfile (.git 등) 기본 노출 — Unix 스크립트 위험. | |

**User's choice:** 점(.) 접두 OR FILE_ATTRIBUTE_HIDDEN (추천)
**Lock:** D-34

### Q5: ls -l의 execute 비트(x)는 어떻게 결정할까요?

| Option | Description | Selected |
|--------|-------------|----------|
| 확장자 기반 (.exe/.cmd/.bat/.ps1/.com) (추천) | 고정 세트. 결정적, 테스트 안정. D-18a which 기본값과 일관. | ✓ |
| PATHEXT 환경변수 참조 | 동적. 테스트 flaky. | |
| 항상 - | 모든 파일 rw-rw-rw-. 단순하지만 정보 없음. | |

**User's choice:** 확장자 기반 (.exe/.cmd/.bat/.ps1/.com) (추천)
**Lock:** D-35

---

## 링크 전략

### Q6: ln -s로 디렉토리 링크 생성 시 CreateSymbolicLinkW가 권한 부족으로 실패하면?

| Option | Description | Selected |
|--------|-------------|----------|
| Junction으로 자동 fallback + stderr 알림 (추천) | Developer Mode 없는 환경에서도 `ln -s dir newdir` 동작. 'symlink privilege unavailable, created junction instead' 경고. | ✓ |
| 엄격 실패 (exit 1) | POSIX 시맨틱 유지. Cygwin/WSL 관례. | |
| 명시적 플래그 (--junction)로만 fallback | 기본은 실패, --junction 있을 때만 junction. 명확하지만 GNU에 없는 플래그. | |

**User's choice:** Junction으로 자동 fallback + stderr 알림 (추천)
**Lock:** D-36
**Notes:** 주의 — junction은 절대경로만 저장되며 로컬 볼륨만 가능함.

### Q7: ls -l에서 symlink과 junction을 어떻게 구분 표시할까요?

| Option | Description | Selected |
|--------|-------------|----------|
| symlink=l, junction=l + target에 [junction] 태그 (추천) | POSIX ls 파싱 호환 + 정보 전달. 'link -> target' vs 'jct -> C:\\target [junction]'. | ✓ |
| 둘 다 l + target 표기만 | 순수 POSIX 스타일. 구분 없음. | |
| symlink=l, junction=J (GOW 전통) | 원조 GOW/uutils 방식. 스크립트 호환성 우려. | |

**User's choice:** symlink=l, junction=l + target에 [junction] 태그 (추천)
**Lock:** D-37

### Q8: ln (옵션 없이) = 하드링크 지원?

| Option | Description | Selected |
|--------|-------------|----------|
| 지원, CreateHardLinkW 사용 (추천) | 동일 볼륨에서만. 크로스 볼륨 시 exit 1. | ✓ |
| 지원하지 않음, 경고 및 exit 1 | Windows 하드링크는 nlink 감지 이슈 등 문제. v2로 연기. | |

**User's choice:** 지원, CreateHardLinkW 사용 (추천)
**Lock:** D-38

---

## tail -f watcher 전략

### Q9: tail -f watcher는 어떤 레이어를 사용할까요?

| Option | Description | Selected |
|--------|-------------|----------|
| notify 8.2 crate (추천) | CLAUDE.md 스택 명시. RecommendedWatcher. 200+ 줄 unsafe 회피. Pitfall #3 parent-dir watch 패턴은 notify 위에 구현 가능. | ✓ |
| Raw ReadDirectoryChangesW (windows-sys) | 제어권 최대, 의존성 최소. 200+ 줄 unsafe 작성 필요. | |
| notify-debouncer-full | 빠른 연속 write 합침. 200ms latency 위반 우려. | |

**User's choice:** notify 8.2 crate (추천)
**Lock:** D-39

### Q10: tail -f 도중 파일이 rotate/truncate되면?

| Option | Description | Selected |
|--------|-------------|----------|
| --follow=descriptor (현 descriptor 유지, 기본) (추천) | GNU 기본. -f = --follow=descriptor. 원래 handle 계속 추적. | ✓ |
| -F = --follow=name + --retry 같이 구현 | -f descriptor, -F name. GNU 사실상 2-set 지원. | |
| --follow=name만 | -f = name. 단순하지만 descriptor 추적 필요 스크립트 깨짐. | |

**User's choice:** --follow=descriptor (현 descriptor 유지, 기본) (추천)
**Lock:** D-40
**Notes:** -F (= --follow=name + --retry) 도 별도로 구현함 (user 선택한 옵션 설명에 포함된 내용).

### Q11: tail -f file1 file2 (다중 파일) 출력 포맷은?

| Option | Description | Selected |
|--------|-------------|----------|
| GNU 표준: 전환 시 헤더 '==> file <==' (추천) | 업데이트 전환 시마다 헤더. 연속 중복 억제. | ✓ |
| 줄 prefix 'file: ' 형식 | grep 스타일. 파싱 쉽지만 GNU 동작과 다름. | |
| -q (quiet)로 헤더 제거 옵션 추가 | 기본은 GNU, -q로 억제. -v로 강제. | |

**User's choice:** GNU 표준: 전환 시 헤더 '==> file <==' (추천)
**Lock:** D-41
**Notes:** -q / -v 옵션도 포함 (GNU 표준 세트).

---

## 파괴적 연산 안전성

### Q12: rm -rf의 root 보호 범위는?

| Option | Description | Selected |
|--------|-------------|----------|
| --preserve-root 기본 ON + 드라이브 루트 보호 (추천) | GNU 기본 + 'C:\\' 등 드라이브 루트도 거부. --no-preserve-root로 override. | ✓ |
| --preserve-root 기본 ON (POSIX 루트만) | GNU 정확 동작. 'rm -rf C:\\' 실행됨. | |
| --preserve-root OFF | 전통 GNU < 6.4. 위험. | |

**User's choice:** --preserve-root 기본 ON + 드라이브 루트 보호 (추천)
**Lock:** D-42

### Q13: rm -i 대화형 prompt는 언제 작동?

| Option | Description | Selected |
|--------|-------------|----------|
| -i 명시할 때만 prompt (GNU 기본) (추천) | GNU 기본. -i 없으면 write-protected 파일 예외. | ✓ |
| -i + write-protected 파일에서 자동 | -i 명시 + write-protected에서 tty면 prompt. -f suppress. | |
| 절대 질문 안 함 (-f 기본) | GNU 호환성 다름. 스크립트 친화. | |

**User's choice:** -i 명시할 때만 prompt (GNU 기본) (추천)
**Lock:** D-43

### Q14: cp -r 시 symlink 기본 처리는?

| Option | Description | Selected |
|--------|-------------|----------|
| symlink을 symlink으로 복사 (-P 기본) (추천) | GNU coreutils 현재 기본 (cp -r = cp -rP). -L로 dereference, -H로 CLI arg만 dereference. | ✓ |
| symlink target을 따라가서 복사 (-L) | 전통 POSIX 기본. | |

**User's choice:** symlink을 symlink으로 복사 (-P 기본) (추천)
**Lock:** D-44

### Q15: rm이 read-only 파일을 만났을 때 (-f 없이)?

| Option | Description | Selected |
|--------|-------------|----------|
| stdin이 tty면 prompt, 아니면 거부 (GNU) (추천) | GNU 표준. 'rm: remove write-protected file X?' y/n. 비-tty는 Permission denied. | ✓ |
| 무조건 거부 (-f 필요) | 더 엄격. | |
| 경고 없이 삭제 | GNU rm -f 기본 동작. | |

**User's choice:** stdin이 tty면 prompt, 아니면 거부 (GNU) (추천)
**Lock:** D-45

---

## Done Check

**Question:** 논의 완료. 추가로 논의할 gray area가 있을까요?

| Option | Description | Selected |
|--------|-------------|----------|
| 텍스트 변환 및 인코딩 (cat -v/-A, dos2unix/unix2dos) | BOM, binary 감지, in-place 원자적 교체, -n/-o 플래그. | |
| head/tail 바이트/라인 모드 | non-UTF-8 처리, bstr 사용, -c/-n 정확성. | |
| 이제 CONTEXT 쓰기 준비됨 (추천) | 15 decision 잠김. research/planner에 위임. | ✓ |

**User's choice:** 이제 CONTEXT 쓰기 준비됨 (추천)

Text conversion/encoding 및 head/tail 세부는 Claude의 재량으로 CONTEXT D-46~D-51 및 Claude's Discretion 섹션에 반영됨. Research phase에서 추가 clarification 필요 시 재방문.

---

## Claude's Discretion (CONTEXT.md §Claude's Discretion 참조)

- cat -v/-A/-T/-E non-printable 표기 세부 구현
- ls --color 디폴트 색상 스키마
- cp/mv progress 표시 (기본 silent)
- head/tail -c byte count 구현 세부
- tail -f 초기 N 라인 seek 전략
- mv 크로스 볼륨 fallback 세부

## Deferred Ideas (CONTEXT.md §Deferred 참조)

- Full ACL 매핑 (v2)
- cp -p hidden/archive/system 속성 (v2)
- cp/mv --progress (v2)
- ls LS_COLORS 파싱 (v2)
- tail --pid=PID (v2)
- chmod --reference=file (v2)
- ln --backup, mv --backup (v2)
- rm --one-file-system (v2)
- dos2unix --info (v2)
- ls -l nlink count (v2 — Windows-specific 개선)

---

*Discussion: 2026-04-21*

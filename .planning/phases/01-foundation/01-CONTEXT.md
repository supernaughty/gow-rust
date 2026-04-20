# Phase 1: Foundation - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

gow-core 공유 라이브러리와 Cargo workspace 인프라를 구축한다. 모든 유틸리티 크레이트가 의존하는 플랫폼 프리미티브(UTF-8, GNU 인자 파싱, 컬러/TTY, 에러 처리, 경로 변환, 심링크 추상화)를 제공하며, Windows 10/11에서 MSVC 툴체인으로 빌드 가능해야 한다.

</domain>

<decisions>
## Implementation Decisions

### GNU 인자 파싱 전략
- **D-01:** clap 4 derive API 위에 GNU 호환 래퍼 레이어를 구축한다. uutils 접근 방식을 따른다.
- **D-02:** GNU exit code 규칙을 따른다 — 잘못된 인자는 exit code 1 (clap 기본값 2가 아님).
- **D-03:** 옵션 퍼뮤테이션을 지원한다 (예: `ls file -l`이 `ls -l file`과 동일하게 동작).
- **D-04:** `--` 이후 모든 인자를 비옵션으로 처리한다.
- **D-05:** 숫자 축약을 지원한다 (`head -5` = `head -n 5`, `tail -20` = `tail -n 20`).

### 경로 변환 설계
- **D-06:** 파일 인자로 해석되는 위치에서만 경로 변환을 적용한다. 플래그 값(-c 등)은 변환하지 않는다.
- **D-07:** MSYS 스타일 경로를 인식한다: `/c/Users/foo` → `C:\Users\foo`. Git Bash/MSYS2 사용자를 위한 편의 기능.
- **D-08:** 변환은 보수적��로 — 확실한 경우에만 변환하고, 모호한 경우 원본을 유지한다.

### 에러 처리 패턴
- **D-09:** gow-core 라이브러리는 `thiserror`로 타입화된 에러를 정의한다 (GowError enum).
- **D-10:** 각 유틸리티 바이너리는 `anyhow`로 에러를 전파하고 main()에서 GNU 형식��로 출력한다.
- **D-11:** 에러 메시지 형식: `{utility}: {message}` (GNU 관례 — 예: `cat: file.txt: No such file or directory`).

### 프로젝트 구조/네이밍
- **D-12:** Cargo workspace 구조: `crates/gow-core/`, `crates/gow-cat/`, `crates/gow-grep/` 등. `gow-` 접두사로 네임스페이스 분리.
- **D-13:** Rust 2024 에디션 사용, resolver = 3.
- **D-14:** 생성되는 실행 파일은 GNU 이름 그대로: `cat.exe`, `grep.exe`, `ls.exe`. 기존 스크립트 호환성 유지.
- **D-15:** workspace 수준에서 공통 의존성 버전 관�� (`[workspace.dependencies]`).

### Claude's Discretion
- gow-core 내부 모듈 구조 (encoding, args, color, path, error를 어떻게 나눌지)
- CI/CD 설정 세부사항
- 테스트 유틸리티/헬퍼 구조

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Context
- `.planning/PROJECT.md` — 프로젝트 비전, 제약조건, 핵심 결정사항
- `.planning/REQUIREMENTS.md` — Phase 1 요구사항: FOUND-01~07, WIN-01~03
- `.planning/ROADMAP.md` — Phase 1 성공 기준 5개

### Research
- `.planning/research/STACK.md` — 기술 스택 결정 (clap 4, windows-sys 0.61, termcolor, thiserror, anyhow)
- `.planning/research/ARCHITECTURE.md` — Cargo workspace 아키텍처 패턴, uutils 구조 참조
- `.planning/research/PITFALLS.md` — clap GNU 비호환성, Windows 코드페이지, 경로 변환 오탐 주의사항

### External Reference
- 원본 GOW 소스: `D:\workspace\gow-utilities-src-0.8.0` — 원본 유틸리��의 옵션/동작 참조용

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- 없음 (그린필드 프로젝트)

### Established Patterns
- 없음 (Phase 1에서 패턴 수립)

### Integration Points
- 모든 후속 Phase (2~6)가 gow-core에 의존함. Phase 1의 API가 전체 프로젝트의 기반.

</code_context>

<specifics>
## Specific Ideas

- uutils/coreutils의 uucore 구조를 참조하되, gow-rust 고유의 Windows 최적화(경로 변환, MSYS 호환)를 추가
- GOW 이슈 #244 (경로 변환 오류)를 Phase 1에서 설계적으로 해결 — 파일 인자 위치에서만 변환

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-04-20*

# gow-rust

## What This Is

GOW (Gnu On Windows) 유틸리티를 Rust로 재작성하는 오픈소스 프로젝트. 원본 GOW 0.8.0의 핵심 GNU 유틸리티들을 현대적인 Rust로 구현하여, Windows 환경에서 높은 GNU 호환성과 네이티브 Windows 통합(UTF-8, 경로 변환, PowerShell 연동)을 제공한다.

## Core Value

Windows 사용자가 별도의 무거운 환경(WSL, Cygwin) 없이 GNU 명령어를 네이티브 성능으로 사용할 수 있어야 한다.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] 핵심 coreutils 명령어를 Rust로 구현 (ls, cat, cp, mv, rm, mkdir, echo, head, tail, wc, sort, uniq, tr, tee, basename, dirname, pwd, env, yes, true, false)
- [ ] grep을 Rust로 구현 (정규식, 컬러 출력, 재귀 검색)
- [ ] sed를 Rust로 구현 (스트림 편집, 인플레이스 편집)
- [ ] find를 Rust로 구현 (파일 검색, -exec 지원, 와일드카드)
- [ ] diff/patch를 Rust로 구현
- [ ] gawk를 Rust로 구현
- [ ] tar/gzip/bzip2 압축 유틸리티를 Rust로 구현
- [ ] curl을 Rust로 구현 (HTTPS, 프록시, 현대적 TLS)
- [ ] less 페이저를 Rust로 구현
- [ ] which를 Rust로 구현 (Windows PATH 정확히 탐색)
- [ ] dos2unix/unix2dos를 Rust로 구현
- [ ] UTF-8을 기본 인코딩으로 지원 (GitHub 이슈 #280, #77 해결)
- [ ] Windows 경로 자동 변환 (Unix ↔ Windows, 이슈 #244, #246 해결)
- [ ] tail -f가 Windows에서 올바르게 동작 (이슈 #169, #75, #89 해결)
- [ ] grep --color가 Windows 터미널에서 동작 (이슈 #85 해결)
- [ ] find에서 공백 포함 경로 처리 (이슈 #209 해결)
- [ ] find -exec 올바르게 동작 (이슈 #208 해결)
- [ ] 각 유틸리티를 독립 바이너리로 빌드
- [ ] MSI 설치 프로그램으로 배포
- [ ] GNU 옵션 높은 호환성 (주요 플래그 대부분 지원)
- [ ] PowerShell 통합 지원

### Out of Scope

- vim/nano 에디터 재작성 — 기존 성숙한 Rust 대안(helix, zed 등) 존재
- bison/flex 파서 생성기 재작성 — 전문 도구이며 별도 프로젝트 규모
- make 빌드 시스템 재작성 — 기존 대안(just, cargo 등) 충분
- putty SSH 클라이언트 재작성 — Windows에 OpenSSH 내장
- bash 셸 자체 재작성 — 별도 프로젝트 규모 (nushell 등 대안)
- indent 코드 포매터 재작성 — 언어별 포매터가 더 우수
- jwhois 재작성 — 사용 빈도 낮음
- 실시간 채팅/IRC 클라이언트 추가 — GOW 범위 밖 (이슈 #131)
- PHP 추가 — GOW 범위 밖 (이슈 #167)

## Context

- 원본 소스: `D:\workspace\gow-utilities-src-0.8.0` (34개 유틸리티 소스)
- 원본 GOW GitHub: https://github.com/bmatzelle/gow (158개 미해결 이슈, 사실상 유지보수 중단)
- 주요 이슈 테마: 오래된 바이너리 버전, UTF-8 깨짐, 경로 변환 버그, 누락 기능, tail -f 등 기본 기능 결함
- Rust 생태계에 uutils/coreutils 프로젝트가 존재하지만, GOW의 Windows 최적화 + 설치 편의성에는 미치지 못함
- 대상: Windows 개발자/시스템 관리자로, WSL 없이 GNU 도구가 필요한 사용자

## Constraints

- **언어**: Rust (안정 채널, 최신 stable)
- **타겟 플랫폼**: Windows 10/11 x86_64 (MSVC 툴체인)
- **호환성**: GNU 옵션 높은 호환성 — 주요 플래그 대부분 지원하여 기존 스크립트가 동작
- **배포**: MSI 설치 프로그램, PATH 자동 등록
- **바이너리 구조**: 유틸리티별 독립 exe
- **인코딩**: UTF-8 기본, Windows 코드페이지 폴백 지원

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust 사용 | 메모리 안전성, 네이티브 성능, 크로스 컴파일, 현대적 에러 처리 | — Pending |
| 핵심 유틸리티 우선 | 가장 많이 사용되는 명령어부터 가치 전달, 이후 확장 | — Pending |
| 독립 바이너리 | 전통적 GOW 방식 유지, 개별 업데이트 가능, 사용자 익숙함 | — Pending |
| vim/make/bash 제외 | 별도 프로젝트 규모이며 성숙한 대안이 존재 | — Pending |
| MSI 배포 | 원본 GOW와 동일한 설치 경험, IT 관리자 친화적 | — Pending |
| Windows 특화 기능 추가 | 이슈 분석 결과 경로/인코딩 문제가 핵심 불만 — Rust로 근본 해결 | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-20 after initialization*

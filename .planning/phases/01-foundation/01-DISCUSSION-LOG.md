# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 01-foundation
**Areas discussed:** GNU 인자 파싱 전략, 경로 변환 설계, 에러 처리 패턴, 프로젝트 구조/네이밍

---

## GNU 인자 파싱 전략

| Option | Description | Selected |
|--------|-------------|----------|
| clap + GNU 래퍼 | clap 4 derive API 위에 exit code/퍼뮤테이션/숫자축약 래퍼. uutils 방식 | ✓ |
| lexopt 직접 사용 | 경량 파서로 완전한 제어. 더 많은 코드 필요 | |
| 자체 파서 구현 | GNU getopt 동작을 정확히 복제. 가장 많은 작업량 | |

**User's choice:** clap + GNU 래퍼 (Recommended)
**Notes:** 숫자 축약 (head -5, tail -20) 도 지원하기로 결정

---

## 경로 변환 설계

| Option | Description | Selected |
|--------|-------------|----------|
| 파일 인자만 | 파일 경로로 해석되는 인자만 변환. 플래그(-c 등)는 변환 안 함 | ✓ |
| 모든 인자 | /로 시작하는 모든 것을 변환 시도 (GOW 원본 방식 — 버그 원인) | |
| 명시적 API만 | 자동 변환 없음. 사용자가 Windows 경로를 직접 사용 | |

**User's choice:** 파일 인자만 (Recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| 네, 인식 | /c/Users → C:\Users 변환. Git Bash/MSYS2 사용자에게 유용 | ✓ |
| 아니오 | Windows 네이티��� 경로만 인식. 단순하지만 Git Bash에서 불편 | |

**User's choice:** MSYS 스타일 경로 인식 (Recommended)

---

## 에러 처리 패턴

| Option | Description | Selected |
|--------|-------------|----------|
| thiserror + anyhow | gow-core는 thiserror로 타입 정의, 바이너리는 anyhow로 간결하게. uutils 패턴 | ✓ |
| thiserror 단독 | 모든 곳에서 타입된 에러. 더 엄격하지만 보일러플레이트 많음 | |
| You decide | Claude가 최적의 패턴 결정 | |

**User's choice:** thiserror + anyhow (Recommended)

---

## 프로젝트 구조/네이밍

| Option | Description | Selected |
|--------|-------------|----------|
| gow-{name} + crates/ | crates/gow-cat/, crates/gow-grep/ 식. 명확한 네임스페이스 | ✓ |
| uu_{name} + src/uu/ | src/uu/cat/, src/uu/grep/ 식. uutils 동일 구조 | |
| 평탄한 구조 | cat/, grep/ 식. 단순하지만 충돌 위험 | |

**User's choice:** gow-{name} + crates/ (Recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| 2024 | 최신 에디션, resolver=3, 새 기능 활용 | ✓ |
| 2021 | 안정적, 더 많은 라이브러리 호환성 | |

**User's choice:** Rust 2024 에디션 (Recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| GNU 이름 그대로 | cat.exe, grep.exe, ls.exe — 기존 스크립트 호환 | ✓ |
| gow- 접두사 | gow-cat.exe, gow-grep.exe — 충돌 방지 | |
| g 접두사 | gcat.exe, ggrep.exe — macOS Homebrew GNU 관례 | |

**User's choice:** GNU 이름 그대로 (Recommended)

---

## Claude's Discretion

- gow-core 내부 모듈 구조
- CI/CD 설정 세부사항
- 테스트 유틸리티/헬퍼 구조

## Deferred Ideas

None

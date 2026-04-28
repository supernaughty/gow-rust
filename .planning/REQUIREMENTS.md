# Requirements

This file is the explicit capability and coverage contract for the project.

## Active

### R008 — 줄 정렬 (-n 숫자, -r 역순, -u 유니크, -k 키필드)
- Class: core-capability
- Status: active
- Description: 줄 정렬 (-n 숫자, -r 역순, -u 유니크, -k 키필드)
- Why it matters: 텍스트 처리 기본 도구
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: none
- Validation: mapped
- Notes: TEXT-04

### R009 — 중복 줄 제거/카운트 (-c 카운트, -d 중복만)
- Class: core-capability
- Status: active
- Description: 중복 줄 제거/카운트 (-c 카운트, -d 중복만)
- Why it matters: 텍스트 처리 기본 도구
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: none
- Validation: mapped
- Notes: TEXT-05

### R010 — 문자 변환/삭제 (-d 삭제, -s 압축, 문자 클래스)
- Class: core-capability
- Status: active
- Description: 문자 변환/삭제 (-d 삭제, -s 압축, 문자 클래스)
- Why it matters: 텍스트 처리 기본 도구
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: none
- Validation: mapped
- Notes: TEXT-06

### R011 — 정규식 패턴 검색 (-i 대소문자, -r 재귀, -n 줄번호, -c 카운트, -v 반전)
- Class: core-capability
- Status: active
- Description: 정규식 패턴 검색 (-i 대소문자, -r 재귀, -n 줄번호, -c 카운트, -v 반전)
- Why it matters: 핵심 텍스트 검색 도구, --color 지원 (GOW #85)
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: none
- Validation: mapped
- Notes: GREP-01, GREP-02, GREP-03

### R012 — 스트림 편집 (s/치환, d/삭제, p/출력, 주소 범위), sed -i 인플레이스 편집
- Class: core-capability
- Status: active
- Description: 스트림 편집 (s/치환, d/삭제, p/출력, 주소 범위), sed -i 인플레이스 편집
- Why it matters: 핵심 스트림 편집 도구, Windows 파일 잠금 처리 필요
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: none
- Validation: mapped
- Notes: SED-01, SED-02

### R013 — 완전한 AWK 인터프리터, 필드 분리, printf, 연관 배열 지원
- Class: core-capability
- Status: active
- Description: 완전한 AWK 인터프리터, 필드 분리, printf, 연관 배열 지원
- Why it matters: 강력한 텍스트 분석 도구
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: none
- Validation: mapped
- Notes: AWK-01, AWK-02

### R014 — 파일 비교 및 패치 적용
- Class: core-capability
- Status: validated
- Description: 파일 비교 및 패치 적용
- Why it matters: 코드 및 텍스트 변경 관리
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: none
- Validation: validated in M001/S04 (Plan 04-06) with 11 integration tests covering identical files, unified diff format, context lines, recursive diff, -N absent-as-empty, patch apply, -p1 strip, --dry-run, -R reverse, and -i input file.
- Notes: DIFF-01, DIFF-02

### R015 — 파일 검색 (-name, -type, -size, -mtime 등), -exec 지원, 공백 처리
- Class: core-capability
- Status: validated
- Description: 파일 검색 (-name, -type, -size, -mtime 등), -exec 지원, 공백 처리
- Why it matters: 파일 탐색 핵심 도구
- Source: user
- Primary owning slice: M001/S05
- Supporting slices: none
- Validation: validated in M001/S05 (Plan 05-02) with 15 unit + 13 integration tests covering -name/-iname/-type/-size/-mtime/-maxdepth/-exec (incl. paths with spaces, GOW #209)/-print0. Cross-binary find -print0 | xargs -0 pipeline verified.
- Notes: FIND-01, FIND-02, FIND-03

### R016 — 표준 입력에서 명령줄 구성 (-0 null 구분, -I 치환)
- Class: core-capability
- Status: validated
- Description: 표준 입력에서 명령줄 구성 (-0 null 구분, -I 치환)
- Why it matters: 파이프라인 연동 핵심 도구
- Source: user
- Primary owning slice: M001/S05
- Supporting slices: none
- Validation: validated in M001/S05 (Plan 05-03) with 11 unit + 8 integration tests covering -0/-I {}/-n/-L, GNU-compatible exit codes (0/123/124/125), and cross-binary find -print0 | xargs -0 pipeline on Windows.
- Notes: XARGS-01

### R017 — 파일 페이저 (스크롤, 검색, 큰 파일 지원)
- Class: core-capability
- Status: validated
- Description: 파일 페이저 (스크롤, 검색, 큰 파일 지원)
- Why it matters: 터미널 텍스트 가독성 도구
- Source: user
- Primary owning slice: M001/S05
- Supporting slices: none
- Validation: validated in M001/S05 (Plan 05-04) with 7 unit + 7 integration tests covering non-TTY passthrough, ANSI byte-exact passthrough (D-08), large-file no-OOM (D-09, 1 MiB), Unicode content, and missing-file error. Interactive TTY behavior (scroll, search, g/G, clean exit) verified by human UAT 2026-04-28.
- Notes: LESS-01

### R018 — 아카이브 생성/추출 (-c 생성, -x 추출, -t 목록, -z gzip, -j bzip2)
- Class: core-capability
- Status: active
- Description: 아카이브 생성/추출 (-c 생성, -x 추출, -t 목록, -z gzip, -j bzip2)
- Why it matters: 파일 아카이브 도구
- Source: user
- Primary owning slice: M001/S06
- Supporting slices: none
- Validation: mapped
- Notes: ARCH-01

### R019 — 압축/해제 도구 세트
- Class: core-capability
- Status: active
- Description: 압축/해제 도구 세트
- Why it matters: 표준 압축 형식 지원
- Source: user
- Primary owning slice: M001/S06
- Supporting slices: none
- Validation: mapped
- Notes: ARCH-02, ARCH-03

### R020 — HTTP/HTTPS 요청, TLS 1.2/1.3 지원, 프록시 인증
- Class: core-capability
- Status: active
- Description: HTTP/HTTPS 요청, TLS 1.2/1.3 지원, 프록시 인증
- Why it matters: 네트워크 데이터 전송 도구
- Source: user
- Primary owning slice: M001/S06
- Supporting slices: none
- Validation: mapped
- Notes: NET-01, NET-02

## Validated

### R001 — 디렉토리 목록 (-l 상세, -a 숨김, -R 재귀, --color 색상)
- Class: core-capability
- Status: validated
- Description: 디렉토리 목록 (-l 상세, -a 숨김, -R 재귀, --color 색상)
- Why it matters: GNU coreutils의 기본 기능
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated in M001/S03 (Plan 03-09) with 19 integration tests covering hidden files, permissions, and junction display.
- Notes: FILE-02

### R002 — 파일 복사 (-r 재귀, -f 강제, -p 권한 보존)
- Class: core-capability
- Status: validated
- Description: 파일 복사 (-r 재귀, -f 강제, -p 권한 보존)
- Why it matters: GNU coreutils의 기본 기능
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated in M001/S03 (Plan 03-07) with 16 integration tests covering recursive copy, timestamps, and symlinks.
- Notes: FILE-03

### R003 — 파일 이동/이름변경 (-f 강제, -i 대화형)
- Class: core-capability
- Status: validated
- Description: 파일 이동/이름변경 (-f 강제, -i 대화형)
- Why it matters: GNU coreutils의 기본 기능
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated in M001/S03 (Plan 03-11) with 12 integration tests covering same-volume rename and directory moves.
- Notes: FILE-04

### R004 — 파일 삭제 (-r 재귀, -f 강제, -i 대화형)
- Class: core-capability
- Status: validated
- Description: 파일 삭제 (-r 재귀, -f 강제, -i 대화형)
- Why it matters: GNU coreutils의 기본 기능
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated in M001/S03 (Plan 03-08) with 18 integration tests covering recursive removal, drive-root safety, and read-only handling.
- Notes: FILE-05

### R005 — 링크 생성 (-s 심볼릭, Windows 심링크/정션 지원)
- Class: core-capability
- Status: validated
- Description: 링크 생성 (-s 심볼릭, Windows 심링크/정션 지원)
- Why it matters: GNU coreutils의 기본 기능
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated in M001/S03 (Plan 03-10) with 12 integration tests covering hard links, symlinks, and junction fallbacks.
- Notes: FILE-09

### R006 — 파일 뒷부분 출력 (-n 줄수, -f 실시간 추적, GOW #169/#75/#89 해결)
- Class: core-capability
- Status: validated
- Description: 파일 뒷부분 출력 (-n 줄수, -f 실시간 추적, GOW #169/#75/#89 해결)
- Why it matters: GNU coreutils의 기본 기능, Windows에서 tail -f가 안정적으로 동작해야 함
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated in M001/S03 (Plan 03-12) with 12 integration tests covering last-N lines/bytes and real-time follow via notify.
- Notes: TEXT-02

### R007 — LF→CRLF 변환
- Class: core-capability
- Status: validated
- Description: LF→CRLF 변환
- Why it matters: Windows 환경에서의 텍스트 인코딩 호환성
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated in M001/S03 (Plan 03-06) with 12 integration tests covering LF to CRLF conversion.
- Notes: CONV-02

### R100 — Cargo workspace 구조로 다중 크레이트 프로젝트 구성
- Class: quality-attribute
- Status: validated
- Description: Cargo workspace 구조로 다중 크레이트 프로젝트 구성
- Why it matters: 프로젝트 확장성 및 공유 라이브러리 구조
- Source: user
- Primary owning slice: M001/S01
- Supporting slices: none
- Validation: validated
- Notes: FOUND-01, Plan 01-01

### R101 — gow-core 공유 라이브러리 — UTF-8 콘솔 초기화 (SetConsoleOutputCP 65001)
- Class: quality-attribute
- Status: validated
- Description: gow-core 공유 라이브러리 — UTF-8 콘솔 초기화 (SetConsoleOutputCP 65001)
- Why it matters: 한글 등 비ASCII 문자 지원을 위한 필수 조건
- Source: user
- Primary owning slice: M001/S01
- Supporting slices: none
- Validation: validated
- Notes: FOUND-02, Plan 01-02, WIN-01

### R102 — gow-core 공유 라이브러리 — GNU 호환 인자 파싱 (옵션 퍼뮤테이션, exit code 1, -- 종료)
- Class: differentiator
- Status: validated
- Description: gow-core 공유 라이브러리 — GNU 호환 인자 파싱 (옵션 퍼뮤테이션, exit code 1, -- 종료)
- Why it matters: 기존 GNU 스크립트 호환성을 위한 핵심
- Source: user
- Primary owning slice: M001/S01
- Supporting slices: none
- Validation: validated
- Notes: FOUND-03, Plan 01-02

### R103 — gow-core 공유 라이브러리 — 컬러/TTY 감지 및 ANSI VT100 활성화
- Class: quality-attribute
- Status: validated
- Description: gow-core 공유 라이브러리 — 컬러/TTY 감지 및 ANSI VT100 활성화
- Why it matters: Windows 터미널에서 컬러 출력 지원
- Source: user
- Primary owning slice: M001/S01
- Supporting slices: none
- Validation: validated
- Notes: FOUND-04, Plan 01-02

### R104 — Unix↔Windows 경로 자동 변환 (컨텍스트 인식, GOW #244 해결)
- Class: core-capability
- Status: validated
- Description: Unix↔Windows 경로 자동 변환 (컨텍스트 인식, GOW #244 해결)
- Why it matters: MSYS/Cygwin 스타일 경로 지원
- Source: user
- Primary owning slice: M001/S01
- Supporting slices: none
- Validation: validated
- Notes: FOUND-06, Plan 01-03

### R105 — 문자열 출력 (-n 개행 없이, -e 이스케이프 해석)
- Class: core-capability
- Status: validated
- Description: 문자열 출력 (-n 개행 없이, -e 이스케이프 해석)
- Why it matters: UTIL-01, Plan 02-03
- Source: user
- Primary owning slice: M001/S02
- Supporting slices: none
- Validation: validated
- Notes: validated in Plan 02-03

### R106 — 단어/줄/바이트 수 카운트, Unicode-aware via bstr
- Class: core-capability
- Status: validated
- Description: 단어/줄/바이트 수 카운트, Unicode-aware via bstr
- Why it matters: TEXT-03, Plan 02-08
- Source: user
- Primary owning slice: M001/S02
- Supporting slices: none
- Validation: validated
- Notes: validated in Plan 02-08

### R107 — 실행파일 위치 탐색 (Windows PATH 정확히 탐색, GOW #276 해결)
- Class: core-capability
- Status: validated
- Description: 실행파일 위치 탐색 (Windows PATH 정확히 탐색, GOW #276 해결)
- Why it matters: WHICH-01, Plan 02-11
- Source: user
- Primary owning slice: M001/S02
- Supporting slices: none
- Validation: validated
- Notes: validated in Plan 02-11

### R108 — 파일 연결 및 표준 출력 (-n 번호, -b 비공백 번호, -s 빈줄 압축)
- Class: core-capability
- Status: validated
- Description: 파일 연결 및 표준 출력 (-n 번호, -b 비공백 번호, -s 빈줄 압축)
- Why it matters: FILE-01, Plan 03-02
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated
- Notes: validated in Plan 03-02

### R109 — 파일 앞부분 출력 (-n 줄수, -c 바이트수, 숫자 축약 -5)
- Class: core-capability
- Status: validated
- Description: 파일 앞부분 출력 (-n 줄수, -c 바이트수, 숫자 축약 -5)
- Why it matters: TEXT-01, Plan 03-03
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated
- Notes: validated in Plan 03-03

### R110 — 파일 권한 변경 (Windows ACL 매핑)
- Class: core-capability
- Status: validated
- Description: 파일 권한 변경 (Windows ACL 매핑)
- Why it matters: FILE-10, Plan 03-04
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated
- Notes: validated in Plan 03-04

### R111 — CRLF→LF 변환, atomic rewrite 지원
- Class: core-capability
- Status: validated
- Description: CRLF→LF 변환, atomic rewrite 지원
- Why it matters: CONV-01, Plan 03-05
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: none
- Validation: validated
- Notes: validated in Plan 03-05

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | core-capability | validated | M001/S03 | none | validated in M001/S03 (Plan 03-09) with 19 integration tests covering hidden files, permissions, and junction display. |
| R002 | core-capability | validated | M001/S03 | none | validated in M001/S03 (Plan 03-07) with 16 integration tests covering recursive copy, timestamps, and symlinks. |
| R003 | core-capability | validated | M001/S03 | none | validated in M001/S03 (Plan 03-11) with 12 integration tests covering same-volume rename and directory moves. |
| R004 | core-capability | validated | M001/S03 | none | validated in M001/S03 (Plan 03-08) with 18 integration tests covering recursive removal, drive-root safety, and read-only handling. |
| R005 | core-capability | validated | M001/S03 | none | validated in M001/S03 (Plan 03-10) with 12 integration tests covering hard links, symlinks, and junction fallbacks. |
| R006 | core-capability | validated | M001/S03 | none | validated in M001/S03 (Plan 03-12) with 12 integration tests covering last-N lines/bytes and real-time follow via notify. |
| R007 | core-capability | validated | M001/S03 | none | validated in M001/S03 (Plan 03-06) with 12 integration tests covering LF to CRLF conversion. |
| R008 | core-capability | active | M001/S04 | none | mapped |
| R009 | core-capability | active | M001/S04 | none | mapped |
| R010 | core-capability | active | M001/S04 | none | mapped |
| R011 | core-capability | active | M001/S04 | none | mapped |
| R012 | core-capability | active | M001/S04 | none | mapped |
| R013 | core-capability | active | M001/S04 | none | mapped |
| R014 | core-capability | validated | M001/S04 | none | validated in M001/S04 (Plan 04-06) with 11 integration tests |
| R015 | core-capability | validated | M001/S05 | none | validated in M001/S05 (Plan 05-02) with 28 tests — find predicates, -exec, -print0, GOW #209 regression |
| R016 | core-capability | validated | M001/S05 | none | validated in M001/S05 (Plan 05-03) with 19 tests — xargs -0/-I/-n/-L, exit codes, cross-binary pipeline |
| R017 | core-capability | validated | M001/S05 | none | validated in M001/S05 (Plan 05-04) with 14 tests + human UAT 2026-04-28 — less pager, ANSI, large-file |
| R018 | core-capability | active | M001/S06 | none | mapped |
| R019 | core-capability | active | M001/S06 | none | mapped |
| R020 | core-capability | active | M001/S06 | none | mapped |
| R100 | quality-attribute | validated | M001/S01 | none | validated |
| R101 | quality-attribute | validated | M001/S01 | none | validated |
| R102 | differentiator | validated | M001/S01 | none | validated |
| R103 | quality-attribute | validated | M001/S01 | none | validated |
| R104 | core-capability | validated | M001/S01 | none | validated |
| R105 | core-capability | validated | M001/S02 | none | validated |
| R106 | core-capability | validated | M001/S02 | none | validated |
| R107 | core-capability | validated | M001/S02 | none | validated |
| R108 | core-capability | validated | M001/S03 | none | validated |
| R109 | core-capability | validated | M001/S03 | none | validated |
| R110 | core-capability | validated | M001/S03 | none | validated |
| R111 | core-capability | validated | M001/S03 | none | validated |

## Coverage Summary

- Active requirements: 10
- Mapped to slices: 10
- Validated: 22 (R001, R002, R003, R004, R005, R006, R007, R015, R016, R017, R100, R101, R102, R103, R104, R105, R106, R107, R108, R109, R110, R111)
- Unmapped active requirements: 0

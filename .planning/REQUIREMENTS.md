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

## M002 Requirements

### REL-01 — git tag v0.1.0 + GitHub Release with x64/x86 MSI files attached
- Class: delivery
- Status: active
- Description: Create git tag v0.1.0 and publish a GitHub Release with x64 and x86 MSI installer files attached as release assets
- Why it matters: First public release; users need downloadable installer to adopt gow-rust
- Source: user
- Primary owning slice: M002/Phase07
- Supporting slices: none
- Validation: pending
- Notes: REL-01

### REL-02 — ARM64 installer build requirements documented in README/CONTRIBUTING.md
- Class: delivery
- Status: active
- Description: Document ARM64 build prerequisites and cross-compilation steps in README or CONTRIBUTING.md
- Why it matters: ARM64 Windows devices exist; community contributors need build guidance
- Source: user
- Primary owning slice: M002/Phase07
- Supporting slices: none
- Validation: pending
- Notes: REL-02

### REL-03 — gow-probe.exe excluded from installer staging
- Class: fix
- Status: active
- Description: Ensure the gow-probe diagnostic binary is not bundled into the MSI installer
- Why it matters: gow-probe is a development tool, not a user-facing utility; pollutes user install
- Source: user
- Primary owning slice: M002/Phase07
- Supporting slices: none
- Validation: pending
- Notes: REL-03

### CI-01 — GitHub Actions workflow: cargo test --workspace on every push/PR
- Class: quality-attribute
- Status: active
- Description: Automated CI runs cargo test --workspace on every push and pull request
- Why it matters: Prevents regressions; required for safe open-source collaboration
- Source: user
- Primary owning slice: M002/Phase07
- Supporting slices: none
- Validation: pending
- Notes: CI-01

### CI-02 — GitHub Actions release workflow: on git tag v*, build x64+x86 release MSIs
- Class: delivery
- Status: active
- Description: A tag-triggered GitHub Actions workflow builds x64 and x86 MSI installers in release mode
- Why it matters: Automates release artifact creation; eliminates manual build steps
- Source: user
- Primary owning slice: M002/Phase07
- Supporting slices: none
- Validation: pending
- Notes: CI-02

### CI-03 — GitHub Actions release workflow: attach built MSIs to GitHub Release automatically
- Class: delivery
- Status: active
- Description: The release workflow uploads the built MSI artifacts to the GitHub Release via gh CLI or actions/upload-release-asset
- Why it matters: Completes the automated release pipeline end-to-end
- Source: user
- Primary owning slice: M002/Phase07
- Supporting slices: none
- Validation: pending
- Notes: CI-03

### FIX-01 — tar uses MultiBzDecoder instead of BzDecoder (multi-stream bzip2 support)
- Class: fix
- Status: active
- Description: Replace BzDecoder with MultiBzDecoder in gow-tar so concatenated .tar.bz2 archives decode fully
- Why it matters: Fixes WR-01 code review finding; truncated extraction is data-loss behavior
- Source: user
- Primary owning slice: M002/Phase08
- Supporting slices: none
- Validation: pending
- Notes: FIX-01, WR-01

### FIX-02 — tar Cli::from_arg_matches graceful error instead of .unwrap() panic
- Class: fix
- Status: active
- Description: Replace .unwrap() with proper error handling and exit 2 on argument parsing failure in gow-tar
- Why it matters: Fixes WR-02; panic on bad CLI args is user-hostile
- Source: user
- Primary owning slice: M002/Phase08
- Supporting slices: none
- Validation: pending
- Notes: FIX-02, WR-02

### FIX-03 — tar extraction returns non-zero exit code on per-entry errors
- Class: fix
- Status: active
- Description: gow-tar exits non-zero when any archive entry fails to extract, not just on fatal errors
- Why it matters: Fixes WR-03; silent partial extraction violates GNU tar behavior
- Source: user
- Primary owning slice: M002/Phase08
- Supporting slices: none
- Validation: pending
- Notes: FIX-03, WR-03

### FIX-04 — xz uses XzDecoder with multi-stream support for concatenated .xz files
- Class: fix
- Status: active
- Description: Update gow-xz to handle concatenated .xz streams so all data is decompressed
- Why it matters: Fixes WR-04; silently truncated output is a correctness bug
- Source: user
- Primary owning slice: M002/Phase08
- Supporting slices: none
- Validation: pending
- Notes: FIX-04, WR-04

### FIX-05 — gzip rejects files without .gz suffix (GNU-compatible error message)
- Class: fix
- Status: active
- Description: gow-gzip errors out with a GNU-compatible message when decompressing a file without .gz extension
- Why it matters: Fixes WR-05; current behavior silently produces .out files, breaking scripts
- Source: user
- Primary owning slice: M002/Phase08
- Supporting slices: none
- Validation: pending
- Notes: FIX-05, WR-05

### FIX-06 — curl -I -s suppresses header output in silent mode
- Class: fix
- Status: active
- Description: gow-curl in silent mode (-s) suppresses header output even when -I is passed
- Why it matters: Fixes WR-06; current behavior prints headers despite -s, breaking scripts
- Source: user
- Primary owning slice: M002/Phase08
- Supporting slices: none
- Validation: pending
- Notes: FIX-06, WR-06

### FIX-07 — curl -o removes partial output file on I/O error
- Class: fix
- Status: active
- Description: gow-curl cleans up the partially written output file when an I/O error occurs during download
- Why it matters: Fixes WR-07; partial files cause silent downstream failures
- Source: user
- Primary owning slice: M002/Phase08
- Supporting slices: none
- Validation: pending
- Notes: FIX-07, WR-07

### BND-01 — download-extras.ps1 script downloads vim portable (v9.2+), wget (v1.21.4), nano portable (v7.2+)
- Class: core-capability
- Status: active
- Description: A PowerShell script downloads versioned vim portable, wget, and nano portable binaries to a local staging directory
- Why it matters: Enables bundling third-party editors/tools without redistribution license issues
- Source: user
- Primary owning slice: M002/Phase09
- Supporting slices: none
- Validation: pending
- Notes: BND-01

### BND-02 — Extras staged to extras/bin/ directory, included in MSI alongside Rust binaries
- Class: core-capability
- Status: active
- Description: Downloaded extras land in extras/bin/ and WiX installer includes them as a separate component group
- Why it matters: Users get vim/nano/wget via the standard installer with no extra steps
- Source: user
- Primary owning slice: M002/Phase09
- Supporting slices: none
- Validation: pending
- Notes: BND-02

### BND-03 — egrep.bat, fgrep.bat, bunzip2.bat, gawk.bat, gfind.bat, gsort.bat batch aliases created
- Class: core-capability
- Status: active
- Description: Batch file shims are created so legacy names (egrep, fgrep, bunzip2, gawk, gfind, gsort) invoke the corresponding Rust binary
- Why it matters: Preserves GOW 0.8.0 naming compatibility so existing scripts keep working
- Source: user
- Primary owning slice: M002/Phase09
- Supporting slices: none
- Validation: pending
- Notes: BND-03

### BND-04 — Installer supports optional "Extras" feature (vim/nano/wget can be deselected)
- Class: core-capability
- Status: active
- Description: WiX installer exposes an optional "Extras" feature that users can deselect during installation
- Why it matters: Power users or minimal installs should not be forced to include large third-party binaries
- Source: user
- Primary owning slice: M002/Phase09
- Supporting slices: none
- Validation: pending
- Notes: BND-04

### U-01 — seq (number sequences: seq 10, seq 1 2 10)
- Class: core-capability
- Status: active
- Description: Implement gow-seq supporting seq LAST, seq FIRST LAST, seq FIRST INCREMENT LAST
- Why it matters: Widely used in shell scripts for iteration
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-01

### U-02 — sleep (delay: sleep 1, sleep 0.5)
- Class: core-capability
- Status: active
- Description: Implement gow-sleep supporting integer and fractional seconds
- Why it matters: Essential for scripted delays and retry loops
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-02

### U-03 — tac (reverse cat)
- Class: core-capability
- Status: active
- Description: Implement gow-tac: output lines in reverse order
- Why it matters: Common text processing utility
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-03

### U-04 — nl (number lines)
- Class: core-capability
- Status: active
- Description: Implement gow-nl with -b, -n, -w, -s options for line numbering
- Why it matters: Standard text formatting utility
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-04

### U-05 — od (octal/hex dump)
- Class: core-capability
- Status: active
- Description: Implement gow-od with -A, -t, -N options for octal and hex file dumps
- Why it matters: Essential for binary file inspection
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-05

### U-06 — fold (line wrapping)
- Class: core-capability
- Status: active
- Description: Implement gow-fold with -w width and -s word-boundary options
- Why it matters: Text formatting and pipeline utility
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-06

### U-07 — expand/unexpand (tab conversion)
- Class: core-capability
- Status: active
- Description: Implement gow-expand (tabs to spaces) and gow-unexpand (spaces to tabs) with -t tabstop option
- Why it matters: Source code and text formatting compatibility
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-07

### U-08 — du (disk usage)
- Class: core-capability
- Status: active
- Description: Implement gow-du with -s, -h, -a, -d options for disk usage reporting
- Why it matters: Essential system administration tool
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-08

### U-09 — df (disk free)
- Class: core-capability
- Status: active
- Description: Implement gow-df with -h human-readable output for disk space reporting
- Why it matters: Essential system administration tool
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-09

### U-10 — md5sum/sha1sum/sha256sum (hash utilities)
- Class: core-capability
- Status: active
- Description: Implement gow-md5sum, gow-sha1sum, gow-sha256sum with -c check mode
- Why it matters: File integrity verification, widely used in CI and packaging
- Source: user
- Primary owning slice: M002/Phase10
- Supporting slices: none
- Validation: pending
- Notes: U-10

### U2-01 — whoami (current user)
- Class: core-capability
- Status: active
- Description: Implement gow-whoami: print current username
- Why it matters: Common scripting utility; uses Windows API
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-01

### U2-02 — uname (system info: uname -a, -s, -r)
- Class: core-capability
- Status: active
- Description: Implement gow-uname with -a, -s, -r, -m options reporting Windows OS info
- Why it matters: Scripts rely on uname to detect OS; Windows-native output needed
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-02

### U2-03 — paste (column merge)
- Class: core-capability
- Status: active
- Description: Implement gow-paste for merging lines from multiple files side by side
- Why it matters: Text processing pipeline utility
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-03

### U2-04 — join (field-based join)
- Class: core-capability
- Status: active
- Description: Implement gow-join for relational joining of sorted files on a common field
- Why it matters: Data processing utility used in shell pipelines
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-04

### U2-05 — split (file splitting)
- Class: core-capability
- Status: active
- Description: Implement gow-split with -b bytes, -l lines, -n chunks options
- Why it matters: Large file handling; widely used in data pipelines
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-05

### U2-06 — printf (formatted output)
- Class: core-capability
- Status: active
- Description: Implement gow-printf supporting format strings with %d, %s, %f, \n, \t escapes
- Why it matters: Portable alternative to echo; used extensively in POSIX scripts
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-06

### U2-07 — expr (expression evaluation)
- Class: core-capability
- Status: active
- Description: Implement gow-expr for arithmetic and string expression evaluation
- Why it matters: Used in legacy shell scripts for arithmetic without bash arithmetic
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-07

### U2-08 — test/[ (condition evaluation)
- Class: core-capability
- Status: active
- Description: Implement gow-test (and [ alias) for POSIX condition evaluation (-f, -d, -z, -n, comparison operators)
- Why it matters: Foundation of POSIX shell scripting; required for scripts that invoke test as external command
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-08

### U2-09 — fmt (text formatting)
- Class: core-capability
- Status: active
- Description: Implement gow-fmt for paragraph-aware line wrapping with -w width option
- Why it matters: Text formatting utility used in documentation pipelines
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-09

### U2-10 — unlink (file removal by fd)
- Class: core-capability
- Status: active
- Description: Implement gow-unlink for removing a single file by path (POSIX unlink syscall semantics)
- Why it matters: Explicit single-file removal without rm flags; used in some POSIX scripts
- Source: user
- Primary owning slice: M002/Phase11
- Supporting slices: none
- Validation: pending
- Notes: U2-10

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
| REL-01 | delivery | active | M002/Phase07 | none | pending |
| REL-02 | delivery | active | M002/Phase07 | none | pending |
| REL-03 | fix | active | M002/Phase07 | none | pending |
| CI-01 | quality-attribute | active | M002/Phase07 | none | pending |
| CI-02 | delivery | active | M002/Phase07 | none | pending |
| CI-03 | delivery | active | M002/Phase07 | none | pending |
| FIX-01 | fix | active | M002/Phase08 | none | pending |
| FIX-02 | fix | active | M002/Phase08 | none | pending |
| FIX-03 | fix | active | M002/Phase08 | none | pending |
| FIX-04 | fix | active | M002/Phase08 | none | pending |
| FIX-05 | fix | active | M002/Phase08 | none | pending |
| FIX-06 | fix | active | M002/Phase08 | none | pending |
| FIX-07 | fix | active | M002/Phase08 | none | pending |
| BND-01 | core-capability | active | M002/Phase09 | none | pending |
| BND-02 | core-capability | active | M002/Phase09 | none | pending |
| BND-03 | core-capability | active | M002/Phase09 | none | pending |
| BND-04 | core-capability | active | M002/Phase09 | none | pending |
| U-01 | core-capability | active | M002/Phase10 | none | pending |
| U-02 | core-capability | active | M002/Phase10 | none | pending |
| U-03 | core-capability | active | M002/Phase10 | none | pending |
| U-04 | core-capability | active | M002/Phase10 | none | pending |
| U-05 | core-capability | active | M002/Phase10 | none | pending |
| U-06 | core-capability | active | M002/Phase10 | none | pending |
| U-07 | core-capability | active | M002/Phase10 | none | pending |
| U-08 | core-capability | active | M002/Phase10 | none | pending |
| U-09 | core-capability | active | M002/Phase10 | none | pending |
| U-10 | core-capability | active | M002/Phase10 | none | pending |
| U2-01 | core-capability | active | M002/Phase11 | none | pending |
| U2-02 | core-capability | active | M002/Phase11 | none | pending |
| U2-03 | core-capability | active | M002/Phase11 | none | pending |
| U2-04 | core-capability | active | M002/Phase11 | none | pending |
| U2-05 | core-capability | active | M002/Phase11 | none | pending |
| U2-06 | core-capability | active | M002/Phase11 | none | pending |
| U2-07 | core-capability | active | M002/Phase11 | none | pending |
| U2-08 | core-capability | active | M002/Phase11 | none | pending |
| U2-09 | core-capability | active | M002/Phase11 | none | pending |
| U2-10 | core-capability | active | M002/Phase11 | none | pending |

## Coverage Summary

- M001 active requirements: 10
- M001 mapped to slices: 10
- M001 validated: 22 (R001, R002, R003, R004, R005, R006, R007, R015, R016, R017, R100, R101, R102, R103, R104, R105, R106, R107, R108, R109, R110, R111)
- M001 unmapped active requirements: 0
- M002 requirements: 30 (REL-01–03, CI-01–03, FIX-01–07, BND-01–04, U-01–10, U2-01–10)
- M002 mapped to phases: 30
- M002 unmapped: 0

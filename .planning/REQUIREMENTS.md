# Requirements: gow-rust

**Defined:** 2026-04-20
**Core Value:** Windows 사용자가 별도의 무거운 환경 없이 GNU 명령어를 네이티브 성능으로 사용할 수 있어야 한다.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Foundation

- [x] **FOUND-01**: Cargo workspace 구조로 다중 크레이트 프로젝트 구성 — completed Plan 01-01 (2026-04-20)
- [x] **FOUND-02**: gow-core 공유 라이브러리 — UTF-8 콘솔 초기화 (SetConsoleOutputCP 65001) — completed Plan 01-02 (2026-04-20)
- [x] **FOUND-03**: gow-core 공유 라이브러리 — GNU 호환 인자 파싱 (옵션 퍼뮤테이션, exit code 1, -- 종료) — completed Plan 01-02 (2026-04-20)
- [x] **FOUND-04**: gow-core 공유 라이브러리 — 컬러/TTY 감지 및 ANSI VT100 활성화 — completed Plan 01-02 (2026-04-20)
- [x] **FOUND-05**: gow-core 공유 라이브러리 — 통합 에러 처리 타입 — completed Plan 01-03 (2026-04-20)
- [x] **FOUND-06**: Unix↔Windows 경로 자동 변환 (컨텍스트 인식, GOW #244 해결) — completed Plan 01-03 (2026-04-20)
- [x] **FOUND-07**: Windows 심볼릭 링크/정션 추상화 레이어 — completed Plan 01-03 (2026-04-20)

### Coreutils — File Operations

- [ ] **FILE-01**: cat — 파일 연결 및 표준 출력 (-n 번호, -b 비공백 번호, -s 빈줄 압축)
- [ ] **FILE-02**: ls — 디렉토리 목록 (-l 상세, -a 숨김, -R 재귀, --color 색상)
- [ ] **FILE-03**: cp — 파일 복사 (-r 재귀, -f 강제, -p 권한 보존)
- [ ] **FILE-04**: mv — 파일 이동/이름변경 (-f 강제, -i 대화형)
- [ ] **FILE-05**: rm — 파일 삭제 (-r 재귀, -f 강제, -i 대화형)
- [ ] **FILE-06**: mkdir — 디렉토리 생성 (-p 부모 포함, GOW #133 해결)
- [ ] **FILE-07**: rmdir — 빈 디렉토리 삭제 (-p 부모 포함)
- [ ] **FILE-08**: touch — 파일 타임스탬프 변경/생성
- [ ] **FILE-09**: ln — 링크 생성 (-s 심볼릭, Windows 심링크/정션 지원)
- [ ] **FILE-10**: chmod — 파일 권한 변경 (Windows ACL 매핑)

### Coreutils — Text Processing

- [ ] **TEXT-01**: head — 파일 앞부분 출력 (-n 줄수, -c 바이트수, 숫자 축약 -5)
- [ ] **TEXT-02**: tail — 파일 뒷부분 출력 (-n 줄수, -f 실시간 추적, GOW #169/#75/#89 해결)
- [ ] **TEXT-03**: wc — 단어/줄/바이트 수 카운트
- [ ] **TEXT-04**: sort — 줄 정렬 (-n 숫자, -r 역순, -u 유니크, -k 키필드)
- [ ] **TEXT-05**: uniq — 중복 줄 제거/카운트 (-c 카운트, -d 중복만)
- [ ] **TEXT-06**: tr — 문자 변환/삭제 (-d 삭제, -s 압축, 문자 클래스)

### Coreutils — Utilities

- [ ] **UTIL-01**: echo — 문자열 출력 (-n 개행 없이, -e 이스케이프 해석)
- [ ] **UTIL-02**: pwd — 현재 디렉토리 출력 (-P 물리 경로)
- [ ] **UTIL-03**: env — 환경변수 출력/설정 후 명령 실행
- [ ] **UTIL-04**: tee — 표준 입력을 파일과 표준 출력에 동시 기록 (-a 추가)
- [ ] **UTIL-05**: basename — 경로에서 파일명 추출
- [ ] **UTIL-06**: dirname — 경로에서 디렉토리 추출
- [ ] **UTIL-07**: yes — 무한 반복 문자열 출력
- [ ] **UTIL-08**: true — 항상 성공 (exit 0)
- [ ] **UTIL-09**: false — 항상 실패 (exit 1)

### Text Search and Processing

- [ ] **GREP-01**: grep — 정규식 패턴 검색 (-i 대소문자, -r 재귀, -n 줄번호, -c 카운트, -v 반전)
- [ ] **GREP-02**: grep --color가 Windows 터미널에서 올바르게 동작 (GOW #85 해결)
- [ ] **GREP-03**: grep -E (확장 정규식) 및 grep -F (고정 문자열) 지원
- [ ] **SED-01**: sed — 스트림 편집 (s/치환, d/삭제, p/출력, 주소 범위)
- [ ] **SED-02**: sed -i 인플레이스 편집 (Windows 파일 잠금 처리)
- [ ] **AWK-01**: awk — 완전한 AWK 인터프리터 (패턴-액션, 내장 변수, 내장 함수)
- [ ] **AWK-02**: awk 필드 분리, printf, 연관 배열 지원
- [ ] **DIFF-01**: diff — 파일 비교 (unified, context, normal 형식)
- [ ] **DIFF-02**: patch — diff 출력으로 파일 패치 적용

### Search and Navigation

- [ ] **FIND-01**: find — 파일 검색 (-name, -type, -size, -mtime 등)
- [ ] **FIND-02**: find -exec가 올바르게 동작 (GOW #208 해결)
- [ ] **FIND-03**: find에서 공백 포함 경로 올바르게 처리 (GOW #209 해결)
- [ ] **WHICH-01**: which — 실행파일 위치 탐색 (Windows PATH 정확히 탐색, GOW #276 해결)
- [ ] **XARGS-01**: xargs — 표준 입력에서 명령줄 구성 (-0 null 구분, -I 치환)
- [ ] **LESS-01**: less — 파일 페이저 (스크롤, 검색, 큰 파일 지원)

### Archive and Compression

- [ ] **ARCH-01**: tar — 아카이브 생성/추출 (-c 생성, -x 추출, -t 목록, -z gzip, -j bzip2)
- [ ] **ARCH-02**: gzip/gunzip — gzip 압축/해제 (GOW #166 gunzip 누락 해결)
- [ ] **ARCH-03**: bzip2/bunzip2 — bzip2 압축/해제
- [ ] **ARCH-04**: zip — zip 아카이브 생성
- [ ] **ARCH-05**: unrar — RAR 아카이브 추출
- [ ] **CONV-01**: dos2unix — CRLF→LF 변환
- [ ] **CONV-02**: unix2dos — LF→CRLF 변환

### Network

- [ ] **NET-01**: curl — HTTP/HTTPS 요청 (-o 출력, -L 리다이렉트, -H 헤더, -d 데이터)
- [ ] **NET-02**: curl TLS 1.2/1.3 지원, 프록시 인증 (GOW #277 해결)

### Binary Tools

- [ ] **BIN-01**: gsar — 바이너리 파일 검색 및 치환

### Windows Integration

- [x] **WIN-01**: UTF-8이 모든 유틸리티의 기본 인코딩 (GOW #280, #77 해결) — completed Plan 01-02 (2026-04-20), end-to-end validation pending Plan 01-04
- [ ] **WIN-02**: Windows 긴 경로 지원 (MAX_PATH 260자 제한 해제) — scaffolded Plan 01-01, validated in Plan 01-04
- [x] **WIN-03**: PowerShell에서 모든 유틸리티 정상 동작 — completed Plan 01-02 (2026-04-20), end-to-end validation pending Plan 01-04

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Distribution

- **DIST-01**: MSI 설치 프로그램으로 배포 (PATH 자동 등록, 업그레이드 지원)
- **DIST-02**: Chocolatey 패키지 배포
- **DIST-03**: Scoop 매니페스트 배포
- **DIST-04**: winget 패키지 배포

### Extended Utilities

- **EXT-01**: rsync — 파일 동기화 (GOW #216, #175 요청)
- **EXT-02**: nohup — 프로세스 백그라운드 실행 (GOW #269 요청)
- **EXT-03**: comm — 정렬된 파일 비교 (GOW #76 요청)
- **EXT-04**: ed — 라인 에디터 (GOW #236 요청)
- **EXT-05**: host/dig — DNS 조회 (GOW #104, #83 요청)
- **EXT-06**: sha256sum — 체크섬 계산 (GOW #137 요청)

### Enhanced Features

- **ENH-01**: PowerShell 모듈 래퍼 (PSObject 출력)
- **ENH-02**: GNU 호환성 테스트 스위트 통과율 추적
- **ENH-03**: 멀티콜 바이너리 옵션 (단일 exe)

## Out of Scope

| Feature | Reason |
|---------|--------|
| vim/nano 에디터 재작성 | 기존 성숙한 Rust 대안 존재 (helix, zed) |
| bison/flex 파서 생성기 | 전문 도구이며 별도 프로젝트 규모 |
| make 빌드 시스템 | 기존 대안 충분 (just, cargo) |
| putty SSH 클라이언트 | Windows에 OpenSSH 내장 |
| bash 셸 자체 | 별도 프로젝트 규모 (nushell 등 대안) |
| indent 코드 포매터 | 언어별 포매터가 더 우수 |
| jwhois | 사용 빈도 낮음 |
| IRC/채팅 클라이언트 | GOW 범위 밖 (GOW #131) |
| PHP | GOW 범위 밖 (GOW #167) |
| readline 라이브러리 | 셸 없이는 불필요 |
| pcre 라이브러리 | Rust regex 크레이트가 대체 |
| libintl (i18n) | v1에서는 영문/유니코드만 지원 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 | Phase 1 | Done (Plan 01-01) |
| FOUND-02 | Phase 1 | Done (Plan 01-02) |
| FOUND-03 | Phase 1 | Done (Plan 01-02) |
| FOUND-04 | Phase 1 | Done (Plan 01-02) |
| FOUND-05 | Phase 1 | Done (Plan 01-03) |
| FOUND-06 | Phase 1 | Done (Plan 01-03) |
| FOUND-07 | Phase 1 | Done (Plan 01-03) |
| WIN-01 | Phase 1 | Done (Plan 01-02) — E2E in Plan 01-04 |
| WIN-02 | Phase 1 | Scaffolded (Plan 01-01) — validated in Plan 01-04 |
| WIN-03 | Phase 1 | Done (Plan 01-02) — E2E in Plan 01-04 |
| UTIL-01 | Phase 2 | Pending |
| UTIL-02 | Phase 2 | Pending |
| UTIL-03 | Phase 2 | Pending |
| UTIL-04 | Phase 2 | Pending |
| UTIL-05 | Phase 2 | Pending |
| UTIL-06 | Phase 2 | Pending |
| UTIL-07 | Phase 2 | Pending |
| UTIL-08 | Phase 2 | Pending |
| UTIL-09 | Phase 2 | Pending |
| TEXT-03 | Phase 2 | Pending |
| FILE-06 | Phase 2 | Pending |
| FILE-07 | Phase 2 | Pending |
| FILE-08 | Phase 2 | Pending |
| WHICH-01 | Phase 2 | Pending |
| FILE-01 | Phase 3 | Pending |
| FILE-02 | Phase 3 | Pending |
| FILE-03 | Phase 3 | Pending |
| FILE-04 | Phase 3 | Pending |
| FILE-05 | Phase 3 | Pending |
| FILE-09 | Phase 3 | Pending |
| FILE-10 | Phase 3 | Pending |
| TEXT-01 | Phase 3 | Pending |
| TEXT-02 | Phase 3 | Pending |
| CONV-01 | Phase 3 | Pending |
| CONV-02 | Phase 3 | Pending |
| TEXT-04 | Phase 4 | Pending |
| TEXT-05 | Phase 4 | Pending |
| TEXT-06 | Phase 4 | Pending |
| GREP-01 | Phase 4 | Pending |
| GREP-02 | Phase 4 | Pending |
| GREP-03 | Phase 4 | Pending |
| SED-01 | Phase 4 | Pending |
| SED-02 | Phase 4 | Pending |
| AWK-01 | Phase 4 | Pending |
| AWK-02 | Phase 4 | Pending |
| DIFF-01 | Phase 4 | Pending |
| DIFF-02 | Phase 4 | Pending |
| FIND-01 | Phase 5 | Pending |
| FIND-02 | Phase 5 | Pending |
| FIND-03 | Phase 5 | Pending |
| XARGS-01 | Phase 5 | Pending |
| LESS-01 | Phase 5 | Pending |
| BIN-01 | Phase 5 | Pending |
| ARCH-01 | Phase 6 | Pending |
| ARCH-02 | Phase 6 | Pending |
| ARCH-03 | Phase 6 | Pending |
| ARCH-04 | Phase 6 | Pending |
| ARCH-05 | Phase 6 | Pending |
| NET-01 | Phase 6 | Pending |
| NET-02 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 59 total (note: initial count of 52 was pre-final; actual count in document is 59)
- Mapped to phases: 59
- Unmapped: 0

---
*Requirements defined: 2026-04-20*
*Last updated: 2026-04-20 after roadmap creation*

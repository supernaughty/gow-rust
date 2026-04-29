---
status: diagnosed
phase: 09-external-bundling
source: [09-VERIFICATION.md]
started: 2026-04-29T00:00:00Z
updated: 2026-04-29T00:00:00Z
---

## Current Test

COMPLETED 2026-04-29 — vim/wget/nano confirmed on PATH. Installer UI issues found (see Gaps).

## Tests

### 1. End-to-End MSI Install
expected: Run `.\download-extras.ps1` followed by `build.bat installer x64`. Install the produced MSI. After installation, `vim --version`, `wget --version`, and `nano --version` all succeed from a standard cmd/PowerShell prompt (no manual PATH edits required).
result: PASSED — `vim --version` (VIM 9.2 확인), `wget --version` (GNU Wget 1.21.4 확인), `nano --version` (GNU nano 7.2-22.1 확인). 외부 도구 PATH 등록 정상.

### 2. Extras Feature Deselection
expected: During MSI installation (with WixUI_FeatureTree visible), uncheck "GOW-Rust Extras". After install, Rust binaries (grep, sed, awk, etc.) work from PATH, but `vim`, `wget`, `nano`, and `rg` are NOT installed/accessible.
result: FAILED — WixUI_FeatureTree 기능 선택 화면이 설치 중 나타나지 않았음. Extras 선택 해제 옵션이 사용자에게 제공되지 않음. 최신 Phase 09 변경사항이 적용된 MSI로 재빌드 후 재확인 필요.

## Summary

total: 2
passed: 1
issues: 2
pending: 0
skipped: 0
blocked: 0

## Gaps

- GAP-1: WixUI_FeatureTree 기능 선택 화면이 설치 중 나타나지 않음. 이전 빌드로 테스트한 가능성 있음. `build.bat installer x64` 재실행 후 확인 필요.
- GAP-2: 인스톨러 UI 브랜딩 문제 — 아이콘/이미지가 GNU 스타일과 맞지 않고, 설치 약관(License.rtf)이 프로젝트 내용과 맞지 않음. wix/License.rtf 및 설치 UI 커스터마이징 필요.

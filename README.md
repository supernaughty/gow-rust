# gow-rust

<div align="center">

**GNU utilities for Windows — rewritten in Rust**

[![Build](https://img.shields.io/github/actions/workflow/status/supernaughty/gow-rust/release.yml?label=build&style=flat-square)](https://github.com/supernaughty/gow-rust/actions)
[![Release](https://img.shields.io/github/v/release/supernaughty/gow-rust?style=flat-square)](https://github.com/supernaughty/gow-rust/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-informational?style=flat-square)](https://github.com/supernaughty/gow-rust/releases)

[English](#english) · [한국어](#한국어)

</div>

---

## English

### What is this?

**gow-rust** is a modern reimplementation of [bmatzelle/gow](https://github.com/bmatzelle/gow) — Gnu On Windows — built with Rust.

The original GOW project gave Windows developers native access to essential GNU command-line tools without needing WSL or Cygwin. After years of happily using it, a few rough edges kept showing up — UTF-8 output issues, path conversion quirks, and `tail -f` behaving inconsistently on Windows.

This project was born from a simple idea: *what if vibe coding could fix all of that?* With modern AI-assisted development, reimplementing GOW's core utilities in Rust turned out to be not just feasible — it was fast and fun. The result is a ground-up rewrite that tackles those pain points natively.

**What you get:**
- Native Windows performance (no MSYS runtime DLL required)
- UTF-8 output by default (`SetConsoleOutputCP(65001)`)
- Correct Unix ↔ Windows path conversion (`/c/Users/foo` ↔ `C:\Users\foo`)
- Modern TLS for `curl` via Windows SChannel (no OpenSSL)
- Static CRT linking — truly self-contained binaries
- Installers for x64, x86, and ARM64 (Windows 10/11)

### Included utilities (v0.1.0)

44 GNU utilities, each as an independent binary:

| Category | Tools |
|----------|-------|
| File ops | `cat` `cp` `mv` `rm` `ln` `ls` `mkdir` `rmdir` `chmod` `touch` |
| Text | `grep` `sed` `awk` `sort` `uniq` `tr` `cut` `wc` `head` `tail` `diff` `patch` |
| Archive | `tar` `gzip` `gunzip` `zcat` `bzip2` `xz` `unxz` |
| Network | `curl` |
| Shell | `echo` `env` `pwd` `find` `xargs` `less` `tee` `which` |
| String | `basename` `dirname` `dos2unix` `unix2dos` |
| Misc | `yes` `true` `false` |

### Installation

Download the installer for your architecture from [Releases](https://github.com/supernaughty/gow-rust/releases):

| File | Architecture |
|------|-------------|
| `gow-rust-v0.1.0-installer-x64.msi` | Windows 10/11 64-bit (most PCs) |
| `gow-rust-v0.1.0-installer-x86.msi` | 32-bit |
| `gow-rust-v0.1.0-installer-arm64.msi` | Surface Pro X / Copilot+ PCs |

The installer places everything in `C:\gow-rust` and optionally adds it to your system `PATH`.

### Build from source

**Prerequisites:** Rust stable (1.85+), MSVC toolchain, WiX Toolset v3

```bat
git clone https://github.com/supernaughty/gow-rust.git
cd gow-rust

:: First-time setup (installs targets + WiX)
setup.bat

:: Build debug binaries
build.bat

:: Build release MSI installer
build.bat installer x64
build.bat installer all     :: x64 + x86 + arm64
```

Binaries land in `target\<triple>\release\`.

### Contributing

This is an open-source project and feedback is genuinely welcome. If something is broken, missing, or behaves differently than the GNU original:

- **Bug reports** → [open an issue](https://github.com/supernaughty/gow-rust/issues)
- **Feature requests** → [open an issue](https://github.com/supernaughty/gow-rust/issues) with the `enhancement` label
- **Pull requests** → welcome, especially for missing GNU utilities

Your feedback shapes what gets prioritized. This is just the beginning — more utilities and improvements are actively in the works.

### License

MIT — see [LICENSE](LICENSE)

---

## 한국어

### 이게 뭔가요?

**gow-rust**는 [bmatzelle/gow](https://github.com/bmatzelle/gow) — Gnu On Windows — 를 Rust로 다시 구현한 프로젝트입니다.

원본 GOW는 WSL이나 Cygwin 없이 Windows에서 GNU 명령어를 바로 쓸 수 있게 해주는 훌륭한 도구입니다. 오랫동안 잘 써왔는데, 사용하다 보면 UTF-8 출력이 깨지거나, 경로 변환이 미묘하게 틀리거나, Windows에서 `tail -f`가 제대로 동작하지 않는 경우가 있었습니다.

그러다 요즘 AI 기반 바이브 코딩이 너무 잘 동작하는 걸 보면서 생각했습니다: *"이거 Rust로 다시 만들어볼 수 있지 않을까?"* 실제로 해보니 생각보다 훨씬 빠르고 재미있었습니다. 그 결과물이 바로 이 프로젝트입니다.

**이 프로젝트가 제공하는 것:**
- 네이티브 Windows 성능 (MSYS 런타임 DLL 불필요)
- UTF-8 기본 출력 (`SetConsoleOutputCP(65001)`)
- Unix ↔ Windows 경로 자동 변환 (`/c/Users/foo` ↔ `C:\Users\foo`)
- Windows SChannel을 통한 최신 TLS (OpenSSL 불필요)
- CRT 정적 링킹 — 완전히 독립적인 바이너리
- x64, x86, ARM64 인스톨러 (Windows 10/11)

### 포함된 유틸리티 (v0.1.0)

독립 바이너리로 44개의 GNU 유틸리티 제공:

| 카테고리 | 도구 |
|---------|------|
| 파일 조작 | `cat` `cp` `mv` `rm` `ln` `ls` `mkdir` `rmdir` `chmod` `touch` |
| 텍스트 처리 | `grep` `sed` `awk` `sort` `uniq` `tr` `cut` `wc` `head` `tail` `diff` `patch` |
| 압축/아카이브 | `tar` `gzip` `gunzip` `zcat` `bzip2` `xz` `unxz` |
| 네트워크 | `curl` |
| 셸 유틸리티 | `echo` `env` `pwd` `find` `xargs` `less` `tee` `which` |
| 문자열 처리 | `basename` `dirname` `dos2unix` `unix2dos` |
| 기타 | `yes` `true` `false` |

### 설치

[Releases](https://github.com/supernaughty/gow-rust/releases)에서 아키텍처에 맞는 인스톨러를 다운로드:

| 파일 | 대상 |
|------|------|
| `gow-rust-v0.1.0-installer-x64.msi` | Windows 10/11 64비트 (일반 PC) |
| `gow-rust-v0.1.0-installer-x86.msi` | 32비트 |
| `gow-rust-v0.1.0-installer-arm64.msi` | Surface Pro X / Copilot+ PC |

인스톨러는 `C:\gow-rust`에 설치하고 시스템 `PATH`에 자동 등록하는 옵션을 제공합니다.

### 소스에서 빌드

**필요 도구:** Rust stable (1.85+), MSVC 툴체인, WiX Toolset v3

```bat
git clone https://github.com/supernaughty/gow-rust.git
cd gow-rust

:: 최초 1회 설정 (러스트 타겟 + WiX 설치)
setup.bat

:: 디버그 빌드
build.bat

:: 릴리즈 MSI 인스톨러 빌드
build.bat installer x64
build.bat installer all     :: x64 + x86 + arm64
```

바이너리는 `target\<triple>\release\`에 생성됩니다.

### 기여 및 피드백

오픈소스 프로젝트로, 피드백을 진심으로 환영합니다. 무언가 동작이 이상하거나, 빠진 기능이 있거나, GNU 원본과 다르게 동작한다면:

- **버그 리포트** → [이슈 등록](https://github.com/supernaughty/gow-rust/issues)
- **기능 요청** → `enhancement` 라벨로 [이슈 등록](https://github.com/supernaughty/gow-rust/issues)
- **풀 리퀘스트** → 환영합니다. 특히 누락된 GNU 유틸리티 구현에 대해서

여러분의 피드백이 우선순위를 결정합니다. 이제 시작이에요 — 더 많은 유틸리티와 개선이 계속 이어집니다.

굉장히 기대됩니다 — 함께 만들어가요 🚀

### 라이선스

MIT — [LICENSE](LICENSE) 참조

# Building gow-rust from Source

This document covers how to build gow-rust MSI installers on Windows.

## Prerequisites (all architectures)

- Windows 10/11 x86_64
- Rust stable toolchain (`rustup` installed)
- WiX Toolset v3.14.1 — run `setup.bat` to install
- Visual Studio 2022 with "Desktop development with C++" workload

## Building from Source

### x64 (default)

```bat
build.bat installer x64
```

Output: `target\gow-rust-v<version>-installer-x64.msi`

### x86

```bat
build.bat installer x86
```

Output: `target\gow-rust-v<version>-installer-x86.msi`

### ARM64

ARM64 MSI is built via cross-compilation on any x64 Windows machine. No ARM64 hardware is required.

**Prerequisites:**

1. Visual Studio 2022 with "Desktop development with C++" workload
2. VS optional component: "MSVC v143 - VS 2022 C++ ARM64 build tools"
3. VS optional component: "Windows 11 SDK (10.0.22000 or later)"
4. `rustup target add aarch64-pc-windows-msvc`
5. WiX Toolset v3.14.1 — run `setup.bat` to install

**Build command:**

```bat
build.bat installer arm64
```

Output: `target\gow-rust-v<version>-installer-arm64.msi`

## Running Tests

```bat
build.bat test
```

This runs `cargo test --workspace` across all utility crates.

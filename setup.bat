@echo off
setlocal EnableDelayedExpansion

echo ===================================================
echo  GOW-Rust toolchain setup
echo  Installs: rustup targets, cargo-wix, WiX v3
echo ===================================================
echo.

set ERRORS=0

echo [1/5] Adding rustup target: i686-pc-windows-msvc
rustup target add i686-pc-windows-msvc
if errorlevel 1 (
    echo [FAILED] Could not add i686-pc-windows-msvc
    set /a ERRORS+=1
) else (
    echo [OK] i686-pc-windows-msvc added
)
echo.

echo [2/5] Adding rustup target: aarch64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc
if errorlevel 1 (
    echo [FAILED] Could not add aarch64-pc-windows-msvc
    set /a ERRORS+=1
) else (
    echo [OK] aarch64-pc-windows-msvc added
)
echo.

echo [3/5] Installing cargo-wix 0.3.9...
cargo install cargo-wix --version 0.3.9
if errorlevel 1 (
    echo [FAILED] cargo-wix install failed
    set /a ERRORS+=1
) else (
    echo [OK] cargo-wix 0.3.9 installed
)
echo.

echo [4/5] Installing WiX Toolset v3 via winget...
winget install --id WiXToolset.WiXToolset --version 3.14.1 --accept-source-agreements --accept-package-agreements
if errorlevel 1 (
    echo [WARN] winget install failed - try manually:
    echo        https://github.com/wixtoolset/wix3/releases
) else (
    echo [OK] WiX Toolset v3 installed
)
echo.

echo [5/5] Verifying heat.exe is on PATH...
where heat.exe >nul 2>&1
if errorlevel 1 (
    echo [WARN] heat.exe not found - open a new terminal or add WiX to PATH:
    echo        set PATH=%%PATH%%;C:\Program Files (x86)\WiX Toolset v3.14\bin
) else (
    echo [OK] heat.exe found
)
echo.

echo ===================================================
if !ERRORS!==0 (
    echo  Setup complete - all steps succeeded.
) else (
    echo  Setup finished with !ERRORS! error(s). See above.
)
echo.
echo  Next steps:
echo    build.bat installer x64       build x64 MSI
echo    build.bat installer x86       build x86 MSI
echo    build.bat installer arm64     build ARM64 MSI
echo    build.bat installer all       build all 3 MSIs
echo ===================================================

endlocal
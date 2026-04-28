@echo off
setlocal EnableDelayedExpansion

echo ===================================================
echo  GOW-Rust toolchain setup
echo  Installs: rustup targets, cargo-wix, WiX v3
echo ===================================================
echo.

set ERRORS=0

:: -------------------------------------------------------
:: Step 1: Add rustup target i686-pc-windows-msvc (x86)
:: -------------------------------------------------------
echo [1/3] Adding rustup target: i686-pc-windows-msvc
rustup target add i686-pc-windows-msvc
if errorlevel 1 (
    echo [FAILED] Could not add i686-pc-windows-msvc target
    set /a ERRORS+=1
) else (
    echo [OK] i686-pc-windows-msvc added
)
echo.

:: -------------------------------------------------------
:: Step 2: Add rustup target aarch64-pc-windows-msvc (ARM64)
:: -------------------------------------------------------
echo [2/3] Adding rustup target: aarch64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc
if errorlevel 1 (
    echo [FAILED] Could not add aarch64-pc-windows-msvc target
    set /a ERRORS+=1
) else (
    echo [OK] aarch64-pc-windows-msvc added
)
echo.

:: -------------------------------------------------------
:: Step 3: Install cargo-wix 0.3.9
:: -------------------------------------------------------
echo [3/5] Installing cargo-wix 0.3.9...
cargo install cargo-wix --version 0.3.9
if errorlevel 1 (
    echo [FAILED] cargo-wix install failed
    set /a ERRORS+=1
) else (
    echo [OK] cargo-wix 0.3.9 installed
)
echo.

:: -------------------------------------------------------
:: Step 4: Install WiX Toolset v3 via winget
:: -------------------------------------------------------
echo [4/5] Installing WiX Toolset v3 via winget...
winget install --id WiXToolset.WiXToolset --version 3.14.1 --accept-source-agreements --accept-package-agreements
if errorlevel 1 (
    echo [FAILED] WiX Toolset v3 install failed
    echo          Try manually: https://github.com/wixtoolset/wix3/releases
    set /a ERRORS+=1
) else (
    echo [OK] WiX Toolset v3 installed
)
echo.

:: -------------------------------------------------------
:: Step 5: Verify heat.exe is on PATH
:: -------------------------------------------------------
echo [5/5] Verifying WiX heat.exe is on PATH...
where heat.exe >nul 2>&1
if errorlevel 1 (
    echo [WARN] heat.exe not found on PATH
    echo        Add WiX bin dir to PATH, e.g.:
    echo        set PATH=%%PATH%%;C:\Program Files (x86)\WiX Toolset v3.14\bin
    echo        You may need to open a new terminal for PATH to take effect.
) else (
    echo [OK] heat.exe found on PATH
)
echo.

:: -------------------------------------------------------
:: Summary
:: -------------------------------------------------------
echo ===================================================
if !ERRORS!==0 (
    echo  Setup complete — all steps succeeded.
) else (
    echo  Setup finished with !ERRORS! error(s). See above.
)
echo.
echo  Next steps:
echo    Run: build.bat installer x64       :: build MSI for x64
echo    Run: build.bat installer x86       :: build MSI for x86
echo    Run: build.bat installer arm64     :: build MSI for ARM64
echo    Run: build.bat installer all       :: build MSI for all arches
echo ===================================================

endlocal
exit /b !ERRORS!

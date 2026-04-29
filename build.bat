@echo off
setlocal EnableDelayedExpansion

set MODE=debug
set TARGET=
set PKG=
set ARCH=
set RUST_TARGET=
set WIX_ARCH=

:parse
if "%1"=="" goto run
if /i "%1"=="release"   ( set MODE=release & set TARGET=--release & shift & goto parse )
if /i "%1"=="debug"     ( set MODE=debug   & set TARGET=          & shift & goto parse )
if /i "%1"=="-p"        ( set PKG=-p %2    & shift & shift & goto parse )
if /i "%1"=="--arch"    ( set ARCH=%2      & shift & shift & goto parse )
if /i "%1"=="clean"     ( goto clean )
if /i "%1"=="test"      ( goto test )
if /i "%1"=="installer" ( shift & goto installer )
if /i "%1"=="help"      ( goto help )
shift & goto parse

:resolve_arch
if /i "!ARCH!"=="x64"   ( set "RUST_TARGET=x86_64-pc-windows-msvc" & set "WIX_ARCH=x64"   & goto :eof )
if /i "!ARCH!"=="x86"   ( set "RUST_TARGET=i686-pc-windows-msvc"   & set "WIX_ARCH=x86"   & goto :eof )
if /i "!ARCH!"=="arm64" ( set "RUST_TARGET=aarch64-pc-windows-msvc"& set "WIX_ARCH=arm64" & goto :eof )
if "!ARCH!"==""         ( set "RUST_TARGET=" & set "WIX_ARCH=" & goto :eof )
echo [ERROR] Unknown --arch value: !ARCH!  Valid: x64  x86  arm64
exit /b 2

:run
call :resolve_arch
if errorlevel 2 exit /b 2

set ARCH_FLAG=
if not "!RUST_TARGET!"=="" set ARCH_FLAG=--target !RUST_TARGET!

if not "!ARCH!"=="" (
    echo [build] mode=!MODE! arch=!ARCH! target=!RUST_TARGET! !PKG!
) else (
    echo [build] mode=!MODE! !PKG!
)

cargo build !TARGET! !ARCH_FLAG! !PKG!
if errorlevel 1 ( echo [FAILED] build failed & exit /b 1 )
echo.

if not "!RUST_TARGET!"=="" (
    echo [ok] binaries in target\!RUST_TARGET!\!MODE!\
) else (
    echo [ok] binaries in target\!MODE!\
)

if "!PKG!"=="" (
    echo   awk  basename  cat  chmod  cp  curl  cut  diff  dirname
    echo   dos2unix  echo  env  false  find  grep  gunzip  gzip
    echo   head  less  ln  ls  mkdir  mv  patch  pwd  rm  rmdir
    echo   sed  sort  tail  tar  tee  touch  tr  true  uniq
    echo   unix2dos  wc  which  xargs  xz  yes  zcat
)
goto end

:read_version
powershell -NoProfile -Command "(Select-String -Path 'Cargo.toml' -Pattern '^version = ').Line.Split([char]34)[1]" > "%TEMP%\gow_ver.tmp" 2>nul
set /p VERSION= < "%TEMP%\gow_ver.tmp"
del "%TEMP%\gow_ver.tmp" 2>nul
goto :eof

:installer
call :read_version
if "!VERSION!"=="" ( echo [ERROR] Could not read version from Cargo.toml & exit /b 1 )
echo [version] !VERSION!

set INS_ARCH=%1
if "%INS_ARCH%"=="" set INS_ARCH=x64
shift

if /i "%INS_ARCH%"=="all" (
    call :build_msi x64
    if errorlevel 1 exit /b 1
    call :build_msi x86
    if errorlevel 1 exit /b 1
    call :build_msi arm64
    if errorlevel 1 exit /b 1
    echo.
    echo ===================================================
    echo  All installers built:
    echo    %CD%\target\gow-rust-v!VERSION!-installer-x64.msi
    echo    %CD%\target\gow-rust-v!VERSION!-installer-x86.msi
    echo    %CD%\target\gow-rust-v!VERSION!-installer-arm64.msi
    echo ===================================================
    goto end
)

call :build_msi %INS_ARCH%
goto end

:build_msi
set _ARCH=%1
set _RT=
set _WA=

if /i "%_ARCH%"=="x64"   ( set "_RT=x86_64-pc-windows-msvc"  & set "_WA=x64"   )
if /i "%_ARCH%"=="x86"   ( set "_RT=i686-pc-windows-msvc"    & set "_WA=x86"   )
if /i "%_ARCH%"=="arm64" ( set "_RT=aarch64-pc-windows-msvc" & set "_WA=arm64" )

if "%_RT%"=="" (
    echo [ERROR] Unknown arch: %_ARCH%  Valid: x64  x86  arm64
    exit /b 2
)

set _OUT=target\gow-rust-v!VERSION!-installer-%_ARCH%.msi

echo.
echo ===================================================
echo  Building: gow-rust-v!VERSION!-installer-%_ARCH%.msi
echo  Target: %_RT%
echo ===================================================

echo [1/5] cargo build --release --target %_RT% %PKG%
cargo build --release --target %_RT% %PKG%
if errorlevel 1 ( echo [FAILED] cargo build & exit /b 1 )

set _STAGE=target\wix-stage\%_ARCH%
set _CORE_STAGE=target\wix-stage\%_ARCH%\core
set _EXTRAS_STAGE=target\wix-stage\%_ARCH%\extras

echo [2/5] Staging Rust binaries to %_CORE_STAGE%
powershell -NoProfile -Command "Remove-Item -Path '%_STAGE%' -Recurse -Force -ErrorAction SilentlyContinue; $null=New-Item '%_CORE_STAGE%' -ItemType Directory -Force; $null=New-Item '%_EXTRAS_STAGE%' -ItemType Directory -Force; Get-ChildItem 'target\%_RT%\release\*.exe' | Where-Object { $_.Name -ne 'gow-probe.exe' } | Copy-Item -Destination '%_CORE_STAGE%'"

echo [3/5] Staging extras (vim, wget, nano, batch aliases) to %_EXTRAS_STAGE%
if exist extras\bin (
    powershell -NoProfile -Command "Get-ChildItem 'extras\bin\*' -Include '*.exe','*.bat' | Copy-Item -Destination '%_EXTRAS_STAGE%'"
    if exist extras\bin\vim-runtime (
        powershell -NoProfile -Command "Copy-Item -Path 'extras\bin\vim-runtime' -Destination '%_EXTRAS_STAGE%\vim-runtime' -Recurse -Force"
    )
)
echo   Core staged:
dir /b %_CORE_STAGE% 2>nul
echo   Extras staged:
dir /b %_EXTRAS_STAGE% 2>nul | findstr /v "^vim-runtime$"

echo [4/5] Harvesting with heat.exe (core + extras)...
heat.exe dir %_CORE_STAGE% -cg CoreComponents -dr APPLICATIONFOLDER -scom -sreg -sfrag -srd -var var.CoreSourceDir -out wix\CoreHarvest-%_ARCH%.wxs
if errorlevel 1 ( echo [FAILED] heat.exe (core) - is WiX v3 installed? Run setup.bat first. & exit /b 1 )
powershell -NoProfile -ExecutionPolicy Bypass -File wix\fix-guids.ps1 -WxsFile wix\CoreHarvest-%_ARCH%.wxs
if errorlevel 1 ( echo [FAILED] fix-guids.ps1 (core) & exit /b 1 )

heat.exe dir %_EXTRAS_STAGE% -cg ExtrasComponents -dr APPLICATIONFOLDER -scom -sreg -sfrag -srd -var var.ExtrasSourceDir -out wix\ExtrasHarvest-%_ARCH%.wxs
if errorlevel 1 ( echo [FAILED] heat.exe (extras) & exit /b 1 )
powershell -NoProfile -ExecutionPolicy Bypass -File wix\fix-guids.ps1 -WxsFile wix\ExtrasHarvest-%_ARCH%.wxs
if errorlevel 1 ( echo [FAILED] fix-guids.ps1 (extras) & exit /b 1 )

echo [5/5] Compiling and linking MSI...
candle.exe wix\main.wxs wix\CoreHarvest-%_ARCH%.wxs wix\ExtrasHarvest-%_ARCH%.wxs -arch %_WA% -dCoreSourceDir=%_CORE_STAGE% -dExtrasSourceDir=%_EXTRAS_STAGE% -dVersion=!VERSION! -dPlatform=%_WA%
if errorlevel 1 ( echo [FAILED] candle.exe & exit /b 1 )

light.exe -b %_CORE_STAGE% -b %_EXTRAS_STAGE% main.wixobj CoreHarvest-%_ARCH%.wixobj ExtrasHarvest-%_ARCH%.wixobj -o !_OUT! -ext WixUIExtension
if errorlevel 1 ( echo [FAILED] light.exe & exit /b 1 )

echo.
echo ===================================================
echo  Installer ready:
echo    %CD%\!_OUT!
echo ===================================================

set _RT=
set _WA=
set _ARCH=
set _STAGE=
set _CORE_STAGE=
set _EXTRAS_STAGE=
set _OUT=
goto :eof

:test
echo [test] running workspace tests...
cargo test --workspace
goto end

:clean
echo [clean] removing target\
cargo clean
goto end

:help
echo Usage: build.bat [debug^|release] [--arch x64^|x86^|arm64] [-p ^<crate^>] [command]
echo.
echo  Build:
echo    build.bat                          debug, all crates
echo    build.bat release                  release, all crates
echo    build.bat release --arch x86       release x86
echo    build.bat --arch arm64             debug ARM64
echo    build.bat -p gow-curl              debug, curl only
echo.
echo  Installer  ^(run setup.bat first^):
echo    build.bat installer                x64 MSI
echo    build.bat installer x64            x64 MSI
echo    build.bat installer x86            x86 MSI
echo    build.bat installer arm64          ARM64 MSI
echo    build.bat installer all            all 3 MSIs
echo.
echo  Output: target\gow-rust-v^<version^>-installer-^<arch^>.msi
echo.
echo  Other:
echo    build.bat test                     run all tests
echo    build.bat clean                    cargo clean
goto end

:end
endlocal
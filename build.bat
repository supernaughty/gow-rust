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
:: Map --arch value to Rust target triple and WiX arch name
if /i "!ARCH!"=="x64"   ( set RUST_TARGET=x86_64-pc-windows-msvc  & set WIX_ARCH=x64   & goto :eof )
if /i "!ARCH!"=="x86"   ( set RUST_TARGET=i686-pc-windows-msvc    & set WIX_ARCH=x86   & goto :eof )
if /i "!ARCH!"=="arm64" ( set RUST_TARGET=aarch64-pc-windows-msvc & set WIX_ARCH=arm64 & goto :eof )
if "!ARCH!"=="" ( set RUST_TARGET= & set WIX_ARCH= & goto :eof )
echo [ERROR] Unknown --arch value: !ARCH!
echo         Valid values: x64  x86  arm64
exit /b 2

:run
call :resolve_arch
if errorlevel 2 exit /b 2

set ARCH_FLAG=
if not "!RUST_TARGET!"=="" set ARCH_FLAG=--target !RUST_TARGET!

if not "!ARCH!"=="" (
    echo [build] mode=!MODE! arch=!ARCH! (!RUST_TARGET!) !PKG!
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

:: -------------------------------------------------------
:: installer subcommand
::   build.bat installer [x64|x86|arm64|all]
:: -------------------------------------------------------
:installer
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
    goto end
)

call :build_msi %INS_ARCH%
goto end

:: -------------------------------------------------------
:: :build_msi <arch>   — build release + stage + WiX MSI
:: -------------------------------------------------------
:build_msi
set _ARCH=%1

:: Resolve target triple
if /i "%_ARCH%"=="x64"   ( set _RT=x86_64-pc-windows-msvc  & set _WA=x64   )
if /i "%_ARCH%"=="x86"   ( set _RT=i686-pc-windows-msvc    & set _WA=x86   )
if /i "%_ARCH%"=="arm64" ( set _RT=aarch64-pc-windows-msvc & set _WA=arm64 )

if "%_RT%"=="" (
    echo [ERROR] Unknown arch for installer: %_ARCH%
    echo         Valid: x64  x86  arm64
    exit /b 2
)

echo.
echo ===================================================
echo  Building MSI for %_ARCH% (target: %_RT%)
echo ===================================================

:: Step 1: Release build
echo [1/4] cargo build --release --target %_RT% %PKG%
cargo build --release --target %_RT% %PKG%
if errorlevel 1 ( echo [FAILED] cargo build & exit /b 1 )

:: Step 2: Stage binaries
set _STAGE=target\wix-stage\%_ARCH%
echo [2/4] Staging binaries to %_STAGE%
if not exist %_STAGE% mkdir %_STAGE%
:: Copy all .exe from release output (skip build-script executables via size filter)
for %%F in (target\%_RT%\release\*.exe) do (
    copy /Y "%%F" "%_STAGE%\" >nul
)
echo       Staged:
dir /b %_STAGE%\*.exe 2>nul | findstr /v "^$" | (for /f %%N in ('more') do echo         %%N) || echo         (none found)

:: Step 3: WiX harvest
echo [3/4] Harvesting with heat.exe...
if not exist wix mkdir wix
heat.exe dir %_STAGE% -cg BinComponents -dr APPLICATIONFOLDER -scom -sreg -sfrag -srd -var var.SourceDir -out wix\BinHarvest-%_ARCH%.wxs
if errorlevel 1 ( echo [FAILED] heat.exe & exit /b 1 )

:: Step 4: Compile + Link MSI
echo [4/4] Compiling and linking MSI...
candle.exe wix\main.wxs wix\BinHarvest-%_ARCH%.wxs -arch %_WA% -dSourceDir=%_STAGE% -dVersion=0.1.0 -dPlatform=%_WA%
if errorlevel 1 ( echo [FAILED] candle.exe & exit /b 1 )

light.exe -b %_STAGE% main.wixobj BinHarvest-%_ARCH%.wixobj -o target\gow-rust-%_ARCH%.msi -ext WixUIExtension
if errorlevel 1 ( echo [FAILED] light.exe & exit /b 1 )

echo.
echo [OK] MSI generated: target\gow-rust-%_ARCH%.msi
set _RT=
set _WA=
set _ARCH=
set _STAGE=
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
echo Usage: build.bat [debug^|release] [--arch x64^|x86^|arm64] [-p ^<crate^>] [subcommand]
echo.
echo  Build subcommands:
echo   build.bat                          debug build, all crates (default x64)
echo   build.bat release                  release build, all crates
echo   build.bat -p gow-curl              debug build, curl only
echo   build.bat release -p gow-gzip      release build, gzip only
echo   build.bat --arch arm64             debug build for ARM64
echo   build.bat release --arch x86       release build for x86
echo   build.bat --arch arm64 -p gow-tar  debug build gow-tar for ARM64
echo.
echo  Installer subcommands (requires WiX v3 + setup.bat run first):
echo   build.bat installer                build MSI for x64 (default)
echo   build.bat installer x64            build MSI for x64
echo   build.bat installer x86            build MSI for x86
echo   build.bat installer arm64          build MSI for ARM64
echo   build.bat installer all            build MSI for x64, x86, and ARM64
echo.
echo  Other subcommands:
echo   build.bat test                     run all workspace tests
echo   build.bat clean                    cargo clean
echo   build.bat help                     show this help
echo.
echo  Architecture mapping:
echo   x64   = x86_64-pc-windows-msvc
echo   x86   = i686-pc-windows-msvc
echo   arm64 = aarch64-pc-windows-msvc
echo.
echo  Crates: gow-gzip  gow-bzip2  gow-xz  gow-tar  gow-curl
echo          gow-find  gow-xargs  gow-less  gow-grep  gow-sed  ...
goto end

:end
endlocal

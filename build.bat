@echo off
setlocal

set MODE=debug
set TARGET=

:parse
if "%1"=="" goto run
if /i "%1"=="release" ( set MODE=release & set TARGET=--release & shift & goto parse )
if /i "%1"=="debug"   ( set MODE=debug   & set TARGET=          & shift & goto parse )
if /i "%1"=="-p"      ( set PKG=-p %2    & shift & shift & goto parse )
if /i "%1"=="clean"   ( goto clean )
if /i "%1"=="test"    ( goto test )
if /i "%1"=="help"    ( goto help )
shift & goto parse

:run
echo [build] mode=%MODE% %PKG%
cargo build %TARGET% %PKG%
if errorlevel 1 ( echo [FAILED] build failed & exit /b 1 )
echo.
echo [ok] binaries in target\%MODE%\
if "%PKG%"=="" (
    echo   awk  basename  cat  chmod  cp  curl  cut  diff  dirname
    echo   dos2unix  echo  env  false  find  grep  gunzip  gzip
    echo   head  less  ln  ls  mkdir  mv  patch  pwd  rm  rmdir
    echo   sed  sort  tail  tar  tee  touch  tr  true  uniq
    echo   unix2dos  wc  which  xargs  xz  yes  zcat
)
goto end

:test
echo [test] running workspace tests...
cargo test --workspace
goto end

:clean
echo [clean] removing target\
cargo clean
goto end

:help
echo Usage: build.bat [debug^|release] [-p ^<crate^>] [test^|clean^|help]
echo.
echo   build.bat                  debug build, all crates
echo   build.bat release          release build, all crates
echo   build.bat -p gow-curl      debug build, curl only
echo   build.bat release -p gow-gzip   release build, gzip only
echo   build.bat test             run all tests
echo   build.bat clean            cargo clean
echo.
echo Crates: gow-gzip  gow-bzip2  gow-xz  gow-tar  gow-curl
echo         gow-find  gow-xargs  gow-less  gow-grep  gow-sed  ...
goto end

:end
endlocal

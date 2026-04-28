#Requires -Version 5.1
<#
.SYNOPSIS
    Downloads portable third-party binaries for inclusion in the gow-rust installer.

.DESCRIPTION
    Downloads vim, wget, and nano portable binaries and extracts them to extras/bin/.
    Run once before building the installer. Re-running skips already-downloaded tools.

.EXAMPLE
    .\download-extras.ps1
    .\download-extras.ps1 -Force    # re-download even if already present
#>
[CmdletBinding()]
param(
    [switch]$Force
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$BinDir = Join-Path $PSScriptRoot 'extras\bin'
$TmpDir = Join-Path $PSScriptRoot 'extras\tmp'

$null = New-Item -ItemType Directory -Path $BinDir -Force
$null = New-Item -ItemType Directory -Path $TmpDir -Force

function Download-File([string]$Url, [string]$Dest) {
    if (-not (Test-Path $Dest) -or $Force) {
        Write-Host "  Downloading $([System.IO.Path]::GetFileName($Dest))..."
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing
    } else {
        Write-Host "  Already downloaded: $([System.IO.Path]::GetFileName($Dest))"
    }
}

function Tool-Done([string]$Name) {
    Write-Host "  [OK] $Name -> extras\bin\" -ForegroundColor Green
}

# ─────────────────────────────────────────────────────────────────
# 1. vim 9.2 (latest portable win64)
# ─────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[1/3] vim portable (win64)" -ForegroundColor Cyan

$VimVersion = "9.2.0407"
$VimZip     = Join-Path $TmpDir "gvim_${VimVersion}_x64.zip"
$VimUrl     = "https://github.com/vim/vim-win32-installer/releases/download/v${VimVersion}/gvim_${VimVersion}_x64.zip"
$VimExe     = Join-Path $BinDir "vim.exe"

if (-not (Test-Path $VimExe) -or $Force) {
    Download-File $VimUrl $VimZip

    Write-Host "  Extracting vim.exe..."
    $ExtractDir = Join-Path $TmpDir "vim_extract"
    $null = New-Item -ItemType Directory -Path $ExtractDir -Force
    Expand-Archive -Path $VimZip -DestinationPath $ExtractDir -Force

    # vim zip layout: vim\vim92\vim.exe (console), gvim.exe (GUI)
    $VimSrc = Get-ChildItem -Path $ExtractDir -Recurse -Filter "vim.exe" |
        Where-Object { $_.Name -eq "vim.exe" -and $_.DirectoryName -notmatch "\\bundle\\" } |
        Select-Object -First 1
    if (-not $VimSrc) { throw "vim.exe not found in zip" }

    # Copy the entire vim subdirectory (vim needs its runtime files)
    $VimSubdir = Join-Path $BinDir "vim-runtime"
    $null = New-Item -ItemType Directory -Path $VimSubdir -Force
    Copy-Item -Path (Split-Path $VimSrc.FullName -Parent) -Destination $VimSubdir -Recurse -Force

    # Copy vim.exe to extras\bin\vim.exe
    Copy-Item -Path $VimSrc.FullName -Destination $VimExe -Force

    # Write vim.bat that sets VIMRUNTIME and calls the real vim.exe
    $VimBat = Join-Path $BinDir "vim.bat"
    @"
@echo off
set "VIMRUNTIME=%~dp0vim-runtime"
"%~dp0vim.exe" %*
"@ | Set-Content -Path $VimBat -Encoding ASCII
} else {
    Write-Host "  Already present: vim.exe"
}
Tool-Done "vim $VimVersion"

# ─────────────────────────────────────────────────────────────────
# 2. wget 1.21.4 (static build, no deps)
# ─────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[2/3] wget 1.21.4 (win64, static)" -ForegroundColor Cyan

$WgetVersion = "1.21.4"
$WgetExe     = Join-Path $BinDir "wget.exe"
$WgetUrl     = "https://eternallybored.org/misc/wget/$WgetVersion/64/wget.exe"

if (-not (Test-Path $WgetExe) -or $Force) {
    Download-File $WgetUrl $WgetExe
} else {
    Write-Host "  Already present: wget.exe"
}
Tool-Done "wget $WgetVersion"

# ─────────────────────────────────────────────────────────────────
# 3. nano 7.2 portable (win64)
# ─────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[3/3] nano 7.2 portable (win64)" -ForegroundColor Cyan

$NanoVersion = "7.2-22.1"
$NanoZip     = Join-Path $TmpDir "nano-for-windows_win64_v${NanoVersion}.zip"
$NanoExe     = Join-Path $BinDir "nano.exe"
$NanoUrl     = "https://github.com/okibcn/nano-for-windows/releases/download/v${NanoVersion}/nano-for-windows_win64_v${NanoVersion}.zip"

if (-not (Test-Path $NanoExe) -or $Force) {
    Download-File $NanoUrl $NanoZip

    Write-Host "  Extracting nano.exe..."
    $NanoExtract = Join-Path $TmpDir "nano_extract"
    $null = New-Item -ItemType Directory -Path $NanoExtract -Force
    Expand-Archive -Path $NanoZip -DestinationPath $NanoExtract -Force

    $NanoSrc = Get-ChildItem -Path $NanoExtract -Recurse -Filter "nano.exe" | Select-Object -First 1
    if (-not $NanoSrc) { throw "nano.exe not found in zip" }
    Copy-Item -Path $NanoSrc.FullName -Destination $NanoExe -Force
} else {
    Write-Host "  Already present: nano.exe"
}
Tool-Done "nano $NanoVersion"

# ─────────────────────────────────────────────────────────────────
# 4. Batch aliases (egrep, fgrep, bunzip2, gawk, gfind, gsort)
# ─────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[+] Writing batch aliases..." -ForegroundColor Cyan

$Aliases = @{
    'egrep.bat'   = '@echo off & "%~dp0grep.exe" -E %*'
    'fgrep.bat'   = '@echo off & "%~dp0grep.exe" -F %*'
    'bunzip2.bat' = '@echo off & "%~dp0bzip2.exe" -d %*'
    'gawk.bat'    = '@echo off & "%~dp0awk.exe" %*'
    'gfind.bat'   = '@echo off & "%~dp0find.exe" %*'
    'gsort.bat'   = '@echo off & "%~dp0sort.exe" %*'
    'gzip.bat'    = '@echo off & "%~dp0gzip.exe" %*'
    'unxz.bat'    = '@echo off & "%~dp0xz.exe" -d %*'
}

foreach ($kv in $Aliases.GetEnumerator()) {
    $path = Join-Path $BinDir $kv.Key
    if (-not (Test-Path $path) -or $Force) {
        $kv.Value | Set-Content -Path $path -Encoding ASCII
        Write-Host "  [OK] $($kv.Key)" -ForegroundColor Green
    } else {
        Write-Host "  Already present: $($kv.Key)"
    }
}

# ─────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host " Extras ready in extras\bin\" -ForegroundColor Cyan
Write-Host "================================================================"
Get-ChildItem -Path $BinDir | Sort-Object Name | ForEach-Object {
    Write-Host "  $($_.Name)"
}
Write-Host ""
Write-Host " Next step: build.bat installer x64" -ForegroundColor Yellow

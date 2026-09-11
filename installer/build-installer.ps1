# 설치 파일 빌드: cargo build --release + VC++ 런타임 준비 + ISCC 컴파일.
# 실행: pwsh -File installer\build-installer.ps1
# 결과물: dist\ 폴더.

[CmdletBinding()]
param(
    [switch]$SkipCargoBuild,
    [string]$BuildDir,
    [string]$OutputDir,
    [string]$RuntimeDir
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if (-not $BuildDir) { $BuildDir = Join-Path $repoRoot 'target\release' }
if (-not $OutputDir) { $OutputDir = Join-Path $repoRoot 'dist' }
if (-not $RuntimeDir) { $RuntimeDir = Join-Path $repoRoot 'target\installer\runtime' }

# 1. 릴리스 실행 파일
if (-not $SkipCargoBuild) {
    Write-Host '==> cargo build --release'
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build 실패 (exit $LASTEXITCODE)" }
}

$exe = Join-Path $BuildDir 'weather-app.exe'
if (-not (Test-Path $exe)) {
    throw "실행 파일을 찾을 수 없습니다: $exe`n먼저 'cargo build --release'를 실행하세요."
}

# 2. 앱 로컬 런타임 DLL (재배포 패키지 대신 실행 파일 옆에 배치)
function Get-VcRedistDlls {
    $roots = @(
        (Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\2022'),
        (Join-Path $env:ProgramFiles 'Microsoft Visual Studio\2022')
    ) | Where-Object { $_ -and (Test-Path $_) }

    $crtDirs = foreach ($root in $roots) {
        Get-ChildItem -Path $root -Directory -ErrorAction SilentlyContinue | ForEach-Object {
            $redist = Join-Path $_.FullName 'VC\Redist\MSVC'
            if (-not (Test-Path $redist)) { continue }
            Get-ChildItem -Path $redist -Directory -ErrorAction SilentlyContinue | ForEach-Object {
                $crt = Join-Path $_.FullName 'x64\Microsoft.VC143.CRT'
                if (Test-Path (Join-Path $crt 'vcruntime140.dll')) {
                    $ver = $null
                    [void][version]::TryParse($_.Name, [ref]$ver)
                    [pscustomobject]@{ Dir = $crt; Version = $ver }
                }
            }
        }
    }

    $best = $crtDirs | Sort-Object Version -Descending | Select-Object -First 1
    if (-not $best) { return @() }
    Get-ChildItem -Path $best.Dir -Filter '*.dll' | Select-Object -ExpandProperty FullName
}

New-Item -ItemType Directory -Path $RuntimeDir -Force | Out-Null
Get-ChildItem -Path $RuntimeDir -File -ErrorAction SilentlyContinue | Remove-Item -Force

$runtimeDlls = Get-VcRedistDlls
if ($runtimeDlls.Count -gt 0) {
    foreach ($dll in $runtimeDlls) {
        Copy-Item -Path $dll -Destination $RuntimeDir -Force
        Write-Host "런타임 DLL 준비: $(Split-Path -Leaf $dll)"
    }
} else {
    Write-Warning @'
VC++ 재배포 DLL을 찾지 못했습니다. 설치 대상 PC에 VC++ 재배포 패키지
(Microsoft Visual Studio 2015-2022 x64)가 없다면 앱이 실행되지 않을 수 있습니다.
'@
}

# 3. Inno Setup 컴파일러 찾기
$isccCandidates = @(
    (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
    (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
    (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe')
)
$iscc = $isccCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) {
    $cmd = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($cmd) { $iscc = $cmd.Source }
}
if (-not $iscc) {
    throw @'
Inno Setup 6을 찾을 수 없습니다. 다음 명령으로 설치하세요:
  winget install --id JRSoftware.InnoSetup -e
'@
}

if (-not (Test-Path $OutputDir)) { New-Item -ItemType Directory -Path $OutputDir | Out-Null }

# 4. 설치 프로그램 컴파일
$iss = Join-Path $PSScriptRoot 'weather-app.iss'
Write-Host "==> ISCC $iss"
& $iscc "/DBuildDir=$BuildDir" "/DOutputDir=$OutputDir" "/DRuntimeDir=$RuntimeDir" $iss
if ($LASTEXITCODE -ne 0) { throw "설치 프로그램 컴파일 실패 (exit $LASTEXITCODE)" }

Write-Host ''
Get-ChildItem $OutputDir -Filter '*.exe' |
    Sort-Object LastWriteTime -Descending |
    ForEach-Object { Write-Host ("생성됨: {0} ({1:N1} MB)" -f $_.FullName, ($_.Length / 1MB)) }

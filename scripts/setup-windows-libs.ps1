$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$archiveName = 'llvm-mingw-20261104-ucrt-x86_64.zip'
$archivePath = Join-Path $projectRoot ('.tools\' + $archiveName)
$uri = 'https://github.com/mstorsjo/llvm-mingw/releases/download/20261104/' + $archiveName
$expected = '552191b5754c49a5a20723276a6fc56541ea1922c4c50b7b669022333fc1b40f'
if (-not (Test-Path -LiteralPath $archivePath)) { Invoke-WebRequest -Uri $uri -OutFile $archivePath }
if ((Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash -ine $expected) { throw 'Checksum LLVM-MinGW nao confere.' }
& tar.exe -xf $archivePath -C (Join-Path $projectRoot '.tools')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Get-Item -LiteralPath '.tools\llvm-mingw-20261104-ucrt-x86_64\x86_64-w64-mingw32\lib\libopengl32.a' | Select-Object FullName,Length

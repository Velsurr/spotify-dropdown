$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$env:CARGO_HOME = Join-Path $projectRoot '.tools\cargo'
$env:RUSTUP_HOME = Join-Path $projectRoot '.tools\rustup'
$env:TEMP = Join-Path $projectRoot '.tools\tmp'
$env:TMP = $env:TEMP
$env:RUSTUP_INIT_SKIP_MSVC_CHECK = 'yes'
New-Item -ItemType Directory -Path $env:TEMP -Force | Out-Null
$installer = Join-Path $projectRoot '.tools\rustup-init.exe'
$checksumUri = 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-gnu/rustup-init.exe.sha256'
$checksumContent = (Invoke-WebRequest -Uri $checksumUri).Content
if ($checksumContent -isnot [string]) { $checksumContent = [Text.Encoding]::UTF8.GetString($checksumContent) }
$expected = ($checksumContent.Trim() -split '\s+')[0]
$actual = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash
if ($expected -ine $actual) { throw 'O checksum do instalador oficial do Rust nao confere.' }
& $installer -y --no-modify-path --default-host x86_64-pc-windows-gnullvm --default-toolchain stable --profile minimal --component rustfmt,clippy,rust-mingw
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& (Join-Path $env:CARGO_HOME 'bin\rustc.exe') --version
& (Join-Path $env:CARGO_HOME 'bin\cargo.exe') --version

param(
    [Parameter(Position = 0)]
    [ValidateSet('fetch', 'check', 'test', 'release', 'fixture', 'fmt', 'clippy', 'run')]
    [string]$Action = 'release'
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$env:CARGO_HOME = Join-Path $projectRoot '.tools\cargo'
$env:RUSTUP_HOME = Join-Path $projectRoot '.tools\rustup'
$env:TEMP = Join-Path $projectRoot '.tools\tmp'
$env:TMP = $env:TEMP
$env:PATH = (Join-Path $env:CARGO_HOME 'bin') + ';' + (Join-Path $projectRoot '.tools\llvm-mingw-20261104-ucrt-x86_64\bin') + ';' + $env:PATH
$env:CARGO_INCREMENTAL = '0'
$cargo = Join-Path $env:CARGO_HOME 'bin\cargo.exe'
if (-not (Test-Path -LiteralPath $cargo)) { throw 'Rust local nao instalado. Execute scripts\setup-rust.ps1 apos autorizar a instalacao.' }
switch ($Action) {
    'fetch' { & $cargo fetch }
    'check' { & $cargo check --locked --offline }
    'test' { & $cargo test --locked --offline }
    'release' { & $cargo build --release --bin spotify-dropdown --locked --offline }
    'fixture' { & $cargo build --release --bin fullscreen-fixture --locked --offline }
    'fmt' { & $cargo fmt --all }
    'clippy' { & $cargo clippy --all-targets --locked --offline -- -D warnings }
    'run' { & $cargo run --release --locked --offline }
}
exit $LASTEXITCODE

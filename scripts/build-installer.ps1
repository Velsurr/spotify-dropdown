param([switch]$QaOnly)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$env:TEMP = Join-Path $projectRoot '.tools\tmp'
$env:TMP = $env:TEMP
$compiler = Join-Path $projectRoot '.tools\nsis-3.13\makensis.exe'
if (-not (Test-Path -LiteralPath $compiler)) { throw 'NSIS portatil ausente. Autorize e execute scripts\setup-nsis.ps1.' }
$manifest = Get-Content -Raw -LiteralPath (Join-Path $projectRoot 'Cargo.toml')
$version = [regex]::Match($manifest, '(?m)^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"').Groups[1].Value
if (-not $version) { throw 'Versao de tres digitos nao encontrada.' }
$arguments = @('/V2', "/DAPP_VERSION=$version", "/DPROJECT_ROOT=$projectRoot")
if ($QaOnly) { $arguments += '/DQA_ONLY' }
$arguments += (Join-Path $projectRoot 'installer\spotify-dropdown.nsi')
& $compiler @arguments
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$output = if ($QaOnly) { Join-Path $projectRoot 'artifacts\SpotifyDropdown-Setup-QA.exe' } else { Join-Path $projectRoot "dist\v$version\SpotifyDropdown-Setup-$version.exe" }
[pscustomobject]@{Path=$output;Bytes=(Get-Item -LiteralPath $output).Length;SHA256=(Get-FileHash -LiteralPath $output -Algorithm SHA256).Hash} | Format-List

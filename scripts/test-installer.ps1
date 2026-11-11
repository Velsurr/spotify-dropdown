$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$env:TEMP = Join-Path $projectRoot '.tools\tmp'
$env:TMP = $env:TEMP
$qaRoot = [IO.Path]::GetFullPath((Join-Path $projectRoot 'artifacts\installer-qa'))
if (-not $qaRoot.StartsWith($projectRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Destino de teste fora do projeto.' }
. (Join-Path $PSScriptRoot 'paths.ps1')
$installer = Join-Path $projectRoot 'artifacts\SpotifyDropdown-Setup-QA.exe'
if (-not (Test-Path -LiteralPath $installer)) { throw 'Gere primeiro build-installer.ps1 -QaOnly.' }
$process = Start-Process -FilePath $installer -ArgumentList '/S' -WorkingDirectory $projectRoot -WindowStyle Hidden -PassThru -Wait
if ($process.ExitCode -ne 0) { throw "Instalacao de teste falhou: $($process.ExitCode)" }
$installed = Join-Path $qaRoot 'Spotify Dropdown.exe'
if ((Get-FileHash -LiteralPath $installed).Hash -ne (Get-FileHash -LiteralPath (Get-ProjectExecutable)).Hash) { throw 'Executavel extraido diferente do original.' }
foreach ($name in @('README.md', 'CHANGELOG.md', 'Desinstalar.exe')) {
    if (-not (Test-Path -LiteralPath (Join-Path $qaRoot $name))) { throw "Arquivo nao extraido: $name" }
}
$preferences = Join-Path $qaRoot 'settings.ini'
[IO.File]::WriteAllText($preferences, '# Preferencias de teste; preservar ao desinstalar.')
$uninstaller = Join-Path $qaRoot 'Desinstalar.exe'
$process = Start-Process -FilePath $uninstaller -ArgumentList "/S _?=$qaRoot" -WorkingDirectory $projectRoot -WindowStyle Hidden -PassThru -Wait
if ($process.ExitCode -ne 0) { throw "Desinstalacao de teste falhou: $($process.ExitCode)" }
foreach ($name in @('Spotify Dropdown.exe', 'README.md', 'CHANGELOG.md')) {
    if (Test-Path -LiteralPath (Join-Path $qaRoot $name)) { throw "Arquivo nao removido: $name" }
}
if (-not (Test-Path -LiteralPath $preferences)) { throw 'As preferencias deveriam ter sido preservadas.' }
'Instalacao e desinstalacao verificadas dentro do projeto; executavel identico e preferencias preservadas. Variante QA sem registro ou atalhos.'

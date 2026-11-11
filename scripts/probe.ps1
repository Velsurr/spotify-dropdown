param([ValidatePattern('^[a-z0-9-]+$')][string]$Tag = 'inicial')
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
. (Join-Path $PSScriptRoot 'paths.ps1')
$exe = Get-ProjectExecutable
$reportPath = 'artifacts\diagnostico-' + $Tag + '.txt'
$process = Start-Process -FilePath $exe -WorkingDirectory $projectRoot -ArgumentList '--diagnostico',$reportPath -WindowStyle Hidden -PassThru
if (-not $process.WaitForExit(20000)) { throw 'O diagnostico excedeu 20 segundos.' }
if (-not (Test-Path -LiteralPath $reportPath)) { throw 'O executavel nao gerou o diagnostico.' }
Get-Content -LiteralPath $reportPath

param([int]$TargetProcessId = 0)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'paths.ps1')
$projectExecutable = Get-ProjectExecutable
if ($TargetProcessId -eq 0) {
    $candidates = @(Get-Process -Name 'Spotify Dropdown' | Where-Object Path -EQ $projectExecutable)
    if ($candidates.Count -ne 1) { throw 'Esperava uma unica instancia do prototipo.' }
    $TargetProcessId = $candidates[0].Id
}
$process = Get-Process -Id $TargetProcessId
if ($process.Path -ne $projectExecutable) { throw 'O processo informado nao pertence ao executavel deste projeto.' }
$started = [DateTime]::UtcNow
$cpuStart = $process.TotalProcessorTime.TotalMilliseconds
Start-Sleep -Seconds 10
$process.Refresh()
$elapsed = ([DateTime]::UtcNow - $started).TotalSeconds
$cpuMilliseconds = $process.TotalProcessorTime.TotalMilliseconds - $cpuStart
$report = [ordered]@{
    ProcessId = $TargetProcessId
    Window = $process.MainWindowTitle
    SampleSeconds = [Math]::Round($elapsed, 2)
    CpuMilliseconds = [Math]::Round($cpuMilliseconds, 2)
    CpuPercentOfOneCore = [Math]::Round($cpuMilliseconds / ($elapsed * 10), 3)
    WorkingSetMiB = [Math]::Round($process.WorkingSet64 / 1MB, 2)
    PrivateMiB = [Math]::Round($process.PrivateMemorySize64 / 1MB, 2)
    Handles = $process.HandleCount
    Threads = $process.Threads.Count
}
$json = $report | ConvertTo-Json
$versionName = Split-Path -Leaf (Split-Path -Parent $projectExecutable)
[IO.File]::WriteAllText((Join-Path $projectRoot "artifacts\consumo-$versionName.json"), $json, [Text.UTF8Encoding]::new($false))
$json

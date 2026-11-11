$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$env:TEMP = Join-Path $projectRoot '.tools\tmp'
$env:TMP = $env:TEMP
$source = Join-Path $projectRoot 'target\release\spotify-dropdown.exe'
if (-not (Test-Path -LiteralPath $source)) { throw 'Compile em release antes de empacotar.' }
. (Join-Path $PSScriptRoot 'paths.ps1')
$destination = Split-Path -Parent (Get-ProjectExecutable)
New-Item -ItemType Directory -Path $destination -Force | Out-Null
Copy-Item -LiteralPath $source -Destination (Join-Path $destination 'Spotify Dropdown.exe') -Force
$files = @((Join-Path $destination 'Spotify Dropdown.exe'))
foreach ($name in @('README.md', 'CHANGELOG.md', 'MANUAL-TESTS.md', 'RELEASE-NOTES.md')) {
    $target = Join-Path $destination $name
    Copy-Item -LiteralPath (Join-Path $projectRoot $name) -Destination $target -Force
    $files += $target
}
# Enumerate only deliverables: never include settings, local logs, tools or caches.
$archive = Get-ProjectArchive
Compress-Archive -LiteralPath $files -DestinationPath $archive -CompressionLevel Optimal -Force
$hashes = foreach ($path in @((Get-ProjectExecutable), $archive)) {
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $(Split-Path -Leaf $path)"
}
[IO.File]::WriteAllLines((Join-Path $destination 'SHA256SUMS.txt'), $hashes, [Text.UTF8Encoding]::new($false))
foreach ($path in @((Get-ProjectExecutable), $archive)) {
    [pscustomobject]@{Path=$path;Bytes=(Get-Item -LiteralPath $path).Length;SHA256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash} | Format-List
}

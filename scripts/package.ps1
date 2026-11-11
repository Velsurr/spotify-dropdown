$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$source = Join-Path $projectRoot 'target\release\spotify-dropdown.exe'
if (-not (Test-Path -LiteralPath $source)) { throw 'Compile em release antes de empacotar.' }
. (Join-Path $PSScriptRoot 'paths.ps1')
$destination = Split-Path -Parent (Get-ProjectExecutable)
New-Item -ItemType Directory -Path $destination -Force | Out-Null
Copy-Item -LiteralPath $source -Destination (Join-Path $destination 'Spotify Dropdown.exe') -Force
Copy-Item -LiteralPath (Join-Path $projectRoot 'README.md') -Destination (Join-Path $destination 'README.md') -Force
Copy-Item -LiteralPath (Join-Path $projectRoot 'CHANGELOG.md') -Destination (Join-Path $destination 'CHANGELOG.md') -Force
Get-FileHash -LiteralPath (Join-Path $destination 'Spotify Dropdown.exe') -Algorithm SHA256 | Format-List
Get-Item -LiteralPath (Join-Path $destination 'Spotify Dropdown.exe') | Select-Object FullName,Length

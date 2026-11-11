function Get-ProjectVersion {
    $projectRoot = Split-Path -Parent $PSScriptRoot
    $manifest = Get-Content -LiteralPath (Join-Path $projectRoot 'Cargo.toml') -Raw
    $packageVersion = [regex]::Match($manifest, '(?m)^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?)"\s*$').Groups[1].Value
    if (-not $packageVersion) { throw 'Versao SemVer nao encontrada em Cargo.toml.' }
    $packageVersion
}

function Get-ProjectExecutable {
    $projectRoot = Split-Path -Parent $PSScriptRoot
    $versionFolder = 'v' + (Get-ProjectVersion)
    Join-Path $projectRoot ('dist\' + $versionFolder + '\Spotify Dropdown.exe')
}

function Get-ProjectArchive {
    $version = Get-ProjectVersion
    Join-Path (Split-Path -Parent (Get-ProjectExecutable)) "SpotifyDropdown-v$version-windows-x64.zip"
}

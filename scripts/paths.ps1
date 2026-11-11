function Get-ProjectExecutable {
    $projectRoot = Split-Path -Parent $PSScriptRoot
    $manifest = Get-Content -LiteralPath (Join-Path $projectRoot 'Cargo.toml') -Raw
    $packageVersion = [version]([regex]::Match($manifest, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value)
    $versionFolder = 'v' + $packageVersion.ToString(3)
    Join-Path $projectRoot ('dist\' + $versionFolder + '\Spotify Dropdown.exe')
}

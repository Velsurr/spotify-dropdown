$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$archive = Join-Path $projectRoot '.tools\nsis-3.13.zip'
$uri = 'https://netix.dl.sourceforge.net/project/nsis/NSIS%203/3.13/nsis-3.13.zip'
$isZip = (Test-Path -LiteralPath $archive) -and ([IO.File]::ReadAllBytes($archive)[0] -eq 80)
if (-not $isZip) { Invoke-WebRequest -Uri $uri -OutFile $archive }
if ([IO.File]::ReadAllBytes($archive)[0] -ne 80) {
    $landing = [IO.File]::ReadAllText($archive)
    $redirect = [regex]::Match($landing, '<meta[^>]+content="5; url=([^"]+)"').Groups[1].Value
    $redirect = [Net.WebUtility]::HtmlDecode($redirect)
    if (-not $redirect.StartsWith('https://downloads.sourceforge.net/project/nsis/NSIS%203/3.13/nsis-3.13.zip?')) { throw 'O servidor nao retornou o download oficial esperado.' }
    Invoke-WebRequest -Uri $redirect -OutFile $archive
}
if ([IO.File]::ReadAllBytes($archive)[0] -ne 80) { throw 'O servidor retornou uma pagina, nao o pacote ZIP do NSIS.' }
Get-FileHash -LiteralPath $archive -Algorithm SHA256
& tar.exe -xf $archive -C (Join-Path $projectRoot '.tools')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& (Join-Path $projectRoot '.tools\nsis-3.13\makensis.exe') /VERSION
exit $LASTEXITCODE

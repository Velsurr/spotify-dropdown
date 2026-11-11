param(
    [Parameter(Mandatory=$true)]
    [ValidatePattern('^[0-9a-fA-F]{40}$')]
    [string]$Commit
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
. (Join-Path $PSScriptRoot 'paths.ps1')
$version = Get-ProjectVersion
if ($version -ne '0.2.2-alpha') { throw 'Este roteiro publica somente a pre-release 0.2.2-alpha.' }
$tag = "v$version"
$repository = 'Velsurr/spotify-dropdown'
$archive = Get-ProjectArchive
if (-not (Test-Path -LiteralPath $archive)) { throw 'Gere o executavel release e o ZIP antes de publicar.' }
$ghCommand = Get-Command gh -ErrorAction SilentlyContinue
if (-not $ghCommand) { throw 'GitHub CLI (gh) ausente. Sua instalacao exige autorizacao; este roteiro nao instala nem autentica ferramentas.' }
$gh = $ghCommand.Source
& $gh auth status
if ($LASTEXITCODE -ne 0) { throw 'Autenticacao do GitHub CLI indisponivel. Concluir login/autorizar acesso antes de publicar.' }

# Require an existing remote commit; this script does not upload source or replace history.
$remoteCommit = & $gh api "repos/$repository/commits/$Commit" --jq '.sha'
if ($LASTEXITCODE -ne 0 -or $remoteCommit.Trim() -ine $Commit) { throw 'O commit informado precisa existir no repositorio remoto.' }
$remoteManifest = & $gh api "repos/$repository/contents/Cargo.toml?ref=$Commit" --jq '.content'
if ($LASTEXITCODE -ne 0) { throw 'Cargo.toml nao encontrado no commit remoto.' }
$manifestText = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String(($remoteManifest -join '').Replace("`n", '').Replace("`r", '')))
if ($manifestText -notmatch '(?m)^version\s*=\s*"0\.2\.2-alpha"\s*$') { throw 'O commit remoto nao corresponde a versao 0.2.2-alpha.' }

$releaseJson = & $gh release list --repo $repository --limit 1000 --json tagName
if ($LASTEXITCODE -ne 0) { throw 'Nao foi possivel verificar releases existentes.' }
if (@(($releaseJson | ConvertFrom-Json) | Where-Object tagName -EQ $tag).Count) { throw 'A release ja existe; nao sera modificada.' }
$referencesJson = & $gh api "repos/$repository/git/matching-refs/tags/$tag"
if ($LASTEXITCODE -ne 0) { throw 'Nao foi possivel verificar tags existentes.' }
$existing = @(($referencesJson | ConvertFrom-Json) | Where-Object ref -EQ "refs/tags/$tag")
if ($existing.Count) {
    $tagCommit = & $gh api "repos/$repository/commits/$tag" --jq '.sha'
    if ($LASTEXITCODE -ne 0 -or $tagCommit.Trim() -ine $Commit) { throw 'A tag existente aponta para outro commit; nenhuma alteracao sera feita.' }
} else {
    & $gh api --method POST "repos/$repository/git/refs" -f "ref=refs/tags/$tag" -f "sha=$Commit" --silent
    if ($LASTEXITCODE -ne 0) { throw 'Nao foi possivel criar a nova tag.' }
}

& $gh release create $tag $archive --repo $repository --target $Commit --verify-tag --title 'Spotify Dropdown 0.2.2 Alpha' --notes-file (Join-Path $projectRoot 'RELEASE-NOTES.md') --prerelease --latest=false
if ($LASTEXITCODE -ne 0) { throw 'A publicacao falhou. A tag pode ter sido criada; a release anterior permanece intacta.' }
& $gh release view $tag --repo $repository --json tagName,isPrerelease,url,assets
if ($LASTEXITCODE -ne 0) { throw 'A criacao foi enviada, mas a verificacao final falhou. Confira o estado remoto antes de tentar novamente.' }

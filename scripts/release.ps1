# Esempio: .\scripts\release.ps1 -Version 0.2.0
[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter(Mandatory, Position = 0)]
    [ValidatePattern('^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$')]
    [string]$Version,
    [switch]$SkipChecks
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path $PSScriptRoot -Parent

function Invoke-Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Program $($Arguments -join ' ') fallito (exit $LASTEXITCODE)."
    }
}

function Replace-One {
    param([string]$Text, [string]$Pattern, [string]$Replacement)
    $regex = [regex]::new($Pattern)
    if ($regex.Matches($Text).Count -ne 1) {
        throw "Formato inatteso: impossibile individuare una sola versione ($Pattern)."
    }
    return $regex.Replace($Text, $Replacement)
}

Push-Location $repo
try {
    foreach ($tool in @('git', 'bun', 'cargo')) {
        Get-Command $tool -ErrorAction Stop | Out-Null
    }
    $tag = "v$Version"
    & git show-ref --verify --quiet "refs/tags/$tag"
    if ($LASTEXITCODE -eq 0) { throw "Il tag $tag esiste gia. Scegli una nuova versione." }
    if ($LASTEXITCODE -ne 1) { throw 'Impossibile controllare i tag Git.' }

    # Prepara tutte le sostituzioni prima di scrivere; conserva formattazione e commenti.
    $changes = [ordered]@{}
    $packagePath = Join-Path $repo 'package.json'
    $packageText = [IO.File]::ReadAllText($packagePath)
    $current = ($packageText | ConvertFrom-Json).version
    if ([version]$Version -lt [version]$current) { throw "La versione deve essere almeno $current." }
    $changes[$packagePath] = Replace-One $packageText '(?m)^(\s*"version"\s*:\s*")[^"]+("\s*,?\s*)$' ('${1}' + $Version + '${2}')
    $manifestPath = Join-Path $repo 'src-tauri/Cargo.toml'
    $changes[$manifestPath] = Replace-One ([IO.File]::ReadAllText($manifestPath)) '(?m)^(version\s*=\s*")[^"]+("\s*)$' ('${1}' + $Version + '${2}')
    $lockPath = Join-Path $repo 'src-tauri/Cargo.lock'
    $changes[$lockPath] = Replace-One ([IO.File]::ReadAllText($lockPath)) '(?m)^(name = "memotape"\r?\nversion = ")[^"]+("\r?$)' ('${1}' + $Version + '${2}')
    $configPath = Join-Path $repo 'src-tauri/tauri.conf.json'
    $configText = [IO.File]::ReadAllText($configPath)
    if (($configText | ConvertFrom-Json).PSObject.Properties.Name -contains 'version') {
        $changes[$configPath] = Replace-One $configText '(?m)^(\s*"version"\s*:\s*")[^"]+("\s*,?\s*)$' ('${1}' + $Version + '${2}')
    }

    Write-Host "Memotape $current -> $Version; tag previsto: $tag"
    Write-Host 'Chiudi tauri dev e attendi la fine di altre build Rust prima di proseguire.'
    if (-not $PSCmdlet.ShouldProcess($repo, "Imposta $Version, esegue i controlli e genera l'installer NSIS")) { return }
    foreach ($path in $changes.Keys) {
        [IO.File]::WriteAllText($path, $changes[$path], [Text.UTF8Encoding]::new($false))
    }
    if (-not $SkipChecks) {
        foreach ($check in @('typecheck', 'test', 'check', 'format:backend', 'lint:backend')) {
            Invoke-Checked 'bun' @('run', $check)
        }
        Invoke-Checked 'cargo' @('test', '--locked', '--manifest-path', 'src-tauri/Cargo.toml')
    }
    $bundle = Join-Path $repo 'src-tauri/target/release/bundle/nsis'
    $name = "Memotape_${Version}_x64-setup.exe"
    $installer = Join-Path $bundle $name
    $signature = "$installer.sig"
    $latest = Join-Path $bundle 'latest.json'
    $notes = Join-Path $repo ".scratch/release-$Version.md"
    # L'Aggiornamento (ADR-0036): l'installer si firma con la chiave degli aggiornamenti.
    $key = Join-Path $env:USERPROFILE '.tauri/memotape.key'
    if (-not (Test-Path -LiteralPath $key)) { throw "Chiave degli aggiornamenti assente: $key" }
    $password = Read-Host 'Password della chiave degli aggiornamenti' -AsSecureString
    $env:TAURI_SIGNING_PRIVATE_KEY = $key
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = [Net.NetworkCredential]::new('', $password).Password
    # Solo qui gli artefatti dell'Aggiornamento: un `bun tauri build` qualunque non chiede la chiave.
    $updaterConfig = Join-Path $env:TEMP 'memotape-updater-artifacts.json'
    [IO.File]::WriteAllText($updaterConfig, '{"bundle":{"createUpdaterArtifacts":true}}', [Text.UTF8Encoding]::new($false))
    $buildStarted = [DateTime]::UtcNow
    try {
        Invoke-Checked 'bun' @('tauri', 'build', '--bundles', 'nsis', '--config', $updaterConfig)
    } finally {
        Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY, Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $updaterConfig -ErrorAction SilentlyContinue
    }
    foreach ($path in @($installer, $signature)) {
        if (-not (Test-Path -LiteralPath $path)) { throw "File assente: $path" }
        $file = Get-Item -LiteralPath $path
        if ($file.LastWriteTimeUtc -lt $buildStarted -or $file.Length -eq 0) {
            throw "La build non ha prodotto un nuovo file: $path"
        }
    }
    # Il manifest che le app leggono da releases/latest/download/latest.json.
    $manifest = [ordered]@{
        version   = $Version
        pub_date  = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssZ')
        platforms = [ordered]@{
            'windows-x86_64' = [ordered]@{
                signature = [IO.File]::ReadAllText($signature).Trim()
                url       = "https://github.com/inerba/memotape/releases/download/$tag/$name"
            }
        }
    }
    [IO.File]::WriteAllText($latest, ($manifest | ConvertTo-Json -Depth 4), [Text.UTF8Encoding]::new($false))
    Write-Host "`nInstaller pronto: $installer"
    Write-Host "SHA256: $((Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash)"
    Write-Host "Firma: $signature"
    Write-Host "Manifest: $latest"
    if (-not (Test-Path -LiteralPath $notes)) { Write-Warning "Note di rilascio assenti: $notes" }
    Write-Host "`nDopo aver revisionato e committato le modifiche di versione:"
    Write-Host "git tag -a $tag -m 'Memotape $Version'"
    Write-Host 'git push origin HEAD'
    Write-Host "git push origin $tag"
    Write-Host "gh release create $tag `"$installer`" `"$signature`" `"$latest`" --verify-tag --draft --title `"Memotape $Version`" --notes-file `"$notes`""
    Write-Host 'Le app vedono la versione quando pubblichi la bozza.'
} finally {
    Pop-Location
}

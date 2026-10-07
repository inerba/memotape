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
    $installer = Join-Path $repo "src-tauri/target/release/bundle/nsis/Memotape_${Version}_x64-setup.exe"
    $buildStarted = [DateTime]::UtcNow
    Invoke-Checked 'bun' @('tauri', 'build', '--bundles', 'nsis')
    if (-not (Test-Path -LiteralPath $installer)) { throw "Installer assente: $installer" }
    $file = Get-Item -LiteralPath $installer
    if ($file.LastWriteTimeUtc -lt $buildStarted -or $file.Length -eq 0) {
        throw "La build non ha prodotto un nuovo installer: $installer"
    }
    Write-Host "`nInstaller pronto: $installer"
    Write-Host "SHA256: $((Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash)"
    Write-Host "`nDopo aver revisionato e committato le modifiche di versione:"
    Write-Host "git tag -a $tag -m 'Memotape $Version'"
    Write-Host 'git push origin HEAD'
    Write-Host "git push origin $tag"
    Write-Host "gh release create $tag `"$installer`" --verify-tag --title `"Memotape $Version`" --generate-notes"
} finally {
    Pop-Location
}

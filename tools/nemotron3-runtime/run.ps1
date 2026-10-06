param(
    [Parameter(Mandatory = $true)][string]$Nemotron3Model,
    [string]$ModelsDirectory = "$env:APPDATA\it.memotape.desktop\models",
    [string]$ProbePath,
    [string]$ResultsDirectory
)

$ErrorActionPreference = 'Stop'
function Assert-Artifact([string]$ArtifactPath, $Metadata) {
    $file = Get-Item -LiteralPath $ArtifactPath
    if ($file.Length -ne $Metadata.size) { throw "dimensione inattesa: $ArtifactPath" }
    $digest = (Get-FileHash -LiteralPath $ArtifactPath -Algorithm SHA256).Hash
    if ($digest -ne $Metadata.sha256) { throw "SHA-256 inatteso: $ArtifactPath" }
}

$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path
if (-not $ProbePath) {
    $ProbePath = Join-Path $repoRoot 'src-tauri\target\nemotron3-proof\probe-target\debug\memotape-runtime-probe.exe'
}
if (-not $ResultsDirectory) {
    $ResultsDirectory = Join-Path $repoRoot 'src-tauri\target\nemotron3-proof\smoke'
}
$probe = (Resolve-Path -LiteralPath $ProbePath).Path
$diarizer = (Resolve-Path -LiteralPath $Nemotron3Model).Path
$catalog = Get-Content -LiteralPath (Join-Path $repoRoot 'src-tauri\src\managers\models.json') -Raw | ConvertFrom-Json
$artifacts = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'artifacts.json') -Raw | ConvertFrom-Json
Assert-Artifact $diarizer $artifacts.nemotron3
foreach ($fixture in $artifacts.fixtures.PSObject.Properties) {
    Assert-Artifact (Join-Path $repoRoot "src-tauri\tests\fixtures\$($fixture.Name)") $fixture.Value
}
New-Item -ItemType Directory -Force -Path $ResultsDirectory | Out-Null
$cases = @(
    @{ name = 'nemotron3'; path = $diarizer; mode = 'diar-offline'; language = 'auto'; fixture = 'parlato-due-voci.wav' },
    @{ name = 'nemotron3'; path = $diarizer; mode = 'diar-stream'; language = 'auto'; fixture = 'parlato-due-voci.wav' }
)
foreach ($model in $catalog.modelli) {
    $modelPath = Join-Path $ModelsDirectory ($model.url.Split('/')[-1])
    if (-not (Test-Path -LiteralPath $modelPath -PathType Leaf)) { throw "modello assente: $modelPath" }
    Assert-Artifact $modelPath $model
    if ($model.tipo -eq 'diarizzazione') {
        $cases += @{ name = $model.id; path = $modelPath; mode = 'diar-offline'; language = 'auto'; fixture = 'parlato-due-voci.wav' }
    } else {
        $languages = if ($model.accettaLingua -eq $false) { @('auto') } else { @('it', 'auto') }
        foreach ($language in $languages) {
            $cases += @{ name = $model.id; path = $modelPath; mode = 'asr-offline'; language = $language; fixture = 'parlato-it.wav' }
            if ($model.modalita -eq 'stream') {
                $cases += @{ name = $model.id; path = $modelPath; mode = 'asr-stream'; language = $language; fixture = 'parlato-it.wav' }
            }
        }
    }
}
$summary = @()
foreach ($backend in @('cpu', 'vulkan')) {
    foreach ($case in $cases) {
        $name = "$($case.name)-$backend-$($case.mode)-$($case.language)"
        $stdout = Join-Path $ResultsDirectory "$name.jsonl"
        $stderr = Join-Path $ResultsDirectory "$name.stderr.log"
        $fixture = Join-Path $repoRoot "src-tauri\tests\fixtures\$($case.fixture)"
        & $probe $backend $case.path $fixture $case.mode $case.language 1> $stdout 2> $stderr
        $exitCode = $LASTEXITCODE
        $records = @(Get-Content -LiteralPath $stdout | Where-Object { $_.StartsWith('{') } | ForEach-Object { $_ | ConvertFrom-Json })
        $failures = @(Get-Content -LiteralPath $stderr | Where-Object { $_.StartsWith('{') } | ForEach-Object { $_ | ConvertFrom-Json })
        $summary += [PSCustomObject]@{ name = $name; exitCode = $exitCode; records = $records; failures = $failures }
        $summary | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $ResultsDirectory 'summary.json')
        [PSCustomObject]@{ name = $name; exitCode = $exitCode; result = $records[-1] } | ConvertTo-Json -Depth 6 -Compress
    }
}
if (@($summary | Where-Object { $_.exitCode -ne 0 }).Count -gt 0) { exit 1 }

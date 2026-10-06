param([string]$Backend='vulkan')
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$root = Join-Path $repo '.scratch/nemotron3-diarizzazione/corpus08'
$exe = (Get-ChildItem -LiteralPath (Join-Path $repo 'src-tauri/target/debug/deps') -Filter 'memotape_lib-*.exe' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
$tests = @(& $exe --list 2>$null)
if (-not ($tests -contains 'engine::transcribe_cpp::windows_benchmark::ticket08_confronto_stessa_asr: test')) { throw 'Comparatore assente nel binario' }
$env:MEMOTAPE_NEMOTRON3_MODEL = 'D:\local\tauri\sbobino-deps\nemotron3-proof\Nemotron-3-Diarization-BF16.gguf'
$env:MEMOTAPE_COMPARE_BACKEND = $Backend
foreach ($case in (Get-Content (Join-Path $root 'corpus-manifest.json') -Raw | ConvertFrom-Json).cases) {
 $folder = Join-Path $root ('cases/' + $case.case)
 $env:MEMOTAPE_COMPARE_AUDIO = Join-Path $folder 'audio.wav'
 $env:MEMOTAPE_COMPARE_OUTPUT = Join-Path $folder ('comparison-' + $Backend)
 if (Test-Path -LiteralPath $env:MEMOTAPE_COMPARE_OUTPUT) { throw 'Risultato già presente: non sovrascrivere' }
 & $exe ticket08_confronto_stessa_asr --ignored --test-threads=1 --nocapture 1> (Join-Path $folder ('native-' + $Backend + '.stdout.log')) 2> (Join-Path $folder ('native-' + $Backend + '.stderr.log'))
 if ($LASTEXITCODE -ne 0) { throw ('Test nativo fallito: ' + $case.case) }
 foreach ($name in @('frozen-asr.json','nemotron3.rttm','sortformer.rttm','nemotron3-units.json','sortformer-units.json')) {
  if (-not (Test-Path (Join-Path $env:MEMOTAPE_COMPARE_OUTPUT $name))) { throw ('Output mancante: ' + $name) }
 }
 Write-Output ($case.case + ' ' + $Backend + ': comparatore nativo passato')
}
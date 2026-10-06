param(

    [int]$Seconds = 300,

    [string[]]$Backends = @('vulkan','cpu'),

    [int[]]$Inputs = @(1,2),

    [int]$TimeoutSeconds = 1200,

    [string]$TestExecutable

)

$ErrorActionPreference = 'Stop'

$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

$outputRoot = Join-Path $repo '.scratch\nemotron3-diarizzazione\benchmark08'

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null

$exe = if ($TestExecutable) { (Resolve-Path -LiteralPath $TestExecutable).Path } else { (Get-ChildItem -LiteralPath (Join-Path $repo 'src-tauri\target\debug\deps') -Filter 'memotape_lib-*.exe' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName }

if (-not (Test-Path -LiteralPath $exe)) { throw 'Eseguire cargo test --no-run e impostare il percorso del binario compilato.' }

$available = @(& $exe --list 2>$null)

if ($LASTEXITCODE -ne 0 -or -not ($available -contains 'engine::transcribe_cpp::windows_benchmark::ticket08_windows_benchmark: test')) { throw 'Il binario non contiene il test ticket08_windows_benchmark.' }

$env:MEMOTAPE_NEMOTRON3_MODEL = 'D:\local\tauri\sbobino-deps\nemotron3-proof\Nemotron-3-Diarization-BF16.gguf'

$env:MEMOTAPE_BENCH_SECONDS = "$Seconds"

$summary = @()

foreach ($backend in $Backends) {

    foreach ($count in $Inputs) {

        $name = "$backend-$count-$Seconds"

        $folder = Join-Path $outputRoot $name

        if (Test-Path -LiteralPath $folder) { throw "Output già presente: $folder" }

        New-Item -ItemType Directory -Path $folder | Out-Null

        $env:MEMOTAPE_BENCH_OUTPUT = $folder

        $env:MEMOTAPE_BENCH_BACKEND = $backend

        $env:MEMOTAPE_BENCH_INPUTS = "$count"

        $process = Start-Process -FilePath $exe -ArgumentList @('ticket08_windows_benchmark','--ignored','--test-threads=1','--nocapture') -WorkingDirectory $repo -WindowStyle Hidden -RedirectStandardOutput (Join-Path $folder 'stdout.log') -RedirectStandardError (Join-Path $folder 'stderr.log') -PassThru

        $watch = [Diagnostics.Stopwatch]::StartNew()

        $samples = @()

        $gpuSamples = @()

        $nextGpu = 0

        $timeout = $false

        while (-not $process.HasExited) {

            $process.Refresh()

            $samples += [PSCustomObject]@{ wallMs = $watch.ElapsedMilliseconds; cpuSeconds = $process.TotalProcessorTime.TotalSeconds; workingSet = $process.WorkingSet64; privateBytes = $process.PrivateMemorySize64 }

            if ($watch.Elapsed.TotalSeconds -ge $nextGpu) {

                $gpu = & nvidia-smi --query-gpu=timestamp,memory.used,utilization.gpu,utilization.memory --format=csv,noheader,nounits 2>$null

                $gpuSamples += [PSCustomObject]@{ wallMs = $watch.ElapsedMilliseconds; wholeGpu = $gpu; exitCode = $LASTEXITCODE }

                $nextGpu += 3

            }

            if ($watch.Elapsed.TotalSeconds -gt $TimeoutSeconds) {

                $timeout = $true

                Stop-Process -Id $process.Id

                break

            }

            Start-Sleep -Milliseconds 1000

        }

        $process.WaitForExit()

        $samples | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $folder 'process-samples.json')

        $gpuSamples | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $folder 'whole-gpu-samples.json')

        $reportPath = Join-Path $folder 'report.json'

        $validReport = $false

        if (Test-Path -LiteralPath $reportPath) {

            $report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json

            $validReport = $report.backend -eq $backend -and $report.inputs -eq $count -and $report.audio_ms -eq ([int][Math]::Floor($Seconds * 16000 / 480) * 30) -and $report.runtime_commit -eq 'e6672a8' -and $report.integrity.asr -and $report.integrity.audio_bytes -and $report.integrity.decoded_duration -and $report.integrity.text_final -and $report.integrity.reopen

        }

        $liveHealthy = $validReport -and (@($report.live_results | Where-Object { $_ -ne 'Ok(())' }).Count -eq 0)
        $record = [PSCustomObject]@{ name = $name; pid = $process.Id; exitCode = $process.ExitCode; timeout = $timeout; validReport = $validReport; liveDiarizationHealthy = $liveHealthy; processWallSeconds = $watch.Elapsed.TotalSeconds; samples = $samples.Count }

        $summary += $record

        $summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $outputRoot "matrix-$Seconds-summary.json")

        $record | ConvertTo-Json -Compress

    }

}

if (@($summary | Where-Object { $_.exitCode -ne 0 -or $_.timeout -or -not $_.validReport -or -not $_.liveDiarizationHealthy }).Count -gt 0) { exit 1 }
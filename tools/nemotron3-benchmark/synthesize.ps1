param([string]$Jobs)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$voices = @($synth.GetInstalledVoices() | ForEach-Object { [PSCustomObject]@{ name=$_.VoiceInfo.Name; id=$_.VoiceInfo.Id; culture=$_.VoiceInfo.Culture.Name; enabled=$_.Enabled } })
$folder = Split-Path -Parent $Jobs
$voices | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $folder 'tts-voices.json') -Encoding utf8
$format = New-Object System.Speech.AudioFormat.SpeechAudioFormatInfo(16000, [System.Speech.AudioFormat.AudioBitsPerSample]::Sixteen, [System.Speech.AudioFormat.AudioChannel]::Mono)
foreach ($job in (Get-Content -LiteralPath $Jobs -Raw -Encoding UTF8 | ConvertFrom-Json)) {
 if (Test-Path -LiteralPath $job.output) { throw 'Stem TTS esistente, non sovrascrivere' }
 $synth.SelectVoice($job.voice)
 $synth.Rate = 0
 $synth.Volume = 100
 $synth.SetOutputToWaveFile($job.output, $format)
 $synth.Speak($job.text)
 $synth.SetOutputToNull()
}
$synth.Dispose()
Write-Output '36 stem TTS creati con voci distinte; Elsa Desktop non conta come identità aggiuntiva.'
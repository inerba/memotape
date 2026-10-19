This release brings eighteen transcription models, a new noise cleaner, a more accurate speech detector and an installer half the size.

## New

- **Eighteen transcription models**
  Settings → Transcription now offers eighteen models that transcribe Italian, up from three: Cohere Transcribe, Canary, Qwen3-ASR, Voxtral, primeLine Parakeet and the whole Whisper family join Parakeet, Nemotron and Whisper Large v3 Turbo. Nemotron Streaming is the default for new installations, and Nemotron, Cohere Transcribe and Whisper Medium are marked as recommended. If you already picked a model, Memotape keeps it. Settings → About lists the credits and license of every model.

- **Vocabulary built into Whisper and Qwen3-ASR**
  With Whisper and Qwen3-ASR, your Vocabulary terms now guide the model while it transcribes. With the other models, Memotape still corrects each sentence after it is transcribed.

## Improvements

- **New audio cleaning with RNNoise**
  Audio cleaning now uses RNNoise instead of DeepFilterNet3, for imported files and for recordings. It starts instantly, so a recording no longer waits for the cleaner to load, and it runs on any processor.

- **More accurate speech detection**
  Memotape now uses Silero VAD v6.2 to find where speech starts and ends. It tells speech from background noise much better: noise no longer turns into a sentence of invented text, and very quiet speech is kept.

- **Smaller installer**
  The installer drops from 30 MB to 16 MB, and the installed app from about 126 MB to 88 MB, because ONNX Runtime and the DeepFilterNet3 model are no longer needed. When you update, the installer also removes the old files that the new version no longer uses.

- **Clearer notices and Windows 11 window buttons**
  Notices now stand out in both the light and the dark theme, and the window buttons look and behave like the ones in Windows 11.

## Fixes

- **No more invented sentences on silence**
  On a muted microphone or on background noise, Whisper could repeat your Vocabulary terms and Cohere could make up whole sentences. Sentences that are only noise are no longer sent to the model, and the Vocabulary is no longer passed as a prompt to models that would copy it.

- **A stuck sentence no longer stops the transcription**
  When a model gets stuck repeating itself, Memotape splits the sentence at its quietest point and transcribes the pieces again. A piece that still fails keeps the text recovered so far, followed by "[unintelligible]" in the interface language. A word repeated many times in a row is reduced to three.

## Changes

- **Speaker recognition uses Sortformer only**
  The Nemotron 3 Diarization option is gone. Recognize speakers always uses Sortformer, also if you had chosen Nemotron 3 before.

**Full Changelog**: https://github.com/inerba/memotape/compare/v0.5.2...v0.6.0

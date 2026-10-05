# Memotape

![Memotape cassette symbol](src-tauri/icons/128x128.png)

**Record, transcribe, and revisit conversations on your Windows PC.**

Memotape turns speech from audio files, videos, and recordings into searchable text. Record your microphone, system audio, or both; transcribe locally; then listen back, correct the text, and export your notes. No account or API key is required. Once you have downloaded a model, transcription works offline.

Built for meetings, interviews, lectures, and recorded conversations. Available for **Windows 10/11 x64**.

[Get started](#getting-started) · [Models](#transcription-models) · [Build from source](#build-from-source) · [Contribute](#contributing)

## Project status

Memotape is under active development. The repository currently identifies the app as version `0.1.0`; public distribution, the installer publisher, and the application license are still being defined. The installer is currently unsigned. See [Build from source](#build-from-source) to run the app or produce a local installer.

## What you can do

| Feature | What it offers |
| --- | --- |
| Local transcription | Transcribe audio and video with Nemotron, Whisper, or Parakeet, without uploading your audio. |
| Recording | Capture your microphone, system audio, or both, with pause/resume, level meters, and independent input gain controls. |
| Live transcription | Read the text while recording. Nemotron also shows provisional text while a phrase is still being spoken. |
| Separate inputs | Transcribe the microphone and system audio independently and read them as a conversation. |
| Speaker recognition | Identify up to four speakers per audio stream with Sortformer and give them names. For live recordings, recognition runs after Stop. |
| Library and Collections | Organize Tape files in folders, rename and move them, and send unwanted files to the Windows Recycle Bin. |
| Search | Find words in Tape titles, transcript phrases, and speaker names, then jump to the matching passage. |
| Audio and text together | Play a Tape, seek through its waveform, change playback speed, and follow the highlighted transcript. |
| Transcript corrections | Edit individual phrases and save corrections directly in the Tape without changing the audio. |
| Copy and export | Copy the full transcript as plain text or Markdown, copy an individual turn, or export a Markdown file. |
| Optional assistant access | Allow Claude or Codex to search and read your Library through a local, read-only MCP server. |

The interface is available in **English, Italian, French, Spanish, German, and Polish**, with light, dark, and Windows-matched themes. Spoken-language options depend on the selected model and are separate from the interface language.

## Getting started

After launching Memotape:

1. Open **Settings → Transcription**, download a transcription model, and select it. Nemotron is the default and recommended option. Downloads are verified before use.
2. In **Settings → General**, choose your **Library folder**, or keep the default `Documents\Memotape`. Collections are folders inside this Library.
3. Import a file or start a recording using one of the workflows below.

### Transcribe an existing file

1. Click **Import a file**, or drag one audio or video file into the window.
2. Choose the model and spoken language in the transcription options. Enable **Recognize speakers** if needed; this requires the separate Sortformer download.
3. Click **Transcribe**. Progress appears while the file is processed; the completed text appears when transcription finishes.
4. Open the resulting Tape to listen, correct phrases, rename speakers, copy text, or export Markdown.

The new Tape is saved in the selected Collection, or the Library root when no Collection is selected. The original file is preserved. If you cancel, the phrases already transcribed remain available to copy, but no new Tape is saved. A file with no detected speech also produces no Tape.

### Record a conversation

1. Open the **New recording** menu and choose **Microphone**, **System audio**, or **Both**. Choose specific devices in **Settings → Recording and audio** if needed.
2. Enable live transcription to see the text during recording. With **Both**, the **Separate inputs** setting transcribes each input independently; this uses two model instances and needs more memory.
3. Start recording. Use the meters and input gain controls to adjust levels, and pause/resume to leave unwanted intervals out of the saved audio.
4. Click **Stop**. If live transcription is enabled, let the remaining transcription and optional speaker recognition finish.

The recording is saved as a Tape. You can also record without live transcription and transcribe the Tape later.

### Review and reuse the text

- Click a phrase to correct it. **Enter** or leaving the phrase saves; **Esc** cancels the edit.
- Use a turn's time or play button to listen from that passage. **Space** toggles playback when you are not typing.
- Search the Library with **Ctrl+K** to find matching text and open the relevant passage.
- Use **Copy text**, **Copy turn**, or **Export Markdown…** to reuse your transcript.

Transcribing an existing Tape again asks for confirmation because it replaces its transcript, corrections, and speaker names. Only one recording or transcription can run at a time; you can still consult other Tape files while it runs.

## Transcription models

Download and manage models in **Settings → Transcription**. You only need one transcription model to get started; Sortformer is optional.

| Model | Role | Text during live recording | Approximate download |
| --- | --- | --- | --- |
| Nemotron Streaming 3.5 0.6B | Default transcription model | Provisional text, then completed phrases | 560 MB |
| Whisper Large v3 Turbo | Alternative transcription model | Completed phrases | 620 MB |
| Parakeet TDT v3 0.6B | Alternative transcription model | Completed phrases | 549 MB |
| Sortformer 4spk v2.1 | Speaker recognition, up to four speakers | Applied after recording | 139 MB |

Sizes are rounded decimal MB for the packaged models, not memory requirements. Processing uses Vulkan when available and falls back to the CPU; performance depends on your hardware and the model. Parakeet detects the spoken language automatically rather than accepting a manual language choice.

Models are stored in `%APPDATA%\it.memotape.desktop\models`. Interrupted downloads can resume; completed files are checked against their expected size and SHA-256. The [model catalog](src-tauri/src/managers/models.json) is the source of truth for download URLs, checksums, sizes, and model licenses.

## Supported files

| Type | Accepted extensions |
| --- | --- |
| Audio | `.mp3`, `.wav`, `.m4a`, `.flac`, `.ogg`, `.opus`, `.mpga`, `.mpeg`, `.aiff` |
| Video / media containers | `.mp4`, `.mkv`, `.mov`, `.m4v`, `.webm` |
| Memotape documents | `.tape` |

An accepted extension does not guarantee that its embedded audio codec is supported. For example, AC-3 audio inside an MKV and MPEG-PS video in a `.mpeg` file are unsupported. AVI, WMV, FLV, TS, and MTS are outside the supported formats. Memotape decodes audio locally without requiring FFmpeg.

### What is a Tape?

A **Tape** is a `.tape` file that keeps audio, transcript, phrase timings, speaker names, and metadata together. Recordings and successfully transcribed files become Tape files in your Library.

Internally, it is a ZIP archive containing the mix in Ogg/Opus, a JSON transcript, and an optional waveform. Separate-input recordings also include the individual audio tracks. You can move a Tape between folders or open one outside the Library. The local search index can be rebuilt from the Tape files.

Markdown export is an explicit action. Phrase timings support playback inside the app; they are not included in copied text or exported Markdown.

## Privacy and assistant access

Recording, transcription, speaker recognition, Library search, and playback happen on your PC. Model downloads and update checks use the network. On a Windows installation without WebView2, the installer also needs a connection to download that runtime.

Assistant access is **off by default**. To enable it, open **Settings → Assistants** and turn on **Allow assistants to read the Library**. That page provides connection snippets using the actual installed executable path for Claude Code, Claude Desktop, and Codex.

The assistant starts `memotape.exe --mcp` as a local stdio server, which also works with the desktop app closed. It can search the Library, list Collections and Tape files, and read transcript passages. It cannot modify Tape files or start recordings or transcriptions.

**Text read by an external assistant may be sent to that assistant's servers.** Enable access only if you want that assistant to read your Library. Files moved manually while Memotape is closed appear in the assistant's index after you reopen the app.

## Build from source

Development and packaging target **Windows x64**. Run the commands below from the repository root in PowerShell.

### Prerequisites

| Dependency | Requirement |
| --- | --- |
| Rust | Stable MSVC toolchain, `x86_64-pc-windows-msvc`, version 1.90 or newer |
| Visual Studio | Visual Studio 2022 with the C++ workload and its VC++ redistributable files |
| Bun | 1.4 |
| Node.js | 22.22 or newer |
| CMake | Available on `PATH` |
| Vulkan SDK | LunarG SDK installed; configure `VULKAN_SDK` below |
| ONNX Runtime | Official Windows x64 1.24.2 package, extracted outside the repository |

Download [ONNX Runtime 1.24.2 for Windows x64](https://github.com/microsoft/onnxruntime/releases/download/v1.24.2/onnxruntime-win-x64-1.24.2.zip). The expected ZIP SHA-256 is:

```text
8e3e9c826375352e29cb2614fe44f3d7a4b0ff7b8028ad7a456af9d949a7e8b0
```

### Configure native dependencies

Create a local `.cargo/config.toml` at the **repository root**, replacing the example paths with your installed locations. This file is ignored by Git.

```toml
[env]
LOCALAPPDATA = { value = "src-tauri/target", relative = true, force = true }
VULKAN_SDK = { value = 'C:\VulkanSDK\<version>', force = false }
ORT_LIB_LOCATION = 'D:\dependencies\onnxruntime-win-x64-1.24.2\lib'
ORT_PREFER_DYNAMIC_LINK = "1"
```

The local `LOCALAPPDATA` override lets the native transcription build use a short junction path. Keep this Cargo configuration at the root because the backend scripts run there. `build.rs` locates the Visual Studio redistributables through `vswhere` or `VCToolsRedistDir` and copies the required runtime DLLs.

### Run the desktop app

```powershell
bun install
bun tauri dev
```

Download a transcription model through the app before trying to transcribe. The frontend development server alone does not provide the native audio or transcription features.

### Run the checks

Run all six checks before submitting a change:

```powershell
bun run typecheck
bun run test
bun run check
bun run format:backend
bun run lint:backend
cargo test --manifest-path src-tauri/Cargo.toml
```

Use `bun run fix` for frontend formatting/lint fixes and `cargo fmt --manifest-path src-tauri/Cargo.toml` for Rust formatting. Only run one Rust build, test, or lint process at a time; they share `src-tauri/target`. Stop `bun tauri dev` before building the installer.

Optional smoke tests with real models require all three transcription models, plus Sortformer for the speaker-recognition test:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -- --ignored --test-threads=1
```

Keep these tests sequential: concurrent model loading in the Vulkan smoke tests can crash the process. Automated checks do not replace manual checks of recording devices, playback, and installation on a clean Windows PC.

### Build the installer

```powershell
bun tauri build
```

The NSIS installer is written to `src-tauri/target/release/bundle/nsis/Memotape_<version>_x64-setup.exe`. It bundles the native runtime libraries and installs for the current user. Models are downloaded separately through Settings. See the [installer notes](AGENTS.md#installer) for packaging details and verification steps.

## Troubleshooting

| Problem | What to check |
| --- | --- |
| The selected model is missing | Download and select it in **Settings → Transcription**. Speaker recognition also needs Sortformer. |
| A file's audio format is unsupported | Check the audio codec as well as the extension; a supported container can contain unsupported audio. |
| A microphone or output device is unavailable | Check Windows devices and the device selected in **Settings → Recording and audio**. |
| Recording levels are too low or clip | Adjust the gain next to each input meter; lower it when the meter reaches red. |
| A native build fails | Check CMake on `PATH`, the Vulkan SDK, the ONNX Runtime `lib` path, and the root `.cargo/config.toml`. |
| Manually moved Tape files are missing from search | Bring the app back to the foreground or reopen it so the Library index refreshes. |

Desktop logs are stored in `%LOCALAPPDATA%\it.memotape.desktop\logs\Memotape.log`.

When reporting a problem, include the app version, Windows version, relevant hardware, selected model, steps to reproduce, and the relevant error or log excerpt. Remove personal transcript content and private paths before sharing logs or examples. Use the repository's issue tracker when available.

## Contributing

For substantial changes, discuss the intended behavior before implementing it. [PRODUCT.md](PRODUCT.md) defines product requirements, and [CONTEXT.md](CONTEXT.md) defines the domain vocabulary. Much of the detailed project documentation is currently in Italian.

- Keep changes focused and run the six checks above.
- Add tests for behavior changes and describe any manual verification in your pull request.
- Update `PRODUCT.md` when changing a product requirement; record architectural decisions in `docs/adr/`.
- Do not edit the generated `src/bindings.ts` by hand; a debug app launch regenerates it from Rust commands and events.
- Use conventional commit prefixes such as `feat:`, `fix:`, `docs:`, and `refactor:`. The pre-commit hook formats staged frontend files.

### Repository guide

| Path | Purpose |
| --- | --- |
| [`src/`](src/) | React interface, feature modules, shared components, and translations |
| [`src-tauri/src/`](src-tauri/src/) | Rust commands, managers, audio pipeline, transcription, Tape format, Library, player, and MCP server |
| [`src-tauri/tests/fixtures/`](src-tauri/tests/fixtures/) | Audio/video fixtures for automated verification |
| [`PRODUCT.md`](PRODUCT.md) | Product requirements and implemented milestones |
| [`CONTEXT.md`](CONTEXT.md) | Domain glossary |
| [`AGENTS.md`](AGENTS.md) | Detailed development instructions, architecture, and known pitfalls |
| [`docs/adr/`](docs/adr/) | Architectural decisions |
| [`docs/research/`](docs/research/) | Technical research and supporting evidence |
| [`docs/brand/`](docs/brand/) | Brand assets and usage guidelines |

## License and acknowledgments

**The application license has not yet been selected; this repository does not currently include an application `LICENSE` file.** Dependency and model licenses are separate and do not define the license of Memotape itself.

The project started from the [create-tauri-react](https://github.com/MrLightful/create-tauri-react) template, distributed under the MIT license. Memotape uses Tauri and React for its desktop interface, transcribe-cpp for local speech processing, and Rust audio components for decoding and recording. Bundled third-party license texts are in [`src-tauri/resources/licenses/`](src-tauri/resources/licenses/); model license identifiers are listed in the [model catalog](src-tauri/src/managers/models.json).

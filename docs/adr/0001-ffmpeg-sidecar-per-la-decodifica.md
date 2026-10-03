---
status: superseded by ADR-0002
---

# ffmpeg come sidecar per la decodifica

Tutta la decodifica di audio e video passa da un `ffmpeg.exe` LGPL (build BtbN `lgpl` statica) incluso come sidecar Tauri e avviato da Rust con `std::process::Command`, invece che da librerie Rust. Symphonia 0.6.1 non legge MPEG, AVI, WMV, FLV, TS e MTS, né AC-3 e HE-AAC dentro MP4, MOV e MKV, mentre il requisito chiede di coprire l'intero elenco dei formati video. Si accetta un sidecar da circa 128 MiB. Una build minimale con soli demuxer e decoder audio è il passo successivo quando il peso dell'installer conta. Le build GPL (gyan.dev, auto-download di `ffmpeg-sidecar`) sono escluse.

## Considered Options

- **Symphonia + adapter libopus**: bundle leggero, ma lascia senza supporto 6 formati su 19.
- **`ffmpeg-next`**: in "maintenance-only mode". Su MSVC servono libclang e vcpkg, e per restare LGPL bisogna comunque spedire le DLL.

Fonte: `docs/research/decodifica-ffmpeg.md`.

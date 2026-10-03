---
status: accepted
---

# Niente ffmpeg: decodifica e Registrazione solo in Rust

L'app non include ffmpeg. "Estrai solo audio" è stato tolto dal prodotto, la decodifica dei file passa da Symphonia (con `symphonia-adapter-libopus` per Opus) e le Registrazioni sono scritte in OGG/Opus dai crate `opus` + `ogg`. Si rinuncia ai formati che Symphonia non legge, in cambio di un installer leggero, senza un binario LGPL da 128 MiB e senza un processo esterno da gestire. Sostituisce l'ADR-0001.

Fonte: `docs/research/decodifica-ffmpeg.md` (§1.1, §1.4).

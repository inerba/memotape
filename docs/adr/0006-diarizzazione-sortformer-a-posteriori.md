---
status: accepted
---

# Diarizzazione con Sortformer, solo a posteriori

La Diarizzazione usa Sortformer 4spk v2.1 tramite `transcribe-cpp` 0.2.4 (GGUF Q8_0, 139 MB, NVIDIA Open Model License), scaricato dal catalogo dei modelli come "modello di diarizzazione". Gira solo sull'audio intero:
- su un file, durante Trascrivi;
- dopo Stop, sul mix o sugli Ingressi scelti.

Il numero di Parlanti è automatico, al massimo 4.

Nemotron-3-Diarization (fino a 8 Parlanti, streaming) è migliore, ma oggi è supportato solo dalla PR #175 di transcribe.cpp, non rilasciata e senza un GGUF pubblicato: si adotta quando entra in una release, e con lui si valuta la Diarizzazione dal vivo. Non c'è un'impostazione per scegliere l'algoritmo finché ne esiste uno solo.

Fonte: `docs/research/diarizzazione.md`.

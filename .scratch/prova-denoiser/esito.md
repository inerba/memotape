# Prova d'ascolto: DFN3 contro RNNoise 0.1 e 0.2

PROTOTIPO usa e getta, binario B di `../alleggerire-installer/spec.md`. Data: 18 ottobre 2026.

## Candidati

- **DFN3**: l'uscita dell'app, generata dal test ignorato `dfn3_campione_reale_confronta_pcm_e_modello` (stesso codice di `PcmStream`/`DeepFilter`, uscita diretta, `atten_lim` 100).
- **RNNoise 0.1**: crate `nnnoiseless` 0.5.2, pesi originali del 2018, gli stessi che usa OBS 32.2.2.
- **RNNoise 0.2**: xiph/rnnoise tag `v0.2` (`904a876d`), modello `0b50c45` (`rnnoise_data-0b50c45.tar.gz`, SHA-256 `4ac81c5c…4b43`, 22 270 507 byte), compilato con MSVC `/arch:AVX2` senza RTCD.

Tutti a 48 kHz mono, frame da 480 campioni. Per i due RNNoise il ritardo di un frame è compensato come in OBS.

## Campioni

`campioni/restaraunt_noisy.mp3` (il campione dell'ADR-0019) e il corpus `../pulizia-audio/verification-05/corpus` (silenzio, rumore continuo e intermittente, soffio sintetico, voce attenuata di 30 dB, «sì/no»), convertiti con ffmpeg in WAV 48 kHz mono.

## Ascolto alla cieca

`ascolto/NN-campione/`: `0-originale.wav` e tre uscite `A`, `B`, `C` in ordine casuale per campione. La chiave è in `chiave-NON-APRIRE.json` (non versionata): si apre dopo aver annotato i giudizi.

## Misure (non alla cieca)

Silero v4 dell'app, soglia 0,4, onset di 2 frame da 30 ms, sull'uscita ricampionata a 16 kHz. Attenuazione = RMS dell'uscita rispetto all'originale.

| Campione | Originale | DFN3 | RNNoise 0.1 | RNNoise 0.2 |
|---|---|---|---|---|
| voce attenuata −30 dB: inizio / parlato | 1590 ms / 5400 ms | **nessuna Frase** / 0 ms (uscita −217 dB, silenzio digitale) | 1860 ms / 4710 ms | 1800 ms / 4890 ms |
| «sì/no»: inizio / parlato | 120 / 510 ms | 150 / 450 ms | 120 / 510 ms | 150 / 480 ms |
| ristorante: inizio / parlato | 150 / 20 160 ms | 450 / 15 690 ms | 150 / 19 140 ms | 330 / 17 100 ms |
| ristorante: attenuazione | | −4,8 dB | −2,5 dB | −2,7 dB |
| rumore intermittente: parlato / attenuazione | 0 ms | 0 ms / −237 dB | **420 ms** (falso avvio) / −5,1 dB | 0 ms / −30,1 dB |
| soffio sintetico: attenuazione | | −54 dB | −0,1 dB | −72 dB |
| silenzio, rumore continuo | nessuna Frase | nessuna | nessuna | nessuna |

CPU, sola elaborazione, Ryzen 7 3700X, un Ingresso, release, RTF (secondi di calcolo per secondo di audio) sul ristorante: DFN3 0,083–0,088 (`DfTract::process`, tre giri), RNNoise 0.1 0,004, RNNoise 0.2 0,015 (AVX2, senza RTCD).

## Esito

Ascolto dell'utente, 18 ottobre 2026: «B è molto buona; siamo quasi ai livelli di C, quindi B va bene». Chiave aperta dopo il giudizio: sul ristorante (e sulla voce attenuata) B è **RNNoise 0.2** e C è **DFN3**. Le lettere cambiano da campione a campione: il giudizio vale per il ristorante, il campione dell'ADR-0019.

Verdetto secondo Q13:

- (a) Ascolto: RNNoise 0.2 accettato dall'utente come quasi al livello di DFN3.
- (b) Silero: RNNoise 0.2 non perde inizi di Frase rispetto a DFN3; sulla voce attenuata di 30 dB li conserva dove DFN3 restituisce silenzio.

Esito secondo Q17: passa RNNoise 0.2 (RNNoise 0.1 attenua poco e ha un falso avvio sul rumore intermittente). Si apre la decisione sulla sua build MSVC. Peso della libreria statica compilata (modello normale, AVX2): 5 701 470 byte, circa 5,0 MB compressi con xz, contro circa 11 MB compressi di DFN3 tra modello e tract.

## Riprodurre

```
cd .scratch/prova-denoiser/banco && cargo build --release
target/release/prova-denoiser nnn|rnn02 <in48.wav> <out48.wav>
target/release/prova-denoiser onset <in16.wav> <silero.onnx>       # serve onnxruntime.dll accanto all'exe
target/release/prova-denoiser dfn3time <in48.wav> <DeepFilterNet3_onnx.tar.gz>
```

I sorgenti di RNNoise 0.2 vanno clonati in `rnnoise-0.2/` con i pesi estratti in `src/`. Il test DFN3 dell'app scrive in `../pulizia-audio/dfn3-diretto/`: farne una copia prima e ripristinarla dopo.

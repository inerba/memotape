# Confronto del campione fornito dall'utente

Archivio della diagnosi prima della correzione. Dopo l'ascolto, l'utente ha
scelto DFN3 diretto: la miscela qui chiamata «attuale» è la versione precedente.
I WAV e le misure qui presenti restano invariati come confronto storico.
Le nuove verifiche e l'uscita dell'app sono in `../dfn3-diretto/`.

Campione: `campioni/restaraunt_noisy.mp3`, 22,097625 s, mono, 48 kHz.
La prova usa il decoder e l'adattatore libDF/Tract effettivi del progetto.
L'MP3 originale non viene modificato. Nessun cambiamento a Impostazioni o Libreria.

- `01-originale.wav`: PCM decodificato, esportato a 16 bit per ascolto.
- `02-integrazione-attuale.wav`: uscita della miscela SNR attuale, con limite a 6 dB.
- `03-dfn3-diretto.wav`: uscita spettrale DFN3 senza miscela con l'originale.
- `confronto.json`: misure su PCM f32 prima dell'esportazione, anche per secondo.

Durata e formato identici, nessuna normalizzazione del volume. Attenuazione
dell'energia complessiva: integrazione 0,675185 dB, DFN3 diretto 4,834699 dB.
Senza riferimento di voce pulita questa non è una misura di SNR, qualità
percettiva o conservazione delle parole. La parte finale silenziosa produce
rapporti non finiti, rappresentati come null nel JSON.

Il test nativo `dfn3_campione_reale_confronta_pcm_e_modello` passa (96,58 s,
debug, compilazione esclusa). Richiede `MEMOTAPE_DFN3_SAMPLE` con il percorso
del campione. Il test diagnostico `dfn3_ticchettio_attenuazione_effettiva`
resta intenzionalmente rosso e ignorato nella suite ordinaria finché il
difetto non è corretto: su impulsi sintetici DFN3 annulla il segnale, la
miscela attuale lo conserva con 0 dB di attenuazione (0,91 s in release).

La curva triangolare usa SNR inferiore a -10 dB come motivo per conservare
interamente l'originale, neutralizzando la soppressione del rumore dominante.
Due correzioni sperimentali sono state provate e RITIRATE:

- curva monotona, limite 6 dB: ticchettio attenuato 5,998 dB, ma onset Silero
  della fixture a -30 dB spostato da 480 a 2820 ms;
- attenuazione 3 dB sotto -10 dB SNR: onset spostato da 480 a 2100 ms.

Il comportamento di produzione è stato ripristinato alla curva precedente.
La diagnosi è confermata, la correzione definitiva resta aperta. Non abbassare
le asserzioni sulla voce debole e non presentare i WAV come un fix già applicato.
Nessun commit o rilascio.

# 08: Verificare qualità italiana, ritardo e stabilità su Windows

**What to build:** La prova produce un resoconto riproducibile che dice se Nemotron migliora l'attribuzione e raggiunge l'obiettivo realtime sul PC verificato, senza confondere test del core, misure native e funzionalità ancora non provate.

**Blocked by:** 06 — Rettificare e dividere il testo già comparso dal vivo; 07 — Riconoscere i Parlanti dal vivo con gli Ingressi separati.

**Status:** ready-for-human

**Modello consigliato:** GPT-6.1 Sol (`gpt-6.1-sol`).

**Sforzo consigliato:** `high` (alto).

**Motivo della scelta:** Collaudo strutturato su casi annotati e misure riproducibili; contano gli esperimenti reali e la precisione del resoconto.

- [ ] Il confronto con Sortformer usa lo stesso campione italiano annotato, gli stessi Ingressi e la stessa ASR su 2–4 voci; riporta separatamente DER del diarizer ed errori di attribuzione nel testo.
- [ ] Si verificano anche 5–8 voci, alternanze rapide, sovrapposizioni, rumore e silenzi. I fixture sintetici restano smoke test e non sostituiscono il confronto sul parlato reale.
- [ ] Le misure audio→attribuzione e testo disponibile→etichetta sono distinte dal ritardo ASR; hardware, runtime/modello, preset, backend e distribuzione dei ritardi sono riportati. Il target 1–2 secondi viene valutato con queste misure.
- [ ] Registrazioni lunghe verificano memoria, carico, durata e code con uno e due Ingressi su CPU/Vulkan; Pausa, Stop, saturazione, guasto e Annulla preservano audio e testo.
- [ ] Il flusso completo include rettifiche/suddivisioni, analisi finale, esiti provvisori/non determinati, copia/esportazione, player, riapertura e lettura dei Tape vecchi nelle sei lingue.
- [x] I sei controlli del repository passano e gli smoke con modelli reali sono sequenziali; il resoconto separa i controlli automatici dalle verifiche native e non certifica PC non provati.
- [x] L'esito può essere una prova non riuscita: i limiti vengono riportati e non mascherati cambiando modello o runtime. Sortformer resta disponibile e non si promuove automaticamente Nemotron a predefinito.
- [x] Continuazione, riconoscimento globale delle persone, allineatore aggiuntivo per Whisper, distribuzione pubblica dei pesi e pubblicazione di installer restano fuori da questi ticket.


Le impostazioni consigliate sono un punto di partenza, non un benchmark sul ticket. Avviare l'implementazione solo su richiesta dell'utente e dopo la chiusura dei ticket bloccanti; `ready-for-agent` indica che il ticket è specificato.


## Esecuzione autorizzata

Il 5 ottobre 2026 sono stati autorizzati i ticket 06, 07 e 08 in sequenza, dopo il completamento verificato dello 05, ciascuno con un agente pulito GPT-6.1 Sol e sforzo high. Avviare solo dopo la chiusura dei prerequisiti; nessun commit o pubblicazione e autorizzato.

## Esito della prova — 6 ottobre 2026

Ticket non chiuso done. [Report riproducibile](../report08.md), [procedura UI](../ui08-da-verificare.md), [strumenti](../../../tools/nemotron3-benchmark/README.md). CPU1/CPU2 attivano realmente `LiveDiarizationLagging`: prova realtime non riuscita. ASR/audio/finale e Tape restano integri. Vulkan p95 callback audio→attribuzione circa 1,73 s sulla fixture sintetica, massimi 2,27–2,32 s; nessuna attestazione UI o qualità italiana.

Mancano campioni italiani reali annotati indipendentemente da 2–4 e 5–8 voci, con licenza/provenienza verificate, e accesso al collaudo della UI Windows (API CUA native disabilitate). Lo scorer DER/testo e il confronto con ASR congelata sono pronti; nessuna groundtruth derivata dal modello. Quattro stress nativi da 300 s non certificano Registrazioni lunghe da un’ora con due dispositivi. Metriche callback/memoria/carico, smoke WASAPI e core Tape sono distinti da UI/clipboard/playback ascoltato. Le prime cinque caselle restano aperte perché la copertura è parziale.

Il coordinatore è stato avvisato e ha richiesto eventuali percorsi audio/annotazioni all’utente. Nessun commit/push/pubblicazione o modifica persistente delle Impostazioni; default Sortformer conservato. Modifiche 01–07 preservate mediante confronto con baseline pre-08, senza ripristinarle.

Controlli finali: typecheck, 106 frontend, Biome (159 file), fmt, clippy, 234 Rust/10 ignored tutti verdi; cinque smoke nativi/runner e matrice 4×300 s eseguiti in sequenza, otto test metriche verdi. Review Standards/Spec documentata in [review-08.md](../review-08.md), finding risolte, nessuna attestazione UI.

## Estensione autorizzata: prove create ed eseguite

Su richiesta «Inventale tu queste prove», costruiti e misurati cinque campioni TTS/letture pubbliche montate con 2/3/2/4/8 SpeakerID. [Addendum](../report08-addendum.md): DER a tre soglie proxy, ASR congelata identica, copertura temporizzata e namespace corretto; N3 Human8 produce7etichette eDER28,22%principale. Reference indipendente ma non goldmanuale, quindi caselle qualità restano parziali e stato ready-for-human. Due integrazioni native nuove sul TapeHuman8 riuscite; sei controlli ripetuti106frontend235Rust12ignored e14testmetriche verdi. Nessuna attestazione UI o promozione automatica.

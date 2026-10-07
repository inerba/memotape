# Verifica dell'avvio della Registrazione

Piano e risultati del 7 ottobre 2026. Le evidenze qui sotto distinguono
test automatici, componente DFN3 reale e browser con IPC simulato dal
collaudo end-to-end Tauri/WASAPI, ancora da svolgere.

## Misure

Misurare per sessione con orologio monotono: clic e feedback visivo, completamento
dei salvataggi delle Impostazioni, validazioni/prenotazioni, preparazione della
Pulizia audio per Ingresso, apertura e avvio dei dispositivi, disponibilità dei
writer, conferma di Registrazione avviata, primo audio conservato e primo testo.
Separare attività in parallelo da attese sul percorso dell'audio; non sommarne
le durate come se fossero tutte sequenziali.

Confrontare prima e dopo sullo stesso PC e con gli stessi dispositivi:

- primo avvio dell'app e clic immediato su Registra;
- clic dopo la preparazione in background;
- Registrazioni successive nella stessa apertura dell'app;
- Microfono, Audio di sistema silenzioso, Entrambi;
- pulizia spenta, attiva su un Ingresso, attiva su entrambi;
- Trascrizione dal vivo spenta/accesa, motore pronto/non pronto;
- Parlanti spenti/accesi, distinguendo la verifica del modello dall'analisi
  finale dopo Stop;
- build debug e release identificate separatamente.

Riportare tempi osservati e memoria residente prima/dopo la preparazione e tra
sessioni. Le ripetizioni devono verificare che il lavoro costoso sia riusato e
che lo stato audio non contamini una sessione successiva. Nessuna promessa di
tempo assoluto senza baseline; nessuna percentuale UI stimata dai log precedenti.

## Criteri funzionali

- La preparazione è visibile già durante `flush`; doppi clic e altre Attività
  incompatibili restano esclusi.
- «Puoi parlare» dipende dalla conferma del backend e non da un timer, dalla
  comparsa del primo testo o dall'ampiezza del segnale. Il loopback silenzioso
  deve poter partire senza aspettare un pacchetto non silenzioso.
- La conferma arriva dopo che tutti gli Ingressi richiesti e il percorso di
  scrittura sono pronti. I timer condividono l'origine dell'audio conservato.
- Annulla prima dell'avvio non produce Tape, non perde la vista precedente e
  impedisce partenze tardive. La corsa fra Annulla e avvio si risolve nel backend:
  se l'avvio ha già vinto, Stop conserva l'audio anziché eliminarlo.
- Eventi tardivi, di altre sessioni o di preparazioni ormai annullate non
  possono riattivare la UI. Un guasto anticipato resta visibile.
- Audio/testo precedenti restano consultabili dopo Annulla o errore prima
  dell'avvio; eventuali Correzioni manuali non vengono perse.
- Dopo il segnale di avvio, una fixture marcata dimostra che la prima parola
  compare nell'Ogg/Tape riaperto, anche con ASR ancora in preparazione.
- Stop e Pausa continuano a funzionare mentre ASR è in preparazione; le code
  non devono crescere senza limite se la preparazione resta bloccata.
- Tastiera, lettore di schermo e le sei lingue coprono preparazione, conferma,
  annullamento ed errori senza ripetere annunci a ogni tick.

Nell'implementazione: sei controlli di progetto richiesti, una compilazione Rust
alla volta. I test automatici non sostituiscono il collaudo nella UI Tauri e la
verifica dell'audio conservato.

## Risultati dell'implementazione

- Readiness esplicita: `preparing`, `cleaning`, `devices`, `recording`, con UUID.
  La fase recording arriva dopo Capture, Mixer e tutti i writer. Non attende ASR
  né il primo pacchetto del loopback. Log monotoni distinguono attesa pulizia,
  dispositivi/writer e avvio del percorso audio.
- Test deterministici: Annulla immediato durante flush senza partenza tardiva,
  Annulla prima dei controlli, UUID obsoleto, Stop dopo avvio, pending DFN3,
  accensione tardiva solo su PCM futuro, spegnimento senza acquisire il filtro,
  bypass e avviso singolo su guasto. OFF/ON mentre il filtro è in attesa,
  anche fra due blocchi PCM, riemette la preparazione per il solo Ingresso. Il limite ASR si verifica saturando la
  coda con un consumatore fermo: nessuna attesa della cattura, guasto esplicito.
- DFN3 reale, senza dispositivi né ASR: test ignored
  `dfn3_preparato_riusa_piano_e_azzera_audio_fra_sessioni`, debug MSVC.
  La preparazione dei due filtri è parallela; il prestito è esclusivo.

| Formato | Ingressi | Preparazione a freddo | Prestito pronto |
| --- | ---: | ---: | ---: |
| 48 kHz mono | 1 | 1,1693 s | 14,9 µs |
| 48 kHz stereo | 2 | 1,1801 s | 6,8 µs |
| 16 kHz mono | 2 | 1,2680 s | 5,9 µs |

Il test confronta l'intero PCM elaborato fra sessioni: uguaglianza esatta,
anche dopo il rilascio di un filtro lasciato a metà senza Finish. Verifica
anche durata e impossibilità di un secondo prestito contemporaneo.
Il prestito pronto non include il reset precedente né l'apertura WASAPI.
Le misure storiche di diagnosi (due filtri sequenziali circa 2,36 s) non sono
una baseline end-to-end comparabile: non si dichiara un'accelerazione totale.

Memoria campionata ogni 50 ms nel processo di test separato, senza ASR:
vedi `native-memory.json`, `native-reuse.log`, `native-reuse.err`.
Il campione iniziale registra 7,79 MiB working set e 2,57 MiB privati;
il massimo privato osservato è 43,50 MiB (working set 61,96 MiB in quel
campione). Include runtime, piani e buffer del test; non è una stima del
consumo totale dell'app né una misura isolata della sola cache. Non si
accumulano configurazioni nella cache; allocazioni transitorie e pagine
trattenute dall'allocatore possono restare visibili nel processo.

Browser sulla build frontend con IPC simulato (`ui-probe.cjs`): avvio,
Annulla prima della conferma, nuova sessione, UUID obsoleto ignorato,
apertura dispositivi, guasto ASR prima della readiness, conferma visiva,
Pausa e Stop. Nessun errore JavaScript. Screenshot a 1200×800 e 880×600:
`preparing-ui.png`, `started-ui.png`, `started-min-ui.png`. Non è il WebView2
nativo; non prova il microfono, la prima parola nell'Ogg o il lettore di schermo.
Il detector Impeccable sui tre file UI modificati ha restituito zero rilievi.

## Da collaudare sul percorso reale

La matrice di Misure sopra resta da eseguire su Tauri/WASAPI e release:
prima parola dopo la conferma nel Tape riaperto, dispositivi assenti/scollegati,
Audio di sistema silenzioso, due Ingressi, Pause/Stop durante caricamento ASR,
clic immediato all'apertura dell'app e latenze visuali effettive.
Queste prove non sono state sostituite da misure del filtro o da IPC simulato.

## Controlli di progetto

Tutti verdi sullo stato finale:

| Controllo | Esito |
| --- | --- |
| `bun run typecheck` | OK |
| `bun run test` | 134 pass, 0 fail |
| `bun run check` | OK |
| `bun run format:backend` | OK |
| `bun run lint:backend` | OK, `-D warnings` |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 304 pass, 0 fail, 35 ignored |

Il test dei bindings generati è incluso e passa. Gli ignored non sono dichiarati
collaudati: il solo benchmark DFN3 di questo ticket è stato eseguito a parte.
Build frontend finale riuscita, inclusa la protezione dagli eventi dopo la
conclusione del comando; nessun installer o release prodotti. Review Spec e
Standards separate, con correzione dei rilievi. Il delta usa una copia del
checkout prima dell'implementazione, perché HEAD non distingue le modifiche
preesistenti dell'utente da quelle di questo ticket.


## Limite del completamento ASR

L'audio è salvato mentre ASR carica il modello. La coda ammette al massimo
2000 messaggi per Ingresso (circa 60 secondi PCM; anche ClosePhrase occupa
uno slot); se satura, la Trascrizione dal vivo fallisce esplicitamente e
la cattura continua. Il caricamento nativo ASR già iniziato non è
interrompibile: Stop può attendere che termini prima di completare il Tape.
Il limite della coda impedisce crescita senza limite; non prova un limite
al tempo necessario per finalizzare il documento.

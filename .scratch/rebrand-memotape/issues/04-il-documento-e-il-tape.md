# 04: Il documento è il Tape, `.tape`

**What to build:** le Registrazioni e i file trascritti diventano `.tape`, e l'utente li vede chiamati Tape:
- estensione `tape` in `SOURCE_EXTENSIONS`, nei due specchi Rust e TS;
- passano a `.tape`:
  - i controlli sul percorso che oggi pretendono `.bino`;
  - `from_args`, che prende il primo `.tape` dopo l'eseguibile;
  - `sync` della Libreria, che cerca solo i `.tape`;
  - i nomi dei file creati (`<nome>.tape`, `Registrazione <data ora>.tape`);
  - `Decoder::open`, che riconosce un `.tape`;
- protocollo del player `tape` (`http://tape.localhost/…`);
- associazione dell'installer con classe `Memotape.Tape`, descrizione "Tape (Memotape)" ed estensione `tape`, e l'hook di disinstallazione che toglie `.tape`;
- nelle sei lingue "Bino/Bini" diventa "Tape", invariabile e maschile in italiano ("un Tape", "i Tape", "the Tape", "die Tape"…), compresi "Audio, video e Tape" e il numero dei Tape nella Libreria;
- PRODUCT.md (tipo "Tape (Memotape)" in Esplora file) e AGENTS.md parlano di Tape e `.tape`.

Il formato dentro lo zip resta lo stesso (`version` 1, stesse voci), e Memotape non legge i `.bino` (ADR-0014).

A controlli verdi si sposta la Libreria del PC di sviluppo, una volta sola:
- i 6 Bini di `Documenti\Sbobino` (uno nella radice, 5 nella Raccolta `Mediolanum`) vanno in `Documenti\Memotape`, rinominati `.tape`, senza perdere la Raccolta;
- gli Ogg delle due Registrazioni interrotte passano da `Documenti\Sbobino\.sbobino` a `Documenti\Memotape\.memotape`;
- i `.ogg` e i `.txt` della vecchia app restano in `Documenti\Sbobino`.

Spec: `.scratch/rebrand-memotape/spec.md` (storie 6–14, 31–33).

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] Nessun file del repo dice Bino/Bini o `.bino`, tranne i documenti storici, l'ADR-0014 e l'_Avoid_ del glossario
- [ ] Il test sullo specchio di `SOURCE_EXTENSIONS` è verde con `tape`
- [ ] Registra e Trascrivi creano file `.tape` in `Documenti\Memotape`, e un `.tape` trascinato da Esplora file si apre
- [ ] Dopo lo spostamento la Libreria mostra i 6 Tape con la Raccolta Mediolanum, e ognuno si apre con testo, player e Forma d'onda
- [ ] I sei controlli sono verdi

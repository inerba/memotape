# 05: Pulizia finale e verifica

**What to build:** il rebrand è completo e verificato, dall'installer agli Assistenti:
- i file di `docs/brand` si chiamano `memotape-*`, e `LINEE-GUIDA.md`, i commenti di `BrandMark` e ogni altro riferimento li seguono;
- DESIGN.md, i contratti in `.impeccable` e gli ultimi commenti parlano di Memotape e Tape;
- i logotipi con "sbobino" disegnato restano fuori dall'app finché non arriva il nuovo wordmark (fuori perimetro, si farà con `/logo-design`).

Spec: `.scratch/rebrand-memotape/spec.md` (storie 3, 7–9, 26, 27, 29, 30).

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] `git grep -i -E 'sbobino|\bbin[oi]\b|\.bino'` trova solo ADR 0001–0013, l'ADR-0014, `.scratch/sbobino*`, `.scratch/rebrand-memotape`, `docs/research`, l'_Avoid_ del glossario e i riferimenti a `sbobino-deps`
- [ ] Nessun file del repo ha `sbobino` o `bino` nel nome, tranne quelli nei percorsi ammessi qui sopra
- [ ] `Memotape_<versione>_x64-setup.exe` supera la "Verifica dell'installer" di AGENTS.md:
  - installa in una cartella di prova, e le DLL vengono caricate da lì;
  - in Esplora file un `.tape` appare come "Tape (Memotape)" con l'icona, e il doppio clic lo apre;
  - la disinstallazione toglie `Memotape.Tape`, `.tape` e la chiave di disinstallazione
- [ ] `memotape.exe --mcp` risponde a `initialize`, e `tools/list` contiene `list_tapes`
- [ ] L'app di sviluppo apre i 6 Tape, e Registra e Trascrivi funzionano
- [ ] I sei controlli sono verdi

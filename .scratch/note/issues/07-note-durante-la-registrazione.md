# 07: Note durante la Registrazione

**What to build:** l'utente scrive Note mentre registra. La vista della Registrazione ha la scheda Note, che si salva sempre da sola (qualunque sia l'interruttore) in una bozza accanto agli Ogg temporanei, così sopravvive a un guasto. Allo Stop le Note entrano nel Tape come `note.json`; se il Tape non si crea e resta solo l'Ogg, ogni Nota non vuota diventa `<titolo> <nome>.md` accanto all'Ogg, senza sovrascrivere. Ctrl+T inserisce il tempo della Registrazione, pause escluse; i tempi diventano cliccabili dopo lo Stop.

**Blocked by:** 03, 05

**Status:** ready-for-agent

- [ ] Test Rust: la bozza entra nel Tape allo Stop; senza Tape diventa file `.md` accanto all'Ogg, numerati se il nome è preso (come `la_trascrizione_dal_vivo_salva_il_markdown_solo_senza_tape`)
- [ ] Il tempo inserito da Ctrl+T coincide con quello del Tape dopo lo Stop, anche con una Pausa nel mezzo
- [ ] Verifica a mano con una Registrazione vera, con e senza Trascrivi dal vivo
- [ ] `docs/sviluppo/registrazione.md` descrive la bozza delle Note
- [ ] I sei controlli verdi

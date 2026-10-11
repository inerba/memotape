# 04: Salva, Salva da solo e uscita protetta

**What to build:** l'utente sceglie come salvare le Note. Nella riga dei nomi, a destra, «Salva» (Ctrl+S) e l'interruttore «Salva da solo», acceso di partenza e uguale per tutti i Tape (impostazione `note_salva_da_solo`, anche in Impostazioni → Generale). Acceso salva dopo 1 s di pausa e uscendo dal campo; spento, una Nota con modifiche ha un pallino accanto al nome e le bozze restano passando da una Nota o da una scheda all'altra. Aprire un altro Tape, la Home, la Libreria o le Impostazioni, chiudere la finestra, Ctrl+N/Ctrl+O, avviare un'Attività o un Taglio sul Tape chiedono «Salvare le modifiche alle Note?» con Salva, Non salvare e Annulla. Durante un'Attività sul Tape le Note sono in sola lettura.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] `note_salva_da_solo` vero di default e quando manca nel file delle Impostazioni
- [ ] Lo stato delle bozze vive in un reducer della route con il suo `*.test.ts`: casi in cui il dialog compare e casi in cui no
- [ ] La chiusura della finestra con bozze aperte passa dal dialog; Annulla lascia la finestra aperta
- [ ] Il pallino si legge «Modifiche non salvate»; Salva è spento senza modifiche
- [ ] Testi nelle sei lingue; il pannello delle scorciatoie elenca Ctrl+S
- [ ] `docs/sviluppo/impostazioni-e-servizi.md` e `docs/sviluppo/frontend.md` aggiornati
- [ ] I sei controlli verdi

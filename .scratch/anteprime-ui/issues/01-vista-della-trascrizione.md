# 01: Vista della trascrizione (Copione, Intervista, Nastro)

**What to build:** l'impostazione `vista_trascrizione` e le tre rese della trascrizione. Regole in `../spec.md`, «1 · Vista della trascrizione»; anteprime nel passo 1 di `../anteprime.html`.

**Blocked by:** nessuno.

**Status:** ready-for-agent

- [ ] `Settings::vista_trascrizione` (enum `VistaTrascrizione`: `copione` predefinita, `intervista`, `nastro`, `#[serde(default)]`), `DEFAULT_SETTINGS` allineato, bindings rigenerati.
- [ ] Impostazioni → Generale: la scelta, con le etichette nelle sei lingue.
- [ ] `transcript-view.tsx`: le tre rese sugli stessi `Turn` di `turnsOf`; la resa di oggi si toglie. Vale per Tape, Registrazione dal vivo (Parziali in corsivo) e Frasi dopo Annulla.
- [ ] Battuta incerta con «?» o nodo vuoto e nome accessibile «Parlante non determinato».
- [ ] Separatore «Pausa di …» per silenzi stimati ≥ 5 s in Copione e Nastro: esportare o riusare la stima di `silenceMs` (`phrases.ts`), con test.
- [ ] Copione con gli Ingressi separati: icona, colonna fino a 140 px, nome tagliato con tooltip.
- [ ] Tempo, ▶, Copia turno, rinomina, editor del Turno, Unisci, Segui l'audio e la pillola di ritorno funzionano in tutte e tre.
- [ ] DESIGN.md (componente firma Turno), PRODUCT.md, AGENTS.md aggiornati.
- [ ] Prova nell'app: tre viste, due temi, 880 px, una Registrazione dal vivo breve.
- [ ] I sei controlli passano.

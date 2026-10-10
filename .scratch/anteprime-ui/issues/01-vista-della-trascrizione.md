# 01: Vista della trascrizione (Copione, Intervista, Nastro)

**What to build:** l'impostazione `vista_trascrizione` e le tre rese della trascrizione. Regole in `../spec.md`, «1 · Vista della trascrizione»; anteprime nel passo 1 di `../anteprime.html`.

**Blocked by:** nessuno.

**Status:** done

- [x] `Settings::vista_trascrizione` (enum `VistaTrascrizione`: `copione` predefinita, `intervista`, `nastro`, `#[serde(default)]`), `DEFAULT_SETTINGS` allineato, bindings rigenerati.
- [x] Impostazioni → Generale: la scelta, con le etichette nelle sei lingue.
- [x] `transcript-view.tsx`: le tre rese sugli stessi `Turn` di `turnsOf`; la resa di oggi si toglie. Vale per Tape, Registrazione dal vivo (Parziali in corsivo) e Frasi dopo Annulla.
- [x] Battuta incerta con «?» o nodo vuoto e nome accessibile «Parlante non determinato».
- [x] Separatore «Pausa di …» per silenzi stimati ≥ 5 s in Copione e Nastro: esportare o riusare la stima di `silenceMs` (`phrases.ts`), con test.
- [x] Copione con gli Ingressi separati: icona, colonna fino a 140 px, nome tagliato con tooltip.
- [x] Tempo, ▶, Copia turno, rinomina, editor del Turno, Unisci, Segui l'audio e la pillola di ritorno funzionano in tutte e tre.
- [x] DESIGN.md (componente firma Turno), PRODUCT.md, AGENTS.md aggiornati.
- [ ] Prova nell'app: tre viste, due temi, 880 px, una Registrazione dal vivo breve.
- [x] I sei controlli passano.

## Comments

### 2026-10-10

- Logica pura in `phrases.ts`: `pausesOf` (silenzio stimato con `silenceMs` prima di ogni turno, ≥ 5 s, dalla fine più tarda delle Frasi precedenti: con gli Ingressi separati una voce può parlare sopra il turno di prima) e `pauseDuration` («9 s», «1 min 20 s», «1 min»). Le pause dentro un turno non si segnano. Rust: test del predefinito e di un `settings.json` senza il campo.
- `TranscriptView` legge la vista da `useSettings`; `TurnBlock` passa gli stessi dati a `CopioneRow`, `IntervistaRow` e `NastroRow`. Il test del componente che c'era ora gira sulle tre viste dentro `SettingsProvider`.
- Deviazione: ▶, Riascolta, Unisci e Copia turno stanno in una barretta sospesa (`shadow-float`) in alto a destra del turno in tutte e tre le viste, non «accanto al tempo»: nell'Intervista e nel Nastro il tempo è a sinistra e una barretta lì coprirebbe l'inizio del testo. Riascolta compare solo in hover o con il focus.
- Intervista con l'editor del Turno: il nome è assoluto sopra il campo e la prima riga rientra della sua larghezza (`--rientro`, misurato con un `ResizeObserver`; la textarea eredita `text-indent`).
- Colonna del tempo a 48 px (non 40) per i Tape oltre l'ora; colonna dei nomi del Copione fissa a 88 o 140 px. Senza etichette (mix non diarizzato) il Copione non ha la colonna dei nomi.
- Provato nell'app (CDP) su un Tape a Ingressi separati e diarizzato: Copione chiaro 1200 px e scuro 880 px, Intervista scura, Nastro chiaro a 880 px con l'ascolto e la barretta col focus, la scelta in Impostazioni (rimessa a Copione). Non provati: la Registrazione dal vivo con i Parziali (coperta solo dal test del componente), la rinomina e Unisci a mano, il lettore di schermo.

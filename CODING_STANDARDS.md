# Standard di codice per la review

Regole che chiedono giudizio, da applicare a ogni diff. Valgono anche le «Regole del codice» di `AGENTS.md`. Le regole meccaniche le fanno rispettare gli strumenti, quindi in review si saltano: Biome (`bun run check`), i colori fissi e le ombre diverse da `shadow-float` (`scripts/check-design.ts`, nello stesso `check`), `src/components/ui` cambiato a mano (pre-commit), chiavi i18n mancanti nelle sei lingue (`src/lib/i18n.test.ts`) e bindings non aggiornati (`i_bindings_committati_sono_aggiornati`).

## Glossario

- Codice, commenti, commit e testi dell'interfaccia usano i termini di `CONTEXT.md` (Sorgente, Attività, Frase, Turno, Parlante, Tape…).
- Le parole in _Avoid_ non sostituiscono un termine del glossario. Molte restano lecite in un altro senso, quello indicato tra parentesi: `live` come nome di modulo, «volume» del player. Segnala solo l'uso come sinonimo del termine.
- Un concetto nuovo che ricorre nel diff e non è nel glossario va aggiunto a `CONTEXT.md` nello stesso commit.

## Dove sta il codice

- Logica pura del frontend: in un `.ts` della sua feature, con il `*.test.ts` accanto; il `.tsx` resta React. Niente test sui componenti (eccezione: `app/routes/home-state.ts`).
- Un modulo che cresce per motivi diversi (per esempio `library.ts` con Recenti, tabella e menu) si divide per motivo.
- `audio_toolkit` ed `engine` restano senza Tauri; la logica dei manager che si può testare senza `AppHandle` va in una funzione pura.

## Interfaccia

- Le regole nominate di `DESIGN.md` che nessuno strumento controlla: Display-Names-Only (Commissioner solo per titoli e marchio), Tabular Time (cifre tabulari per tempi e conteggi), la salvia solo per l'audio in ascolto, colori dei Parlanti solo nei loro segni.
- Testi: voce sobria di `PRODUCT.md` («Brand Commitments»), frasi brevi, termini del glossario, nessuna battuta. Le traduzioni dicono la stessa cosa dell'italiano.
- Ogni scorciatoia segue lo stato del suo pulsante (`src/features/shortcuts/`).

## Documenti

- Un comportamento cambiato aggiorna nello stesso commit il suo documento di `docs/sviluppo/` (tabella in `AGENTS.md`), e `PRODUCT.md` o `DESIGN.md` se tocca un requisito o il sistema visivo.
- Una scelta architetturale non ovvia ha un ADR in `docs/adr/`.

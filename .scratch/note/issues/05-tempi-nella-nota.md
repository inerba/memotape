# 05: Tempi nella Nota

**What to build:** un tempo tra quadre in una Nota, `[m:ss]`, `[mm:ss]` o `[h:mm:ss]`, è un Tempo nella Nota: fuori dalla riga del cursore diventa una pillola che porta lì il player, come il tempo di un Turno; oltre la durata del Tape resta testo. Ctrl+T, nella scheda Note, inserisce al cursore il tempo attuale del player. Un Taglio, dall'app o da un Assistente, sposta i tempi di tutte le Note con la linea del tempo e toglie le quadre a quelli dentro il tratto tolto, lasciando il resto del Markdown com'è.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] Una regola sola per riconoscere i tempi (fuori dal codice inline e dai link) e per spostarli dato un elenco di tratti tolti, in Rust e in TypeScript, con gli stessi casi di test nei due lati
- [ ] Il Taglio riscrive `note.json` insieme all'audio e al documento; un test prova lo spostamento, lo smontaggio e la conservazione carattere per carattere del resto
- [ ] La pillola si legge «Ascolta da 12:40»; in sola lettura resta cliccabile
- [ ] Ctrl+T elencato nel pannello delle scorciatoie
- [ ] I sei controlli verdi

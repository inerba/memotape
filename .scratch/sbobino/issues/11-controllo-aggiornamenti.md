# 11: Controllo aggiornamenti

**What to build:** all'avvio, se c'è connessione, l'app controlla se esiste una versione più recente e propone nella status bar il link per scaricarla. Offline il controllo fallisce in silenzio.

**Blocked by:** 01 (Scaffold), repo GitHub deciso (`inerba/memotape`); l'editore resta da definire ma non blocca

**Status:** done

- [x] `GET https://api.github.com/repos/<owner>/<repo>/releases/latest` con User-Agent, ignorando le prerelease
- [x] Il `v` iniziale di `tag_name` viene tolto, e il confronto con `semver` usa `cmp_precedence` rispetto a `package_info().version`
- [x] Il link `html_url` si apre con `tauri-plugin-opener`. Errori di rete e limiti di rate sono silenziosi
- [x] Test del confronto delle versioni (prefisso `v`, prerelease, versione uguale)
- [x] `AGENTS.md` / `PRODUCT.md` aggiornati, controlli verdi

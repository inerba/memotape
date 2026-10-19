---
status: accepted
---

# Aggiornamento con tauri-plugin-updater e firma degli aggiornamenti

Data: 19 ottobre 2026. Termine in `CONTEXT.md` (Aggiornamento).

## Contesto

Fino alla 0.6.0 Memotape controllava all'avvio l'ultima release di GitHub e l'avviso apriva la pagina della release: l'utente scaricava e lanciava l'installer a mano. L'installazione automatica era fuori dal perimetro di `PRODUCT.md`.

## Decisione

- **Plugin:** l'Aggiornamento usa `tauri-plugin-updater` (con `tauri-plugin-process` per il riavvio). All'avvio il plugin legge `latest.json` dall'ultima release di GitHub (`releases/latest/download/latest.json`); questo controllo sostituisce quello di `updates.rs` sull'API delle release. Offline o con un errore non c'è Aggiornamento, in silenzio, e una prerelease non conta.
- **Firma:** l'installer di ogni release è firmato con la chiave degli aggiornamenti di Tauri (minisign), non con Authenticode, che resta fuori perimetro. La chiave pubblica è in `tauri.conf.json`; il plugin rifiuta un installer che non le corrisponde.
- **Chiave privata:** sta fuori dal repo in `$env:USERPROFILE\.tauri\memotape.key`, protetta da password, con una copia di chiave e password nel password manager. La password si chiede a ogni release e non si salva.
- **Un clic:** «Scarica e installa» scarica con l'avanzamento nell'avviso, poi avvia l'installer NSIS in modalità `passive`; l'app si chiude e si riapre aggiornata. Il pulsante è disattivato durante un'Attività, e prima di installare passano le bozze delle Note (`proteggi`). Se qualcosa fallisce l'avviso offre «Riprova» e il link alla release.
- **Release:** `bun run release` controlla `main` pulito e allineato, fa la build firmata, scrive `latest.json` dal `.sig`, crea il tag `v<versione>` e la release in bozza con le note di `.scratch/release-<versione>.md` e i tre file (installer, `.sig`, `latest.json`). Le app vedono la versione solo quando la bozza è pubblicata.

## Conseguenze

- **Se la chiave privata si perde** (o la sua password), le installazioni esistenti rifiutano ogni Aggiornamento firmato con una chiave nuova: ogni utente deve reinstallare a mano una volta la versione con la chiave pubblica nuova. Non rigenerare la chiave senza questo prezzo in mente.
- Una release senza `latest.json`, o con un `latest.json` sbagliato, non arriva alle app: per questo lo scrive lo script.
- Chi ha la 0.6.0 o precedenti non ha il plugin: aggiorna a mano un'ultima volta alla 0.6.1.

## Alternative scartate

- **Download e lancio dell'installer con codice nostro:** niente chiave da custodire, ma nessuna verifica di autenticità (chi entrasse nell'account GitHub farebbe installare a tutti un file suo) e più codice da mantenere per download, lancio e riavvio, che il plugin ha già risolto.
- **Un passo «scaricato, riavvia quando vuoi»:** l'installer pesa circa 16 MB e si scarica in pochi secondi; un clic solo basta.

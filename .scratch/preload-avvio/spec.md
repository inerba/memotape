# Preload di avvio — accordo del 7 ottobre 2026

- Seconda proposta: logo originale con il solo mozzo che ruota, senza barrette.
- Tema e lingua scelti prima della prima visualizzazione; Sistema segue Windows.
- Nessuna durata minima: sparisce al primo contenuto React, compresa la pagina di errore.
- Libreria, ASR e filtri audio continuano in background.
- Dopo 10 s compare il messaggio di attesa lunga, senza dichiarare un errore.
- Rispetta movimento ridotto; i controlli della finestra restano operativi.
- Errori effettivi del bootstrap espongono la stessa copia Ricarica della pagina di errore.

La finestra `main` non viene creata automaticamente. `startup::create_main`
la crea nascosta dopo il caricamento di SettingsStore e Models, con il tema
salvato e i testi delle sei traduzioni in uno script di inizializzazione.
Allinea il fondo nativo al tema effettivo prima di mostrarla. Il documento
iniziale contiene markup, stili e timer indipendenti dal bundle React.
Un MutationObserver rimuove il preload appena `#root` ha un elemento; non
lo elimina con una fallback Suspense vuota. Il piccolo modulo dei controlli
usa la stessa eccezione Window API di WindowControls, senza nuovi comandi IPC.
Il modulo dei controlli importa React dinamicamente: il chunk iniziale è
circa 18 KB e resta operativo mentre si carica quello dell'app. Il tema
esplicito viene anche iniettato nel documento, perché nella prova WebView2
le media query seguivano Windows anziché la scelta scura salvata.
Lo stesso tema resta in `data-theme` anche dopo la rimozione del preload:
colori, varianti Tailwind e marchio della Home lo seguono. SettingsProvider
lo aggiorna prima della pittura; soltanto Sistema ascolta i cambi di Windows.

Originali dei file modificati in `prima/`, per distinguere questo intervento
dalle modifiche preesistenti. Nessun commit o pubblicazione autorizzati.

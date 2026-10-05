---
status: accepted
---

# Il drop dei file passa da Tauri, niente trascinamento HTML5 nella pagina

Un file trascinato da Esplora file sulla finestra si apre come Sorgente, come con Importa un file. Per avere il suo percorso si usa `dragDropEnabled: true` e l'evento `onDragDropEvent` della webview, che dà i percorsi già durante il passaggio sopra la finestra. Così il velo può dire se il file è accettato prima che venga rilasciato.

Il prezzo è il trascinamento HTML5 dentro la pagina. Su Windows, con il drop attivo, wry (0.57) registra un suo `IDropTarget` sulla WebView2 e non inoltra a Chromium i trascinamenti senza file: `dragstart`, `dragover` e `drop` tra elementi della pagina non arrivano più. Per questo sparisce il trascinamento di un Bino su una pillola delle Raccolte: per spostarlo resta Sposta in….

Alternative scartate:
- lasciare `dragDropEnabled: false` e ricavare il percorso dal `File` del `drop` HTML5 con `chrome.webview.postMessageWithAdditionalObjects` e un secondo handler di `WebMessageReceived` (`ICoreWebView2File::Path`). Servono `webview2-com` e `windows` come dipendenze dirette, fissate alle versioni di Tauri e riallineate a ogni aggiornamento, codice COM `unsafe` e il presupposto che l'handler di wry continui a ignorare i messaggi che non sono stringhe. In più, durante il passaggio si sa solo che sono file, non quali;
- tenere il trascinamento sulle Raccolte riscrivendolo con i pointer event (`pointerdown/move/up`, `elementFromPoint`, un'anteprima disegnata a mano): funziona, ma è codice in più per un gesto che Sposta in… copre già.

Conseguenza: niente `draggable` né eventi `drag*` HTML5 nel frontend. Su Windows non danno errori, semplicemente non scattano. Un trascinamento interno, se un giorno servisse, va fatto con i pointer event.

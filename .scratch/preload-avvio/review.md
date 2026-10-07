disposition: fix

## persistence

Pass. `PRODUCT.md`, `DESIGN.md` e `docs/brand/LINEE-GUIDA.md` riportano il contratto del preload; il lavoro è code-led e il riferimento HTML è solo di critica, quindi non è richiesto uno stato di comp-round. Tutti i nove capture richiesti esistono, hanno le dimensioni dichiarate e mostrano il viewport corretto senza regioni nere o vuote. `ui-report.json` e `native-report.json` confermano i percorsi browser e WebView2; i sei controlli standard sono verdi (134 test frontend, 305 test Rust, 35 ignorati, build frontend e Rust passate). Il detector Impeccable non è stato eseguito per la superficie Windows nativa; il giudizio usa i capture, le sonde e la lettura del codice forniti.

## fidelity

| Elemento | Esito | Evidenza |
| --- | --- | --- |
| Lockup Memotape centrato, simbolo da 76 px e logotipo da 184 px | match | `startup-light-1200.png`, `startup-dark-1200.png`, `startup-dark-880.png`; misure e asset coerenti con `DESIGN.md` e la guida del brand |
| Solo il mozzo ruota in 2,8 s; quadrato, strisce, bobina esterna e logotipo restano fermi | match | `index.html` anima solo `.startup-hub`; `ui-report.json` rileva il cambio di trasformazione senza barrette o percentuali |
| Carta/inchiostro e varianti del marchio in chiaro e scuro durante il preload | match | capture chiari/scuri con OS opposto; `native-report.json` mostra tema `dark` iniettato da Rust mentre WebView2 segnala `dark: false` |
| Tema scelto dopo il montaggio React | contradicted | `startup-native-ready.png` è chiaro anche se `native-report.json` riporta `savedTheme: "scuro"`; `index.html` elimina `data-startup-theme` quando `#root` diventa pronto e `global.css` torna alla sola media query di Windows |
| Copia di avvio e messaggio lento a 10 s senza spostare il gruppo | match | i tre capture slow e `slowWithoutShift: true` |
| Movimento ridotto | match | `reducedMotion: true`; il CSS porta l'animazione a `none` |
| Errore reale con animazione ferma e Ricarica | match | `startup-error-1200.png`; `moduleFailureShowsReload` e `animationPausedOnFailure` |
| Riduci, Ingrandisci/Ripristina e Chiudi prima del chunk React | match | `startup-native.png`, `startup-native-ready.png` e la prova WebView2 con ritardo di 5 s |
| Titolo/logo piccolo nella barra custom | adaptation | convenzione Windows coerente con la barra del titolo di `DESIGN.md`; mantiene i controlli nativi disponibili e non altera il lockup centrale |
| Lingue, minimo 880×600 e rimozione prima della Libreria | match | probe in italiano/polacco, capture 1200×800 e 880×600, `removedBeforeLibraryReady: true` |

## ceiling

Raggiunto per la superficie approvata: logo reale, carta e inchiostro coerenti, una sola animazione funzionale, nessun gradiente, materiale finto, ombra dura, glyph o barra di progresso; il messaggio lento è subordinato e non introduce movimento. Nessun dispositivo nativo del perimetro Windows è inutilizzato. La copertura resta quella dei capture e delle sonde dichiarate; non include installer, altre macchine o un detector automatico.

## material_fixes

1. **P1 — contratto tema:** conserva il tema esplicito anche dopo la rimozione del preload e prima del primo frame React; con `savedTheme: "scuro"` il capture `startup-native-ready.png` deve restare scuro anche quando Windows è chiaro, poi ricattura lo stato ready e la variante opposta per verificare il passaggio.

## keep

Preservare il lockup originale, la sola rotazione del mozzo a 2,8 s, l'assenza di barrette, il messaggio a 10 s senza salto, la rimozione immediata al contenuto reale e il percorso di errore con Ricarica.

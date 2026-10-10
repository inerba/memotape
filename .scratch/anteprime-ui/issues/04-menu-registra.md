# 04: Menu Nuova registrazione

**What to build:** il menu con un riquadro per Ingresso. Regole in `../spec.md`, «4 · Menu Nuova registrazione».

**Blocked by:** 03.

**Status:** done

- [x] `record-menu.tsx`: riquadro per Ingresso, interruttore nella testata, «Spento» in chiaro, un solo rientro, Sensibilità a 13 px; l'ultimo Ingresso acceso non si spegne, come oggi.
- [x] Trascrivi dal vivo in fondo con la riga di spiegazione nelle sei lingue.
- [x] Documenti aggiornati, i sei controlli passano.

## Comments

### 2026-10-10

- Nuove chiavi `recording.inputOff` e `recording.liveHint` nelle sei lingue. «Spento» è `aria-hidden`: lo stato lo dice già l'interruttore.
- Deviazione: il menu è largo 352 px e non 320 come nell'anteprima. Misurato nell'app, a 13 px «Bilanciato» non ci stava già in italiano a 320 px; a 352 px la Sensibilità sta su una riga anche con le etichette tedesche e polacche.
- `Segmented` con `fill` passa da griglia a colonne uguali a `flex` con `flex-1`: un'etichetta lunga allarga il suo segmento invece di uscirne. `fill` lo usa solo il menu.
- Nessuna logica pura nuova, quindi nessun test nuovo. Provato nell'app via CDP nei due temi, senza avviare Registrazioni né cambiare impostazioni; non provato a 880 px né con l'interfaccia in altre lingue (solo le larghezze delle etichette, sostituite nel DOM).

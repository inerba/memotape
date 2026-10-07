# Opzioni della Registrazione, Copia turno e nome del Microfono

Status: ready-for-agent

Data: 7 ottobre 2026. Requisiti in `PRODUCT.md` ("Opzioni della Registrazione e
nome del Microfono"), decisione sul nome in ADR-0027. Usare la skill
`impeccable` (shape → craft → polish) per menu e barra, contro `DESIGN.md` e
`.impeccable/surfaces/src-app-routes-home-tsx.md`.

## 1. Copia turno solo testo

- `turnText` / `turnTextWithBody` (`src/features/transcription/phrases.ts`):
  niente etichetta, solo il corpo. `CopyTurn` resta com'è.
- Aggiornare i test in `phrases.test.ts` che si aspettano `Nome: …`.

## 2. Toggle al posto delle checkbox

- `shadcn add switch` (mai modificato a mano).
- Sostituire `Checkbox` in `SettingCheckbox`, `RecordingInputs` (anche la
  variante a riquadri di Impostazioni), `CleaningProfile`. Il nome
  `SettingCheckbox` può restare o diventare `SettingSwitch`: scelta di chi
  implementa, con un solo componente.
- Controllare gli altri usi di `Checkbox` (`rg "ui/checkbox"`): resta solo dove
  non è un'impostazione salvata subito.

## 3. Sensibilità segmentata

- Un componente (radiogroup accessibile, frecce da tastiera) con i quattro
  livelli di `settings.protection.levels.*`, usato da `CleaningProfile` al
  posto del `NativeSelect` in tutte e tre le superfici. Le spiegazioni dei
  livelli (`FieldHelp`, `aria-describedby`) restano.

## 4. Menu ▾ di Nuova registrazione (`record-menu.tsx`)

```
Microfono                    [switch]   ← registra da
   Pulisci il rumore         [switch]
   Sensibilità  [Spento|Sensibile|Bilanciato|Selettivo]
   Riconosci i parlanti      [switch]   (disattivato senza Trascrivi dal vivo)
Audio di sistema             [switch]
   …idem
────────────
Trascrivi dal vivo           [switch]
```

- Le opzioni di un Ingresso spento si nascondono.
- Riconosci i parlanti: con Entrambi `parlantiMicrofono`/`parlantiSistema`
  sotto il loro Ingresso; con un solo Ingresso `parlantiMix` sotto quello acceso
  (`parlantiRegistrazione`).
- Larghezza circa 320px, nessuno scorrimento verticale.

## 5. Barra della Registrazione (`recording-panel.tsx`)

- Intestazione: pallino, timer, stato a sinistra; Pausa e Stop a destra.
- Una striscia per Ingresso: icona (`Mic`/`Speaker`) e nome completo
  (`settings.recording.inputs.*`, non `recording.levels.*` "Sistema"),
  indicatore, `GuadagnoSelect` a destra.
- Indicatore disegnato (non `<meter>`): circa 10–12px di altezza, angoli
  arrotondati, scala in dB da −60 a 0 (`levelPercent`), zone verde/giallo/rosso
  agli stessi punti di oggi (giallo da −15 dBFS, rosso vicino a 0), segno di
  picco che decade. `role="meter"` con `aria-valuenow/min/max` e l'etichetta
  attuale. Solo token del tema.
- Sotto l'indicatore: `CleaningProfile compact` con switch e Sensibilità
  segmentata; messaggi di preparazione e bypass invariati.

## 6. Nome predefinito del Microfono

- Nuovo campo `nome_microfono: Option<String>` in `Settings` con
  `#[serde(default)]` (facoltativo nei bindings), validato come i nomi dei
  Parlanti (non vuoto dopo trim, altrimenti `None`). Specchio in
  `DEFAULT_SETTINGS` e nello schema zod.
- Impostazioni → Generale: campo "Il tuo nome" con la spiegazione "Usato per il
  Microfono quando non riconosci più parlanti". Chiavi i18n in tutte e sei le
  lingue.
- Applicazione (ADR-0027): quando si crea un Tape con Ingressi separati e il
  Microfono non diarizzato (`!parlanti_microfono` per la Registrazione,
  `Settings::parlanti_trascrivi` per la ritrascrizione), scrivere
  `parlanti["microfono"] = nome`. Il punto unico va trovato dove nasce il
  `Document` (`Document::new` / composizione del Tape in `record` e in
  Trascrivi su un Tape), non nei chiamanti.
- Dal vivo: la `Conversation` della Registrazione parte con lo stesso nome
  (`withNome(conv, "microfono", null, nome)`), così etichette e Copia testo
  non cambiano allo Stop.
- Mai sul mix, mai su un Microfono diarizzato; i Tape esistenti non si toccano.

## Verifiche

- Test: `turnText` senza etichetta; composizione del Tape con e senza nome,
  con Microfono diarizzato e sul mix; validazione dell'impostazione; chiavi
  i18n (test esistente).
- Sei controlli verdi. Prova a mano: menu e barra in tema chiaro e scuro,
  tastiera sul controllo segmentato, Registrazione da Entrambi con il nome
  impostato, poi riapertura del Tape.

## Design brief (impeccable shape, 7 ottobre 2026)

Mondo esistente, nessun ridisegno: carta/inchiostro di `DESIGN.md`, modalità
Operate. Menu e barra si usano accanto a una videochiamata: si leggono in un
colpo d'occhio, si toccano poco.

- **Interruttore**: `shadcn add switch`. **Acceso = salvia** (`play`), come Segui
  l'audio, che migra allo stesso componente. Spento: traccia `rule-input`,
  pollice foglio con `shadow-sm`. Va emendata in `DESIGN.md` la One Accent Rule:
  la salvia vale anche per "interruttore acceso", mai per pulsanti, link o schede.
- **Sensibilità segmentata**: traccia `paper-shade` da 8 px di angolo, segmento
  scelto in foglio con filo e inchiostro, gli altri `ink-muted`; altezza 28 px
  nel menu e nella barra, 32 px in Impostazioni. **Etichette brevi** con chiavi
  nuove (`settings.protection.short.*`, sei lingue); il nome completo resta in
  `aria-label` e nella spiegazione.
- **Menu**: righe da 32 px; l'Ingresso è una riga di peso 500 con icona tenue e
  switch a destra; le opzioni sotto rientrano di 24 px allineate al testo
  dell'Ingresso, a 14 px. Filo da 1 px prima di Trascrivi dal vivo. Le opzioni
  di un Ingresso che si accende compaiono con una dissolvenza `motion-safe`.
- **Barra**: intestazione con pallino mattone pulsante, timer a 1.5rem tabulare,
  stato in `ink-muted`; Pausa (outline) e Stop (pieno) a destra. Strisce degli
  Ingressi separate da un filo; nome a sinistra (colonna fissa, larga quanto il
  nome più lungo tradotto), indicatore al centro, Guadagno a destra.
- **Indicatore**: 10 px, angoli pieni, traccia `paper-shade`; riempimento
  **salvia → senape → mattone** (senape da −15 dBFS, mattone da −3 dBFS) come
  zone fisse, non un gradiente che scorre. Nuovo token `level-warm` (senape
  del marchio, con gemello scuro) in `global.css`. Segno di picco: tratto da
  2 px in inchiostro che resta 1 s e scende; in Pausa tutto a zero e tenue.
  Animazione del riempimento a ogni tick (100 ms) con transizione breve;
  con `prefers-reduced-motion` salta senza transizione.
- **Riga sotto l'indicatore**: Pulisci il rumore (switch) e Sensibilità
  segmentata sulla stessa riga, allineate all'inizio dell'indicatore; i `?`
  restano come `FieldHelp`. Preparazione e bypass come testo sotto, invariati.
- **Fuori ambito**: Player, Impostazioni oltre a switch e segmentato, logica
  di salvataggio (`SettingsProvider`), Guadagno (resta `NativeSelect`).

### Revisione dopo l'anteprima (7 ottobre 2026)

Riferimento approvato: [mock.html](mock.html), barra "Compatta, due righe",
termine "Filtra rumore". Prevale sul brief sopra dove diverge.

- **Etichette**: "Filtra rumore" al posto di "Pulisci il rumore"
  (`settings.cleaning.enable`, sei lingue); "Sensibilità" al posto di
  "Sensibilità del parlato" nelle tre superfici, con il nome completo in
  `aria-label` e nella spiegazione.
- **Ingressi**: nel menu icona piena da 20 px e nome ("Microfono", "Audio di
  sistema"); con l'interruttore spento le opzioni sotto spariscono (non solo
  disabilitate) e ricompaiono con una breve dissolvenza `motion-safe`; l'ultimo
  Ingresso acceso ha l'interruttore disabilitato, tooltip "Serve almeno una
  fonte" (`withInput` già ignora lo spegnimento). Nella barra solo l'icona,
  con il nome in `title` e per i lettori di schermo. Icone piene disegnate
  (microfono a capsula piena, altoparlante pieno con i coni ritagliati; vedi i
  `symbol` `mic-fill` e `speaker-fill` del mock), in `foreground`: 20 px nel
  menu, 32 px nella barra.
- **Barra compatta su due righe**: padding 10×16 px; intestazione con pallino
  da 9 px, timer a 1.25rem, Pausa e Stop da 30 px. Per Ingresso: icona centrata
  verticalmente sulle due righe; riga 1 indicatore da 8 px e Guadagno (colonna
  a larghezza fissa, così gli indicatori sono allineati); riga 2 Filtra rumore
  (interruttore 28×16) e Sensibilità segmentata da 22 px, con i ⓘ. Niente scala
  in dB. A finestra minima la riga 2 va a capo.

## 7. Barra riducibile

- Pulsante icona da 30 px (lucide `ChevronsDownUp` / `ChevronsUpDown`) a
  sinistra di Pausa, `aria-expanded`, tooltip "Riduci la barra" / "Espandi la
  barra". Ridotta: una riga con pallino, timer, per Ingresso icona da 18 px e
  indicatore da 6 px (max ~220 px), poi Pausa e Stop con l'etichetta. Lo stato
  sparisce. Le righe degli Ingressi si chiudono con la transizione
  `grid-template-rows` 1fr → 0fr (≈260 ms, `motion-safe`).
- Ricordata in `localStorage` (`memotape.recordingBar`), letta con try/catch;
  non è un'impostazione.

## 8. Muto di un Ingresso

Glossario: **Muto** (`CONTEXT.md`). Requisiti in `PRODUCT.md`.

- **UI**: l'icona dell'Ingresso (barra intera e ridotta) è un pulsante con
  `aria-pressed`, nome "Muto del Microfono" / "Muto dell'Audio di sistema".
  In Muto: icona al 40%, segno di divieto `destructive` sovrapposto (entra con
  scala + dissolvenza), indicatore in grigio con tratteggio diagonale, nella
  barra intera "In muto" in `destructive` accanto al Guadagno. Vedi il mock.
- **Backend**: comando nuovo `set_muto(ingresso, muto)` sul `Recorder` (atomico
  per Ingresso, letto dal worker a ogni giro come i Guadagni); `record` parte
  sempre senza Muto. Nel `Mixer` il Muto è un fattore 0 sullo stesso percorso
  del Guadagno, con la rampa `GUADAGNO_RAMP_NS`: vale per mix, `tracks` e
  Trascrizione dal vivo; la linea del tempo non cambia.
- **Livello**: `take_peaks` deve dare il picco dopo il Guadagno ma prima del
  Muto, così l'indicatore grigio mostra il parlato reale.
- **Trascrizione dal vivo**: niente `ClosePhrase` esplicito. Il silenzio del
  Muto chiude la Frase con il VAD (hangover ≤ 700 ms); un `ClosePhrase` al clic
  arriverebbe prima dell'audio ancora nel mixer e taglierebbe la Frase in due
  (deciso in implementazione, 7 ottobre 2026).
- **Fuori ambito**: metadati dei tratti in Muto nel Tape, scorciatoie da
  tastiera, Muto prima dell'avvio (nel menu resta "Registra da").
- **Test**: Mixer con Muto a metà (silenzio dopo la rampa, durata invariata,
  picchi prima del Muto), Muto durante la Pausa, entrambi in Muto.

## 9. Dispositivo nel menu

- Estrarre `DeviceSelect` da `src/app/routes/settings.tsx` in
  `features/settings/device-select.tsx` e usarlo in Impostazioni e nel menu,
  come prima riga sotto l'Ingresso acceso (h-7, testo 12.5px, larghezza piena).
  Salva `microphone` / `outputDevice` come oggi, passando da `save(update)`.
- Variante `compact` per il menu: testo breve con la funzione pura
  `shortDeviceName` (toglie il prefisso generico `Xxx (…)` di Windows:
  "Microfono (Anker PowerConf C200)" → "Anker PowerConf C200"; lascia il nome
  com'è se non combacia; test accanto) e "Predefinito · <nome breve>" (chiave
  i18n nuova, sei lingue). Nome intero in `title` di ogni opzione e del select.
- `listMicrophones` / `listOutputDevices` a ogni apertura del popover
  (evento `toggle` del popover), errori nell'`onError` del menu.
- Dispositivo salvato ma assente: opzione "Non collegato" scelta, select con
  bordo e testo `destructive`. Registra non si blocca (errore attuale).
- Verificare a mano che aprire il `<select>` nativo dentro il popover
  `popover="auto"` non lo chiuda (light dismiss).
- Fuori ambito: cambio di dispositivo durante la Registrazione.

// Aggiornamento riproducibile delle sei lingue per la specifica approvata.
const translations = {
  it: ["Unisci", "Unisci al turno sopra", "Unisci al turno sotto", "Annulla", "Corretto a mano", "Questo Tape contiene correzioni manuali. Il testo corretto, le attribuzioni modificate a mano e i nomi personalizzati saranno conservati. La Diarizzazione aggiornerà le altre attribuzioni.", "Questo Tape contiene correzioni manuali. Una nuova Trascrizione sostituirà il testo, le attribuzioni e i nomi dei Parlanti, comprese le correzioni fatte a mano."],
  en: ["Merge", "Merge with the turn above", "Merge with the turn below", "Cancel", "Manually corrected", "This Tape contains manual corrections. Edited text, manually changed speaker assignments and custom names will be preserved. Diarization will update the other assignments.", "This Tape contains manual corrections. A new transcription will replace the text, speaker assignments and speaker names, including your manual corrections."],
  de: ["Zusammenführen", "Mit dem vorherigen Beitrag zusammenführen", "Mit dem nächsten Beitrag zusammenführen", "Abbrechen", "Manuell korrigiert", "Dieses Tape enthält manuelle Korrekturen. Bearbeiteter Text, manuell geänderte Sprecherzuordnungen und eigene Namen bleiben erhalten. Die Diarisierung aktualisiert die übrigen Zuordnungen.", "Dieses Tape enthält manuelle Korrekturen. Eine neue Transkription ersetzt den Text, die Sprecherzuordnungen und die Sprechernamen einschließlich der manuellen Korrekturen."],
  es: ["Unir", "Unir con la intervención anterior", "Unir con la intervención siguiente", "Cancelar", "Corregido a mano", "Este Tape contiene correcciones manuales. Se conservarán el texto corregido, las atribuciones modificadas a mano y los nombres personalizados. La diarización actualizará las demás atribuciones.", "Este Tape contiene correcciones manuales. Una nueva transcripción sustituirá el texto, las atribuciones y los nombres de los hablantes, incluidas las correcciones manuales."],
  fr: ["Fusionner", "Fusionner avec l’intervention précédente", "Fusionner avec l’intervention suivante", "Annuler", "Corrigé manuellement", "Ce Tape contient des corrections manuelles. Le texte corrigé, les attributions modifiées manuellement et les noms personnalisés seront conservés. La diarisation mettra à jour les autres attributions.", "Ce Tape contient des corrections manuelles. Une nouvelle transcription remplacera le texte, les attributions et les noms des locuteurs, y compris les corrections manuelles."],
  pl: ["Połącz", "Połącz z poprzednią wypowiedzią", "Połącz z następną wypowiedzią", "Anuluj", "Poprawiono ręcznie", "Ten Tape zawiera ręczne poprawki. Poprawiony tekst, ręcznie zmienione przypisania mówców i własne nazwy zostaną zachowane. Diaryzacja zaktualizuje pozostałe przypisania.", "Ten Tape zawiera ręczne poprawki. Nowa transkrypcja zastąpi tekst, przypisania mówców i ich nazwy, w tym ręczne poprawki."],
};
for (const [language, values] of Object.entries(translations)) {
  const path = `src/locales/${language}.json`;
  const locale = await Bun.file(path).json();
  const [merge, mergeAbove, mergeBelow, mergeCancel, manuallyCorrected, description, manualDescription] = values;
  Object.assign(locale.transcription, { merge, mergeAbove, mergeBelow, mergeCancel });
  locale.transcription.replace.manualDescription = manualDescription;
  locale.diarization.replace.description = description;
  locale.diarization.replace.keep = mergeCancel;
  locale.tape.manuallyCorrected = manuallyCorrected;
  await Bun.write(path, `${JSON.stringify(locale, null, 2)}\n`);
}

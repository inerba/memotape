/** Accoda una Frase al testo dell'area: una Frase per riga. */
export function appendPhrase(text: string, phrase: string) {
  return text ? `${text}\n${phrase}` : phrase;
}

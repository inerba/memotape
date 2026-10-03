import type { TranscriptPartial } from "@/bindings";

/** Accoda una Frase al testo dell'area: una Frase per riga. */
export function appendPhrase(text: string, phrase: string) {
  return text ? `${text}\n${phrase}` : phrase;
}

/** Il testo dell'area: le Frasi e, nella riga in corso, il Parziale se non è vuoto. */
export function withPartial(text: string, partial: TranscriptPartial | null) {
  return partial?.text ? appendPhrase(text, partial.text) : text;
}

/** Il Parziale dopo l'arrivo della Frase `phraseId`: se è suo, la Frase lo sostituisce. */
export function afterPhrase(
  partial: TranscriptPartial | null,
  phraseId: number
) {
  return partial?.phraseId === phraseId ? null : partial;
}

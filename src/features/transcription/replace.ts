import { isBino } from "@/features/source/file-name";

/** Cosa sostituisce il testo nell'area dopo la conferma. */
export type ReplaceAction = "transcribe" | "record" | "open";

/** Se Trascrivi chiede conferma: con testo nell'area e, su un Bino, sempre (cambia anche il Bino). */
export function transcribeNeedsConfirm(text: string, source: string | null) {
  return text.trim() !== "" || (source !== null && isBino(source));
}

/** La chiave del testo della conferma: aprire un Bino, ritrascriverlo o il resto. */
export function replaceDescription(
  action: ReplaceAction | undefined,
  source: string | null
) {
  if (action === "open") {
    return "transcription.replace.descriptionOpen";
  }
  return action === "transcribe" && source !== null && isBino(source)
    ? "transcription.replace.descriptionBino"
    : "transcription.replace.description";
}

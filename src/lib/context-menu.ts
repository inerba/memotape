/**
 * Se il clic destro su `target` lascia il menu della WebView: nei campi (Taglia, Copia, Incolla) e
 * con del testo selezionato (Copia), anche se il clic cade fuori dalla selezione. Altrove il menu
 * da browser (Indietro, Aggiorna, Stampa) non compare; i nostri menu chiamano già `preventDefault`.
 */
export function lasciaMenuNativo(
  target: EventTarget | null,
  selezione: string
): boolean {
  if (selezione !== "") {
    return true;
  }
  const el = target as HTMLElement | null;
  return Boolean(el?.closest?.("input, textarea") || el?.isContentEditable);
}

/** Nella build di release toglie il menu della WebView dove `lasciaMenuNativo` non lo vuole. */
export function bloccaMenuNativo() {
  if (!import.meta.env.PROD) {
    return;
  }
  document.addEventListener("contextmenu", (e) => {
    if (!lasciaMenuNativo(e.target, window.getSelection()?.toString() ?? "")) {
      e.preventDefault();
    }
  });
}

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

/** I tasti di ricarica e cronologia della WebView: F5, Ctrl+R (anche con Maiusc), Alt+← e Alt+→, i tasti media. */
export function tastoDelBrowser(
  e: Pick<KeyboardEvent, "altKey" | "ctrlKey" | "key">
): boolean {
  if (
    ["F5", "BrowserBack", "BrowserForward", "BrowserRefresh"].includes(e.key)
  ) {
    return true;
  }
  if (e.ctrlKey && !e.altKey) {
    return e.key.toLowerCase() === "r";
  }
  return (
    e.altKey && !e.ctrlKey && (e.key === "ArrowLeft" || e.key === "ArrowRight")
  );
}

/**
 * Anche in sviluppo, ferma i tasti di `tastoDelBrowser` e i tasti laterali del mouse (Indietro,
 * Avanti): ricaricare o tornare indietro perderebbe lo stato dell'app. Ctrl+F e F12 restano.
 */
export function bloccaTastiDelBrowser() {
  document.addEventListener("keydown", (e) => {
    if (tastoDelBrowser(e)) {
      e.preventDefault();
    }
  });
  document.addEventListener("mouseup", (e) => {
    if (e.button === 3 || e.button === 4) {
      e.preventDefault();
    }
  });
}

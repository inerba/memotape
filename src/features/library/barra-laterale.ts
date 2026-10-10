import { useCallback, useState } from "react";

/** Dove si ricorda, su questo PC, se la barra laterale è chiusa. */
const CHIAVE = "memotape.barraLaterale";

/**
 * La barra laterale è aperta, se non è stata chiusa. `memoria` restituisce `localStorage`: in una
 * finestra privata o con i dati bloccati anche il solo accesso fallisce, e allora parte aperta.
 */
export function barraAperta(memoria: () => Pick<Storage, "getItem">): boolean {
  try {
    return memoria().getItem(CHIAVE) !== "chiusa";
  } catch {
    return true;
  }
}

/** Le icone della striscia, la barra laterale chiusa, dall'alto in basso. */
export const ICONE_DELLA_STRISCIA = [
  "home",
  "nuovaRegistrazione",
  "importa",
  "cerca",
  "recenti",
  "libreria",
  "impostazioni",
] as const;

export type IconaDellaStriscia = (typeof ICONE_DELLA_STRISCIA)[number];

/**
 * Cerca e Recenti servono la barra aperta: la riaprono e portano il focus nella ricerca o nei
 * Recenti. Le altre agiscono subito e la barra resta chiusa (`null`).
 */
export function riapertura(
  icona: IconaDellaStriscia
): "ricerca" | "recenti" | null {
  if (icona === "cerca") {
    return "ricerca";
  }
  return icona === "recenti" ? "recenti" : null;
}

/** Ricorda la barra laterale `aperta` o chiusa; senza memoria la scelta vale fino alla chiusura. */
export function ricordaBarra(
  memoria: () => Pick<Storage, "setItem">,
  aperta: boolean
): void {
  try {
    memoria().setItem(CHIAVE, aperta ? "aperta" : "chiusa");
  } catch {
    // Senza memoria del browser la scelta vale finché la finestra resta aperta.
  }
}

const memoriaDelBrowser = () => localStorage;

/** La barra laterale aperta o chiusa, ricordata su questo PC, e il comando che la alterna. */
export function useBarraLaterale(): { alterna: () => void; aperta: boolean } {
  const [aperta, setAperta] = useState(() => barraAperta(memoriaDelBrowser));
  const alterna = useCallback(
    () =>
      setAperta((corrente) => {
        ricordaBarra(memoriaDelBrowser, !corrente);
        return !corrente;
      }),
    []
  );
  return { alterna, aperta };
}

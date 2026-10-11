import type { TFunction } from "i18next";

/**
 * Le scorciatoie da tastiera della finestra principale: azione, tasti, gruppo del pannello e
 * quando valgono. Pura: il listener globale (`ScorciatoieProvider`) le applica agli eventi.
 */

export type Azione =
  | "nuovaRegistrazione"
  | "importa"
  | "cerca"
  | "barraLaterale"
  | "impostazioni"
  | "riproduci"
  | "indietro"
  | "avanti"
  | "piuLento"
  | "piuVeloce"
  | "tornaAlPunto"
  | "copiaTesto"
  | "pausaRegistrazione"
  | "pannello"
  | "apriTape"
  | "rinominaTape"
  | "cestinaTape";

export type Gruppo =
  | "generale"
  | "ascolto"
  | "testo"
  | "registrazione"
  | "libreria";

/**
 * Dove sta il focus: in un campo di `testo` (si scrive), su un `pulsante` o un link (Spazio lo
 * attiva), su un `cursore` (le frecce lo spostano), o altrove.
 */
export type Campo = "testo" | "pulsante" | "cursore" | null;

export interface Stato {
  /** Un'Attività in corso: niente Nuova registrazione né Importa un file. */
  attivita: boolean;
  /** Impostazioni o un dialog coprono la finestra: vale solo il pannello. */
  coperta: boolean;
  /** Una Registrazione in corso: il player è disabilitato, la Pausa sì. */
  registrazione: boolean;
}

/** Le proprietà di un `KeyboardEvent` che servono. */
export interface Tasto {
  altKey: boolean;
  ctrlKey: boolean;
  key: string;
  metaKey: boolean;
  repeat: boolean;
  shiftKey: boolean;
}

export interface Scorciatoia {
  azione: Azione;
  gruppo: Gruppo;
  /** Agisce sul Tape con il focus nella tabella della Libreria, non sulla finestra. */
  nellaTabella?: true;
  quando?: (stato: Stato) => boolean;
  /** La riga del pannello (chiave `shortcuts.lines.*`): indietro e avanti ne condividono una. */
  riga: string;
  /** Tenendo premuto il tasto si ripete. */
  ripeti?: true;
  /**
   * I tasti in forma canonica: modificatori `Ctrl` e `Shift`, poi il tasto (`Space`, `ArrowLeft`
   * o il carattere). Il pannello e i tooltip li traducono.
   */
  tasti: readonly string[];
  /**
   * I campi che con il focus tengono il tasto per sé: il testo per chi scrive, il pulsante per
   * Spazio, il cursore per le frecce.
   */
  trattenutaDa?: readonly NonNullable<Campo>[];
}

const NEL_TESTO = ["testo"] as const;
const LIBERA = (s: Stato) => !s.attivita;
const ASCOLTO = (s: Stato) => !s.registrazione;

export const SCORCIATOIE: readonly Scorciatoia[] = [
  {
    azione: "nuovaRegistrazione",
    gruppo: "generale",
    quando: LIBERA,
    riga: "nuovaRegistrazione",
    tasti: ["Ctrl", "N"],
  },
  {
    azione: "importa",
    gruppo: "generale",
    quando: LIBERA,
    riga: "importa",
    tasti: ["Ctrl", "O"],
  },
  { azione: "cerca", gruppo: "generale", riga: "cerca", tasti: ["Ctrl", "K"] },
  {
    azione: "barraLaterale",
    gruppo: "generale",
    riga: "barraLaterale",
    tasti: ["Ctrl", "B"],
  },
  {
    azione: "impostazioni",
    gruppo: "generale",
    riga: "impostazioni",
    tasti: ["Ctrl", ","],
  },
  {
    azione: "riproduci",
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "riproduci",
    tasti: ["Space"],
    trattenutaDa: ["testo", "pulsante"],
  },
  {
    azione: "indietro",
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "indietroAvanti",
    ripeti: true,
    tasti: ["ArrowLeft"],
    trattenutaDa: ["testo", "cursore"],
  },
  {
    azione: "avanti",
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "indietroAvanti",
    ripeti: true,
    tasti: ["ArrowRight"],
    trattenutaDa: ["testo", "cursore"],
  },
  {
    azione: "piuLento",
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "velocita",
    tasti: ["["],
    trattenutaDa: NEL_TESTO,
  },
  {
    azione: "piuVeloce",
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "velocita",
    tasti: ["]"],
    trattenutaDa: NEL_TESTO,
  },
  {
    azione: "tornaAlPunto",
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "tornaAlPunto",
    tasti: ["Ctrl", "J"],
  },
  {
    azione: "copiaTesto",
    gruppo: "testo",
    riga: "copiaTesto",
    tasti: ["Ctrl", "Shift", "C"],
  },
  {
    azione: "pausaRegistrazione",
    gruppo: "registrazione",
    quando: (s) => s.registrazione,
    riga: "pausaRegistrazione",
    tasti: ["Ctrl", "P"],
  },
  {
    azione: "pannello",
    gruppo: "generale",
    riga: "pannello",
    tasti: ["Ctrl", "/"],
  },
  {
    azione: "apriTape",
    gruppo: "libreria",
    nellaTabella: true,
    riga: "apriTape",
    tasti: ["Enter"],
  },
  {
    azione: "rinominaTape",
    gruppo: "libreria",
    nellaTabella: true,
    riga: "rinominaTape",
    tasti: ["F2"],
  },
  {
    azione: "cestinaTape",
    gruppo: "libreria",
    nellaTabella: true,
    riga: "cestinaTape",
    tasti: ["Delete"],
  },
];

const KEYS: Record<string, string> = { Space: " " };
const LETTERA = /^[a-z]$/i;

function corrisponde(s: Scorciatoia, t: Tasto): boolean {
  const key = s.tasti.find((k) => k !== "Ctrl" && k !== "Shift") ?? "";
  if (t.metaKey || t.key.toLowerCase() !== (KEYS[key] ?? key).toLowerCase()) {
    return false;
  }
  if (s.tasti.includes("Ctrl")) {
    // Ctrl+Alt è AltGr: scrive un altro carattere.
    if (!t.ctrlKey || t.altKey) {
      return false;
    }
  } else if (t.ctrlKey !== t.altKey) {
    // Senza Ctrl il tasto vale da solo o con AltGr (`[` e `]` sulla tastiera italiana).
    return false;
  }
  // Maiuscole solo per le lettere: `/` su molte tastiere chiede già Maiusc.
  return !LETTERA.test(key) || t.shiftKey === s.tasti.includes("Shift");
}

/** Il campo con il focus tiene il tasto per sé. */
function trattenuto(s: Scorciatoia, campo: Campo): boolean {
  return campo !== null && (s.trattenutaDa?.includes(campo) ?? false);
}

/**
 * La scorciatoia di `tasto` con il focus su `campo`, o `null`. `esegui`: l'azione vale in `stato`;
 * `riservata`: il tasto non deve arrivare a WebView2 nemmeno quando non si esegue.
 */
export function scorciatoia(
  tasto: Tasto,
  campo: Campo,
  stato: Stato
): { azione: Azione; esegui: boolean; riservata: boolean } | null {
  const s = SCORCIATOIE.find(
    (candidata) => !candidata.nellaTabella && corrisponde(candidata, tasto)
  );
  if (!s || trattenuto(s, campo)) {
    return null;
  }
  const esegui =
    (s.azione === "pannello" || !stato.coperta) &&
    (s.ripeti === true || !tasto.repeat) &&
    (s.quando?.(stato) ?? true);
  // Con Ctrl il tasto non arriva mai a WebView2, che ne fa acceleratori del browser (Ctrl+P la
  // stampa, Ctrl+Maiusc+C l'ispettore): il preventDefault della pagina li ferma.
  return { azione: s.azione, esegui, riservata: s.tasti.includes("Ctrl") };
}

/**
 * L'azione di `tasto` sul Tape con il focus nella tabella della Libreria (o nei Recenti), o `null`.
 * Le gestisce l'elemento con il focus, non il listener della finestra.
 */
export function tastoDellaTabella(tasto: Tasto): Azione | null {
  return (
    SCORCIATOIE.find((s) => s.nellaTabella && corrisponde(s, tasto))?.azione ??
    null
  );
}

/** I tasti di `azione`, in forma canonica. */
export function tastiDi(azione: Azione): readonly string[] {
  return SCORCIATOIE.find((s) => s.azione === azione)?.tasti ?? [];
}

const FRECCE: Record<string, string> = { ArrowLeft: "←", ArrowRight: "→" };

/** I tasti che hanno un nome nella lingua dell'interfaccia. */
const TRADOTTI = new Set(["Ctrl", "Shift", "Space", "Enter", "Delete"]);

/** Il nome di un tasto in forma canonica nella lingua dell'interfaccia: «Maiusc», «←», «F2». */
export function nomeTasto(tasto: string, t: TFunction): string {
  if (TRADOTTI.has(tasto)) {
    return t(`shortcuts.keys.${tasto}`);
  }
  return FRECCE[tasto] ?? tasto;
}

/** I tasti di `azione` nella lingua dell'interfaccia: «Ctrl+Maiusc+C», «Canc». */
export function nomeTasti(t: TFunction, azione: Azione): string {
  return tastiDi(azione)
    .map((tasto) => nomeTasto(tasto, t))
    .join("+");
}

/**
 * Il tooltip di un pulsante con le sue scorciatoie: «Copia testo (Ctrl+Maiusc+C)»; più azioni si
 * separano con uno spazio («Velocità ([ ])»).
 */
export function conTasti(t: TFunction, testo: string, ...azioni: Azione[]) {
  const tasti = azioni.map((azione) => nomeTasti(t, azione)).join(" ");
  return `${testo} (${tasti})`;
}

/** Il valore di `aria-keyshortcuts` di `azione`. */
export function ariaTasti(azione: Azione): string {
  return tastiDi(azione)
    .map((tasto) => (tasto === "Ctrl" ? "Control" : tasto))
    .join("+");
}

/** Le righe del pannello di `gruppo`: indietro e avanti, e le due velocità, in una sola. */
export function righe(gruppo: Gruppo): { riga: string; tasti: string[] }[] {
  const lista = new Map<string, string[]>();
  for (const s of SCORCIATOIE) {
    if (s.gruppo !== gruppo) {
      continue;
    }
    lista.set(s.riga, [...(lista.get(s.riga) ?? []), ...s.tasti]);
  }
  return [...lista].map(([riga, tasti]) => ({ riga, tasti }));
}

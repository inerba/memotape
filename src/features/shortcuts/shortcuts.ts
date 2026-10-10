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
  | "pannello";

export type Gruppo = "generale" | "ascolto" | "testo" | "registrazione";

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
  /** Solo fuori dai campi, come Spazio. */
  fuoriDaiCampi?: true;
  gruppo: Gruppo;
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
}

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
    fuoriDaiCampi: true,
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "riproduci",
    tasti: ["Space"],
  },
  {
    azione: "indietro",
    fuoriDaiCampi: true,
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "indietroAvanti",
    ripeti: true,
    tasti: ["ArrowLeft"],
  },
  {
    azione: "avanti",
    fuoriDaiCampi: true,
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "indietroAvanti",
    ripeti: true,
    tasti: ["ArrowRight"],
  },
  {
    azione: "piuLento",
    fuoriDaiCampi: true,
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "velocita",
    tasti: ["["],
  },
  {
    azione: "piuVeloce",
    fuoriDaiCampi: true,
    gruppo: "ascolto",
    quando: ASCOLTO,
    riga: "velocita",
    tasti: ["]"],
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

/** Il campo che tiene il tasto per sé, se c'è. */
function trattenuto(s: Scorciatoia, campo: Campo): boolean {
  if (!s.fuoriDaiCampi || campo === null) {
    return false;
  }
  if (campo === "testo") {
    return true;
  }
  if (s.azione === "riproduci") {
    return campo === "pulsante";
  }
  return campo === "cursore" && s.riga === "indietroAvanti";
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
  const s = SCORCIATOIE.find((candidata) => corrisponde(candidata, tasto));
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

/** I tasti di `azione`, in forma canonica. */
export function tastiDi(azione: Azione): readonly string[] {
  return SCORCIATOIE.find((s) => s.azione === azione)?.tasti ?? [];
}

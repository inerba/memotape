import { expect, test } from "bun:test";
import {
  type Azione,
  type Campo,
  type Stato,
  scorciatoia,
  tastoDellaTabella,
} from "@/features/shortcuts/shortcuts";

const LIBERO: Stato = { attivita: false, coperta: false, registrazione: false };

const tasto = (key: string, mod: Partial<Record<string, boolean>> = {}) => ({
  altKey: false,
  ctrlKey: false,
  metaKey: false,
  repeat: false,
  shiftKey: false,
  ...mod,
  key,
});

const ctrl = (key: string, mod: Partial<Record<string, boolean>> = {}) =>
  tasto(key, { ctrlKey: true, ...mod });

test("Ctrl+N avvia una Nuova registrazione fuori da un'Attività", () => {
  expect(scorciatoia(ctrl("n"), null, LIBERO)).toEqual({
    azione: "nuovaRegistrazione",
    esegui: true,
    riservata: true,
  });
});

test("durante un'Attività Ctrl+N e Ctrl+O non agiscono, ma non arrivano a WebView2", () => {
  const busy = { ...LIBERO, attivita: true };
  expect(scorciatoia(ctrl("n"), null, busy)).toEqual({
    azione: "nuovaRegistrazione",
    esegui: false,
    riservata: true,
  });
  expect(scorciatoia(ctrl("o"), "testo", busy)?.esegui).toBe(false);
  expect(scorciatoia(ctrl("o"), null, LIBERO)?.azione).toBe("importa");
});

test("Ctrl+P mette in pausa solo durante una Registrazione, e non stampa mai", () => {
  expect(scorciatoia(ctrl("p"), null, LIBERO)).toEqual({
    azione: "pausaRegistrazione",
    esegui: false,
    riservata: true,
  });
  const registrando = { ...LIBERO, attivita: true, registrazione: true };
  expect(scorciatoia(ctrl("p"), null, registrando)?.esegui).toBe(true);
  // Anche correggendo un Turno il tasto non apre la stampa.
  expect(scorciatoia(ctrl("p"), "testo", LIBERO)?.riservata).toBe(true);
});

test("Ctrl+Maiusc+C copia tutto il testo; Ctrl+C resta la copia della selezione", () => {
  expect(scorciatoia(ctrl("C", { shiftKey: true }), "testo", LIBERO)).toEqual({
    azione: "copiaTesto",
    esegui: true,
    riservata: true,
  });
  expect(scorciatoia(ctrl("c"), null, LIBERO)).toBeNull();
});

test("le lettere senza Ctrl sono testo, non scorciatoie", () => {
  expect(scorciatoia(tasto("n"), null, LIBERO)).toBeNull();
  expect(scorciatoia(ctrl("n", { altKey: true }), null, LIBERO)).toBeNull();
  expect(scorciatoia(ctrl("n", { shiftKey: true }), null, LIBERO)).toBeNull();
});

test("Spazio, frecce e parentesi valgono solo fuori dai campi di testo", () => {
  const cases: [string, Azione][] = [
    [" ", "riproduci"],
    ["ArrowLeft", "indietro"],
    ["ArrowRight", "avanti"],
    ["[", "piuLento"],
    ["]", "piuVeloce"],
  ];
  for (const [key, azione] of cases) {
    expect(scorciatoia(tasto(key), null, LIBERO)?.azione).toBe(azione);
    expect(scorciatoia(tasto(key), "testo", LIBERO)).toBeNull();
  }
});

test("Spazio su un pulsante lo attiva; le frecce su un cursore lo spostano", () => {
  const su = (key: string, campo: Campo) =>
    scorciatoia(tasto(key), campo, LIBERO)?.azione ?? null;
  expect(su(" ", "pulsante")).toBeNull();
  expect(su(" ", "cursore")).toBe("riproduci");
  expect(su("ArrowLeft", "cursore")).toBeNull();
  expect(su("ArrowLeft", "pulsante")).toBe("indietro");
  expect(su("]", "cursore")).toBe("piuVeloce");
});

test("[ e ] valgono anche con AltGr, come sulla tastiera italiana", () => {
  const altGr = { altKey: true, ctrlKey: true };
  expect(scorciatoia(tasto("[", altGr), null, LIBERO)?.azione).toBe("piuLento");
  expect(scorciatoia(tasto("]", altGr), null, LIBERO)?.azione).toBe(
    "piuVeloce"
  );
  // Ctrl senza Alt non è AltGr.
  expect(scorciatoia(tasto("[", { ctrlKey: true }), null, LIBERO)).toBeNull();
});

test("Ctrl+/ apre il pannello anche quando / chiede Maiusc", () => {
  expect(scorciatoia(ctrl("/"), null, LIBERO)?.azione).toBe("pannello");
  expect(scorciatoia(ctrl("/", { shiftKey: true }), null, LIBERO)?.azione).toBe(
    "pannello"
  );
});

test("durante una Registrazione il player non risponde ai tasti", () => {
  const registrando = { ...LIBERO, attivita: true, registrazione: true };
  for (const key of [" ", "ArrowLeft", "["]) {
    expect(scorciatoia(tasto(key), null, registrando)?.esegui).toBe(false);
  }
  expect(scorciatoia(ctrl("j"), null, registrando)?.esegui).toBe(false);
  expect(scorciatoia(ctrl("j"), null, LIBERO)?.azione).toBe("tornaAlPunto");
});

test("con Impostazioni o un dialog sopra la finestra vale solo il pannello", () => {
  const coperta = { ...LIBERO, coperta: true };
  for (const key of ["n", "o", "k", ",", "j"]) {
    expect(scorciatoia(ctrl(key), null, coperta)?.esegui).toBe(false);
  }
  expect(scorciatoia(tasto(" "), null, coperta)?.esegui).toBe(false);
  expect(scorciatoia(ctrl("/"), null, coperta)?.esegui).toBe(true);
});

test("tenendo premuto si ripetono solo le frecce", () => {
  const ripetuto = { repeat: true };
  expect(scorciatoia(tasto(" ", ripetuto), null, LIBERO)?.esegui).toBe(false);
  expect(scorciatoia(ctrl("n", ripetuto), null, LIBERO)?.esegui).toBe(false);
  expect(scorciatoia(tasto("ArrowRight", ripetuto), null, LIBERO)?.esegui).toBe(
    true
  );
});

test("Invio, F2 e Canc agiscono sul Tape della tabella, non sulla finestra", () => {
  expect(tastoDellaTabella(tasto("Enter"))).toBe("apriTape");
  expect(tastoDellaTabella(tasto("F2"))).toBe("rinominaTape");
  expect(tastoDellaTabella(tasto("Delete"))).toBe("cestinaTape");
  expect(tastoDellaTabella(ctrl("Delete"))).toBeNull();
  expect(tastoDellaTabella(tasto(" "))).toBeNull();
  for (const key of ["Enter", "F2", "Delete"]) {
    expect(scorciatoia(tasto(key), null, LIBERO)).toBeNull();
  }
});

test("Ctrl+B, Ctrl+K e Ctrl+, sono nella mappa", () => {
  expect(scorciatoia(ctrl("b"), null, LIBERO)?.azione).toBe("barraLaterale");
  expect(scorciatoia(ctrl("K"), null, LIBERO)?.azione).toBe("cerca");
  expect(scorciatoia(ctrl(","), null, LIBERO)?.azione).toBe("impostazioni");
});

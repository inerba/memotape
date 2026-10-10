import { describe, expect, test } from "bun:test";
import "@/lib/i18n";
import i18n from "i18next";
import type { TapeInfo } from "@/bindings";
import { tapeDetails } from "./tape-details";

describe("Dettagli del Tape", () => {
  const t = i18n.t.bind(i18n);
  const info: TapeInfo = {
    completa: true,
    correttoAMano: false,
    creato: "2026-07-08T15:01:12+02:00",
    diarizzazione: null,
    durataMs: 1_540_000,
    ingressiSeparati: false,
    linguaParlato: "it",
    modello: "Whisper Large v3 Turbo",
    origine: "2026-07-08 15-01-12.mp4",
  };

  test("un file trascritto: il nome del file, modello, lingua e il solo mix", () => {
    expect(tapeDetails(info, t, "it")).toEqual([
      { label: "Origine", value: "2026-07-08 15-01-12.mp4" },
      { label: "Modello", value: "Whisper Large v3 Turbo" },
      { label: "Lingua del parlato", value: "Italiano" },
      { label: "Ingressi", value: "Mix" },
    ]);
  });

  test("una Registrazione con gli Ingressi separati e la lingua automatica", () => {
    const registrazione: TapeInfo = {
      ...info,
      ingressiSeparati: true,
      linguaParlato: "auto",
      origine: null,
    };
    expect(tapeDetails(registrazione, t, "it")).toEqual([
      { label: "Origine", value: "Registrazione" },
      { label: "Modello", value: "Whisper Large v3 Turbo" },
      { label: "Lingua del parlato", value: "Automatica" },
      { label: "Ingressi", value: "Microfono + audio di sistema" },
    ]);
  });

  test("senza testo niente modello né lingua", () => {
    const vuoto: TapeInfo = { ...info, modello: null, origine: null };
    expect(tapeDetails(vuoto, t, "it")).toEqual([
      { label: "Origine", value: "Registrazione" },
      { label: "Ingressi", value: "Mix" },
    ]);
  });
});

import { expect, test } from "bun:test";
import type { Settings } from "@/bindings";
import {
  settingsSchema,
  speechLanguageChoice,
} from "@/features/settings/settings";

const defaults: Settings = {
  bitrateKbps: 32,
  channels: "mono",
  interfaceLanguage: null,
  microphone: null,
  model: "nemotron-3.5-streaming-0.6b-q5km",
  outputDevice: null,
  recordingSource: "mic",
  recordingsFolder: null,
  sampleRate: 48_000,
  speechLanguage: "auto",
};

test("lo schema accetta le impostazioni predefinite e quelle complete", () => {
  expect(settingsSchema.parse(defaults)).toEqual(defaults);
  const full: Settings = {
    ...defaults,
    bitrateKbps: 320,
    channels: "stereo",
    interfaceLanguage: "pl",
    microphone: "Microfono USB",
    outputDevice: "Cuffie",
    recordingSource: "both",
    recordingsFolder: "D:\\Registrazioni",
    sampleRate: 8000,
    speechLanguage: "it",
  };
  expect(settingsSchema.parse(full)).toEqual(full);
});

test("lo schema rifiuta valori fuori dagli elenchi e campi mancanti", () => {
  const invalid = [
    { ...defaults, bitrateKbps: 33 },
    { ...defaults, sampleRate: 44_100 },
    { ...defaults, speechLanguage: "ja" },
    { ...defaults, speechLanguage: null },
    { ...defaults, recordingSource: "line-in" },
    { ...defaults, model: "" },
    { model: defaults.model },
  ];
  for (const value of invalid) {
    expect(settingsSchema.safeParse(value).success).toBe(false);
  }
});

test("il selettore offre le lingue dell'app che il modello accetta, anche come locale", () => {
  // Nemotron: locale.
  expect(
    speechLanguageChoice(["en-US", "it-IT", "de-DE", "ja-JP"], "it")
  ).toEqual({ options: ["it", "en", "de"], value: "it" });
  // Whisper e Parakeet: codici.
  expect(
    speechLanguageChoice(["pl", "es", "fr", "de", "en", "it", "ja"], "auto")
      .options
  ).toEqual(["it", "en", "fr", "es", "de", "pl"]);
  // Un prefisso che non è il codice non vale; un modello senza lingue offre solo Automatica.
  expect(speechLanguageChoice(["ita", "eng"], "auto").options).toEqual([]);
  expect(speechLanguageChoice([], "auto").options).toEqual([]);
});

test("una lingua salvata che il modello non accetta vale Automatica", () => {
  expect(speechLanguageChoice(["en-US", "it-IT"], "pl")).toEqual({
    options: ["it", "en"],
    value: "auto",
  });
});

test("finché le lingue del modello non sono note resta offerta solo la scelta salvata", () => {
  expect(speechLanguageChoice(null, "auto")).toEqual({
    options: [],
    value: "auto",
  });
  expect(speechLanguageChoice(null, "fr")).toEqual({
    options: ["fr"],
    value: "fr",
  });
});

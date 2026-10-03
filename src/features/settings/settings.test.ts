import { expect, test } from "bun:test";
import type { Settings } from "@/bindings";
import { languageOptions, settingsSchema } from "@/features/settings/settings";

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
  speechLanguage: null,
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
    { ...defaults, speechLanguage: "auto" },
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
  expect(languageOptions(["en-US", "it-IT", "de-DE", "ja-JP"], null)).toEqual([
    "it",
    "en",
    "de",
  ]);
  // Whisper e Parakeet: codici.
  expect(
    languageOptions(["pl", "es", "fr", "de", "en", "it", "ja"], null)
  ).toEqual(["it", "en", "fr", "es", "de", "pl"]);
  // Un prefisso che non è il codice non vale; un modello senza lingue offre solo Automatica.
  expect(languageOptions(["ita", "eng"], null)).toEqual([]);
  expect(languageOptions([], "it")).toEqual([]);
});

test("finché le lingue del modello non sono note resta offerta solo la scelta salvata", () => {
  expect(languageOptions(null, null)).toEqual([]);
  expect(languageOptions(null, "fr")).toEqual(["fr"]);
});

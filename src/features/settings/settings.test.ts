import { expect, test } from "bun:test";
import type { Settings } from "@/bindings";
import {
  DEFAULT_SETTINGS as defaults,
  languageOf,
  parlantiRegistrazione,
  settingsSchema,
  speechLanguageChoice,
} from "@/features/settings/settings";

test("i predefiniti sono quelli di Rust", () => {
  expect(defaults.model).toBe("nemotron-3.5-streaming-0.6b-q5km");
  expect([
    defaults.bitrateKbps,
    defaults.channels,
    defaults.sampleRate,
  ]).toEqual([32, "mono", 48_000]);
  expect(defaults.interfaceLanguage).toBeNull();
  // Trascrivi dal vivo è spenta finché l'utente non la attiva.
  expect(defaults.trascrizioneDalVivo).toBe(false);
  // Dal vivo si trascrive il mix finché l'utente non sceglie gli Ingressi separati.
  expect(defaults.modalitaDalVivo).toBe("mix");
  // Copia testo copia testo semplice finché l'utente non sceglie Markdown.
  expect(defaults.copiaCome).toBe("testo");
  // Riconosci i parlanti è spenta finché l'utente non la attiva.
  expect(defaults.parlantiFile).toBe(false);
  expect([
    defaults.parlantiMix,
    defaults.parlantiMicrofono,
    defaults.parlantiSistema,
  ]).toEqual([false, false, false]);
  // Il tema segue Windows finché l'utente non ne sceglie uno.
  expect(defaults.tema).toBe("sistema");
});

test("Riconosci i parlanti delle Registrazioni ha una casella per Ingresso solo con gli Ingressi separati", () => {
  const both = { ...defaults, recordingSource: "both" as const };
  expect(parlantiRegistrazione(both)).toEqual(["parlantiMix"]);
  const separati = { ...both, modalitaDalVivo: "ingressiSeparati" as const };
  expect(parlantiRegistrazione(separati)).toEqual([
    "parlantiMicrofono",
    "parlantiSistema",
  ]);
  // Ingressi separati vale solo registrando da Entrambi.
  expect(
    parlantiRegistrazione({ ...separati, recordingSource: "system" })
  ).toEqual(["parlantiMix"]);
});

test("senza backend la Lingua dell'interfaccia è quella di sistema se supportata, altrimenti l'inglese", () => {
  expect(languageOf("it-IT")).toBe("it");
  expect(languageOf("PL")).toBe("pl");
  expect(languageOf("es-419")).toBe("es");
  expect(languageOf("pt-BR")).toBe("en");
  expect(languageOf("ita")).toBe("en");
  expect(languageOf("")).toBe("en");
});

test("lo schema accetta le impostazioni predefinite e quelle complete", () => {
  expect(settingsSchema.parse(defaults)).toEqual(defaults);
  const full: Settings = {
    ...defaults,
    bitrateKbps: 320,
    channels: "stereo",
    copiaCome: "markdown",
    interfaceLanguage: "pl",
    microphone: "Microfono USB",
    modalitaDalVivo: "ingressiSeparati",
    outputDevice: "Cuffie",
    parlantiFile: true,
    parlantiMicrofono: false,
    parlantiMix: true,
    parlantiSistema: true,
    recordingSource: "both",
    recordingsFolder: "D:\\Registrazioni",
    sampleRate: 8000,
    speechLanguage: "it",
    trascrizioneDalVivo: true,
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
    { ...defaults, trascrizioneDalVivo: "sì" },
    { ...defaults, copiaCome: "html" },
    { ...defaults, modalitaDalVivo: "canali" },
    { ...defaults, parlantiFile: "sì" },
    { ...defaults, parlantiSistema: 1 },
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

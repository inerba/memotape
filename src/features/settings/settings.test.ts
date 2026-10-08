import { expect, test } from "bun:test";
import type { Settings } from "@/bindings";
import {
  DEFAULT_SETTINGS as defaults,
  GUADAGNI,
  guadagnoText,
  languageOf,
  nomeMicrofonoRegistrazione,
  parlantiRegistrazione,
  settingsSchema,
  speechLanguageChoice,
  speechLanguageName,
  withInput,
} from "@/features/settings/settings";

test("la pulizia è spenta nei tre profili e il profilo File si salva indipendentemente", () => {
  expect(defaults.audioMicrofono).toEqual({
    pulizia: false,
    sensibilita: "bilanciato",
  });
  expect(defaults.audioSistema).toEqual({
    pulizia: false,
    sensibilita: "bilanciato",
  });
  expect(defaults.audioFileMisto).toEqual({
    pulizia: false,
    sensibilita: "bilanciato",
  });
  const saved = settingsSchema.parse({
    ...defaults,
    audioFileMisto: { pulizia: true },
  });
  expect(saved.audioFileMisto).toEqual({ pulizia: true });
  expect(saved.audioMicrofono).toEqual({
    pulizia: false,
    sensibilita: "bilanciato",
  });
  expect(saved.audioSistema).toEqual({
    pulizia: false,
    sensibilita: "bilanciato",
  });
  expect(
    settingsSchema.safeParse({ ...defaults, audioFileMisto: { pulizia: "sì" } })
      .success
  ).toBe(false);
});

test("il diarizer si sceglie separatamente e il percorso locale non va perso", () => {
  const parsed = settingsSchema.parse({
    ...defaults,
    diarizer: "nemotron3",
    nemotron3Path: "D:\\modelli\\Nemotron-3-Diarization-BF16.gguf",
  });
  expect(parsed.diarizer).toBe("nemotron3");
  expect(parsed.nemotron3Path).toBe(
    "D:\\modelli\\Nemotron-3-Diarization-BF16.gguf"
  );
  expect(parsed.model).toBe(defaults.model);
  expect(defaults.diarizer).toBe("sortformer");
});

test("i predefiniti sono quelli di Rust", () => {
  expect(defaults.model).toBe("nemotron-3.5-streaming-0.6b-q5km");
  expect([
    defaults.bitrateKbps,
    defaults.channels,
    defaults.sampleRate,
  ]).toEqual([16, "mono", 16_000]);
  expect(defaults.interfaceLanguage).toBeNull();
  // Trascrivi dal vivo è spenta finché l'utente non la attiva.
  expect(defaults.trascrizioneDalVivo).toBe(false);
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
  // Gli Assistenti non leggono la Libreria finché l'utente non lo consente.
  expect(defaults.assistenti).toBe(false);
  // Il Microfono non ha un nome finché l'utente non lo sceglie.
  expect(defaults.nomeMicrofono).toBeNull();
});

test("il nome predefinito del Microfono vale solo da Entrambi, con il Microfono persona sola", () => {
  const both = {
    ...defaults,
    nomeMicrofono: " Francesco ",
    recordingSource: "both",
  } satisfies Settings;
  expect(nomeMicrofonoRegistrazione(both)).toBe("Francesco");
  // Riconosci i parlanti sul Microfono vale solo dal vivo, come in Rust.
  expect(nomeMicrofonoRegistrazione({ ...both, parlantiMicrofono: true })).toBe(
    "Francesco"
  );
  expect(
    nomeMicrofonoRegistrazione({
      ...both,
      parlantiMicrofono: true,
      trascrizioneDalVivo: true,
    })
  ).toBeNull();
  expect(
    nomeMicrofonoRegistrazione({ ...both, recordingSource: "mic" })
  ).toBeNull();
  expect(
    nomeMicrofonoRegistrazione({ ...both, nomeMicrofono: "  " })
  ).toBeNull();
});

test("Riconosci i parlanti delle Registrazioni da Entrambi ha una casella per Ingresso, mai il mix", () => {
  const both = { ...defaults, recordingSource: "both" as const };
  expect(parlantiRegistrazione(both)).toEqual([
    "parlantiMicrofono",
    "parlantiSistema",
  ]);
  for (const recordingSource of ["mic", "system"] as const) {
    expect(parlantiRegistrazione({ ...both, recordingSource })).toEqual([
      "parlantiMix",
    ]);
  }
});

test("le impostazioni salvate con la modalità dal vivo di prima si leggono ancora", () => {
  expect(
    settingsSchema.safeParse({ ...defaults, modalitaDalVivo: "mix" }).success
  ).toBe(true);
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
    vocabolario: ["ChargeBee", "Niccolò"],
  };
  expect(settingsSchema.parse(full)).toEqual(full);
});

test("lo schema rifiuta valori fuori dagli elenchi e campi mancanti", () => {
  const invalid = [
    { ...defaults, bitrateKbps: 33 },
    { ...defaults, sampleRate: 44_100 },
    { ...defaults, speechLanguage: "JA" },
    { ...defaults, speechLanguage: "it-IT" },
    { ...defaults, speechLanguage: null },
    { ...defaults, recordingSource: "line-in" },
    { ...defaults, model: "" },
    { ...defaults, trascrizioneDalVivo: "sì" },
    { ...defaults, copiaCome: "html" },
    { ...defaults, parlantiFile: "sì" },
    { ...defaults, parlantiSistema: 1 },
    { ...defaults, vocabolario: "ChargeBee" },
    { model: defaults.model },
  ];
  for (const value of invalid) {
    expect(settingsSchema.safeParse(value).success).toBe(false);
  }
});

test("il selettore offre tutte le lingue del modello, senza regione e in ordine di nome", () => {
  // Nemotron: locale, anche due per la stessa lingua.
  expect(
    speechLanguageChoice(
      ["en-US", "en-GB", "it-IT", "ja-JP", "zh-CN"],
      "it",
      "it"
    )
  ).toEqual({ options: ["zh", "ja", "en", "it"], value: "it" });
  // Whisper e Parakeet: codici, anche di tre lettere. L'ordine segue la lingua dell'interfaccia.
  expect(
    speechLanguageChoice(["de", "yue", "en"], "auto", "en").options
  ).toEqual(["yue", "en", "de"]);
  expect(
    speechLanguageChoice(["de", "yue", "en"], "auto", "de").options
  ).toEqual(["de", "en", "yue"]);
  // Un modello senza lingue offre solo Automatica.
  expect(speechLanguageChoice([], "auto", "it").options).toEqual([]);
});

test("il nome della lingua è nella lingua dell'interfaccia, con la maiuscola", () => {
  expect(speechLanguageName("ja", "it")).toBe("Giapponese");
  expect(speechLanguageName("it", "en")).toBe("Italian");
  expect(speechLanguageName("pl", "pl")).toBe("Polski");
});

test("una lingua salvata che il modello non accetta vale Automatica", () => {
  expect(speechLanguageChoice(["en-US", "it-IT"], "pl", "it")).toEqual({
    options: ["en", "it"],
    value: "auto",
  });
});

test("finché le lingue del modello non sono note resta offerta solo la scelta salvata", () => {
  expect(speechLanguageChoice(null, "auto", "it")).toEqual({
    options: [],
    value: "auto",
  });
  expect(speechLanguageChoice(null, "ja", "it")).toEqual({
    options: ["ja"],
    value: "ja",
  });
});

test("le caselle di Registra da compongono la sorgente e non si spengono tutte", () => {
  expect(withInput("mic", "system", true)).toBe("both");
  expect(withInput("both", "mic", false)).toBe("system");
  expect(withInput("both", "system", false)).toBe("mic");
  expect(withInput("system", "mic", true)).toBe("both");
  expect(withInput("mic", "mic", false)).toBe("mic");
  expect(withInput("system", "system", false)).toBe("system");
});

test("la tendina del Guadagno va da −12 a +24 dB a passi di 3, con il segno", () => {
  expect(GUADAGNI).toEqual([-12, -9, -6, -3, 0, 3, 6, 9, 12, 15, 18, 21, 24]);
  expect([
    guadagnoText(0),
    guadagnoText(6),
    guadagnoText(-3),
    guadagnoText(24),
  ]).toEqual(["0 dB", "+6 dB", "−3 dB", "+24 dB"]);
});

test("sensibilita distinta dalla pulizia e livelli validati", () => {
  for (const name of [
    "audioMicrofono",
    "audioSistema",
    "audioFileMisto",
  ] as const) {
    expect(defaults[name]?.sensibilita).toBe("bilanciato");
    for (const sensibilita of [
      "spento",
      "sensibile",
      "bilanciato",
      "selettivo",
    ]) {
      expect(
        settingsSchema.safeParse({
          ...defaults,
          [name]: { pulizia: true, sensibilita },
        }).success
      ).toBe(true);
    }
    expect(
      settingsSchema.safeParse({
        ...defaults,
        [name]: { pulizia: false, sensibilita: "altro" },
      }).success
    ).toBe(false);
  }
});

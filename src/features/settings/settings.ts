import { z } from "zod";
import type { Language, Settings, SpeechLanguage } from "@/bindings";
import catalog from "../../../src-tauri/src/managers/models.json";

/** Le sei lingue dell'app, nell'ordine dei selettori. */
export const LANGUAGES = [
  "it",
  "en",
  "fr",
  "es",
  "de",
  "pl",
] as const satisfies Language[];

const language = z.enum(LANGUAGES);

/** Il nome di ogni lingua nella lingua stessa, per il selettore della Lingua dell'interfaccia. */
export const LANGUAGE_NAMES: Record<Language, string> = {
  de: "Deutsch",
  en: "English",
  es: "Español",
  fr: "Français",
  it: "Italiano",
  pl: "Polski",
};

const LOCALE_SEPARATOR = /[-_]/;

/**
 * La lingua dell'app per un locale BCP 47 (`it-IT`), altrimenti l'inglese, come
 * `Language::from_locale` in Rust. Serve solo se il backend non risponde all'avvio.
 */
export function languageOf(locale: string): Language {
  const code = locale.split(LOCALE_SEPARATOR)[0]?.toLowerCase();
  return LANGUAGES.find((l) => l === code) ?? "en";
}

/** I bitrate di una Registrazione, in kbps, come in Rust. */
export const BITRATES_KBPS = [16, 24, 32, 48, 64, 96, 128, 192, 320];
/** Le frequenze di una Registrazione, in Hz, come in Rust. */
export const SAMPLE_RATES = [8000, 16_000, 24_000, 48_000];

/** I predefiniti, come `Settings::default` in Rust: valgono se all'avvio non si leggono. */
export const DEFAULT_SETTINGS: Settings = {
  bitrateKbps: 32,
  channels: "mono",
  interfaceLanguage: null,
  microphone: null,
  model: catalog.predefinito,
  outputDevice: null,
  recordingSource: "mic",
  recordingsFolder: null,
  sampleRate: 48_000,
  speechLanguage: "auto",
};

/** Un numero tra `values`: il tipo resta `number`, come nei bindings. */
const oneOf = (values: number[]) =>
  z.number().refine((value) => values.includes(value));

/** Le stesse regole che Rust applica al file impostazioni. */
export const settingsSchema = z.object({
  bitrateKbps: oneOf(BITRATES_KBPS),
  channels: z.enum(["mono", "stereo"]),
  interfaceLanguage: language.nullable(),
  microphone: z.string().nullable(),
  model: z.string().min(1),
  outputDevice: z.string().nullable(),
  recordingSource: z.enum(["mic", "system", "both"]),
  recordingsFolder: z.string().nullable(),
  sampleRate: oneOf(SAMPLE_RATES),
  speechLanguage: z.enum(["auto", ...LANGUAGES]),
}) satisfies z.ZodType<Settings>;

/** Cosa mostra il selettore della Lingua del parlato, oltre ad Automatica. */
export interface SpeechLanguageChoice {
  options: Language[];
  value: SpeechLanguage;
}

/**
 * Le Lingue del parlato da offrire: quelle dell'app che il modello accetta, come codice (`it`) o
 * come locale (`it-IT`). `modelLanguages` è `null` finché il modello non è stato caricato: allora
 * si offre solo la scelta salvata, per non perderla. Una scelta salvata che il modello non accetta
 * vale Automatica, come per il backend.
 */
export function speechLanguageChoice(
  modelLanguages: string[] | null,
  current: SpeechLanguage
): SpeechLanguageChoice {
  let options: Language[];
  if (modelLanguages === null) {
    options = current === "auto" ? [] : [current];
  } else {
    const codes = new Set(
      modelLanguages.map((l) => l.split("-")[0]?.toLowerCase())
    );
    options = LANGUAGES.filter((l) => codes.has(l));
  }
  const value = options.find((l) => l === current) ?? "auto";
  return { options, value };
}

import { z } from "zod";
import type {
  Language,
  RecordingSource,
  Settings,
  SpeechLanguage,
} from "@/bindings";
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

/** I valori del Guadagno in dB, come `valid_gain` in Rust. */
export const GUADAGNI = Array.from({ length: 13 }, (_, i) => i * 3 - 12);

/** I predefiniti, come `Settings::default` in Rust: valgono se all'avvio non si leggono. */
export const DEFAULT_SETTINGS: Settings = {
  assistenti: false,
  bitrateKbps: 16,
  channels: "mono",
  copiaCome: "testo",
  guadagnoMicrofono: 0,
  guadagnoSistema: 0,
  interfaceLanguage: null,
  microphone: null,
  model: catalog.predefinito,
  outputDevice: null,
  parlantiFile: false,
  parlantiMicrofono: false,
  parlantiMix: false,
  parlantiSistema: false,
  raccolta: null,
  recordingSource: "mic",
  recordingsFolder: null,
  sampleRate: 16_000,
  speechLanguage: "auto",
  tema: "sistema",
  trascrizioneDalVivo: false,
};

/** `auto` o il codice ISO 639 di una lingua senza regione, come in Rust. */
const SPEECH_LANGUAGE = /^(auto|[a-z]{2,3})$/;

/** Un numero tra `values`: il tipo resta `number`, come nei bindings. */
const oneOf = (values: number[]) =>
  z.number().refine((value) => values.includes(value));

/** Le stesse regole che Rust applica al file impostazioni. */
export const settingsSchema = z.object({
  // Facoltativa come nei bindings: i file salvati prima che esistesse non ce l'hanno.
  assistenti: z.boolean().optional(),
  bitrateKbps: oneOf(BITRATES_KBPS),
  channels: z.enum(["mono", "stereo"]),
  // Facoltativa come nei bindings: i file salvati prima che esistesse non ce l'hanno.
  copiaCome: z.enum(["testo", "markdown"]).optional(),
  // Facoltative come nei bindings: i file salvati prima che esistessero non ce l'hanno.
  guadagnoMicrofono: oneOf(GUADAGNI).optional(),
  guadagnoSistema: oneOf(GUADAGNI).optional(),
  interfaceLanguage: language.nullable(),
  microphone: z.string().nullable(),
  model: z.string().min(1),
  outputDevice: z.string().nullable(),
  // Facoltativa come nei bindings: i file salvati prima che esistesse non ce l'hanno.
  parlantiFile: z.boolean().optional(),
  // Facoltative come nei bindings: i file salvati prima che esistessero non ce l'hanno.
  parlantiMicrofono: z.boolean().optional(),
  parlantiMix: z.boolean().optional(),
  parlantiSistema: z.boolean().optional(),
  // Facoltativa come nei bindings: i file salvati prima che esistesse non ce l'hanno.
  raccolta: z.string().nullable().optional(),
  recordingSource: z.enum(["mic", "system", "both"]),
  recordingsFolder: z.string().nullable(),
  sampleRate: oneOf(SAMPLE_RATES),
  speechLanguage: z.string().regex(SPEECH_LANGUAGE),
  // Facoltativa come nei bindings: i file salvati prima che esistesse non ce l'hanno.
  tema: z.enum(["sistema", "chiaro", "scuro"]).optional(),
  // Facoltativa come nei bindings: i file salvati prima che esistesse non ce l'hanno.
  trascrizioneDalVivo: z.boolean().optional(),
}) satisfies z.ZodType<Settings>;

/** Cosa mostra il selettore della Lingua del parlato, oltre ad Automatica. */
export interface SpeechLanguageChoice {
  options: SpeechLanguage[];
  value: SpeechLanguage;
}

/**
 * Il nome della Lingua del parlato `code` (`ja`) nella lingua `locale`, con l'iniziale maiuscola,
 * come `SpeechLanguage::name` in Rust: "Giapponese".
 */
export function speechLanguageName(code: SpeechLanguage, locale: string) {
  const name =
    new Intl.DisplayNames([locale], { type: "language" }).of(code) ?? code;
  return name.charAt(0).toLocaleUpperCase(locale) + name.slice(1);
}

/**
 * Le Lingue del parlato da offrire: tutte quelle del modello, come codice senza regione (`it-IT` di
 * Nemotron diventa `it`), in ordine di nome nella lingua `locale`. `modelLanguages` è `null` finché
 * il modello non è stato caricato: allora si offre solo la scelta salvata, per non perderla. Una
 * scelta salvata che il modello non accetta vale Automatica, come per il backend.
 */
export function speechLanguageChoice(
  modelLanguages: string[] | null,
  current: SpeechLanguage,
  locale: string
): SpeechLanguageChoice {
  if (modelLanguages === null) {
    return { options: current === "auto" ? [] : [current], value: current };
  }
  const codes = new Set(
    modelLanguages
      .map((l) => l.split(LOCALE_SEPARATOR)[0]?.toLowerCase() ?? "")
      .filter((code) => SPEECH_LANGUAGE.test(code) && code !== "auto")
  );
  const named = [...codes].map((code) => ({
    code,
    name: speechLanguageName(code, locale),
  }));
  named.sort((a, b) => a.name.localeCompare(b.name, locale));
  const options = named.map((l) => l.code);
  return { options, value: codes.has(current) ? current : "auto" };
}

/** Una casella di Riconosci i parlanti delle Registrazioni. */
export type ParlantiRegistrazione =
  | "parlantiMix"
  | "parlantiMicrofono"
  | "parlantiSistema";

/** L'etichetta di ogni casella di Riconosci i parlanti delle Registrazioni. */
export const PARLANTI_LABELS: Record<ParlantiRegistrazione, string> = {
  parlantiMicrofono: "settings.recording.inputs.mic",
  parlantiMix: "settings.recording.parlantiMix",
  parlantiSistema: "settings.recording.inputs.system",
};

/**
 * Le caselle di Riconosci i parlanti delle Registrazioni da mostrare: registrando da Entrambi (sempre
 * a Ingressi separati, ADR-0015) una per Ingresso, altrimenti una per il mix. È la scelta degli
 * Ingressi di `Settings::parlanti_registrazione` in Rust, che in più vale solo con la Trascrizione
 * dal vivo e guarda quali caselle sono attive.
 */
export function parlantiRegistrazione(
  settings: Settings
): ParlantiRegistrazione[] {
  return settings.recordingSource === "both"
    ? ["parlantiMicrofono", "parlantiSistema"]
    : ["parlantiMix"];
}

/** Le due caselle di "Registra da": accese, o spente, compongono la sorgente di registrazione. */
export type RecordingInput = "mic" | "system";

/**
 * La sorgente con la casella `input` accesa o spenta. Spegnere l'ultima accesa non cambia nulla:
 * si registra sempre da qualcosa.
 */
export function withInput(
  source: RecordingSource,
  input: RecordingInput,
  on: boolean
): RecordingSource {
  const mic = input === "mic" ? on : source !== "system";
  const system = input === "system" ? on : source !== "mic";
  if (mic && system) {
    return "both";
  }
  if (mic) {
    return "mic";
  }
  return system ? "system" : source;
}

/** Il Guadagno nella tendina: "0 dB", gli altri con il segno ("+6 dB", "−3 dB"). */
export function guadagnoText(db: number): string {
  if (db === 0) {
    return "0 dB";
  }
  return db > 0 ? `+${db} dB` : `−${-db} dB`;
}

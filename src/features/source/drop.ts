import { fileName, isBino } from "@/features/source/file-name";

/** Le estensioni di Apri file: specchio di `SOURCE_EXTENSIONS` in `commands/mod.rs`. */
export const SOURCE_EXTENSIONS = [
  "mp3",
  "wav",
  "m4a",
  "flac",
  "ogg",
  "opus",
  "webm",
  "mpga",
  "mpeg",
  "aiff",
  "mp4",
  "mkv",
  "mov",
  "m4v",
  "bino",
];

export type DropVerdict =
  | { accepted: true; path: string }
  | { accepted: false; reason: "busy" | "format" | "many" };

/**
 * Cosa succede ai percorsi trascinati da Esplora file: un solo file accettato da Apri file si apre;
 * durante un'Attività solo un Bino, in consultazione. Una cartella non ha un'estensione accettata.
 */
export function dropVerdict(paths: string[], busy: boolean): DropVerdict {
  const [path] = paths;
  if (paths.length > 1) {
    return { accepted: false, reason: "many" };
  }
  const name = path ? fileName(path) : "";
  const extension = name.includes(".")
    ? name.slice(name.lastIndexOf(".") + 1).toLowerCase()
    : "";
  if (!(path && SOURCE_EXTENSIONS.includes(extension))) {
    return { accepted: false, reason: "format" };
  }
  if (busy && !isBino(path)) {
    return { accepted: false, reason: "busy" };
  }
  return { accepted: true, path };
}

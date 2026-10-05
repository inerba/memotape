const PATH_SEPARATOR = /[\\/]/;
const LAST_SEGMENT = /[\\/][^\\/]*$/;

/** Il nome della Sorgente, dall'ultimo segmento del percorso. */
export function fileName(path: string) {
  return path.split(PATH_SEPARATOR).pop() || path;
}

/** La cartella che contiene `path`, senza il separatore finale. */
export function folderOf(path: string) {
  return path.replace(LAST_SEGMENT, "");
}

/**
 * Il percorso di `path` dopo che il Tape o la cartella `from`, che lo è o lo contiene, è diventato
 * `to`; altrimenti `path`. Windows non distingue maiuscole e minuscole.
 */
export function movedPath(path: string, from: string, to: string): string {
  const lower = path.toLowerCase();
  const prefix = from.toLowerCase();
  if (lower === prefix) {
    return to;
  }
  const inside =
    lower.startsWith(prefix) && PATH_SEPARATOR.test(path[from.length] ?? "");
  return inside ? to + path.slice(from.length) : path;
}

/** Se la Sorgente è un Tape: si apre con il suo testo e il clic sul nome la mostra nella cartella. */
export function isTape(path: string) {
  return fileName(path).toLowerCase().endsWith(".bino");
}

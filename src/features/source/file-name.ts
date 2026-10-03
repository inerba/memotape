const PATH_SEPARATOR = /[\\/]/;

/** Il nome della Sorgente, dall'ultimo segmento del percorso. */
export function fileName(path: string) {
  return path.split(PATH_SEPARATOR).pop() || path;
}

/** Se la Sorgente è un Bino: si apre con il suo testo e il clic sul nome la mostra nella cartella. */
export function isBino(path: string) {
  return fileName(path).toLowerCase().endsWith(".bino");
}

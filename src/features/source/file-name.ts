const PATH_SEPARATOR = /[\\/]/;

/** Il nome della Sorgente, dall'ultimo segmento del percorso. */
export function fileName(path: string) {
  return path.split(PATH_SEPARATOR).pop() || path;
}

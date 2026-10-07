import { useCallback, useEffect, useState } from "react";
import { isTape, movedPath } from "@/features/source/file-name";

const LAST_TAPE_KEY = "memotape.last-tape";

function readLastTape(): string | null {
  try {
    const path = localStorage.getItem(LAST_TAPE_KEY);
    return path && isTape(path) ? path : null;
  } catch {
    return null;
  }
}

/** Memoria della navigazione, separata dalle Impostazioni e dal contenuto del Tape. */
export function useLastTape() {
  const [lastPath, remember] = useState(readLastTape);
  useEffect(() => {
    try {
      if (lastPath) {
        localStorage.setItem(LAST_TAPE_KEY, lastPath);
      } else {
        localStorage.removeItem(LAST_TAPE_KEY);
      }
    } catch {
      // Se la memoria del browser non è disponibile, resta valida per questa sessione.
    }
  }, [lastPath]);
  const move = useCallback((from: string, to: string) => {
    remember((current) => current && movedPath(current, from, to));
  }, []);
  const forget = useCallback((path: string) => {
    remember((current) =>
      current?.toLowerCase() === path.toLowerCase() ? null : current
    );
  }, []);
  return { forget, lastPath, move, remember };
}

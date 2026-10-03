import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useMemo,
  useState,
} from "react";
import { type AppError, commands, type Settings } from "@/bindings";

interface SettingsContextValue {
  /** Perché all'avvio le impostazioni non si sono lette: allora valgono i predefiniti. */
  loadError: AppError | null;
  /** Salva le impostazioni; restituisce l'errore se Rust le rifiuta o non riesce a scriverle. */
  save: (next: Settings) => Promise<AppError | null>;
  settings: Settings;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

/** Le impostazioni, lette da Rust prima del primo render (`initial`). */
export function SettingsProvider({
  children,
  initial,
  loadError,
}: {
  children: ReactNode;
  initial: Settings;
  loadError: AppError | null;
}) {
  const [settings, setSettings] = useState(initial);
  const save = useCallback(async (next: Settings) => {
    const result = await commands.setSettings(next);
    if (result.status === "error") {
      return result.error;
    }
    // Non sempre `next`: se all'avvio il file non si è letto, è il file con sopra la modifica.
    setSettings(result.data);
    return null;
  }, []);
  const value = useMemo(
    () => ({ loadError, save, settings }),
    [loadError, save, settings]
  );
  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettings(): SettingsContextValue {
  const value = useContext(SettingsContext);
  if (!value) {
    throw new Error("useSettings fuori da SettingsProvider");
  }
  return value;
}

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
  /** Salva le impostazioni; restituisce l'errore se Rust le rifiuta o non riesce a scriverle. */
  save: (next: Settings) => Promise<AppError | null>;
  settings: Settings;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

/** Le impostazioni, lette da Rust prima del primo render (`initial`). */
export function SettingsProvider({
  children,
  initial,
}: {
  children: ReactNode;
  initial: Settings;
}) {
  const [settings, setSettings] = useState(initial);
  const save = useCallback(async (next: Settings) => {
    const result = await commands.setSettings(next);
    if (result.status === "error") {
      return result.error;
    }
    setSettings(next);
    return null;
  }, []);
  const value = useMemo(() => ({ save, settings }), [save, settings]);
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

import {
  createContext,
  type ReactNode,
  useContext,
  useLayoutEffect,
  useMemo,
  useState,
} from "react";
import { type AppError, commands, type Settings } from "@/bindings";
import {
  createSettingsWriter,
  type SaveFeedback,
  type SettingField,
} from "./settings-writer";

interface SettingsContextValue {
  clearFeedback: ReturnType<typeof createSettingsWriter>["clearFeedback"];
  feedback: Partial<Record<SettingField, SaveFeedback>>;
  flush: () => Promise<AppError | null>;
  /** Perché all'avvio le impostazioni non si sono lette: allora valgono i predefiniti. */
  loadError: AppError | null;
  retry: ReturnType<typeof createSettingsWriter>["retry"];
  /** Salva le impostazioni; restituisce l'errore se Rust le rifiuta o non riesce a scriverle. */
  save: ReturnType<typeof createSettingsWriter>["save"];
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
  useLayoutEffect(() => {
    const system = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      document.documentElement.dataset.theme =
        settings.tema === "scuro" ||
        (settings.tema !== "chiaro" && system.matches)
          ? "dark"
          : "light";
    };
    apply();
    if (!settings.tema || settings.tema === "sistema") {
      system.addEventListener("change", apply);
      return () => system.removeEventListener("change", apply);
    }
  }, [settings.tema]);
  const [feedback, setFeedback] = useState<
    Partial<Record<SettingField, SaveFeedback>>
  >({});
  const [writer] = useState(() =>
    createSettingsWriter(
      initial,
      commands.setSettings,
      setSettings,
      (field, state) =>
        setFeedback((current) => ({ ...current, [field]: state }))
    )
  );
  const { clearFeedback, save, flush, retry } = writer;
  const value = useMemo(
    () => ({
      clearFeedback,
      feedback,
      flush,
      loadError,
      retry,
      save,
      settings,
    }),
    [clearFeedback, feedback, loadError, retry, save, flush, settings]
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
